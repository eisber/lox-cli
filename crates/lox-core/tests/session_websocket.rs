#![cfg(feature = "runtime-websocket")]

use lox_core::runtime::Cancellation;
use lox_core::runtime::session_endpoints::{ProxyPolicy, SessionEndpoints};
use lox_core::runtime::session_websocket::{SessionWebSocket, SessionWebSocketError};
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener as ProbeListener};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::timeout;
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{
    Callback, ErrorResponse, Request, Response,
};

const FIXTURE_DEADLINE: Duration = Duration::from_secs(5);
const WS_PATH: &str = "/fixture/site/ws";

fn probe_listener() -> ProbeListener {
    let listener = ProbeListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    listener
}

fn assert_no_contact(listener: &ProbeListener) {
    assert_eq!(listener.accept().unwrap_err().kind(), ErrorKind::WouldBlock);
}

fn direct(ws: SocketAddr, http: SocketAddr) -> SessionEndpoints {
    SessionEndpoints::parse(
        &format!("http://{http}/unused"),
        &format!("ws://{ws}{WS_PATH}"),
        ProxyPolicy::Direct,
    )
    .unwrap()
}

async fn assert_released(peer: &mut TcpStream) {
    match peer.read(&mut [0]).await {
        Ok(0) => {}
        Err(error) if error.kind() == ErrorKind::ConnectionReset => {}
        result => panic!("expected peer connection release without further bytes, got {result:?}"),
    }
}

async fn wait_for_upgrade_request(peer: &mut TcpStream) {
    let mut reader = BufReader::new(peer);
    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    assert_eq!(line, format!("GET {WS_PATH} HTTP/1.1\r\n"));
    loop {
        line.clear();
        assert!(reader.read_line(&mut line).await.unwrap() > 0);
        if line == "\r\n" {
            return;
        }
    }
}

struct CheckUpgradeRequest(SocketAddr);

impl Callback for CheckUpgradeRequest {
    fn on_request(self, request: &Request, response: Response) -> Result<Response, ErrorResponse> {
        assert_eq!(request.uri().path(), WS_PATH);
        assert_eq!(
            request.headers()["host"].to_str().unwrap(),
            self.0.to_string()
        );
        for header in ["authorization", "proxy-authorization", "cookie"] {
            assert!(!request.headers().contains_key(header));
        }
        Ok(response)
    }
}

#[tokio::test]
async fn upgrades_exact_websocket_route_and_close_or_drop_releases_it() {
    timeout(FIXTURE_DEADLINE, async {
        let http = probe_listener();
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let ws = server.local_addr().unwrap();
        let endpoints = direct(ws, http.local_addr().unwrap());
        let cancellation = Cancellation::new();
        let disposals: [fn(SessionWebSocket); 2] = [SessionWebSocket::close, drop];
        for dispose in disposals {
            tokio::join!(
                async {
                    let (peer, _) = server.accept().await.unwrap();
                    let mut socket = accept_hdr_async(peer, CheckUpgradeRequest(ws))
                        .await
                        .unwrap();
                    assert_released(socket.get_mut()).await;
                },
                async {
                    let handle = SessionWebSocket::connect(&endpoints, &cancellation)
                        .await
                        .unwrap();
                    dispose(handle);
                }
            );
        }
        assert_no_contact(&http);
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn refuses_proxy_plaintext_non_loopback_and_sticky_cancellation_without_contact() {
    let probe = probe_listener();
    let address = probe.local_addr().unwrap();
    let port = address.port();
    let http = format!("http://{address}/unused");
    let cancellation = Cancellation::new();
    use SessionWebSocketError as Error;

    for (host, proxy_policy, expected) in [
        (
            "127.0.0.1",
            ProxyPolicy::System,
            Error::UnsupportedProxyPolicy(ProxyPolicy::System),
        ),
        (
            "localhost",
            ProxyPolicy::Direct,
            Error::PlaintextRequiresLoopback,
        ),
        (
            "0.0.0.0",
            ProxyPolicy::Direct,
            Error::PlaintextRequiresLoopback,
        ),
    ] {
        let endpoints =
            SessionEndpoints::parse(&http, &format!("ws://{host}:{port}{WS_PATH}"), proxy_policy)
                .unwrap();
        assert_eq!(
            SessionWebSocket::connect(&endpoints, &cancellation)
                .await
                .unwrap_err(),
            expected
        );
        assert_no_contact(&probe);
    }

    cancellation.cancel();
    assert_eq!(
        SessionWebSocket::connect(&direct(address, address), &cancellation)
            .await
            .unwrap_err(),
        Error::Cancelled
    );
    assert_no_contact(&probe);
}

#[tokio::test]
async fn cancellation_during_upgrade_releases_the_peer_socket() {
    timeout(FIXTURE_DEADLINE, async {
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = server.local_addr().unwrap();
        let endpoints = direct(address, address);
        let cancellation = Cancellation::new();

        let (result, ()) = tokio::join!(
            SessionWebSocket::connect(&endpoints, &cancellation),
            async {
                let (mut peer, _) = server.accept().await.unwrap();
                wait_for_upgrade_request(&mut peer).await;
                cancellation.cancel();
                assert_released(&mut peer).await;
            }
        );
        assert_eq!(result.unwrap_err(), SessionWebSocketError::Cancelled);
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn refuses_redirect_without_contacting_location() {
    timeout(FIXTURE_DEADLINE, async {
        let redirect = probe_listener();
        let target = redirect.local_addr().unwrap();
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = server.local_addr().unwrap();
        let endpoints = direct(address, target);
        let cancellation = Cancellation::new();
        let (result, ()) = tokio::join!(
            SessionWebSocket::connect(&endpoints, &cancellation),
            async {
                let (mut peer, _) = server.accept().await.unwrap();
                wait_for_upgrade_request(&mut peer).await;
                peer.write_all(
                    format!(
                        "HTTP/1.1 302 Found\r\nLocation: ws://{target}/elsewhere\r\nContent-Length: 0\r\n\r\n"
                    )
                    .as_bytes(),
                )
                .await
                .unwrap();
                assert_released(&mut peer).await;
            }
        );
        assert_eq!(result.unwrap_err(), SessionWebSocketError::Redirect);
        assert_no_contact(&redirect);
    })
    .await
    .unwrap();
}
