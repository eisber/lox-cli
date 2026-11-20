#![cfg(feature = "runtime-websocket")]

use lox_core::runtime::Cancellation;
use lox_core::runtime::session_endpoints::{ProxyPolicy, SessionEndpoints};
use lox_core::runtime::session_websocket::{SessionWebSocket, SessionWebSocketError};
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener as ProbeListener};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Instant, timeout};
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
    assert_upgrade_and_release("127.0.0.1:0").await;
}

#[tokio::test]
#[ignore = "requires an operational IPv6 loopback interface"]
async fn upgrades_numeric_ipv6_loopback_route_and_releases_it() {
    assert_upgrade_and_release("[::1]:0").await;
}

async fn assert_upgrade_and_release(bind: &str) {
    timeout(FIXTURE_DEADLINE, async {
        let http = probe_listener();
        let server = TcpListener::bind(bind).await.unwrap();
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
#[ignore = "exercises the production ten-second deadline on a real clock"]
async fn upgrade_deadline_expires_and_releases_the_peer_socket() {
    const CONNECT_DEADLINE: Duration = Duration::from_secs(10);

    timeout(CONNECT_DEADLINE + FIXTURE_DEADLINE, async {
        let http = probe_listener();
        let server = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoints = direct(server.local_addr().unwrap(), http.local_addr().unwrap());
        let cancellation = Cancellation::new();

        let ((result, elapsed), ()) = tokio::join!(
            async {
                let start = Instant::now();
                let result = SessionWebSocket::connect(&endpoints, &cancellation).await;
                (result, start.elapsed())
            },
            async {
                let (mut peer, _) = server.accept().await.unwrap();
                wait_for_upgrade_request(&mut peer).await;
                assert_released(&mut peer).await;
            }
        );
        assert_eq!(result.unwrap_err(), SessionWebSocketError::Timeout);
        assert!(
            elapsed >= CONNECT_DEADLINE,
            "premature timeout: {elapsed:?}"
        );
        assert_no_contact(&http);
    })
    .await
    .expect("stalled upgrade did not time out and release its socket");
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

mod local_wss {
    use super::*;
    use std::{
        env,
        path::{Path, PathBuf},
        process::Command,
        sync::Arc,
    };
    use tokio_rustls::TlsAcceptor;
    use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
    use tokio_rustls::rustls::{AlertDescription, Error as TlsError, ServerConfig};

    const FIXTURE_DIR: &str = "LOX_WSS_FIXTURE_DIR";
    const CHILD_CASE: &str = "LOX_WSS_FIXTURE_CHILD_CASE";
    const CHILD_TEST: &str = "local_wss::strict_direct_wss_child";
    const CHILD_PASSED: &str = "local WSS child passed: ";

    struct Case {
        name: &'static str,
        leaf: &'static str,
        root: &'static str,
        alert: Option<AlertDescription>,
    }

    impl Case {
        fn required_files(&self, fixture: &Path) -> [PathBuf; 3] {
            let files = [
                fixture.join(self.root),
                fixture.join(format!("{}.pem", self.leaf)),
                fixture.join(format!("{}.key", self.leaf)),
            ];
            for file in &files {
                assert!(
                    file.is_file(),
                    "missing required WSS fixture file: {file:?}"
                );
            }
            files
        }
    }

    const CASES: [Case; 3] = [
        Case {
            name: "trusted-loopback",
            leaf: "loopback",
            root: "trusted-root.pem",
            alert: None,
        },
        Case {
            name: "wrong-ip",
            leaf: "wrong-ip",
            root: "trusted-root.pem",
            alert: Some(AlertDescription::BadCertificate),
        },
        Case {
            name: "untrusted-chain",
            leaf: "loopback",
            root: "unrelated-root.pem",
            alert: Some(AlertDescription::UnknownCA),
        },
    ];

    fn fixture_dir() -> PathBuf {
        let fixture = PathBuf::from(
            env::var_os(FIXTURE_DIR).expect("set LOX_WSS_FIXTURE_DIR; see Core README"),
        );
        assert!(
            fixture.is_absolute() && fixture.is_dir(),
            "expected an absolute WSS fixture directory: {fixture:?}"
        );
        fixture
    }

    #[test]
    #[ignore = "requires generated LOX_WSS_FIXTURE_DIR; see Core README: Local WSS fixture"]
    fn strict_direct_wss() {
        let fixture = fixture_dir();
        for case in &CASES {
            case.required_files(&fixture);
        }

        for case in &CASES {
            let mut child = Command::new(env::current_exe().unwrap());
            child
                .args(["--ignored", "--exact", CHILD_TEST, "--nocapture"])
                .env(FIXTURE_DIR, &fixture)
                .env(CHILD_CASE, case.name)
                .env("SSL_CERT_FILE", fixture.join(case.root))
                .env_remove("SSL_CERT_DIR");
            for var in ["HTTP_PROXY", "HTTPS_PROXY", "ALL_PROXY", "NO_PROXY"] {
                child.env_remove(var).env_remove(var.to_ascii_lowercase());
            }
            let output = child.output().expect("run isolated WSS fixture child");
            let stdout = String::from_utf8_lossy(&output.stdout);
            let passed = format!("{CHILD_PASSED}{}", case.name);
            assert!(
                output.status.success() && stdout.lines().any(|line| line == passed),
                "WSS fixture case {} did not report success ({}):\n{stdout}\n{}",
                case.name,
                output.status,
                String::from_utf8_lossy(&output.stderr)
            );
            eprintln!("{passed}");
        }
    }

    #[test]
    #[ignore = "internal child; run local_wss::strict_direct_wss exactly"]
    fn strict_direct_wss_child() {
        let name = env::var(CHILD_CASE).expect("WSS fixture child requires an explicit case");
        let case = CASES
            .iter()
            .find(|case| case.name == name)
            .expect("unknown WSS fixture case");
        let [root, cert, key] = case.required_files(&fixture_dir());
        assert_eq!(
            env::var_os("SSL_CERT_FILE").as_deref(),
            Some(root.as_os_str())
        );
        assert!(env::var_os("SSL_CERT_DIR").is_none());
        let cert = CertificateDer::from_pem_file(cert).expect("read fixture server certificate");
        let key = PrivateKeyDer::from_pem_file(key).expect("read fixture server key");
        let config = ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(vec![cert], key)
            .expect("configure fixture TLS server");
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(check_case(case, TlsAcceptor::from(Arc::new(config))));
        println!("\n{CHILD_PASSED}{}", case.name);
    }

    async fn check_case(case: &Case, acceptor: TlsAcceptor) {
        timeout(FIXTURE_DEADLINE, async {
            let http = probe_listener();
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            let endpoints = SessionEndpoints::parse(
                &format!("http://{}/unused", http.local_addr().unwrap()),
                &format!("wss://{address}{WS_PATH}"),
                ProxyPolicy::Direct,
            )
            .unwrap();
            let cancellation = Cancellation::new();
            tokio::join!(
                async {
                    let (peer, remote) = listener.accept().await.unwrap();
                    assert_eq!(remote.ip(), address.ip());
                    let tls = acceptor.accept(peer).into_fallible().await;
                    if let Some(expected) = case.alert {
                        let Err((error, mut peer)) = tls else {
                            panic!("TLS unexpectedly completed before WebSocket upgrade");
                        };
                        assert_eq!(
                            error
                                .get_ref()
                                .and_then(|error| error.downcast_ref::<TlsError>()),
                            Some(&TlsError::AlertReceived(expected)),
                            "expected client TLS alert, got {error:?}"
                        );
                        assert_released(&mut peer).await;
                    } else {
                        let mut ws = accept_hdr_async(tls.unwrap(), CheckUpgradeRequest(address))
                            .await
                            .unwrap();
                        assert_released(ws.get_mut().get_mut().0).await;
                    }
                },
                async {
                    let result = SessionWebSocket::connect(&endpoints, &cancellation).await;
                    if case.alert.is_some() {
                        assert_eq!(result.unwrap_err(), SessionWebSocketError::Upgrade);
                    } else {
                        result.unwrap().close();
                    }
                }
            );
            assert_no_contact(&http);
        })
        .await
        .expect("local WSS handshake or socket release exceeded fixture deadline");
    }
}
