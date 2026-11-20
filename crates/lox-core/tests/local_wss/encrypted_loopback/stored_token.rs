use super::*;
use lox_core::runtime::session_websocket::encrypted_loopback::{
    LoopbackStoredTokenError, StoredTokenReplyError, authenticate_loopback_with_token,
};
use std::time::SystemTime;

const TOKEN_MAC: &str = "dbddb87d14573b91d6dc63b12edf163b8dad437cf3389d4d2cdf08495806e57f";
const OTHER_TOKEN: &str = "different.synthetic.jwt";
const OTHER_MAC: &str = "b24c1f6e9ecf820a8d0396573437c40bb37d6b1f12f37d8d5dd31ea79efaaed9";

#[derive(Clone, Copy)]
pub(in super::super::super) enum Case {
    ReadyClose,
    ReadyDrop,
    RefusedToken,
    WrongEcho,
    DuplicateExpiry,
    ExpiredAck,
    CancelAwaitingAck,
}

pub(in super::super::super) const CASES: [(Case, &str); 7] = [
    (Case::ReadyClose, "stored-token-ready-owned-close"),
    (Case::ReadyDrop, "stored-token-ready-owned-drop"),
    (Case::RefusedToken, "stored-token-refused-no-fallback"),
    (Case::WrongEcho, "stored-token-wrong-echo-released"),
    (
        Case::DuplicateExpiry,
        "stored-token-duplicate-expiry-released",
    ),
    (Case::ExpiredAck, "stored-token-expired-ack-released"),
    (
        Case::CancelAwaitingAck,
        "stored-token-cancel-before-ack-released",
    ),
];

#[tokio::test]
async fn rejects_expired_unscoped_or_missing_token_without_dialing() {
    use LoopbackStoredTokenError::IneligibleToken;
    use SessionWebSocketTokenError::{Empty, Expired, SubjectMismatch, UnknownExpiry};

    let http = probe_listener();
    let ws = probe_listener();
    let addr = ws.local_addr().unwrap();
    let endpoints = SessionEndpoints::parse(
        &format!("http://{}/unused", http.local_addr().unwrap()),
        &format!("wss://{addr}{WS_PATH}"),
        ProxyPolicy::Direct,
    )
    .unwrap();
    let alternate = SessionEndpoints::parse(
        &format!("http://{}/different", http.local_addr().unwrap()),
        &format!("wss://{addr}{WS_PATH}"),
        ProxyPolicy::Direct,
    )
    .unwrap();
    let command = GetKey2Command::for_principal(PRINCIPAL).unwrap();
    let later = TokenExpiry::At(SystemTime::now() + Duration::from_secs(3600));
    for (route, principal, expiry, expected) in [
        (&endpoints, PRINCIPAL, None, Empty),
        (
            &endpoints,
            PRINCIPAL,
            Some(TokenExpiry::Unknown),
            UnknownExpiry,
        ),
        (
            &endpoints,
            PRINCIPAL,
            Some(TokenExpiry::At(UNIX_EPOCH)),
            Expired,
        ),
        (
            &endpoints,
            "other fixture user",
            Some(later),
            SubjectMismatch,
        ),
        (&alternate, PRINCIPAL, Some(later), SubjectMismatch),
    ] {
        let subject = TokenSubject::new(route, principal).unwrap();
        let mut slot = TokenSlot::new(&subject);
        if let Some(expiry) = expiry {
            slot.replace(&subject, TOKEN.to_owned(), expiry).unwrap();
        }
        let error = authenticate_loopback_with_token(
            &endpoints,
            &command,
            HashAlgorithm::Sha256,
            &slot,
            &[],
            &Cancellation::new(),
        )
        .await
        .unwrap_err();
        assert_eq!(error, IneligibleToken(expected));
    }
    assert_no_contact(&http);
    assert_no_contact(&ws);
}

pub(in super::super::super) async fn check(
    case: Case,
    acceptor: &TlsAcceptor,
    application_key: &RsaPrivateKey,
) {
    use self::Case::*;
    use LoopbackStoredTokenError as Error;
    use StoredTokenReplyError as Reply;

    let spki = application_key.to_public_key().to_public_key_der().unwrap();
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
        let command = GetKey2Command::for_principal(PRINCIPAL).unwrap();
        let cancellation = Cancellation::new();
        let subject = TokenSubject::new(&endpoints, PRINCIPAL).unwrap();
        let mut slot = TokenSlot::new(&subject);
        let token = if matches!(case, RefusedToken) {
            OTHER_TOKEN
        } else {
            TOKEN
        };
        let now = SystemTime::now();
        let TokenExpiry::At(epoch) = TokenExpiry::from_loxone_seconds(0).unwrap() else {
            unreachable!();
        };
        let expires = now.duration_since(epoch).unwrap().as_secs() + 7200;
        slot.replace(
            &subject,
            token.to_owned(),
            TokenExpiry::At(now + Duration::from_secs(3600)),
        )
        .unwrap();
        let (ready_tx, ready_rx) = oneshot::channel();
        let (owned_tx, owned_rx) = oneshot::channel();
        let server = async {
            let (peer, remote) = listener.accept().await.unwrap();
            assert_eq!(remote.ip(), address.ip());
            let tls = acceptor.accept(peer).await.unwrap();
            let mut socket = accept_hdr_async(tls, CheckUpgradeRequest(address))
                .await
                .unwrap();
            let (exchange, key, iv) = read_application_material(&mut socket, application_key).await;
            send_ll(&mut socket, &exchange, r#""""#).await;
            let challenge = decrypt_request(&read_text(&mut socket).await, &key, &iv);
            let (salt, actual) = challenge
                .strip_prefix("salt/")
                .unwrap()
                .split_once('/')
                .unwrap();
            assert_eq!(actual, format!("jdev/sys/getkey2/{ENCODED_PRINCIPAL}"));
            assert_salt(salt);
            send_ll(
                &mut socket,
                &encrypt_echo(actual, &key, &iv, false),
                CHALLENGE,
            )
            .await;
            let auth = decrypt_request(&read_text(&mut socket).await, &key, &iv);
            let (old, rest) = auth
                .strip_prefix("nextSalt/")
                .unwrap()
                .split_once('/')
                .unwrap();
            let (next, actual) = rest.split_once('/').unwrap();
            assert_eq!(old, salt);
            assert_ne!(old, next);
            assert_salt(next);
            let expected_mac = if matches!(case, RefusedToken) {
                OTHER_MAC
            } else {
                TOKEN_MAC
            };
            assert_eq!(
                actual,
                format!("authwithtoken/{expected_mac}/{ENCODED_PRINCIPAL}")
            );
            let echo = if matches!(case, WrongEcho) {
                "jdev/sys/authwithtoken"
            } else {
                "authwithtoken"
            };
            let echo = encrypt_echo(echo, &key, &iv, false);
            match case {
                RefusedToken => send_ll_with_status(&mut socket, &echo, 404, r#""""#).await,
                DuplicateExpiry => {
                    send_ll(
                        &mut socket,
                        &echo,
                        &format!(r#"{{"validUntil":{expires},"validUntil":1}}"#),
                    )
                    .await
                }
                ExpiredAck => send_ll(&mut socket, &echo, r#"{"validUntil":1}"#).await,
                CancelAwaitingAck => cancellation.cancel(),
                ReadyClose | ReadyDrop | WrongEcho => {
                    send_ll(
                        &mut socket,
                        &echo,
                        &format!(
                            r#"{{"validUntil":{expires},"unsecurePass":false,"tokenRights":1666}}"#
                        ),
                    )
                    .await;
                    if matches!(case, ReadyClose | ReadyDrop) {
                        ready_rx.await.unwrap();
                        assert!(socket.next().now_or_never().is_none());
                        owned_tx.send(()).unwrap();
                    }
                }
            }
            assert_wss_released(&mut socket).await;
        };
        let client = async {
            let result = authenticate_loopback_with_token(
                &endpoints,
                &command,
                HashAlgorithm::Sha256,
                &slot,
                spki.as_bytes(),
                &cancellation,
            )
            .await;
            let expected = match case {
                ReadyClose | ReadyDrop => {
                    let ready = result.unwrap();
                    assert_eq!(
                        TokenExpiry::At(ready.reported_expiry()),
                        TokenExpiry::from_loxone_seconds(expires).unwrap()
                    );
                    assert_eq!(format!("{ready:?}"), "ReadyLoopbackSession { .. }");
                    ready_tx.send(()).unwrap();
                    owned_rx.await.unwrap();
                    if matches!(case, ReadyClose) {
                        ready.close()
                    } else {
                        drop(ready)
                    }
                    return;
                }
                RefusedToken => Error::Reply(Reply::StatusNotAccepted),
                WrongEcho => Error::Reply(Reply::EchoMismatch),
                DuplicateExpiry => Error::Reply(Reply::InvalidValue),
                ExpiredAck => Error::Reply(Reply::Expired),
                CancelAwaitingAck => Error::Handshake(LoopbackJwtError::Cancelled),
            };
            let error = result.unwrap_err();
            assert_eq!(error, expected);
            let diagnostics = format!("{error:?}: {error}");
            for private in [PRINCIPAL, TOKEN, OTHER_TOKEN, TOKEN_MAC, OTHER_MAC, WS_PATH] {
                assert!(!diagnostics.contains(private));
            }
        };
        tokio::join!(server, client);
        assert_no_contact(&http);
        assert!(listener.accept().now_or_never().is_none());
    })
    .await
    .expect("stored-token WSS transcript or socket release exceeded fixture deadline");
}
