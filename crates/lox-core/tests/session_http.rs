#![cfg(feature = "runtime")]

use lox_core::runtime::TransportFailure;
use lox_core::runtime::session_endpoints::{ProxyPolicy, SessionEndpoints};
use lox_core::runtime::session_http::{SessionHttpClient, SessionHttpError};
use reqwest::StatusCode;
use std::error::Error;
use std::io::ErrorKind;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

const LOOPBACK: &str = "127.0.0.1:0";
const FIXTURE_DEADLINE: Duration = Duration::from_secs(5);

async fn exchange(
    origin: TcpListener,
    path: &str,
    reply: &str,
) -> (Result<StatusCode, SessionHttpError>, String) {
    let address = origin.local_addr().unwrap();
    let endpoints = SessionEndpoints::parse(
        &format!("http://{address}{path}"),
        &format!("ws://{address}/not-the-http-route"),
        ProxyPolicy::Direct,
    )
    .unwrap();
    let client = SessionHttpClient::new(&endpoints).unwrap();
    let (finished, request_done) = oneshot::channel();
    let get_status = async {
        let result = client.get_status().await;
        finished.send(()).unwrap();
        result
    };
    let serve = async {
        let (socket, _) = origin.accept().await.unwrap();
        let mut headers = Vec::new();
        let mut buffer = [0; 1024];
        while !headers.ends_with(b"\r\n\r\n") {
            socket.readable().await.unwrap();
            match socket.try_read(&mut buffer) {
                Ok(0) => panic!("request ended before the header terminator"),
                Ok(read) => headers.extend_from_slice(&buffer[..read]),
                Err(error) if error.kind() == ErrorKind::WouldBlock => continue,
                Err(error) => panic!("fixture read: {error}"),
            }
            assert!(headers.len() < 8192);
        }
        let mut reply = reply.as_bytes();
        while !reply.is_empty() {
            socket.writable().await.unwrap();
            match socket.try_write(reply) {
                Ok(0) => panic!("fixture could not write the response"),
                Ok(written) => reply = &reply[written..],
                Err(error) if error.kind() == ErrorKind::WouldBlock => continue,
                Err(error) => panic!("fixture write: {error}"),
            }
        }
        drop(socket);
        tokio::select! {
            biased;
            _ = origin.accept() => panic!("a second connection to the HTTP origin was attempted"),
            done = request_done => done.unwrap(),
        }
        String::from_utf8(headers).unwrap()
    };
    tokio::time::timeout(FIXTURE_DEADLINE, async { tokio::join!(get_status, serve) })
        .await
        .expect("scripted HTTP exchange did not finish")
}

fn redirect_reply(location: &str) -> String {
    format!(
        "HTTP/1.1 302 Found\r\nLocation: {location}\r\n\
         Content-Length: 0\r\nConnection: close\r\n\r\n"
    )
}

#[tokio::test(flavor = "current_thread")]
async fn gets_the_exact_admitted_path_and_port_without_credentials() {
    let origin = TcpListener::bind(LOOPBACK).await.unwrap();
    let address = origin.local_addr().unwrap();
    let (result, request) = exchange(
        origin,
        "/Site/v1/~status/",
        "HTTP/1.1 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"fixture\"\r\n\
         Content-Length: 0\r\nConnection: close\r\n\r\n",
    )
    .await;

    assert_eq!(result, Ok(StatusCode::UNAUTHORIZED));
    assert_eq!(
        request.lines().next(),
        Some("GET /Site/v1/~status/ HTTP/1.1")
    );
    let host = format!("host: {address}");
    assert!(request.lines().any(|line| line.eq_ignore_ascii_case(&host)));
    let headers = request.to_ascii_lowercase();
    for name in ["authorization:", "proxy-authorization:", "cookie:"] {
        assert!(!headers.lines().any(|line| line.starts_with(name)));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn reports_redirect_status_without_following_same_origin_or_same_host_other_port() {
    let sink = TcpListener::bind(LOOPBACK).await.unwrap();
    let other_port = format!("http://{}/not-admitted", sink.local_addr().unwrap());
    for location in [other_port.as_str(), "/not-admitted"] {
        let origin = TcpListener::bind(LOOPBACK).await.unwrap();
        let reply = redirect_reply(location);
        let (status, _) = tokio::select! {
            biased;
            _ = sink.accept() => panic!("an independent redirect sink was contacted"),
            result = exchange(origin, "/admitted", &reply) => result,
        };
        assert_eq!(status, Ok(StatusCode::FOUND));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn malformed_redirect_location_yields_a_route_free_request_failure_without_following() {
    let origin = TcpListener::bind(LOOPBACK).await.unwrap();
    let sink = TcpListener::bind(LOOPBACK).await.unwrap();
    let sink_address = sink.local_addr().unwrap();
    // The oversized numeric port survives HTTP URI parsing but fails URL parsing.
    let invalid_port = u32::from(sink_address.port()) + 65536;
    let location = format!(
        "http://{}:{invalid_port}/private-redirect",
        sink_address.ip()
    );
    let reply = redirect_reply(&location);
    let (result, _) = tokio::select! {
        biased;
        _ = sink.accept() => panic!("the malformed redirect's loopback sink was contacted"),
        result = exchange(origin, "/private-admitted", &reply) => result,
    };

    let error = result.unwrap_err();
    assert_eq!(
        error,
        SessionHttpError::Transport(TransportFailure::Request)
    );
    let rendered = format!("{error} {error:?}");
    assert!(!rendered.contains("private-"));
    assert!(!rendered.contains("127.0.0.1"));
    assert!(error.source().is_none());
}

#[test]
fn refuses_system_proxy_policy_without_retaining_routes() {
    let endpoints = SessionEndpoints::parse(
        "https://private-http.test/private-http-path",
        "wss://private-ws.test/private-ws-path",
        ProxyPolicy::System,
    )
    .unwrap();
    let error = SessionHttpClient::new(&endpoints).unwrap_err();
    assert_eq!(
        error,
        SessionHttpError::UnsupportedProxyPolicy(ProxyPolicy::System)
    );
    assert!(!format!("{error} {error:?}").contains("private-"));
    assert!(error.source().is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn a_malformed_response_yields_a_route_free_transport_error() {
    let origin = TcpListener::bind(LOOPBACK).await.unwrap();
    let address = origin.local_addr().unwrap().to_string();
    let (result, _) = exchange(origin, "/private-prefix", "not an HTTP response\r\n\r\n").await;

    let error = result.unwrap_err();
    assert_eq!(
        error,
        SessionHttpError::Transport(TransportFailure::Request)
    );
    let rendered = format!("{error} {error:?}");
    assert!(!rendered.contains(&address));
    assert!(!rendered.contains("private-prefix"));
    assert!(!rendered.contains("not an HTTP response"));
    assert!(error.source().is_none());
}
