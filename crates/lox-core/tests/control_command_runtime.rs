#![cfg(feature = "runtime")]

use httpmock::{Method::GET, MockServer};
use lox_core::runtime::*;
use serde_json::json;
use std::time::Duration;

fn options(host: String) -> ControlReadOptions {
    let mut options = ControlReadOptions::new(host, "command", "synthetic-only");
    options.use_system_proxy = false;
    options.retry_delays = vec![Duration::from_millis(1); 3];
    options
}

#[tokio::test(flavor = "current_thread")]
async fn legacy_command_preserves_literal_path_and_its_distinct_ll_projection() {
    let server = MockServer::start();
    let response = server.mock(|when, then| {
        when.method(GET)
            .path("/prefix//jdev/sps/io/abcdef/on/child")
            .query_param("flag", "1")
            .header("authorization", "Basic Y29tbWFuZDpzeW50aGV0aWMtb25seQ==");
        then.json_body(json!({"LL":{"value":-0.0,"Code":401,"code":"odd"}}));
    });
    let client =
        LegacyControlCommandClient::new(options(format!("{}/prefix/", server.base_url()))).unwrap();
    let command = LegacyControlCommand::new("abcdef", "on/child?flag=1#fragment").unwrap();
    let observation = client
        .send_command(&command, &Cancellation::new())
        .await
        .unwrap();
    assert_eq!(observation.legacy_text().value, "-0");
    assert_eq!(observation.legacy_text().code.as_deref(), Some("odd"));
    assert_eq!(
        observation.validate(),
        Err(ControlValidationIssue::RejectedCode)
    );
    assert!(!format!("{command:?} {observation:?}").contains("flag"));
    response.assert_hits(1);
    client.close().await.unwrap();
    client.close().await.unwrap();
    assert_eq!(client.active_operations(), 0);
    assert_eq!(
        client
            .send_command(&command, &Cancellation::new())
            .await
            .unwrap_err()
            .kind,
        ControlReadFailure::Closed
    );
}

#[tokio::test(flavor = "current_thread")]
async fn legacy_mutating_retries_are_explicit_and_never_imply_exactly_once() {
    let server = MockServer::start();
    for (status, body, hits, kind) in [
        (401, "{", 1, ControlReadFailure::Http { status: 401 }),
        (503, "{", 4, ControlReadFailure::Http { status: 503 }),
        (200, "{", 4, ControlReadFailure::Decode),
    ] {
        let mut response = server.mock(|when, then| {
            when.method(GET).path("/jdev/sps/io/ab/pulse");
            then.status(status).body(body);
        });
        let client = LegacyControlCommandClient::new(options(server.base_url())).unwrap();
        let command = LegacyControlCommand::new("ab", "pulse").unwrap();
        let error = client
            .send_command(&command, &Cancellation::new())
            .await
            .unwrap_err();
        assert_eq!(error.kind, kind);
        assert!(std::error::Error::source(&error).is_none());
        if status == 200 {
            assert!(error.into_source().unwrap().is::<reqwest::Error>());
        }
        response.assert_hits(hits);
        client.close().await.unwrap();
        response.delete();
    }
}

#[cfg(feature = "runtime-blocking")]
#[test]
fn owned_driver_precancellation_sends_nothing_and_close_joins_repeatedly() {
    let server = MockServer::start();
    let response = server.mock(|when, then| {
        when.any_request();
        then.json_body(json!({"LL":{"value":"ok","Code":200}}));
    });
    for invalid in ["", "a/b", "a%20", "a..b", "xyz"] {
        assert!(LegacyControlCommand::new(invalid, "pulse").is_err());
    }
    let command = LegacyControlCommand::new("ab", "pulse").unwrap();
    for _ in 0..3 {
        let driver = LegacyControlCommandDriver::new(options(server.base_url())).unwrap();
        let cancellation = Cancellation::new();
        cancellation.cancel();
        assert_eq!(
            driver
                .send_command(&command, &cancellation)
                .unwrap_err()
                .kind,
            ControlReadFailure::Cancelled(CancelReason::Requested)
        );
        assert_eq!(driver.active_operations(), 0);
        driver.close().unwrap();
        driver.close().unwrap();
        assert!(driver.is_closed());
        assert_eq!(
            driver
                .send_command(&command, &Cancellation::new())
                .unwrap_err()
                .kind,
            ControlReadFailure::Closed
        );
    }
    response.assert_hits(0);
}

#[cfg(feature = "runtime-blocking")]
#[test]
fn owned_command_worker_cancels_pending_headers_and_body_and_releases_real_sockets() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, mpsc};
    use std::thread;

    for body_phase in [false, true] {
        for close in [false, true] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let host = format!("http://{}", listener.local_addr().unwrap());
            let (started_tx, started_rx) = mpsc::sync_channel(1);
            let fixture = thread::spawn(move || {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut headers = Vec::new();
                while !headers.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    socket.read_exact(&mut byte).unwrap();
                    headers.push(byte[0]);
                }
                assert!(
                    String::from_utf8_lossy(&headers).starts_with("GET /jdev/sps/io/ab/pulse ")
                );
                if body_phase {
                    socket
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 10000\r\n\r\n{\"LL\":")
                        .unwrap();
                    socket.flush().unwrap();
                }
                started_tx.send(()).unwrap();
                match socket.read(&mut [0]) {
                    Ok(0) => {}
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::ConnectionReset
                                | std::io::ErrorKind::ConnectionAborted
                        ) => {}
                    result => panic!("expected socket release, got {result:?}"),
                }
            });
            let mut config = options(host);
            config.timeout = Duration::from_secs(10);
            config.shutdown_timeout = Duration::from_secs(2);
            let driver = Arc::new(LegacyControlCommandDriver::new(config).unwrap());
            let command = LegacyControlCommand::new("ab", "pulse").unwrap();
            let cancellation = Cancellation::new();
            let handle = driver.clone();
            let request = command.clone();
            let cancel = cancellation.clone();
            let operation = thread::spawn(move || handle.send_command(&request, &cancel));
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();
            assert_eq!(driver.active_operations(), 1);
            assert_eq!(
                driver
                    .send_command(&command, &Cancellation::new())
                    .unwrap_err()
                    .kind,
                ControlReadFailure::Busy
            );
            let reason = if close {
                driver.close().unwrap();
                CancelReason::Shutdown
            } else {
                cancellation.cancel();
                CancelReason::Requested
            };
            assert_eq!(
                operation.join().unwrap().unwrap_err().kind,
                ControlReadFailure::Cancelled(reason)
            );
            fixture.join().unwrap();
            assert_eq!(driver.active_operations(), 0);
            driver.close().unwrap();
            driver.close().unwrap();
        }
    }
}
