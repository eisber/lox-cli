#![cfg(feature = "runtime")]

use lox_core::runtime::TransportFailure;
use lox_core::runtime::session_endpoints::{ProxyPolicy, SessionEndpoints};
use lox_core::runtime::session_http::{
    BootstrapCapabilities, BootstrapError, BootstrapPayloadIssue, Capability, SessionHttpClient,
    SessionHttpError, TlsStatus,
};
use reqwest::StatusCode;
use serde_json::json;
use std::error::Error;
use std::future::Future;
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
    exchange_with(origin, path, reply, |client| async move {
        client.get_status().await
    })
    .await
}

async fn exchange_with<T, F: Future<Output = T>>(
    origin: TcpListener,
    path: &str,
    reply: &str,
    request: impl FnOnce(SessionHttpClient) -> F,
) -> (T, String) {
    let address = origin.local_addr().unwrap();
    let endpoints = SessionEndpoints::parse(
        &format!("http://{address}{path}"),
        &format!("ws://{address}/not-the-http-route"),
        ProxyPolicy::Direct,
    )
    .unwrap();
    let client = SessionHttpClient::new(&endpoints).unwrap();
    let (finished, request_done) = oneshot::channel();
    let request_result = async {
        let result = request(client).await;
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
    tokio::time::timeout(FIXTURE_DEADLINE, async {
        tokio::join!(request_result, serve)
    })
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

async fn bootstrap_exchange(
    origin: TcpListener,
    reply: &str,
) -> (Result<BootstrapCapabilities, BootstrapError>, String) {
    exchange_with(origin, "/jdev/cfg/api", reply, |client| async move {
        client.bootstrap_preflight().await
    })
    .await
}

fn ok_reply(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

// Original synthetic LL fixtures, not a firmware capture or an upstream example.
#[tokio::test(flavor = "current_thread")]
async fn bootstrap_reads_advisory_capabilities_at_the_exact_port_and_command_path() {
    let origin = TcpListener::bind(LOOPBACK).await.unwrap();
    let address = origin.local_addr().unwrap();
    let body = json!({"LL": {
        "control": "dev/cfg/api",
        "Code": "200",
        "value": r#"{'hasEventSlots':false,'httpsStatus':2,"snr":"synthetic'private-marker"}"#,
    }});
    let (result, request) = bootstrap_exchange(origin, &ok_reply(&body.to_string())).await;

    assert_eq!(
        result,
        Ok(BootstrapCapabilities {
            event_slots: Capability::Reported(false),
            tls_status: Capability::Reported(TlsStatus::CertificateExpired),
        })
    );
    assert!(!format!("{result:?}").contains("private-marker"));
    assert_eq!(request.lines().next(), Some("GET /jdev/cfg/api HTTP/1.1"));
    let host = format!("host: {address}");
    assert!(request.lines().any(|line| line.eq_ignore_ascii_case(&host)));
    let headers = request.to_ascii_lowercase();
    for name in ["authorization:", "proxy-authorization:", "cookie:"] {
        assert!(!headers.lines().any(|line| line.starts_with(name)));
    }
}

#[tokio::test(flavor = "current_thread")]
async fn bootstrap_distinguishes_reported_missing_and_unknown_values() {
    for (value, event_slots, tls_status) in [
        (
            r#"{"hasEventSlots":true,"httpsStatus":1}"#,
            Capability::Reported(true),
            Capability::Reported(TlsStatus::Available),
        ),
        (
            r#"{"httpsStatus":77}"#,
            Capability::Missing,
            Capability::Unknown,
        ),
        (
            r#"{"hasEventSlots":"false"}"#,
            Capability::Unknown,
            Capability::Missing,
        ),
        (
            r#"{"hasEventSlots":null,"syntheticIgnored":1,"syntheticIgnored":2}"#,
            Capability::Unknown,
            Capability::Missing,
        ),
    ] {
        let origin = TcpListener::bind(LOOPBACK).await.unwrap();
        let body = json!({"LL": {"control":"dev/cfg/api", "code":200, "value":value}});
        let (result, _) = bootstrap_exchange(origin, &ok_reply(&body.to_string())).await;
        assert_eq!(
            result,
            Ok(BootstrapCapabilities {
                event_slots,
                tls_status,
            })
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn bootstrap_sanitizes_status_and_payload_refusals() {
    use BootstrapError::Payload;
    use BootstrapPayloadIssue as Issue;

    for (reply, expected) in [
        (
            redirect_reply("/private-marker"),
            BootstrapError::HttpStatus(StatusCode::FOUND),
        ),
        (
            ok_reply("malformed-private-marker"),
            Payload(Issue::InvalidEnvelope),
        ),
        (
            ok_reply(
                r#"{"LL":{"control":"dev/cfg/api","code":503,"value":"synthetic-private-marker"}}"#,
            ),
            Payload(Issue::CommandStatus(503)),
        ),
        (
            ok_reply(r#"{"LL":{"control":"dev/cfg/private-marker","code":200,"value":"{}"}}"#),
            Payload(Issue::UnexpectedControl),
        ),
        (
            ok_reply(
                r#"{"LL":{"control":"dev/cfg/api","Code":200,"value":"{'note':'private-marker\\n'}"}}"#,
            ),
            Payload(Issue::UnsupportedValueEncoding),
        ),
    ] {
        let origin = TcpListener::bind(LOOPBACK).await.unwrap();
        let address = origin.local_addr().unwrap().to_string();
        let (result, _) = bootstrap_exchange(origin, &reply).await;
        let error = result.unwrap_err();
        assert_eq!(error, expected);
        let rendered = format!("{error} {error:?}");
        assert!(!rendered.contains(&address));
        assert!(!rendered.contains("private-marker"));
        assert!(error.source().is_none());
    }
}

#[tokio::test(flavor = "current_thread")]
async fn bootstrap_refuses_duplicate_outer_and_command_members_in_either_order() {
    let accepted = r#""LL":{"control":"dev/cfg/api","code":200,"value":"{}"}"#;
    for duplicate in [
        r#""LL":{"control":"private-marker","code":200,"value":"{}"}"#,
        r#""\u004cL":{"control":"private-marker","code":200,"value":"{}"}"#,
    ] {
        for (first, second) in [(accepted, duplicate), (duplicate, accepted)] {
            let origin = TcpListener::bind(LOOPBACK).await.unwrap();
            let (result, _) =
                bootstrap_exchange(origin, &ok_reply(&format!("{{{first},{second}}}"))).await;
            let error = result.unwrap_err();
            assert_eq!(
                error,
                BootstrapError::Payload(BootstrapPayloadIssue::InvalidEnvelope)
            );
            assert!(!format!("{error} {error:?}").contains("private-marker"));
        }
    }

    for (first, second, remaining) in [
        (
            r#""control":"dev/cfg/api""#,
            r#""contr\u006fl":"private-marker""#,
            r#""Code":200,"value":"{}""#,
        ),
        (
            r#""Code":200"#,
            r#""Code":503"#,
            r#""control":"dev/cfg/api","value":"{}""#,
        ),
        (
            r#""Code":200"#,
            r#""\u0063ode":503"#,
            r#""control":"dev/cfg/api","value":"{}""#,
        ),
        (
            r#""code":200"#,
            r#""CODE":503"#,
            r#""control":"dev/cfg/api","value":"{}""#,
        ),
        (
            r#""value":"{}""#,
            r#""value":"private-marker""#,
            r#""control":"dev/cfg/api","Code":200"#,
        ),
    ] {
        for (left, right) in [(first, second), (second, first)] {
            let origin = TcpListener::bind(LOOPBACK).await.unwrap();
            let body = format!(r#"{{"LL":{{{left},{right},{remaining}}}}}"#);
            let (result, _) = bootstrap_exchange(origin, &ok_reply(&body)).await;
            assert_eq!(
                result,
                Err(BootstrapError::Payload(
                    BootstrapPayloadIssue::InvalidEnvelope
                ))
            );
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn bootstrap_refuses_duplicate_capabilities_in_json_and_apostrophe_forms() {
    for (first, second) in [
        (r#""hasEventSlots":false"#, r#""hasEvent\u0053lots":true"#),
        (r#""httpsStatus":1"#, r#""httpsStatus":2"#),
        (r#"'hasEventSlots':false"#, r#""hasEvent\u0053lots":true"#),
        (r#"'httpsStatus':1"#, r#"'httpsStatus':2"#),
    ] {
        for (left, right) in [(first, second), (second, first)] {
            let origin = TcpListener::bind(LOOPBACK).await.unwrap();
            let value = format!("{{{left},{right}}}");
            let body = json!({"LL": {"control":"dev/cfg/api", "code":200, "value":value}});
            let (result, _) = bootstrap_exchange(origin, &ok_reply(&body.to_string())).await;
            assert_eq!(
                result,
                Err(BootstrapError::Payload(
                    BootstrapPayloadIssue::UnsupportedValueEncoding
                ))
            );
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn bootstrap_bounds_fixed_length_and_chunked_response_bodies_in_bytes() {
    let json = r#"{"LL":{"control":"dev/cfg/api","code":200,"value":"{\"hasEventSlots\":false}"}}"#;
    for size in [8192, 8193] {
        let body = format!("{json}{}", " ".repeat(size - json.len()));
        let fixed = ok_reply(&body);
        let (first, second) = body.split_at(4096);
        let chunked = format!(
            "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n\
             {:x}\r\n{first}\r\n{:x}\r\n{second}\r\n0\r\n\r\n",
            first.len(),
            second.len()
        );
        let expected = if size == 8192 {
            Ok(BootstrapCapabilities {
                event_slots: Capability::Reported(false),
                tls_status: Capability::Missing,
            })
        } else {
            Err(BootstrapError::Payload(BootstrapPayloadIssue::TooLarge))
        };
        for reply in [fixed, chunked] {
            let origin = TcpListener::bind(LOOPBACK).await.unwrap();
            let (result, _) = bootstrap_exchange(origin, &reply).await;
            assert_eq!(result, expected);
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn bootstrap_rejects_non_command_routes_without_contacting_the_origin() {
    let origin = TcpListener::bind(LOOPBACK).await.unwrap();
    let address = origin.local_addr().unwrap();
    for path in [
        "/",
        "/jdev/cfg/apiKey",
        "/prefix/jdev/cfg/api",
        "/jdev/cfg/api/",
    ] {
        let endpoints = SessionEndpoints::parse(
            &format!("http://{address}{path}"),
            &format!("ws://{address}/jdev/cfg/api"),
            ProxyPolicy::Direct,
        )
        .unwrap();
        let client = SessionHttpClient::new(&endpoints).unwrap();
        let result = tokio::time::timeout(FIXTURE_DEADLINE, async {
            tokio::select! {
                biased;
                _ = origin.accept() => panic!("a non-command route was contacted"),
                result = client.bootstrap_preflight() => result,
            }
        })
        .await
        .unwrap();
        assert_eq!(result, Err(BootstrapError::WrongRoute));
    }
}
