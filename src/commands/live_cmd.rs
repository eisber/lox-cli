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

    fn read_timeout_fixture_headers(socket: &mut impl std::io::Read) -> std::io::Result<bool> {
        Ok(read_timeout_fixture_request(socket)?.is_some())
    }

    fn read_timeout_fixture_request(
        socket: &mut impl std::io::Read,
    ) -> std::io::Result<Option<Vec<u8>>> {
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            if let Err(error) = socket.read_exact(&mut byte) {
                match error.kind() {
                    std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::ConnectionReset => {
                        return Ok(None);
                    }
                    _ => return Err(error),
                }
            }
            request.push(byte[0]);
        }
        Ok(Some(request))
    }

    #[test]
    fn timeout_fixture_accepts_a_client_cancelled_before_complete_headers() {
        use std::io::Write;
        use std::net::{Shutdown, TcpListener, TcpStream};

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        client.write_all(b"GET / HTTP/1.1\r\n").unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        assert!(!read_timeout_fixture_headers(&mut socket).unwrap());
        assert!(read_timeout_fixture_headers(&mut &b"GET / HTTP/1.1\r\n\r\n"[..]).unwrap());
    }

    #[test]
    fn timeout_fixture_retains_complete_headers_after_client_cancellation() {
        use std::io::Write;
        use std::net::{Shutdown, TcpListener, TcpStream};

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let mut client = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
        let request = b"GET /jdev/sys/date HTTP/1.1\r\nHost: localhost\r\n\r\n";
        client.write_all(request).unwrap();
        client.shutdown(Shutdown::Both).unwrap();
        drop(client);
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        assert_eq!(
            read_timeout_fixture_request(&mut socket)
                .unwrap()
                .as_deref(),
            Some(request.as_slice())
        );
    }

    #[test]
    fn legacy_phase_timeouts_preserve_actual_blocking_messages_and_categories() {
        use std::io::Write;
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
                            if !read_timeout_fixture_headers(&mut socket).unwrap() {
                                eprintln!(
                                    "timeout fixture: client cancelled before complete headers"
                                );
                                return;
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
        use std::net::TcpListener;

        // Keep the peer listening without replying. A total request timeout can
        // expire before headers are sent, so the wire witness is separate.
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let host = format!("http://{}", listener.local_addr().unwrap());
        let timeout = Duration::from_millis(20);
        let legacy = legacy_clock_fixture(&host, timeout);
        let legacy_error = legacy.get_json("/jdev/sys/date").unwrap_err();
        let driver = runtime_clock_fixture(host, timeout);
        let error = clock_error(driver.read_clock(&Cancellation::new()).unwrap_err());
        driver.close().unwrap();
        drop(legacy);
        drop(listener);
        assert!(
            legacy_error
                .downcast_ref::<reqwest::Error>()
                .unwrap()
                .is_timeout()
        );
        assert!(matches!(
            error.downcast_ref::<ClockError>(),
            Some(ClockError::Transport {
                endpoint: lox_core::runtime::ClockEndpoint::Date,
                kind: lox_core::runtime::TransportFailure::Timeout,
            })
        ));
        assert_eq!(
            crate::categorize_error(&error),
            crate::categorize_error(&legacy_error)
        );
    }

    fn legacy_clock_fixture(host: &str, timeout: Duration) -> LoxClient {
        let mut legacy = LoxClient::new(Config {
            host: host.into(),
            user: "clock-test".into(),
            pass: "synthetic-only".into(),
            ..Config::default()
        })
        .unwrap();
        legacy.client = reqwest::blocking::Client::builder()
            .no_proxy()
            .timeout(timeout)
            .build()
            .unwrap();
        legacy
    }

    fn runtime_clock_fixture(host: String, timeout: Duration) -> ClockDriver {
        let mut options = ClockOptions::new(host, "clock-test", "synthetic-only");
        options.use_system_proxy = false;
        options.timeout = timeout;
        options.retry_delays.clear();
        ClockDriver::new(options).unwrap()
    }

    #[test]
    fn clock_wire_witness_observes_four_legacy_attempts_and_one_runtime_attempt() {
        use std::io::Write;
        use std::net::TcpListener;
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::thread;
        use std::time::Instant;

        let requests = AtomicUsize::new(0);
        thread::scope(|scope| {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let host = format!("http://{}", listener.local_addr().unwrap());
            let fixture_timeout = Duration::from_secs(3);
            let count = &requests;
            let worker = scope.spawn(move || {
                let deadline = Instant::now() + fixture_timeout;
                while count.load(Ordering::Relaxed) < 5 {
                    match listener.accept() {
                        Ok((mut socket, _)) => {
                            socket.set_nonblocking(false).unwrap();
                            socket.set_read_timeout(Some(fixture_timeout)).unwrap();
                            socket.set_write_timeout(Some(fixture_timeout)).unwrap();
                            let request = read_timeout_fixture_request(&mut socket)
                                .unwrap()
                                .expect("Expected complete clock request headers");
                            assert!(request.starts_with(b"GET /jdev/sys/date HTTP/1.1\r\n"));
                            count.fetch_add(1, Ordering::Relaxed);
                            socket
                                .write_all(
                                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                                )
                                .unwrap();
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            assert!(
                                Instant::now() < deadline,
                                "Missing clock request connection"
                            );
                            thread::park_timeout(Duration::from_millis(1));
                        }
                        Err(error) => panic!("Clock fixture accept failed: {error}"),
                    }
                }
                listener
            });
            let legacy = legacy_clock_fixture(&host, fixture_timeout);
            let legacy_error = legacy.get_json("/jdev/sys/date").unwrap_err();
            assert_eq!(
                legacy_error
                    .downcast_ref::<HttpStatusError>()
                    .unwrap()
                    .status,
                503
            );
            assert_eq!(requests.load(Ordering::Relaxed), 4);
            let driver = runtime_clock_fixture(host, fixture_timeout);
            assert_eq!(
                driver.read_clock(&Cancellation::new()).unwrap_err(),
                ClockError::Http {
                    endpoint: lox_core::runtime::ClockEndpoint::Date,
                    status: 503,
                }
            );
            driver.close().unwrap();
            drop(legacy);
            let listener = worker.join().unwrap();
            assert_eq!(requests.load(Ordering::Relaxed), 5);
            assert!(matches!(
                listener.accept(),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
            ));
        });
    }
}
