#![cfg(feature = "runtime")]

use httpmock::{Method::GET, MockServer};
use lox_core::runtime::*;
use serde_json::json;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

fn options(host: String) -> ClockOptions {
    assert!(
        reqwest::Url::parse(&host)
            .unwrap()
            .host_str()
            .unwrap()
            .parse::<std::net::IpAddr>()
            .unwrap()
            .is_loopback()
    );
    let mut options = ClockOptions::new(host, "clock-test", "synthetic-only");
    options.use_system_proxy = false;
    options.timeout = Duration::from_secs(2);
    options.shutdown_timeout = Duration::from_secs(2);
    options
}

#[tokio::test(flavor = "current_thread")]
async fn typed_observations_validate_and_preserve_explicit_legacy_policy() {
    let server = MockServer::start();
    for (wire, value, envelope, issue, legacy) in [
        (
            json!({}),
            ClockValue::Missing,
            EnvelopeState::Missing,
            ClockValidationIssue::Envelope,
            "",
        ),
        (
            json!({"LL":null}),
            ClockValue::Missing,
            EnvelopeState::Unsupported(JsonKind::Null),
            ClockValidationIssue::Envelope,
            "",
        ),
        (
            json!({"LL":{"Code":200}}),
            ClockValue::Missing,
            EnvelopeState::Present,
            ClockValidationIssue::MissingValue,
            "",
        ),
        (
            json!({"LL":{"value":true,"Code":200}}),
            ClockValue::Unsupported(JsonKind::Boolean),
            EnvelopeState::Present,
            ClockValidationIssue::UnsupportedValue,
            "",
        ),
        (
            json!({"LL":{"value":null,"Code":200}}),
            ClockValue::Unsupported(JsonKind::Null),
            EnvelopeState::Present,
            ClockValidationIssue::UnsupportedValue,
            "",
        ),
        (
            json!({"LL":{"value":[],"Code":200}}),
            ClockValue::Unsupported(JsonKind::Array),
            EnvelopeState::Present,
            ClockValidationIssue::UnsupportedValue,
            "",
        ),
        (
            json!({"LL":{"value":"","Code":200}}),
            ClockValue::Text("".into()),
            EnvelopeState::Present,
            ClockValidationIssue::EmptyValue,
            "",
        ),
        (
            json!({"LL":{"value":"kept","Code":"401"}}),
            ClockValue::Text("kept".into()),
            EnvelopeState::Present,
            ClockValidationIssue::RejectedCode,
            "kept",
        ),
        (
            json!({"LL":{"value":"kept"}}),
            ClockValue::Text("kept".into()),
            EnvelopeState::Present,
            ClockValidationIssue::MissingCode,
            "kept",
        ),
        (
            json!({"LL":{"value":1.5,"Code":false,"code":"200"}}),
            ClockValue::Number(1.5),
            EnvelopeState::Present,
            ClockValidationIssue::UnsupportedCode,
            "1.5",
        ),
    ] {
        let mut date = server.mock(|when, then| {
            when.method(GET).path(ClockEndpoint::Date.path());
            then.json_body(wire);
        });
        let mut time = server.mock(|when, then| {
            when.method(GET).path(ClockEndpoint::Time.path());
            then.json_body(json!({"LL":{"value":42,"code":"200"}}));
        });
        let client = ClockClient::new(options(server.base_url())).unwrap();
        let observation = client.read_clock(&Cancellation::new()).await.unwrap();
        assert_eq!(observation.date.envelope, envelope);
        assert_eq!(observation.date.value, value);
        assert_eq!(
            observation.validate(),
            Err(ClockValidationError {
                endpoint: ClockEndpoint::Date,
                issue
            })
        );
        assert_eq!(observation.legacy_text().date, legacy);
        assert_eq!(observation.legacy_text().time, "42");
        date.assert_hits(1);
        time.assert_hits(1);
        client.close().await.unwrap();
        date.delete();
        time.delete();
    }
}

#[tokio::test(flavor = "current_thread")]
async fn ordered_success_uses_basic_auth_and_allows_empty_credentials() {
    let server = MockServer::start();
    let events = Arc::new(Mutex::new(Vec::new()));
    let mut config = options(server.base_url());
    config.user.clear();
    config.password.clear();
    config.user_agent = "clock-core-test".into();
    let capture = events.clone();
    config.diagnostics = Some(Arc::new(move |event| capture.lock().unwrap().push(event)));
    let date = server.mock(|when, then| {
        when.method(GET)
            .path(ClockEndpoint::Date.path())
            .header("authorization", "Basic Og==")
            .header("user-agent", "clock-core-test");
        then.json_body(json!({"LL":{"value":"2026-11-14","Code":"200"}}));
    });
    let time = server.mock(|when, then| {
        when.method(GET)
            .path(ClockEndpoint::Time.path())
            .header("authorization", "Basic Og==");
        then.json_body(json!({"LL":{"value":56.25,"Code":200}}));
    });
    let client = ClockClient::new(config).unwrap();
    let observation = client.read_clock(&Cancellation::new()).await.unwrap();
    assert_eq!(
        observation.validate().unwrap(),
        ClockText {
            date: "2026-11-14".into(),
            time: "56.25".into()
        }
    );
    let events = events.lock().unwrap().clone();
    let order: Vec<_> = events
        .iter()
        .filter_map(|event| match event {
            Diagnostic::Request(endpoint) => Some(*endpoint),
            _ => None,
        })
        .collect();
    assert_eq!(order, [ClockEndpoint::Date, ClockEndpoint::Time]);
    date.assert_hits(1);
    time.assert_hits(1);
    client.close().await.unwrap();
    client.close().await.unwrap();
    assert_eq!(
        client.read_clock(&Cancellation::new()).await,
        Err(ClockError::Closed)
    );
}

#[tokio::test(flavor = "current_thread")]
async fn typed_http_and_decode_failures_retry_and_stop_at_the_failing_endpoint() {
    for (endpoint, status, body, hits) in [
        (ClockEndpoint::Date, 401, "denied", 1),
        (ClockEndpoint::Date, 503, "unavailable", 4),
        (ClockEndpoint::Time, 200, "{", 4),
    ] {
        let server = MockServer::start();
        let failure = server.mock(|when, then| {
            when.method(GET).path(endpoint.path());
            then.status(status).body(body);
        });
        let other_endpoint = if endpoint == ClockEndpoint::Date {
            ClockEndpoint::Time
        } else {
            ClockEndpoint::Date
        };
        let other = server.mock(|when, then| {
            when.method(GET).path(other_endpoint.path());
            then.json_body(json!({"LL":{"value":"ok","Code":200}}));
        });
        let mut config = options(server.base_url());
        config.retry_delays = vec![Duration::from_millis(1); 3];
        let client = ClockClient::new(config).unwrap();
        let error = client.read_clock(&Cancellation::new()).await.unwrap_err();
        assert_eq!(
            error,
            if status == 200 {
                ClockError::Decode { endpoint }
            } else {
                ClockError::Http { endpoint, status }
            }
        );
        failure.assert_hits(hits);
        other.assert_hits(if endpoint == ClockEndpoint::Date {
            0
        } else {
            1
        });
        assert_eq!(client.active_operations(), 0);
        client.close().await.unwrap();
    }
}

/// A real loopback socket that withholds headers, then proves client disconnect.
/// Optional date response forces cancellation/timeout specifically during time.
struct StalledClock {
    host: String,
    started: mpsc::Receiver<()>,
    disconnected: mpsc::Receiver<bool>,
    paths: Arc<Mutex<Vec<String>>>,
    worker: Option<thread::JoinHandle<()>>,
}

impl StalledClock {
    fn new(stall_time: bool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let host = format!("http://{}", listener.local_addr().unwrap());
        let (start_tx, started) = mpsc::sync_channel(1);
        let (end_tx, disconnected) = mpsc::sync_channel(1);
        let paths = Arc::new(Mutex::new(Vec::new()));
        let capture = paths.clone();
        let worker = thread::spawn(move || {
            let accept = || {
                let deadline = Instant::now() + Duration::from_secs(5);
                loop {
                    match listener.accept() {
                        Ok((socket, _)) => return socket,
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && Instant::now() < deadline =>
                        {
                            thread::sleep(Duration::from_millis(2))
                        }
                        Err(error) => panic!("loopback accept failed: {error}"),
                    }
                }
            };
            for index in 0..=usize::from(stall_time) {
                let mut socket = accept();
                socket.set_nonblocking(false).unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let request = read_headers(&mut socket);
                capture
                    .lock()
                    .unwrap()
                    .push(request.lines().next().unwrap().to_string());
                if stall_time && index == 0 {
                    let body = r#"{"LL":{"value":"date","Code":200}}"#;
                    write!(
                        socket,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    )
                    .unwrap();
                } else {
                    start_tx.send(()).unwrap();
                    let mut byte = [0];
                    let eof = match socket.read(&mut byte) {
                        Ok(0) => true,
                        Err(error)
                            if matches!(
                                error.kind(),
                                std::io::ErrorKind::ConnectionReset
                                    | std::io::ErrorKind::ConnectionAborted
                            ) =>
                        {
                            true
                        }
                        _ => false,
                    };
                    let _ = end_tx.send(eof);
                }
            }
        });
        Self {
            host,
            started,
            disconnected,
            paths,
            worker: Some(worker),
        }
    }

    fn assert_disconnected(&mut self, stall_time: bool) {
        assert!(
            self.disconnected
                .recv_timeout(Duration::from_secs(5))
                .unwrap()
        );
        self.finish(stall_time);
    }

    async fn assert_disconnected_async(&mut self, stall_time: bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match self.disconnected.try_recv() {
                Ok(eof) => {
                    assert!(eof);
                    break;
                }
                Err(mpsc::TryRecvError::Empty) if Instant::now() < deadline => {
                    tokio::time::sleep(Duration::from_millis(2)).await
                }
                result => panic!("socket did not disconnect: {result:?}"),
            }
        }
        self.finish(stall_time);
    }

    fn finish(&mut self, stall_time: bool) {
        self.worker.take().unwrap().join().unwrap();
        assert_eq!(
            *self.paths.lock().unwrap(),
            if stall_time {
                vec!["GET /jdev/sys/date HTTP/1.1", "GET /jdev/sys/time HTTP/1.1"]
            } else {
                vec!["GET /jdev/sys/date HTTP/1.1"]
            }
        );
    }
}

fn read_headers(socket: &mut TcpStream) -> String {
    let mut request = Vec::new();
    while !request.ends_with(b"\r\n\r\n") {
        let mut byte = [0];
        socket.read_exact(&mut byte).unwrap();
        request.push(byte[0]);
        assert!(request.len() < 8192);
    }
    String::from_utf8(request).unwrap()
}

async fn wait_started(server: &StalledClock) {
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        match server.started.try_recv() {
            Ok(()) => return,
            Err(mpsc::TryRecvError::Empty) if Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(2)).await
            }
            result => panic!("request did not start: {result:?}"),
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn caller_cancellation_and_shutdown_abort_real_sockets_without_later_requests() {
    for shutdown in [false, true] {
        let mut server = StalledClock::new(!shutdown);
        let client = ClockClient::new(options(server.host.clone())).unwrap();
        let token = Cancellation::new();
        let handle = client.clone();
        let cancel = token.clone();
        let task = tokio::spawn(async move { handle.read_clock(&cancel).await });
        wait_started(&server).await;
        assert_eq!(client.active_operations(), 1);
        let start = Instant::now();
        if shutdown {
            client.close().await.unwrap()
        } else {
            token.cancel()
        };
        assert_eq!(
            task.await.unwrap(),
            Err(ClockError::Cancelled(if shutdown {
                CancelReason::Shutdown
            } else {
                CancelReason::Requested
            }))
        );
        assert!(start.elapsed() < Duration::from_secs(2));
        assert_eq!(client.active_operations(), 0);
        client.close().await.unwrap();
        server.assert_disconnected_async(!shutdown).await;
    }
}

#[tokio::test(flavor = "current_thread")]
async fn timeout_is_typed_and_drops_the_real_socket() {
    let mut server = StalledClock::new(false);
    let mut config = options(server.host.clone());
    config.timeout = Duration::from_millis(100);
    config.retry_delays.clear();
    let client = ClockClient::new(config).unwrap();
    let start = Instant::now();
    assert_eq!(
        client.read_clock(&Cancellation::new()).await,
        Err(ClockError::Transport {
            endpoint: ClockEndpoint::Date,
            kind: TransportFailure::Timeout,
        })
    );
    assert!(start.elapsed() < Duration::from_secs(2));
    assert_eq!(client.active_operations(), 0);
    client.close().await.unwrap();
    server.assert_disconnected_async(false).await;
}

#[tokio::test(flavor = "current_thread")]
async fn precancel_and_cancel_during_retry_delay_make_no_extra_requests() {
    let server = MockServer::start();
    let date = server.mock(|when, then| {
        when.method(GET).path(ClockEndpoint::Date.path());
        then.status(503);
    });
    let mut config = options(server.base_url());
    config.retry_delays = vec![Duration::from_secs(10); 3];
    let (responded, mut response) = tokio::sync::mpsc::unbounded_channel();
    config.diagnostics = Some(Arc::new(move |event| {
        if matches!(event, Diagnostic::Response { .. }) {
            let _ = responded.send(());
        }
    }));
    let client = ClockClient::new(config).unwrap();
    let token = Cancellation::new();
    token.cancel();
    assert_eq!(
        client.read_clock(&token).await,
        Err(ClockError::Cancelled(CancelReason::Requested))
    );
    date.assert_hits(0);
    let token = Cancellation::new();
    let handle = client.clone();
    let cancel = token.clone();
    let task = tokio::spawn(async move { handle.read_clock(&cancel).await });
    tokio::time::timeout(Duration::from_secs(3), response.recv())
        .await
        .unwrap()
        .unwrap();
    token.cancel();
    assert_eq!(
        task.await.unwrap(),
        Err(ClockError::Cancelled(CancelReason::Requested))
    );
    date.assert_hits(1);
    client.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn retained_unpolled_future_reports_bounded_drain_timeout_then_recovers() {
    let server = MockServer::start();
    let mut config = options(server.base_url());
    config.shutdown_timeout = Duration::from_millis(20);
    let client = ClockClient::new(config).unwrap();
    let token = Cancellation::new();
    let mut pending = Box::pin(client.read_clock(&token));
    let waker = std::task::Waker::noop();
    let mut context = std::task::Context::from_waker(waker);
    assert!(std::future::Future::poll(pending.as_mut(), &mut context).is_pending());
    assert_eq!(client.active_operations(), 1);
    assert_eq!(
        client.close().await,
        Err(ClockError::ShutdownTimeout { active: 1 })
    );
    drop(pending);
    assert_eq!(client.active_operations(), 0);
    client.close().await.unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn secrets_are_not_in_debug_errors_or_opt_in_diagnostics() {
    let server = MockServer::start();
    let secret = "private-test-password";
    let failure = server.mock(|when, then| {
        when.method(GET).path(ClockEndpoint::Date.path());
        then.status(401).body("private-test-password");
    });
    let mut config = options(server.base_url());
    config.password = secret.into();
    config.user = "private-test-user".into();
    let debug = format!("{config:?}");
    assert!(!debug.contains(secret));
    assert!(!debug.contains("private-test-user"));
    let events = Arc::new(Mutex::new(Vec::new()));
    let capture = events.clone();
    config.diagnostics = Some(Arc::new(move |event| capture.lock().unwrap().push(event)));
    let client = ClockClient::new(config).unwrap();
    let error = client.read_clock(&Cancellation::new()).await.unwrap_err();
    assert!(!format!("{error} {error:?} {:?}", events.lock().unwrap()).contains(secret));
    assert!(std::error::Error::source(&error).is_none());
    failure.assert_hits(1);
    client.close().await.unwrap();
}

#[cfg(feature = "runtime-blocking")]
#[tokio::test(flavor = "current_thread")]
async fn owned_driver_works_and_disposes_under_an_existing_executor() {
    let server = MockServer::start();
    let date = server.mock(|when, then| {
        when.method(GET).path(ClockEndpoint::Date.path());
        then.json_body(json!({"LL":{"value":"date","Code":200}}));
    });
    let time = server.mock(|when, then| {
        when.method(GET).path(ClockEndpoint::Time.path());
        then.json_body(json!({"LL":{"value":"time","Code":200}}));
    });
    let driver = ClockDriver::new(options(server.base_url())).unwrap();
    assert_eq!(
        driver
            .read_clock(&Cancellation::new())
            .unwrap()
            .validate()
            .unwrap(),
        ClockText {
            date: "date".into(),
            time: "time".into()
        }
    );
    driver.close().unwrap();
    driver.close().unwrap();
    assert_eq!(
        driver.read_clock(&Cancellation::new()),
        Err(ClockError::Closed)
    );
    date.assert_hits(1);
    time.assert_hits(1);
    drop(driver);
}

#[cfg(feature = "runtime-blocking")]
#[test]
fn owned_driver_cancellation_and_concurrent_close_join_the_worker_and_disconnect() {
    for shutdown in [false, true] {
        let mut server = StalledClock::new(false);
        let driver = Arc::new(ClockDriver::new(options(server.host.clone())).unwrap());
        let handle = driver.clone();
        let token = Cancellation::new();
        let cancel = token.clone();
        let operation = thread::spawn(move || handle.read_clock(&cancel));
        server.started.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(
            driver.read_clock(&Cancellation::new()),
            Err(ClockError::Busy)
        );
        let start = Instant::now();
        if shutdown {
            driver.close().unwrap()
        } else {
            token.cancel()
        };
        assert_eq!(
            operation.join().unwrap(),
            Err(ClockError::Cancelled(if shutdown {
                CancelReason::Shutdown
            } else {
                CancelReason::Requested
            }))
        );
        driver.close().unwrap();
        driver.close().unwrap();
        assert!(driver.is_closed());
        assert_eq!(driver.active_operations(), 0);
        assert!(start.elapsed() < Duration::from_secs(2));
        server.assert_disconnected(false);
    }
}
