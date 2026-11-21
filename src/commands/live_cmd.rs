//! `lox live` — thin, read-only-by-default helpers over the Miniserver's
//! `jdev/sps/io` HTTP API.
//!
//! These exist so live verification (does the Virtual Input echo? what is the
//! actor showing? what is the Miniserver clock?) does not require hand-rolled
//! `curl`/`Invoke-WebRequest` with basic-auth and `--SkipCertificateCheck`.
//! Reads are unrestricted; writes are gated behind `--write`.
//!
//! Note: color/composite (`<v.col>`) states return the literal template string
//! over HTTP polling, not a number — perceived color cannot be read back here
//! (it needs the websocket/visu channel), so color verification stays a
//! human-in-the-loop step.

use anyhow::{Context, Result, bail};
use clap::Subcommand;
use lox_core::runtime::{
    Cancellation, ClockDriver, ClockError, ClockOptions, ClockResponsePolicy,
    ControlReadCompatibility, ControlReadDiagnostic, ControlReadDriver, ControlReadError,
    ControlReadFailure, ControlReadOptions, ControlReadPhase, ControlReadPhaseTimeout,
    ControlReadResponsePolicy, Diagnostic,
};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::client::{self, HttpStatusError, LoxClient};
use crate::commands::RunContext;
use crate::config::Config;

#[derive(Subcommand)]
pub enum LiveCmd {
    /// Read a control/connector state: GET /jdev/sps/io/<uuid>
    Get {
        /// Control UUID (or name resolvable via the structure cache)
        uuid: String,
    },
    /// Write a value/command to a control: GET /jdev/sps/io/<uuid>/<value>
    Set {
        /// Control UUID (or name resolvable via the structure cache)
        uuid: String,
        /// Value or command string (e.g. "1", "0", "on", "hsv(120,100,30)")
        value: String,
        /// Required to perform the write (live, mutating operation)
        #[arg(long)]
        write: bool,
    },
    /// Show the Miniserver date and time
    Time,
}

pub fn cmd_live(ctx: &RunContext, action: LiveCmd) -> Result<()> {
    let cfg = Config::load()?;
    match action {
        LiveCmd::Get { uuid } => get(ctx, &mut LoxClient::new(cfg)?, &uuid),
        LiveCmd::Set { uuid, value, write } => {
            set(ctx, &mut LoxClient::new(cfg)?, &uuid, &value, write)
        }
        LiveCmd::Time => time(ctx, cfg),
    }
}

/// Extract `(value, code)` from a Loxone `{"LL": {...}}` envelope.
fn parse_ll(v: &Value) -> (String, Option<String>) {
    let ll = &v["LL"];
    let value = ll["value"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| ll["value"].as_f64().map(|f| f.to_string()))
        .unwrap_or_default();
    let code = ll["Code"]
        .as_str()
        .map(|s| s.to_string())
        .or_else(|| ll["code"].as_str().map(|s| s.to_string()))
        .or_else(|| ll["Code"].as_i64().map(|n| n.to_string()));
    (value, code)
}

fn get(ctx: &RunContext, lox: &mut LoxClient, name_or_uuid: &str) -> Result<()> {
    let uuid = lox.resolve(name_or_uuid)?;
    let path = format!("/jdev/sps/io/{uuid}");
    let cfg = &lox.cfg;
    let mut options = ControlReadOptions::new(&cfg.host, &cfg.user, &cfg.pass);
    options.compatibility = ControlReadCompatibility::LegacyCli;
    options.verify_tls = cfg.verify_ssl.unwrap_or(false);
    options.user_agent = client::USER_AGENT.into();
    let verbosity = client::verbose();
    if verbosity >= 2 {
        options.response_policy = ControlReadResponsePolicy::LegacyCliVerbose {
            body_observer: Arc::new(client::log_body),
        };
    }
    if verbosity >= 1 {
        let url = client::redact_url(&format!("{}/{}", cfg.host, path.trim_start_matches('/')));
        options.diagnostics = Some(Arc::new(move |event| match event {
            ControlReadDiagnostic::Request => eprintln!("GET {url}"),
            ControlReadDiagnostic::Response { status } => {
                let reason = reqwest::StatusCode::from_u16(status)
                    .ok()
                    .and_then(|status| status.canonical_reason())
                    .unwrap_or("");
                eprintln!("  → {status} {reason}");
            }
            _ => {}
        }));
    }
    let read = || -> Result<_> {
        let driver =
            ControlReadDriver::new(options).map_err(|error| control_read_error(error, &path))?;
        let result = driver.read_control(&uuid, &Cancellation::new());
        let closed = driver.close();
        let observation = result.map_err(|error| control_read_error(error, &path))?;
        closed?;
        Ok(observation.legacy_text())
    };
    let state = read().with_context(|| format!("reading state of {uuid}"))?;
    let (value, code) = (state.value, state.code);
    if ctx.json {
        println!("{}", json!({ "uuid": uuid, "value": value, "code": code }));
    } else {
        println!("{value}");
        if value.contains("<v.col>") || value.contains("<v>") {
            eprintln!(
                "note: this is a color/template state — HTTP polling returns the literal \
                 template, not a number. Perceived color is not readable here."
            );
        }
    }
    Ok(())
}

fn control_read_error(error: ControlReadError, path: &str) -> anyhow::Error {
    if let ControlReadFailure::Http { status } = error.kind {
        return HttpStatusError {
            status,
            path: path.into(),
        }
        .into();
    }
    // Explicit legacy adapter: preserve the old reqwest/serde source chain and
    // category. Core's normal Debug/Display never disclose these backend details.
    let message = error.to_string();
    match error.into_source() {
        Some(source) => match source.downcast::<reqwest::Error>() {
            Ok(source) => anyhow::Error::new(*source),
            Err(source) => match source.downcast::<serde_json::Error>() {
                Ok(source) => anyhow::Error::new(*source),
                Err(source) => match source.downcast::<ControlReadPhaseTimeout>() {
                    Ok(source) => {
                        let context = match source.phase() {
                            ControlReadPhase::Headers => {
                                format!("error sending request for url ({})", source.request_url())
                            }
                            ControlReadPhase::Body => "error decoding response body".into(),
                        };
                        anyhow::Error::new(*source).context(context)
                    }
                    Err(source) => anyhow::Error::from_boxed(source),
                },
            },
        },
        None => anyhow::anyhow!("{message}"),
    }
}

fn set(
    ctx: &RunContext,
    lox: &mut LoxClient,
    name_or_uuid: &str,
    value: &str,
    write: bool,
) -> Result<()> {
    if !write {
        bail!(
            "refusing to write to the live Miniserver without --write \
             (would send '{value}' to '{name_or_uuid}')"
        );
    }
    let uuid = lox.resolve(name_or_uuid)?;
    let resp = lox
        .send_cmd(&uuid, value)
        .with_context(|| format!("writing '{value}' to {uuid}"))?;
    let (ret, code) = parse_ll(&resp);
    if ctx.json {
        println!(
            "{}",
            json!({ "uuid": uuid, "sent": value, "value": ret, "code": code })
        );
    } else if !ctx.quiet {
        println!(
            "✓ {uuid} ← {value}  (Code {})",
            code.as_deref().unwrap_or("?")
        );
        if !ret.is_empty() {
            println!("  value: {ret}");
        }
    }
    Ok(())
}

fn time(ctx: &RunContext, cfg: Config) -> Result<()> {
    let mut options = ClockOptions::new(&cfg.host, &cfg.user, &cfg.pass);
    options.verify_tls = cfg.verify_ssl.unwrap_or(false);
    options.user_agent = client::USER_AGENT.into();
    let verbosity = client::verbose();
    if verbosity >= 2 {
        options.response_policy = ClockResponsePolicy::LegacyCliVerbose {
            body_observer: Arc::new(|_, body| client::log_body(body)),
        };
    }
    if verbosity >= 1 {
        if !options.verify_tls {
            eprintln!("warning: TLS certificate verification is disabled (verify_ssl not set)");
        }
        let mut host = reqwest::Url::parse(&cfg.host)?;
        let _ = host.set_username("");
        let _ = host.set_password(None);
        let host = client::redact_url(host.as_str());
        options.diagnostics = Some(Arc::new(move |event| match event {
            Diagnostic::Request(endpoint) => {
                eprintln!("GET {}{}", host.trim_end_matches('/'), endpoint.path());
            }
            Diagnostic::Response { status, .. } => {
                let reason = reqwest::StatusCode::from_u16(status)
                    .ok()
                    .and_then(|status| status.canonical_reason())
                    .unwrap_or("");
                eprintln!("  → {status} {reason}");
            }
            _ => {}
        }));
    }
    let driver = ClockDriver::new(options)?;
    let result = driver.read_clock(&Cancellation::new());
    // Always close before rendering, including failure, without replacing the
    // operation's original diagnostic with a secondary cleanup error.
    let closed = driver.close();
    let observation = result.map_err(clock_error)?;
    closed?;
    let clock = observation.legacy_text();
    let (date, time) = (clock.date, clock.time);
    if ctx.json {
        println!("{}", json!({ "date": date, "time": time }));
    } else {
        println!("{date} {time}");
    }
    Ok(())
}

fn clock_error(error: ClockError) -> anyhow::Error {
    match error {
        ClockError::Http { endpoint, status } => anyhow::Error::new(HttpStatusError {
            status,
            path: endpoint.path().into(),
        }),
        error => error.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::{Method::GET, MockServer};
    use std::time::Duration;

    #[test]
    fn legacy_phase_timeouts_preserve_actual_blocking_messages_and_categories() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;

        for body_phase in [false, true] {
            for verbose in [false, true] {
                let listener = TcpListener::bind("127.0.0.1:0").unwrap();
                let host = format!("http://{}", listener.local_addr().unwrap());
                let worker = thread::spawn(move || {
                    let mut responses = Vec::new();
                    for _ in 0..2 {
                        let (mut socket, _) = listener.accept().unwrap();
                        responses.push(thread::spawn(move || {
                            socket
                                .set_read_timeout(Some(Duration::from_secs(3)))
                                .unwrap();
                            let mut request = Vec::new();
                            while !request.ends_with(b"\r\n\r\n") {
                                let mut byte = [0];
                                socket.read_exact(&mut byte).unwrap();
                                request.push(byte[0]);
                            }
                            if body_phase {
                                socket
                                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n{")
                                    .unwrap();
                            }
                            thread::sleep(Duration::from_millis(150));
                        }));
                    }
                    for response in responses {
                        response.join().unwrap();
                    }
                });
                let path = "/jdev/sps/io/abcdef";
                let legacy = reqwest::blocking::Client::builder()
                    .no_proxy()
                    .timeout(Duration::from_millis(80))
                    .build()
                    .unwrap();
                let reference = (|| -> Result<Value> {
                    let response = legacy.get(format!("{host}{path}")).send()?;
                    if verbose {
                        Ok(serde_json::from_str(&response.text()?)?)
                    } else {
                        Ok(response.json()?)
                    }
                })()
                .unwrap_err();
                let mut options = ControlReadOptions::new(&host, "", "");
                options.compatibility = ControlReadCompatibility::LegacyCli;
                options.use_system_proxy = false;
                options.timeout = Duration::from_millis(80);
                options.retry_delays.clear();
                if verbose {
                    options.response_policy = ControlReadResponsePolicy::LegacyCliVerbose {
                        body_observer: Arc::new(|_| panic!("No complete body on timeout")),
                    };
                }
                let driver = ControlReadDriver::new(options).unwrap();
                let actual = control_read_error(
                    driver
                        .read_control("abcdef", &Cancellation::new())
                        .unwrap_err(),
                    path,
                );
                driver.close().unwrap();
                assert_eq!(format!("{actual:#}"), format!("{reference:#}"));
                assert_eq!(
                    crate::categorize_error(&actual),
                    crate::categorize_error(&reference)
                );
                assert!(
                    reference
                        .downcast_ref::<reqwest::Error>()
                        .unwrap()
                        .is_timeout()
                );
                assert!(actual.downcast_ref::<ControlReadPhaseTimeout>().is_some());
                worker.join().unwrap();
            }
        }
    }

    #[test]
    fn control_read_adapter_preserves_actual_legacy_http_and_decode_sources() {
        let server = MockServer::start();
        let path = "/jdev/sps/io/abcdef";
        for (status, body, attempts) in [(401, "{", 1), (503, "{", 4), (200, "{", 4)] {
            let mut response = server.mock(|when, then| {
                when.method(GET).path(path);
                then.status(status).body(body);
            });
            let legacy = LoxClient::new(Config {
                host: server.base_url(),
                user: "read-test".into(),
                pass: "synthetic-only".into(),
                ..Config::default()
            })
            .unwrap();
            let expected = legacy.get_json(path).unwrap_err();
            let mut options =
                ControlReadOptions::new(server.base_url(), "read-test", "synthetic-only");
            options.use_system_proxy = false;
            options.retry_delays = vec![Duration::from_millis(1); 3];
            let driver = ControlReadDriver::new(options).unwrap();
            let actual = control_read_error(
                driver
                    .read_control("abcdef", &Cancellation::new())
                    .unwrap_err(),
                path,
            );
            driver.close().unwrap();
            assert_eq!(format!("{actual:#}"), format!("{expected:#}"));
            assert_eq!(
                crate::categorize_error(&actual),
                crate::categorize_error(&expected)
            );
            if status == 200 {
                assert!(actual.downcast_ref::<reqwest::Error>().is_some());
            } else {
                assert!(actual.downcast_ref::<HttpStatusError>().is_some());
            }
            response.assert_hits(attempts * 2);
            response.delete();
        }
    }

    #[test]
    fn clock_timeout_category_matches_legacy_transport() {
        let server = MockServer::start();
        let date = server.mock(|when, then| {
            when.method(GET).path("/jdev/sys/date");
            then.delay(Duration::from_millis(200)).body("{}");
        });
        let mut legacy = LoxClient::new(Config {
            host: server.base_url(),
            user: "clock-test".into(),
            pass: "synthetic-only".into(),
            ..Config::default()
        })
        .unwrap();
        legacy.client = reqwest::blocking::Client::builder()
            .no_proxy()
            .timeout(Duration::from_millis(20))
            .build()
            .unwrap();
        let legacy_error = legacy.get_json("/jdev/sys/date").unwrap_err();
        let mut options = ClockOptions::new(server.base_url(), "clock-test", "synthetic-only");
        options.use_system_proxy = false;
        options.timeout = Duration::from_millis(20);
        options.retry_delays.clear();
        let driver = ClockDriver::new(options).unwrap();
        let error = clock_error(driver.read_clock(&Cancellation::new()).unwrap_err());
        driver.close().unwrap();
        assert_eq!(
            crate::categorize_error(&error),
            crate::categorize_error(&legacy_error)
        );
        date.assert_hits(5);
    }
}
