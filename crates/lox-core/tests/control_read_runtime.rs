#![cfg(feature = "runtime")]

use httpmock::{Method::GET, MockServer};
use lox_core::runtime::*;
use serde_json::json;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

const ID: &str = "1fbc668c-005c-7471-ffffed57184a04d2";

fn options(host: String) -> ControlReadOptions {
    let mut options = ControlReadOptions::new(host, "read", "synthetic-only");
    options.use_system_proxy = false;
    options.timeout = Duration::from_secs(3);
    options.shutdown_timeout = Duration::from_secs(2);
    options.retry_delays = vec![Duration::from_millis(1); 3];
    options
}

#[tokio::test(flavor = "current_thread")]
async fn observations_preserve_shapes_and_legacy_code_precedence_without_clock_validation() {
    let server = MockServer::start();
    for (wire, value, code, issue) in [
        (
            json!(null),
            "",
            None,
            Some(ControlValidationIssue::Envelope),
        ),
        (json!([]), "", None, Some(ControlValidationIssue::Envelope)),
        (
            json!({"LL":{"value":true,"Code":200}}),
            "",
            Some("200"),
            Some(ControlValidationIssue::UnsupportedValue),
        ),
        (json!({"LL":{"value":"","Code":200}}), "", Some("200"), None),
        (
            json!({"LL":{"value":42,"Code":401,"code":"200"}}),
            "42",
            Some("200"),
            Some(ControlValidationIssue::RejectedCode),
        ),
        (
            json!({"LL":{"value":-0.0,"Code":"200","code":"401"}}),
            "-0",
            Some("200"),
            None,
        ),
        (
            json!({"LL":{"value":"kept","code":200}}),
            "kept",
            None,
            None,
        ),
    ] {
        let mut response = server.mock(|when, then| {
            when.method(GET)
                .path(format!("/jdev/sps/io/{ID}"))
                .header("authorization", "Basic cmVhZDpzeW50aGV0aWMtb25seQ==");
            then.json_body(wire);
        });
        let client = ControlReadClient::new(options(server.base_url())).unwrap();
        let observation = client.read_control(ID, &Cancellation::new()).await.unwrap();
        assert_eq!(
            observation.legacy_text(),
            ControlState {
                value: value.into(),
                code: code.map(str::to_owned),
            }
        );
        assert_eq!(observation.validate().err(), issue);
        assert_eq!(client.active_operations(), 0);
        client.close().await.unwrap();
        response.assert_hits(1);
        response.delete();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn strict_targets_are_distinct_from_explicit_legacy_path_and_host_admission() {
    let server = MockServer::start();
    let state = server.mock(|when, then| {
        when.method(GET)
            .path("/prefix//jdev/sps/io/child/state")
            .query_param("flag", "1")
            .header("authorization", "Basic Og==")
            .header("user-agent", "read-test");
        then.json_body(json!({"LL":{"value":"ok","Code":200}}));
    });
    let mut config = options(format!("{}/prefix/", server.base_url()));
    config.user.clear();
    config.password.clear();
    config.user_agent = "read-test".into();
    let strict = ControlReadClient::new(config.clone()).unwrap();
    assert!(matches!(
        strict
            .read_control("child/state?flag=1", &Cancellation::new())
            .await
            .unwrap_err()
            .kind,
        ControlReadFailure::InvalidTarget(_)
    ));
    strict.close().await.unwrap();
    state.assert_hits(0);
    config.compatibility = ControlReadCompatibility::LegacyCli;
    let legacy = ControlReadClient::new(config).unwrap();
    assert_eq!(
        legacy
            .read_control("child/state?flag=1", &Cancellation::new())
            .await
            .unwrap()
            .validate()
            .unwrap()
            .value,
        "ok"
    );
    legacy.close().await.unwrap();
    state.assert_hits(1);

    let mut invalid = options("http://[invalid".into());
    assert!(matches!(
        ControlReadClient::new(invalid.clone()).err().unwrap().kind,
        ControlReadFailure::InvalidHost
    ));
    invalid.compatibility = ControlReadCompatibility::LegacyCli;
    invalid.retry_delays.clear();
    let legacy = ControlReadClient::new(invalid).unwrap();
    assert!(matches!(
        legacy
            .read_control(ID, &Cancellation::new())
            .await
            .unwrap_err()
            .kind,
        ControlReadFailure::Transport {
            kind: TransportFailure::Request
        }
    ));
    legacy.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn status_precedes_body_decode_and_failures_have_no_partial_observation() {
    let server = MockServer::start();
    for (status, body, hits, kind) in [
        (401, "{", 1, ControlReadFailure::Http { status: 401 }),
        (403, "{", 1, ControlReadFailure::Http { status: 403 }),
        (503, "{", 4, ControlReadFailure::Http { status: 503 }),
        (200, "{", 4, ControlReadFailure::Decode),
    ] {
        let mut response = server.mock(|when, then| {
            when.method(GET).path(format!("/jdev/sps/io/{ID}"));
            then.status(status).body(body);
        });
        let client = ControlReadClient::new(options(server.base_url())).unwrap();
        let error = client
            .read_control(ID, &Cancellation::new())
            .await
            .unwrap_err();
        assert_eq!(error.kind, kind);
        assert_eq!(client.active_operations(), 0);
        response.assert_hits(hits);
        client.close().await.unwrap();
        response.delete();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn strict_bytes_and_explicit_verbose_lossy_observer_keep_original_error_sources() {
    let server = MockServer::start();
    for body in [
        b"{".as_slice(),
        b"{\"LL\":{\"value\":\"synthetic-body-\xff\",\"Code\":200}}",
    ] {
        let mut response = server.mock(|when, then| {
            when.method(GET).path(format!("/jdev/sps/io/{ID}"));
            then.body(body);
        });
        let mut config = options(server.base_url());
        config.retry_delays.clear();
        let client = ControlReadClient::new(config.clone()).unwrap();
        let error = client
            .read_control(ID, &Cancellation::new())
            .await
            .unwrap_err();
        assert_eq!(error.kind, ControlReadFailure::Decode);
        assert!(!format!("{error:?} {error}").contains("synthetic-body"));
        assert!(std::error::Error::source(&error).is_none());
        assert!(error.into_source().unwrap().is::<reqwest::Error>());
        client.close().await.unwrap();

        let bodies = Arc::new(Mutex::new(Vec::new()));
        let capture = bodies.clone();
        config.response_policy = ControlReadResponsePolicy::LegacyCliVerbose {
            body_observer: Arc::new(move |text| capture.lock().unwrap().push(text.to_owned())),
        };
        let client = ControlReadClient::new(config).unwrap();
        let result = client.read_control(ID, &Cancellation::new()).await;
        if body == b"{" {
            let error = result.unwrap_err();
            assert_eq!(error.kind, ControlReadFailure::Decode);
            assert!(error.into_source().unwrap().is::<serde_json::Error>());
        } else {
            let state = result.unwrap().validate().unwrap();
            assert_eq!(state.value, "synthetic-body-\u{fffd}");
            assert!(!format!("{state:?}").contains("synthetic-body"));
        }
        assert_eq!(*bodies.lock().unwrap(), [String::from_utf8_lossy(body)]);
        client.close().await.unwrap();
        response.assert_hits(2);
        response.delete();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn host_only_redirect_profile_and_reqwest_cross_port_auth_stripping_are_retained() {
    let source = MockServer::start();
    let target = MockServer::start();
    let redirected = target.mock(|when, then| {
        when.method(GET).path("/state").matches(|request| {
            !request
                .headers
                .as_ref()
                .unwrap()
                .iter()
                .any(|(name, _)| name.eq_ignore_ascii_case("authorization"))
        });
        then.json_body(json!({"LL":{"value":"redirected","Code":200}}));
    });
    let mut redirect = source.mock(|when, then| {
        when.method(GET).path(format!("/jdev/sps/io/{ID}"));
        then.status(302)
            .header("location", format!("{}/state", target.base_url()));
    });
    let mut config = options(source.base_url());
    config.retry_delays.clear();
    let client = ControlReadClient::new(config.clone()).unwrap();
    assert_eq!(
        client
            .read_control(ID, &Cancellation::new())
            .await
            .unwrap()
            .validate()
            .unwrap()
            .value,
        "redirected"
    );
    client.close().await.unwrap();
    redirect.assert_hits(1);
    redirected.assert_hits(1);
    redirect.delete();
    let redirect = source.mock(|when, then| {
        when.method(GET).path(format!("/jdev/sps/io/{ID}"));
        then.status(302).header(
            "location",
            format!("http://localhost:{}/state", target.port()),
        );
    });
    let client = ControlReadClient::new(config).unwrap();
    let error = client
        .read_control(ID, &Cancellation::new())
        .await
        .unwrap_err();
    assert_eq!(
        error.kind,
        ControlReadFailure::Transport {
            kind: TransportFailure::Request
        }
    );
    assert!(
        error
            .into_source()
            .unwrap()
            .to_string()
            .contains("error following redirect")
    );
    redirected.assert_hits(1);
    redirect.assert_hits(1);
    client.close().await.unwrap();
}

struct SocketFixture {
    host: String,
    started: mpsc::Receiver<()>,
    ended: mpsc::Receiver<bool>,
    worker: thread::JoinHandle<()>,
}

fn headers(socket: &mut TcpStream) {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") && bytes.len() < 8192 {
        let mut byte = [0];
        socket.read_exact(&mut byte).unwrap();
        bytes.push(byte[0]);
    }
}

impl SocketFixture {
    fn new(response: Option<&'static str>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let host = format!("http://{}", listener.local_addr().unwrap());
        let (start_tx, started) = mpsc::sync_channel(1);
        let (end_tx, ended) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(error)
                        if error.kind() == std::io::ErrorKind::WouldBlock
                            && Instant::now() < deadline =>
                    {
                        thread::sleep(Duration::from_millis(2))
                    }
                    Err(error) => panic!("fixture accept: {error}"),
                }
            };
            socket.set_nonblocking(false).unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            headers(&mut socket);
            if let Some(response) = response {
                socket.write_all(response.as_bytes()).unwrap();
                socket.flush().unwrap();
            }
            start_tx.send(()).unwrap();
            let ended = match socket.read(&mut [0]) {
                Ok(0) => true,
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
                    ) =>
                {
                    true
                }
                _ => false,
            };
            end_tx.send(ended).unwrap();
        });
        Self {
            host,
            started,
            ended,
            worker,
        }
    }
}

async fn receive<T>(receiver: &mpsc::Receiver<T>) -> T {
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        match receiver.try_recv() {
            Ok(value) => return value,
            Err(mpsc::TryRecvError::Empty) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(2)).await
            }
            _ => panic!("fixture deadline/disconnected"),
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn cancellation_close_and_future_drop_actually_disconnect_pending_headers_and_body() {
    pending_disconnects(ControlReadCompatibility::Strict).await;
    pending_disconnects(ControlReadCompatibility::LegacyCli).await;
}

async fn pending_disconnects(compatibility: ControlReadCompatibility) {
    for body in [false, true] {
        for action in 0..3 {
            let fixture = SocketFixture::new(
                body.then_some("HTTP/1.1 200 OK\r\nContent-Length: 10000\r\n\r\n{\"LL\":"),
            );
            let mut config = options(fixture.host);
            config.compatibility = compatibility;
            let client = ControlReadClient::new(config).unwrap();
            let handle = client.clone();
            let cancellation = Cancellation::new();
            let cancel = cancellation.clone();
            let task = tokio::spawn(async move { handle.read_control(ID, &cancel).await });
            receive(&fixture.started).await;
            assert_eq!(client.active_operations(), 1);
            match action {
                0 => {
                    cancellation.cancel();
                    assert_eq!(
                        task.await.unwrap().unwrap_err().kind,
                        ControlReadFailure::Cancelled(CancelReason::Requested)
                    );
                }
                1 => {
                    client.close().await.unwrap();
                    assert_eq!(
                        task.await.unwrap().unwrap_err().kind,
                        ControlReadFailure::Cancelled(CancelReason::Shutdown)
                    );
                }
                _ => {
                    task.abort();
                    assert!(task.await.unwrap_err().is_cancelled());
                }
            }
            assert!(receive(&fixture.ended).await);
            assert_eq!(client.active_operations(), 0);
            client.close().await.unwrap();
            client.close().await.unwrap();
            assert_eq!(
                client
                    .read_control(ID, &Cancellation::new())
                    .await
                    .unwrap_err()
                    .kind,
                ControlReadFailure::Closed
            );
            fixture.worker.join().unwrap();
        }
    }
}

fn phased_response(header_ms: u64, body_ms: u64) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let host = format!("http://{}", listener.local_addr().unwrap());
    let worker = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        headers(&mut socket);
        thread::sleep(Duration::from_millis(header_ms));
        if socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 32\r\nConnection: close\r\n\r\n")
            .is_ok()
        {
            thread::sleep(Duration::from_millis(body_ms));
            let _ = socket.write_all(b"{\"LL\":{\"value\":\"ok\",\"Code\":200}}");
        }
    });
    (host, worker)
}

#[tokio::test(flavor = "current_thread")]
async fn legacy_phase_budgets_match_original_blocking_reference_not_a_total_deadline() {
    for (header_ms, body_ms, expired) in [
        (60, 60, None),
        (130, 0, Some(ControlReadPhase::Headers)),
        (0, 130, Some(ControlReadPhase::Body)),
    ] {
        let (host, worker) = phased_response(header_ms, body_ms);
        let reference = thread::spawn(move || {
            reqwest::blocking::Client::builder()
                .no_proxy()
                .timeout(Duration::from_millis(80))
                .build()
                .unwrap()
                .get(format!("{host}/jdev/sps/io/{ID}"))
                .send()?
                .json::<serde_json::Value>()
        })
        .join()
        .unwrap();
        worker.join().unwrap();
        assert_eq!(reference.is_err(), expired.is_some());
        if let Err(error) = reference {
            assert!(error.is_timeout());
            assert_eq!(
                error.is_request(),
                expired == Some(ControlReadPhase::Headers)
            );
            assert_eq!(error.is_decode(), expired == Some(ControlReadPhase::Body));
        }

        for verbose in [false, true] {
            let (host, worker) = phased_response(header_ms, body_ms);
            let mut config = options(host);
            config.compatibility = ControlReadCompatibility::LegacyCli;
            config.timeout = Duration::from_millis(80);
            config.retry_delays.clear();
            if verbose {
                config.response_policy = ControlReadResponsePolicy::LegacyCliVerbose {
                    body_observer: Arc::new(|_| {}),
                };
            }
            let client = ControlReadClient::new(config).unwrap();
            let result = client.read_control(ID, &Cancellation::new()).await;
            match expired {
                None => assert_eq!(result.unwrap().validate().unwrap().value, "ok"),
                Some(phase) => {
                    let error = result.unwrap_err();
                    assert_eq!(
                        error.kind,
                        ControlReadFailure::Transport {
                            kind: TransportFailure::Timeout
                        }
                    );
                    assert!(std::error::Error::source(&error).is_none());
                    let timeout = error
                        .into_source()
                        .unwrap()
                        .downcast::<ControlReadPhaseTimeout>()
                        .unwrap();
                    assert_eq!(timeout.phase(), phase);
                    assert_eq!(timeout.to_string(), "operation timed out");
                    assert!(!format!("{timeout:?}").contains("127.0.0.1"));
                }
            }
            assert_eq!(client.active_operations(), 0);
            client.close().await.unwrap();
            worker.join().unwrap();
        }
    }

    let (host, worker) = phased_response(60, 60);
    let mut config = options(host);
    config.timeout = Duration::from_millis(80);
    config.retry_delays.clear();
    let strict = ControlReadClient::new(config).unwrap();
    assert_eq!(
        strict
            .read_control(ID, &Cancellation::new())
            .await
            .unwrap_err()
            .kind,
        ControlReadFailure::Transport {
            kind: TransportFailure::Timeout
        }
    );
    strict.close().await.unwrap();
    worker.join().unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn precancellation_and_retry_wait_cancellation_do_not_issue_another_request() {
    let server = MockServer::start();
    let response = server.mock(|when, then| {
        when.method(GET).path(format!("/jdev/sps/io/{ID}"));
        then.status(503);
    });
    let mut config = options(server.base_url());
    config.retry_delays = vec![Duration::from_secs(30)];
    let client = ControlReadClient::new(config).unwrap();
    let cancellation = Cancellation::new();
    cancellation.cancel();
    assert_eq!(
        client
            .read_control(ID, &cancellation)
            .await
            .unwrap_err()
            .kind,
        ControlReadFailure::Cancelled(CancelReason::Requested)
    );
    response.assert_hits(0);
    let cancellation = Cancellation::new();
    let cancel = cancellation.clone();
    let handle = client.clone();
    let task = tokio::spawn(async move { handle.read_control(ID, &cancel).await });
    let deadline = Instant::now() + Duration::from_secs(3);
    while response.hits() == 0 && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
    response.assert_hits(1);
    cancellation.cancel();
    assert_eq!(
        task.await.unwrap().unwrap_err().kind,
        ControlReadFailure::Cancelled(CancelReason::Requested)
    );
    response.assert_hits(1);
    client.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn close_releases_successful_pooled_socket_and_timeout_is_typed() {
    pooled_close_and_timeout(ControlReadCompatibility::Strict).await;
    pooled_close_and_timeout(ControlReadCompatibility::LegacyCli).await;
}

async fn pooled_close_and_timeout(compatibility: ControlReadCompatibility) {
    let fixture = SocketFixture::new(Some(
        "HTTP/1.1 200 OK\r\nContent-Length: 32\r\n\r\n{\"LL\":{\"value\":\"ok\",\"Code\":200}}",
    ));
    let mut config = options(fixture.host);
    config.compatibility = compatibility;
    let client = ControlReadClient::new(config).unwrap();
    client.read_control(ID, &Cancellation::new()).await.unwrap();
    client.close().await.unwrap();
    assert!(receive(&fixture.ended).await);
    fixture.worker.join().unwrap();

    for response in [
        None,
        Some("HTTP/1.1 200 OK\r\nContent-Length: 10000\r\n\r\n{\"LL\":"),
    ] {
        let fixture = SocketFixture::new(response);
        let mut config = options(fixture.host);
        config.compatibility = compatibility;
        config.timeout = Duration::from_millis(30);
        config.retry_delays.clear();
        let client = ControlReadClient::new(config).unwrap();
        assert_eq!(
            client
                .read_control(ID, &Cancellation::new())
                .await
                .unwrap_err()
                .kind,
            ControlReadFailure::Transport {
                kind: TransportFailure::Timeout
            }
        );
        assert!(receive(&fixture.ended).await);
        client.close().await.unwrap();
        fixture.worker.join().unwrap();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn retained_future_reports_bounded_shutdown_failure_and_can_then_drain() {
    let server = MockServer::start();
    let mut config = options(server.base_url());
    config.shutdown_timeout = Duration::from_millis(20);
    let client = ControlReadClient::new(config).unwrap();
    let cancellation = Cancellation::new();
    let mut pending = Box::pin(client.read_control(ID, &cancellation));
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    assert!(std::future::Future::poll(pending.as_mut(), &mut context).is_pending());
    assert_eq!(client.active_operations(), 1);
    assert_eq!(
        client.close().await.unwrap_err().kind,
        ControlReadFailure::ShutdownTimeout { active: 1 }
    );
    drop(pending);
    assert_eq!(client.active_operations(), 0);
    client.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn default_debug_and_diagnostics_do_not_disclose_credentials_or_control_payloads() {
    let server = MockServer::start();
    let response = server.mock(|when, then| {
        when.method(GET).path(format!("/jdev/sps/io/{ID}"));
        then.json_body(json!({"LL":{"value":"private-control-payload","Code":"private-code"}}));
    });
    let mut config = options(server.base_url());
    config.user = "private-user".into();
    config.password = "private-password".into();
    config.user_agent = "private-agent".into();
    assert!(!format!("{config:?}").contains("private-"));
    let events = Arc::new(Mutex::new(Vec::new()));
    let capture = events.clone();
    config.diagnostics = Some(Arc::new(move |event| capture.lock().unwrap().push(event)));
    let client = ControlReadClient::new(config).unwrap();
    let observation = client.read_control(ID, &Cancellation::new()).await.unwrap();
    assert!(
        !format!(
            "{observation:?} {:?} {:?}",
            observation.legacy_text(),
            events.lock().unwrap()
        )
        .contains("private-")
    );
    response.assert_hits(1);
    client.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn truncated_transfer_is_body_transport_not_a_successful_empty_state_or_json_error() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let host = format!("http://{}", listener.local_addr().unwrap());
    let worker = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        socket
            .set_read_timeout(Some(Duration::from_secs(3)))
            .unwrap();
        headers(&mut socket);
        socket
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\nConnection: close\r\n\r\n{\"LL\":",
            )
            .unwrap();
    });
    let mut config = options(host);
    config.retry_delays.clear();
    let client = ControlReadClient::new(config).unwrap();
    let error = client
        .read_control(ID, &Cancellation::new())
        .await
        .unwrap_err();
    assert_eq!(
        error.kind,
        ControlReadFailure::Transport {
            kind: TransportFailure::Body
        }
    );
    assert!(error.into_source().unwrap().is::<reqwest::Error>());
    assert_eq!(client.active_operations(), 0);
    client.close().await.unwrap();
    worker.join().unwrap();
}

#[cfg(feature = "runtime-blocking")]
#[tokio::test(flavor = "current_thread")]
async fn owned_driver_busy_close_cancellation_and_drop_join_and_release_real_sockets() {
    owned_sockets(ControlReadCompatibility::Strict).await;
    owned_sockets(ControlReadCompatibility::LegacyCli).await;
}

#[cfg(feature = "runtime-blocking")]
async fn owned_sockets(compatibility: ControlReadCompatibility) {
    let fixture = SocketFixture::new(None);
    let mut config = options(fixture.host);
    config.compatibility = compatibility;
    let driver = Arc::new(ControlReadDriver::new(config).unwrap());
    let handle = driver.clone();
    let task = thread::spawn(move || handle.read_control(ID, &Cancellation::new()));
    receive(&fixture.started).await;
    assert_eq!(driver.active_operations(), 1);
    assert_eq!(
        driver
            .read_control(ID, &Cancellation::new())
            .unwrap_err()
            .kind,
        ControlReadFailure::Busy
    );
    driver.close().unwrap();
    assert_eq!(
        task.join().unwrap().unwrap_err().kind,
        ControlReadFailure::Cancelled(CancelReason::Shutdown)
    );
    assert!(receive(&fixture.ended).await);
    assert_eq!(driver.active_operations(), 0);
    driver.close().unwrap();
    assert_eq!(
        driver
            .read_control(ID, &Cancellation::new())
            .unwrap_err()
            .kind,
        ControlReadFailure::Closed
    );
    fixture.worker.join().unwrap();

    let fixture = SocketFixture::new(Some(
        "HTTP/1.1 200 OK\r\nContent-Length: 32\r\n\r\n{\"LL\":{\"value\":\"ok\",\"Code\":200}}",
    ));
    let mut config = options(fixture.host);
    config.compatibility = compatibility;
    let driver = ControlReadDriver::new(config).unwrap();
    driver
        .read_control(ID, &Cancellation::new())
        .unwrap()
        .validate()
        .unwrap();
    drop(driver);
    assert!(receive(&fixture.ended).await);
    fixture.worker.join().unwrap();
}
