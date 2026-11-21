use super::*;
pub(in super::super) mod stored_token;
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding};
use base64::{Engine, engine::general_purpose::STANDARD};
use lox_core::runtime::jwt_grant_envelope::JwtGrantEnvelopeError;
use lox_core::runtime::jwt_grant_value::JwtGrantKey;
use lox_core::runtime::session_websocket::encrypted_loopback::{
    LoopbackJwtError, acquire_loopback_jwt,
};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, pkcs8::DecodePrivateKey, pkcs8::EncodePublicKey};

const PRINCIPAL: &str = "Fixture / caf\u{e9} %";
const PASSWORD: &str = "private fixture password";
const ENCODED_PRINCIPAL: &str = "Fixture%20%2F%20caf%C3%A9%20%25";
const MAC: &str = "dbe5fde29bd7925dea0b6574e5af0620ac540738ffe79945ac05366255a30985";
const TOKEN: &str = "opaque.synthetic.jwt";
const ENC: &str = "jdev/sys/enc/";
const CHALLENGE: &str = r#"{"key":"Aa01Bb23","salt":"fixture-only salt","hashAlg":"SHA256"}"#;

#[derive(Clone, Copy)]
pub(super) enum Case {
    Granted,
    WrongKeyExchangeAck,
    PlaintextChallengeEcho,
    ZeroPaddedChallengeEcho,
    AlgorithmMismatch,
    WrongGrantEcho,
    CancelAfterGrantSent,
    DropAfterGrantSent,
    StalledGrant,
}

pub(super) const CASES: [(Case, &str); 9] = [
    (
        Case::Granted,
        "encrypted-acquisition-exact-round-trip-released",
    ),
    (
        Case::WrongKeyExchangeAck,
        "encrypted-acquisition-key-ack-refused",
    ),
    (
        Case::PlaintextChallengeEcho,
        "encrypted-acquisition-no-plaintext-fallback",
    ),
    (
        Case::ZeroPaddedChallengeEcho,
        "encrypted-acquisition-no-padding-fallback",
    ),
    (
        Case::AlgorithmMismatch,
        "encrypted-acquisition-algorithm-refused-before-grant",
    ),
    (
        Case::WrongGrantEcho,
        "encrypted-acquisition-grant-echo-refused",
    ),
    (
        Case::CancelAfterGrantSent,
        "encrypted-acquisition-cancel-after-grant-released",
    ),
    (
        Case::DropAfterGrantSent,
        "encrypted-acquisition-drop-after-grant-released",
    ),
    (
        Case::StalledGrant,
        "encrypted-acquisition-stalled-grant-deadline-released",
    ),
];

pub(super) fn application_key() -> RsaPrivateKey {
    // Use a distinct fixture key from the TLS peer's loopback.key.
    RsaPrivateKey::read_pkcs8_pem_file(fixture_dir().join("wrong-ip.key"))
        .expect("read test RSA application key")
}

#[tokio::test]
async fn refuses_unqualified_transport_key_and_cancelled_input_before_dial() {
    use LoopbackJwtError as Error;

    let http = probe_listener();
    let ws = probe_listener();
    let http_route = format!("http://{}/unused", http.local_addr().unwrap());
    let route = format!("wss://{}{}", ws.local_addr().unwrap(), WS_PATH);
    let plaintext = format!("ws://{}{}", ws.local_addr().unwrap(), WS_PATH);
    let command = GetKey2Command::for_principal(PRINCIPAL).unwrap();
    let cancelled = Cancellation::new();
    cancelled.cancel();
    let active = Cancellation::new();
    for (wire, proxy, cancel, expected) in [
        (
            plaintext.as_str(),
            ProxyPolicy::Direct,
            &active,
            Error::DirectLoopbackWssRequired,
        ),
        (
            "wss://192.0.2.1/fixture",
            ProxyPolicy::Direct,
            &active,
            Error::DirectLoopbackWssRequired,
        ),
        (
            route.as_str(),
            ProxyPolicy::System,
            &active,
            Error::DirectLoopbackWssRequired,
        ),
        (
            route.as_str(),
            ProxyPolicy::Direct,
            &cancelled,
            Error::Cancelled,
        ),
        (
            route.as_str(),
            ProxyPolicy::Direct,
            &active,
            Error::InvalidApplicationKey,
        ),
    ] {
        let endpoints = SessionEndpoints::parse(&http_route, wire, proxy).unwrap();
        let error = acquire_loopback_jwt(
            &endpoints,
            &command,
            HashAlgorithm::Sha256,
            PASSWORD,
            &[],
            cancel,
        )
        .await
        .unwrap_err();
        assert_eq!(error, expected);
    }
    assert_no_contact(&http);
    assert_no_contact(&ws);
}

pub(super) async fn check(case: Case, acceptor: &TlsAcceptor, application_key: &RsaPrivateKey) {
    use self::Case::*;
    use LoopbackJwtError as Error;

    let spki = application_key.to_public_key().to_public_key_der().unwrap();
    let advertised_deadline = Duration::from_secs(10);
    let fixture_timeout = if matches!(case, StalledGrant) {
        advertised_deadline + FIXTURE_DEADLINE
    } else {
        FIXTURE_DEADLINE
    };
    timeout(fixture_timeout, async {
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
        let (grant_seen, wait_for_grant) = oneshot::channel();
        let server = async {
            let (peer, _) = listener.accept().await.unwrap();
            let tls = acceptor.accept(peer).await.unwrap();
            let mut socket = accept_hdr_async(tls, CheckUpgradeRequest(address)).await.unwrap();
            let (exchange, key, iv) = read_application_material(&mut socket, application_key).await;
            if matches!(case, WrongKeyExchangeAck) {
                send_ll(&mut socket, "dev/sys/keyexchange", r#""""#).await;
            } else {
                send_ll(&mut socket, &exchange, r#""""#).await;
                let challenge = decrypt_request(&read_text(&mut socket).await, &key, &iv);
                let (salt, actual) = challenge.strip_prefix("salt/").unwrap().split_once('/').unwrap();
                let expected = format!("jdev/sys/getkey2/{ENCODED_PRINCIPAL}");
                assert_eq!(actual, expected);
                assert_salt(salt);
                let echo = if matches!(case, PlaintextChallengeEcho) {
                    expected
                } else {
                    encrypt_echo(&expected, &key, &iv, matches!(case, ZeroPaddedChallengeEcho))
                };
                send_ll(&mut socket, &echo, CHALLENGE).await;
                if matches!(case, Granted | WrongGrantEcho | CancelAfterGrantSent | DropAfterGrantSent | StalledGrant) {
                    let grant = decrypt_request(&read_text(&mut socket).await, &key, &iv);
                    let (old, rest) = grant.strip_prefix("nextSalt/").unwrap().split_once('/').unwrap();
                    let (next, actual) = rest.split_once('/').unwrap();
                    assert_eq!(old, salt);
                    assert_ne!(old, next);
                    assert_salt(next);
                    let logical = format!("jdev/sys/getjwt/{MAC}");
                    assert_eq!(
                        actual,
                        format!("{logical}/{ENCODED_PRINCIPAL}/2/00010203-0405-4607-88090a0b0c0d0e0f/lox-core%20loopback"),
                    );
                    match case {
                        CancelAfterGrantSent => cancellation.cancel(),
                        DropAfterGrantSent => grant_seen.send(()).unwrap(),
                        StalledGrant => {}
                        _ => {
                            let echo = encrypt_echo(if matches!(case, WrongGrantEcho) { actual } else { &logical }, &key, &iv, false);
                            send_ll(&mut socket, &echo, &format!(r#"{{"token":"{TOKEN}","validUntil":123456,"key":""}}"#)).await;
                        }
                    }
                }
            }
            assert_wss_released(&mut socket).await;
        };
        let client = async {
            let algorithm = if matches!(case, AlgorithmMismatch) { HashAlgorithm::Sha1 } else { HashAlgorithm::Sha256 };
            let started = Instant::now();
            let exchange = acquire_loopback_jwt(
                &endpoints, &command, algorithm, PASSWORD, spki.as_bytes(), &cancellation,
            );
            if matches!(case, DropAfterGrantSent) {
                tokio::select! {
                    reply = exchange => panic!("exchange finished before its future was dropped: {reply:?}"),
                    seen = wait_for_grant => seen.unwrap(),
                }
                return;
            }
            let result = exchange.await;
            let expected = match case {
                Granted => {
                    let grant = result.unwrap();
                    assert_eq!(grant.token(), TOKEN);
                    assert_eq!(grant.reported_key(), JwtGrantKey::ReportedEmpty);
                    assert_eq!(grant.expiry(), TokenExpiry::from_loxone_seconds(123456).unwrap());
                    return;
                }
                WrongKeyExchangeAck => Error::KeyExchangeReply,
                PlaintextChallengeEcho | ZeroPaddedChallengeEcho => {
                    Error::Challenge(AuthChallengeEnvelopeError::EchoMismatch)
                }
                AlgorithmMismatch => Error::AlgorithmMismatch,
                WrongGrantEcho => Error::Grant(JwtGrantEnvelopeError::EchoMismatch),
                CancelAfterGrantSent => Error::Cancelled,
                DropAfterGrantSent => unreachable!("checked above"),
                StalledGrant => {
                    assert!(started.elapsed() >= advertised_deadline);
                    Error::Timeout
                }
            };
            let error = result.unwrap_err();
            assert_eq!(error, expected);
            let diagnostics = format!("{error:?}: {error}");
            for private in [PRINCIPAL, PASSWORD, MAC, TOKEN, WS_PATH] {
                assert!(!diagnostics.contains(private));
            }
        };
        tokio::join!(server, client);
        assert_no_contact(&http);
        assert!(listener.accept().now_or_never().is_none());
    })
    .await
    .expect("encrypted WSS transcript or socket release exceeded fixture deadline");
}

async fn read_text(socket: &mut WebSocketStream<TlsStream<TcpStream>>) -> String {
    let Some(Ok(Message::Text(text))) = socket.next().await else {
        panic!("expected exactly one application text message");
    };
    text
}

async fn read_application_material(
    socket: &mut WebSocketStream<TlsStream<TcpStream>>,
    application_key: &RsaPrivateKey,
) -> (String, [u8; 32], [u8; 16]) {
    let exchange = read_text(socket).await;
    let wrapped = STANDARD
        .decode(exchange.strip_prefix("jdev/sys/keyexchange/").unwrap())
        .unwrap();
    let unwrapped = application_key.decrypt(Pkcs1v15Encrypt, &wrapped).unwrap();
    let (key, iv) = std::str::from_utf8(&unwrapped)
        .unwrap()
        .split_once(':')
        .unwrap();
    let key = hex::decode(key).unwrap().try_into().unwrap();
    let iv = hex::decode(iv).unwrap().try_into().unwrap();
    (exchange, key, iv)
}

async fn send_ll(socket: &mut WebSocketStream<TlsStream<TcpStream>>, echo: &str, value: &str) {
    send_ll_with_status(socket, echo, 200, value).await;
}

async fn send_ll_with_status(
    socket: &mut WebSocketStream<TlsStream<TcpStream>>,
    echo: &str,
    status: u16,
    value: &str,
) {
    let body = format!(r#"{{"LL":{{"control":"{echo}","code":"{status}","value":{value}}}}}"#);
    let header = MessageHeader::new(
        MessageType::Text as u8,
        0,
        0,
        body.len().try_into().unwrap(),
    );
    socket
        .send(Message::Binary(header.to_wire_bytes().to_vec()))
        .await
        .unwrap();
    socket.send(Message::Text(body)).await.unwrap();
}

fn decrypt_request(wire: &str, key: &[u8; 32], iv: &[u8; 16]) -> String {
    let encoded = wire.strip_prefix(ENC).unwrap();
    assert!(!encoded.contains(['+', '/', '=']));
    let decoded = encoded
        .replace("%2B", "+")
        .replace("%2F", "/")
        .replace("%3D", "=");
    let cipher = STANDARD.decode(decoded).unwrap();
    let decrypted = cbc::Decryptor::<aes::Aes256>::new(key.into(), iv.into())
        .decrypt_padded_vec_mut::<block_padding::Pkcs7>(&cipher)
        .unwrap();
    String::from_utf8(decrypted.strip_suffix(&[0]).unwrap().to_vec()).unwrap()
}

fn encrypt_echo(control: &str, key: &[u8; 32], iv: &[u8; 16], zero_padded: bool) -> String {
    let plaintext = format!("nextSalt/0a0b0c0d/0e0f1011/{control}\0");
    let cipher = cbc::Encryptor::<aes::Aes256>::new(key.into(), iv.into());
    let ciphertext = if zero_padded {
        cipher.encrypt_padded_vec_mut::<block_padding::ZeroPadding>(plaintext.as_bytes())
    } else {
        cipher.encrypt_padded_vec_mut::<block_padding::Pkcs7>(plaintext.as_bytes())
    };
    format!("{ENC}{}", STANDARD.encode(ciphertext))
}

fn assert_salt(value: &str) {
    assert_eq!(value.len(), 32);
    assert_eq!(hex::encode(hex::decode(value).unwrap()), value);
}
