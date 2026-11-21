#![cfg(feature = "runtime-websocket")]

use lox_core::runtime::Cancellation;
use lox_core::runtime::auth_challenge_envelope::GetKey2Command;
use lox_core::runtime::session_endpoints::{ProxyPolicy, SessionEndpoints};
use lox_core::runtime::session_token::{TokenExpiry, TokenSlot, TokenSubject};
use lox_core::runtime::session_websocket::{
    GetKey2ExchangeError, SessionWebSocket, SessionWebSocketError, SessionWebSocketTokenError,
};
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener as ProbeListener};
use std::time::{Duration, UNIX_EPOCH};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::{Instant, timeout};
use tokio_tungstenite::accept_hdr_async;
use tokio_tungstenite::tungstenite::handshake::server::{
    Callback, ErrorResponse, Request, Response,
};

const FIXTURE_DEADLINE: Duration = Duration::from_secs(5);
const WS_PATH: &str = "/fixture/site/ws";
const MOCK_TOKEN: &str = "opaque synthetic WebSocket token";

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
        let subject = TokenSubject::new(&endpoints, "fixture account").unwrap();
        let mut slot = TokenSlot::new(&subject);
        slot.replace(
            &subject,
            MOCK_TOKEN.into(),
            TokenExpiry::At(UNIX_EPOCH + Duration::from_secs(1)),
        )
        .unwrap();
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
                    assert_eq!(
                        handle
                            .eligible_token(&slot, &subject, UNIX_EPOCH)
                            .unwrap_err(),
                        SessionWebSocketTokenError::UnverifiedTransport
                    );
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
async fn getkey2_refuses_plaintext_and_oversize_encoded_request_before_dial() {
    let ws = probe_listener();
    let http = probe_listener();
    let address = ws.local_addr().unwrap();
    let http_address = http.local_addr().unwrap();
    let endpoints = direct(address, http_address);
    let cancellation = Cancellation::new();
    let command = GetKey2Command::for_principal("synthetic loopback account").unwrap();
    assert_eq!(
        SessionWebSocket::getkey2(&endpoints, &command, &cancellation)
            .await
            .unwrap_err(),
        GetKey2ExchangeError::VerifiedWssRequired
    );
    assert_no_contact(&ws);
    assert_no_contact(&http);
    println!("getkey2-plaintext-no-dial");

    let endpoints = SessionEndpoints::parse(
        &format!("http://{http_address}/unused"),
        &format!("wss://{address}{WS_PATH}"),
        ProxyPolicy::Direct,
    )
    .unwrap();
    // The raw principal fits, but its escaped command does not.
    let command = GetKey2Command::for_principal(&"/".repeat(400)).unwrap();
    assert_eq!(
        SessionWebSocket::getkey2(&endpoints, &command, &cancellation)
            .await
            .unwrap_err(),
        GetKey2ExchangeError::RequestTooLarge
    );
    assert_no_contact(&ws);
    assert_no_contact(&http);
    println!("getkey2-encoded-bound-no-dial");
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
    #[cfg(feature = "runtime-websocket-encrypted-loopback")]
    mod encrypted_loopback;
    use futures_util::{FutureExt, SinkExt, StreamExt};
    use lox_core::message_header::{MessageHeader, MessageType};
    use lox_core::runtime::auth_challenge_envelope::AuthChallengeEnvelopeError;
    use lox_core::runtime::auth_reply_framing::AuthReplyFramingError;
    use lox_core::runtime::login_challenge::HashAlgorithm;
    #[cfg(feature = "auth-hash")]
    use lox_core::runtime::session_websocket::PrepareLoginResponseError;
    use std::{
        env,
        net::{IpAddr, Ipv4Addr},
        path::{Path, PathBuf},
        process::Command,
        sync::Arc,
    };
    use tokio::sync::oneshot;
    use tokio_rustls::TlsAcceptor;
    use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject};
    use tokio_rustls::rustls::{AlertDescription, Error as TlsError, ServerConfig};
    use tokio_rustls::server::TlsStream;
    use tokio_tungstenite::WebSocketStream;
    use tokio_tungstenite::tungstenite::error::ProtocolError;
    use tokio_tungstenite::tungstenite::protocol::frame::{
        Frame,
        coding::{Data, OpCode},
    };
    use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message};

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

    #[derive(Clone, Copy)]
    enum GetKey2Case {
        Success,
        WrongEcho,
        CancelAfterRequest,
        DropAfterRequest,
        ControlInPair,
        OversizeFrame,
        OversizeMessage,
    }

    const GETKEY2_CASES: [(GetKey2Case, &str); 7] = [
        (GetKey2Case::Success, "getkey2-trusted-same-socket"),
        (GetKey2Case::WrongEcho, "getkey2-wrong-echo-released"),
        (
            GetKey2Case::CancelAfterRequest,
            "getkey2-cancel-after-request-released",
        ),
        (
            GetKey2Case::DropAfterRequest,
            "getkey2-drop-after-request-released",
        ),
        (
            GetKey2Case::ControlInPair,
            "getkey2-control-in-pair-released",
        ),
        (
            GetKey2Case::OversizeFrame,
            "getkey2-oversize-frame-released",
        ),
        (
            GetKey2Case::OversizeMessage,
            "getkey2-oversize-message-released",
        ),
    ];

    #[cfg(feature = "auth-hash")]
    #[derive(Clone, Copy)]
    enum BoundResponseCase {
        Close,
        Drop,
        AlgorithmMismatch,
        PreCancelled,
    }

    #[cfg(feature = "auth-hash")]
    const BOUND_RESPONSE_CASES: [(BoundResponseCase, &str); 4] = [
        (
            BoundResponseCase::Close,
            "bound-response-raw-principal-no-send-close",
        ),
        (BoundResponseCase::Drop, "bound-response-no-send-drop"),
        (
            BoundResponseCase::AlgorithmMismatch,
            "bound-response-algorithm-mismatch-released",
        ),
        (
            BoundResponseCase::PreCancelled,
            "bound-response-pre-cancelled-released",
        ),
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
            let names = std::iter::once(case.name).chain(
                GETKEY2_CASES
                    .iter()
                    .filter(|_| case.alert.is_none())
                    .map(|&(_, name)| name),
            );
            #[cfg(feature = "auth-hash")]
            let names = names.chain(
                BOUND_RESPONSE_CASES
                    .iter()
                    .filter(|_| case.alert.is_none())
                    .map(|&(_, name)| name),
            );
            #[cfg(feature = "runtime-websocket-encrypted-loopback")]
            let names = names.chain(
                encrypted_loopback::CASES
                    .iter()
                    .filter(|_| case.alert.is_none())
                    .map(|&(_, name)| name),
            );
            for name in names {
                let passed = format!("{CHILD_PASSED}{name}");
                assert!(
                    output.status.success() && stdout.lines().any(|line| line == passed),
                    "WSS fixture case {name} did not report success ({}):\n{stdout}\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stderr)
                );
                eprintln!("{passed}");
            }
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
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let acceptor = TlsAcceptor::from(Arc::new(config));
        runtime.block_on(check_case(case, acceptor.clone()));
        println!("\n{CHILD_PASSED}{}", case.name);
        if case.alert.is_none() {
            for (case, name) in GETKEY2_CASES {
                runtime.block_on(check_getkey2_case(case, &acceptor));
                println!("{CHILD_PASSED}{name}");
            }
            #[cfg(feature = "auth-hash")]
            for (case, name) in BOUND_RESPONSE_CASES {
                runtime.block_on(check_bound_response_case(case, &acceptor));
                println!("{CHILD_PASSED}{name}");
            }
            #[cfg(feature = "runtime-websocket-encrypted-loopback")]
            {
                let application_key = encrypted_loopback::application_key();
                for (case, name) in encrypted_loopback::CASES {
                    runtime.block_on(encrypted_loopback::check(case, &acceptor, &application_key));
                    println!("{CHILD_PASSED}{name}");
                }
            }
        }
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
                        let handle = result.unwrap();
                        check_token_eligibility(&handle, &endpoints);
                        handle.close();
                    }
                }
            );
            assert_no_contact(&http);
        })
        .await
        .expect("local WSS handshake or socket release exceeded fixture deadline");
    }

    async fn assert_wss_released(ws: &mut WebSocketStream<TlsStream<TcpStream>>) {
        match ws.next().await {
            None
            | Some(Err(WebSocketError::Protocol(ProtocolError::ResetWithoutClosingHandshake))) => {}
            Some(Err(WebSocketError::Io(error)))
                if matches!(
                    error.kind(),
                    ErrorKind::UnexpectedEof
                        | ErrorKind::ConnectionReset
                        | ErrorKind::ConnectionAborted
                ) => {}
            result => {
                panic!("expected release without a follow-up WebSocket message, got {result:?}")
            }
        }
    }

    async fn check_getkey2_case(case: GetKey2Case, acceptor: &TlsAcceptor) {
        use GetKey2Case::*;
        use GetKey2ExchangeError as Error;

        const PRINCIPAL: &str = "Fixture / café %";
        const COMMAND: &str = "jdev/sys/getkey2/Fixture%20%2F%20caf%C3%A9%20%25";
        const WRONG_ECHO: &str = "jdev/sys/getkey2/another%20fixture";
        const KEY: &str = "Aa01Bb23";
        const SALT: &str = "fixture-only salt";

        timeout(FIXTURE_DEADLINE, async {
            let http = probe_listener();
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            assert_eq!(address.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
            let endpoints = SessionEndpoints::parse(
                &format!("http://{}/unused", http.local_addr().unwrap()),
                &format!("wss://{address}{WS_PATH}"),
                ProxyPolicy::Direct,
            )
            .unwrap();
            let command = GetKey2Command::for_principal(PRINCIPAL).unwrap();
            let cancellation = Cancellation::new();
            let (request_received, wait_for_request) = oneshot::channel();
            let (admitted, wait_for_admission) = oneshot::channel();
            let (still_open, wait_for_open_check) = oneshot::channel();

            let server = async {
                    let (peer, remote) = listener.accept().await.unwrap();
                    assert_eq!(remote.ip(), address.ip());
                    let tls = acceptor.accept(peer).await.unwrap();
                    let mut ws = accept_hdr_async(tls, CheckUpgradeRequest(address))
                        .await
                        .unwrap();
                    assert_eq!(ws.next().await.unwrap().unwrap(), Message::Text(COMMAND.into()));
                    match case {
                        Success | WrongEcho | ControlInPair => {
                            let echo = if matches!(case, WrongEcho) { WRONG_ECHO } else { COMMAND };
                            let body = format!(
                                r#"{{"LL":{{"control":"{echo}","code":"200","value":{{"key":"{KEY}","salt":"{SALT}","hashAlg":"SHA256"}}}}}}"#
                            );
                            let header = MessageHeader::new(
                                MessageType::Text as u8,
                                0,
                                0,
                                body.len().try_into().unwrap(),
                            );
                            ws.send(Message::Binary(header.to_wire_bytes().to_vec())).await.unwrap();
                            if matches!(case, ControlInPair) {
                                ws.send(Message::Ping(b"fixture control".to_vec())).await.unwrap();
                            } else {
                                ws.send(Message::Text(body)).await.unwrap();
                            }
                            if matches!(case, Success) {
                                wait_for_admission.await.unwrap();
                                assert!(ws.next().now_or_never().is_none());
                                still_open.send(()).unwrap();
                            }
                        }
                        CancelAfterRequest => cancellation.cancel(),
                        DropAfterRequest => request_received.send(()).unwrap(),
                        OversizeFrame => {
                            // Unmasked FIN text frame declaring 16 KiB + 1; never send its body.
                            ws.get_mut().write_all(&[0x81, 126, 0x40, 0x01]).await.unwrap();
                            ws.get_mut().flush().await.unwrap();
                        }
                        OversizeMessage => {
                            for (opcode, size, last) in [(Data::Text, 8192, false), (Data::Continue, 8193, true)] {
                                let frame = Frame::message(vec![b'x'; size], OpCode::Data(opcode), last);
                                ws.send(Message::Frame(frame)).await.unwrap();
                            }
                        }
                    }
                    assert_wss_released(&mut ws).await;
            };
            let client = async {
                    let exchange = SessionWebSocket::getkey2(&endpoints, &command, &cancellation);
                    if matches!(case, DropAfterRequest) {
                        tokio::select! {
                            _ = wait_for_request => {}
                            result = exchange => panic!("exchange finished before drop: {result:?}"),
                        }
                        return;
                    }
                    let result = exchange.await;
                    let diagnostics = format!("{result:?}");
                    let expected = match case {
                        Success => {
                            let pending = result.unwrap();
                            let challenge = pending.challenge();
                            assert_eq!(challenge.key_hex(), KEY);
                            assert_eq!(challenge.salt(), SALT);
                            assert_eq!(challenge.algorithm(), HashAlgorithm::Sha256);
                            assert!(!format!("{challenge:?}").contains(SALT));
                            for private in [PRINCIPAL, COMMAND, KEY, SALT, WS_PATH] {
                                assert!(!diagnostics.contains(private));
                            }
                            admitted.send(()).unwrap();
                            wait_for_open_check.await.unwrap();
                            pending.close();
                            return;
                        }
                        WrongEcho => Error::Envelope(AuthChallengeEnvelopeError::EchoMismatch),
                        CancelAfterRequest => Error::Cancelled,
                        ControlInPair => Error::Framing(AuthReplyFramingError::PayloadMessageKind),
                        OversizeFrame | OversizeMessage => Error::ReplyTooLarge,
                        DropAfterRequest => unreachable!(),
                    };
                    let error = result.unwrap_err();
                    assert_eq!(error, expected);
                    assert!(std::error::Error::source(&error).is_none());
                    for private in [PRINCIPAL, COMMAND, WRONG_ECHO, KEY, SALT, WS_PATH, "127.0.0.1", "fixture control"] {
                        assert!(!diagnostics.contains(private));
                        assert!(!error.to_string().contains(private));
                    }
            };
            tokio::join!(server, client);
            assert!(listener.accept().now_or_never().is_none());
            assert_no_contact(&http);
        })
        .await
        .expect("local WSS getkey2 exchange or socket release exceeded fixture deadline");
    }

    #[cfg(feature = "auth-hash")]
    async fn check_bound_response_case(case: BoundResponseCase, acceptor: &TlsAcceptor) {
        const PRINCIPAL: &str = " Fixture\u{0301}:/%2f ";
        const COMMAND: &str = "jdev/sys/getkey2/%20Fixture%CC%81%3A%2F%252f%20";
        const PASSWORD: &str = "synthetic password /:";
        const KEY: &str = "Aa01Bb23";
        const SALT: &str = " fixture-only:salt ";
        // Independent .NET SHA256/HMACSHA256 over the original UTF-8 preimages.
        const EXPECTED: &str = "0018456961a7d14b6c98c94345ff3f3fe97c9f115004d9d67364cd781bab43b3";

        timeout(FIXTURE_DEADLINE, async {
            let http = probe_listener();
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            assert_eq!(address.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
            let endpoints = SessionEndpoints::parse(
                &format!("http://{}/unused", http.local_addr().unwrap()),
                &format!("wss://{address}{WS_PATH}"),
                ProxyPolicy::Direct,
            )
            .unwrap();
            let (prepared, wait_for_prepared) = oneshot::channel();
            let (open, wait_for_open) = oneshot::channel();
            let expected_error = match case {
                BoundResponseCase::Close | BoundResponseCase::Drop => None,
                BoundResponseCase::AlgorithmMismatch => {
                    Some(PrepareLoginResponseError::AlgorithmMismatch)
                }
                BoundResponseCase::PreCancelled => Some(PrepareLoginResponseError::Cancelled),
            };

            let server = async {
                let (peer, remote) = listener.accept().await.unwrap();
                assert_eq!(remote.ip(), address.ip());
                let tls = acceptor.accept(peer).await.unwrap();
                let mut ws = accept_hdr_async(tls, CheckUpgradeRequest(address))
                    .await
                    .unwrap();
                assert!(
                    matches!(ws.next().await, Some(Ok(Message::Text(text))) if text == COMMAND),
                    "expected one exact getkey2 application request"
                );
                let value = format!(r#"{{"key":"{KEY}","salt":"{SALT}","hashAlg":"SHA256"}}"#);
                let body =
                    format!(r#"{{"LL":{{"control":"{COMMAND}","code":200,"value":{value}}}}}"#);
                let header = MessageHeader::new(
                    MessageType::Text as u8,
                    0,
                    0,
                    body.len().try_into().unwrap(),
                );
                ws.send(Message::Binary(header.to_wire_bytes().to_vec()))
                    .await
                    .unwrap();
                ws.send(Message::Text(body)).await.unwrap();
                // Inspect decrypted bytes after the challenge, even if a new frame were incomplete.
                let tls = ws.get_mut();
                let mut byte = [0];
                if expected_error.is_none() {
                    wait_for_prepared.await.unwrap();
                    assert!(
                        tls.read(&mut byte).now_or_never().is_none(),
                        "the same connection must still be held without follow-up application bytes"
                    );
                    open.send(()).unwrap();
                }
                match tls.read(&mut byte).await {
                    Ok(0) => {}
                    Err(error)
                        if matches!(
                            error.kind(),
                            ErrorKind::UnexpectedEof
                                | ErrorKind::ConnectionReset
                                | ErrorKind::ConnectionAborted
                        ) => {}
                    _ => panic!("expected release without follow-up application bytes"),
                }
            };
            let client = async {
                let cancellation = Cancellation::new();
                let pending = {
                    let command = GetKey2Command::for_principal(PRINCIPAL).unwrap();
                    SessionWebSocket::getkey2(&endpoints, &command, &cancellation)
                        .await
                        .unwrap()
                };
                if matches!(case, BoundResponseCase::PreCancelled) {
                    cancellation.cancel();
                }
                let permitted = if matches!(case, BoundResponseCase::AlgorithmMismatch) {
                    HashAlgorithm::Sha1
                } else {
                    HashAlgorithm::Sha256
                };
                let result = pending.prepare_response(permitted, PASSWORD, &cancellation);
                let assert_redacted = |text: &str| {
                    for private in [PRINCIPAL, COMMAND, PASSWORD, KEY, SALT, EXPECTED, WS_PATH] {
                        assert!(
                            !text.contains(private),
                            "unexpected fixture value in diagnostics"
                        );
                    }
                };
                assert_redacted(&format!("{result:#?}"));
                if let Some(expected) = expected_error {
                    let error = result.unwrap_err();
                    assert_eq!(error, expected);
                    assert_redacted(&error.to_string());
                    assert!(std::error::Error::source(&error).is_none());
                } else {
                    let response = result.unwrap();
                    assert!(
                        response.response().to_lower_hex() == EXPECTED,
                        "response must use the exact original preimage"
                    );
                    assert_redacted(&format!("{:#?}", response.response()));
                    prepared.send(()).unwrap();
                    wait_for_open.await.unwrap();
                    if matches!(case, BoundResponseCase::Close) {
                        response.close();
                    } else {
                        drop(response);
                    }
                }
            };
            tokio::join!(server, client);
            assert!(listener.accept().now_or_never().is_none());
            assert_no_contact(&http);
        })
        .await
        .expect("bound response preparation or socket release exceeded fixture deadline");
    }

    fn check_token_eligibility(handle: &SessionWebSocket, endpoints: &SessionEndpoints) {
        use SessionWebSocketTokenError as Error;

        const PRINCIPAL: &str = " Fixture WSS account ";
        let equivalent = SessionEndpoints::parse(
            &endpoints.http_url().replacen("http://", "HTTP://", 1),
            &endpoints.websocket_url().replacen("wss://", "WSS://", 1),
            ProxyPolicy::Direct,
        )
        .unwrap();
        let original = TokenSubject::new(endpoints, PRINCIPAL).unwrap();
        let subject = TokenSubject::new(&equivalent, PRINCIPAL).unwrap();
        let mut slot = TokenSlot::new(&original);
        let tick = Duration::from_millis(1);
        let expires_at = UNIX_EPOCH + Duration::from_secs(1);
        let now = expires_at - tick;
        let expiry = TokenExpiry::At(expires_at);

        let assert_redacted = |diagnostics: &str| {
            for private in [
                MOCK_TOKEN,
                PRINCIPAL.trim(),
                "127.0.0.1",
                "127.0.0.2",
                WS_PATH,
                "/unused",
                "/different-ws",
                "/different-http",
            ] {
                assert!(!diagnostics.contains(private));
            }
        };
        let denied = |slot: &TokenSlot, subject: &TokenSubject<'_>, time, expected| {
            let result = handle.eligible_token(slot, subject, time);
            assert_redacted(&format!("{result:?}"));
            let error = result.unwrap_err();
            assert_eq!(error, expected);
            assert_redacted(&error.to_string());
            assert!(std::error::Error::source(&error).is_none());
        };

        denied(&slot, &subject, now, Error::Empty);
        slot.replace(&original, MOCK_TOKEN.into(), expiry).unwrap();
        let result = handle.eligible_token(&slot, &subject, now);
        assert_redacted(&format!("{handle:#?} {result:#?}"));
        assert_eq!(result.unwrap().expose_token(), MOCK_TOKEN);

        let http = endpoints.http_url();
        let ws = endpoints.websocket_url();
        for (http, ws, proxy) in [
            (
                http.to_owned(),
                ws.replace("127.0.0.1", "127.0.0.2"),
                ProxyPolicy::Direct,
            ),
            (
                http.to_owned(),
                ws.replace(WS_PATH, "/different-ws"),
                ProxyPolicy::Direct,
            ),
            (
                http.replace("/unused", "/different-http"),
                ws.to_owned(),
                ProxyPolicy::Direct,
            ),
            (http.to_owned(), ws.to_owned(), ProxyPolicy::System),
        ] {
            let changed = SessionEndpoints::parse(&http, &ws, proxy).unwrap();
            let other = TokenSubject::new(&changed, PRINCIPAL).unwrap();
            let mut other_slot = TokenSlot::new(&other);
            other_slot
                .replace(&other, MOCK_TOKEN.into(), expiry)
                .unwrap();
            denied(&other_slot, &other, now, Error::RouteMismatch);
            denied(&other_slot, &subject, now, Error::SubjectMismatch);
        }

        for principal in [PRINCIPAL.trim(), " fixture WSS account "] {
            let other = TokenSubject::new(endpoints, principal).unwrap();
            denied(&slot, &other, now, Error::SubjectMismatch);
        }
        for time in [expires_at, expires_at + tick] {
            denied(&slot, &subject, time, Error::Expired);
        }
        slot.replace(&original, MOCK_TOKEN.into(), TokenExpiry::Unknown)
            .unwrap();
        denied(&slot, &subject, now, Error::UnknownExpiry);
    }
}
