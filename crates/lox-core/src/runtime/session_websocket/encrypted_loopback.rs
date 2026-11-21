//! Encrypted JWT acquisition and stored-token login for a synthetic loopback dialect.

pub use super::auth_wire_error::AuthWireError;
use super::{CONNECT_TIMEOUT, MAX_GETKEY2_REQUEST_BYTES};
use super::{SessionWebSocket, SessionWebSocketError};
use crate::runtime::Cancellation;
use crate::runtime::auth_challenge_envelope::{
    AuthChallengeEnvelopeError, GetKey2Command, admit_matching_getkey2,
};
use crate::runtime::auth_path_component::AuthPathComponent;
use crate::runtime::auth_reply_envelope::admit_value;
use crate::runtime::jwt_grant_envelope::{JwtGrantEnvelopeError, admit_matching_getjwt};
use crate::runtime::jwt_grant_value::JwtGrantValue;
use crate::runtime::login_challenge::HashAlgorithm;
use crate::runtime::login_challenge::LoginChallenge;
use crate::runtime::login_response;
use crate::runtime::session_endpoints::{ProxyPolicy, SessionEndpoints};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding};
use base64::{Engine, engine::general_purpose::STANDARD};
use rand::{RngCore, SeedableRng, rngs::OsRng, rngs::StdRng};
use rsa::{Pkcs1v15Encrypt, RsaPublicKey, pkcs8::DecodePublicKey, traits::PublicKeyParts};
use std::{fmt, net::IpAddr};
use tokio::time::{Instant, sleep_until};

mod stored_token;
pub use stored_token::{
    LoopbackStoredTokenError, ReadyLoopbackSession, StoredTokenReplyError,
    authenticate_loopback_with_token,
};

const ENC_PREFIX: &str = "jdev/sys/enc/";
const MAX_SPKI_BYTES: usize = 1024;
const MAX_PASSWORD_BYTES: usize = 4096;
const SYNTHETIC_CLIENT: &str = "2/00010203-0405-4607-88090a0b0c0d0e0f/lox-core%20loopback";

/// Wire stage; errors do not include the associated command, reply or secrets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoginStage {
    KeyExchange,
    Challenge,
    Grant,
}

/// An acquisition refusal without routes, principals, crypto or parser details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoopbackJwtError {
    DirectLoopbackWssRequired,
    RequestTooLarge,
    InvalidApplicationKey,
    KeyPreparationFailed,
    Cancelled,
    Timeout,
    Connect(SessionWebSocketError),
    Wire(LoginStage, AuthWireError),
    KeyExchangeReply,
    CommandPreparationFailed,
    Challenge(AuthChallengeEnvelopeError),
    AlgorithmMismatch,
    ResponseDerivationFailed,
    Grant(JwtGrantEnvelopeError),
}

impl fmt::Display for LoopbackJwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::DirectLoopbackWssRequired => {
                "encrypted loopback login requires direct numeric loopback WSS"
            }
            Self::RequestTooLarge => "encrypted loopback login request exceeds the local limit",
            Self::InvalidApplicationKey => "invalid loopback RSA application key",
            Self::KeyPreparationFailed => "loopback application key preparation failed",
            Self::Cancelled => "encrypted loopback login cancelled",
            Self::Timeout => "encrypted loopback login timed out",
            Self::Connect(error) => {
                return write!(f, "encrypted loopback connection refused: {error}");
            }
            Self::Wire(stage, error) => {
                return write!(f, "encrypted loopback {stage:?} exchange failed: {error}");
            }
            Self::KeyExchangeReply => "encrypted loopback key exchange reply refused",
            Self::CommandPreparationFailed => "encrypted loopback command preparation failed",
            Self::Challenge(error) => {
                return write!(f, "encrypted loopback challenge refused: {error}");
            }
            Self::AlgorithmMismatch => {
                "loopback challenge does not match the permitted hash algorithm"
            }
            Self::ResponseDerivationFailed => "encrypted loopback response derivation failed",
            Self::Grant(error) => {
                return write!(f, "encrypted loopback token grant refused: {error}");
            }
        })
    }
}

impl std::error::Error for LoopbackJwtError {}

/// Run key exchange, encrypted `getkey2` and encrypted `getjwt` on one verified
/// WSS socket using only the local synthetic dialect, including its fixed
/// mock client identity. `application_spki` is a separately supplied RSA public
/// key in SPKI DER; TLS does not discover or authenticate that application key.
///
/// The socket is released on every outcome and when the future is dropped. A
/// single ten-second deadline covers preparation and all I/O. Crypto is
/// synchronous; cancellation does not interrupt an individual computation.
/// There is no worker, fallback, token installation, JWT verification or
/// authenticated handle. Credential and key buffers are not promised zeroized.
pub async fn acquire_loopback_jwt(
    endpoints: &SessionEndpoints,
    command: &GetKey2Command,
    permitted_algorithm: HashAlgorithm,
    password: &str,
    application_spki: &[u8],
    cancellation: &Cancellation,
) -> Result<JwtGrantValue, LoopbackJwtError> {
    use LoopbackJwtError as Error;

    let deadline = Instant::now() + CONNECT_TIMEOUT;
    check_active(cancellation, deadline)?;
    check_login_request(endpoints, command)?;
    if password.len() > MAX_PASSWORD_BYTES {
        return Err(Error::RequestTooLarge);
    }
    let attempt = async {
        let (mut session, challenge) = encrypted_challenge(
            endpoints,
            command,
            permitted_algorithm,
            application_spki,
            cancellation,
            deadline,
        )
        .await?;
        let response = login_response::derive(&challenge, &command.clone_principal(), password)
            .map_err(|_| Error::ResponseDerivationFailed)?;
        let grant = session.encrypt_next(&format!(
            "jdev/sys/getjwt/{}/{}/{SYNTHETIC_CLIENT}",
            response.to_lower_hex(),
            command.encoded_principal(),
        ))?;
        check_active(cancellation, deadline)?;
        session
            .connection
            .exchange_auth_reply(grant, |reply| {
                admit_matching_getjwt(reply, &response, |wire, logical| {
                    session.channel.echoes(wire, logical)
                })
            })
            .await
            .map_err(|error| Error::Wire(LoginStage::Grant, error))?
            .map_err(Error::Grant)
    };
    within_login_budget(attempt, cancellation, deadline).await
}

fn check_login_request(
    endpoints: &SessionEndpoints,
    command: &GetKey2Command,
) -> Result<(), LoopbackJwtError> {
    use LoopbackJwtError as Error;

    let route = endpoints.websocket_route();
    let loopback = route
        .host_str()
        .and_then(|host| host.trim_matches(['[', ']']).parse::<IpAddr>().ok())
        .is_some_and(|address| address.is_loopback());
    if route.scheme() != "wss" || !loopback || endpoints.proxy_policy() != ProxyPolicy::Direct {
        return Err(Error::DirectLoopbackWssRequired);
    }
    if command.as_str().len() > MAX_GETKEY2_REQUEST_BYTES {
        return Err(Error::RequestTooLarge);
    }
    Ok(())
}

struct EncryptedSocket {
    connection: SessionWebSocket,
    channel: Channel,
    rng: StdRng,
    last_salt: String,
}

impl EncryptedSocket {
    fn encrypt_next(&mut self, command: &str) -> Result<String, LoopbackJwtError> {
        let next_salt = salt(&mut self.rng);
        if next_salt == self.last_salt {
            return Err(LoopbackJwtError::CommandPreparationFailed);
        }
        let encrypted = self.channel.encrypt_command(&format!(
            "nextSalt/{}/{next_salt}/{command}",
            self.last_salt
        ))?;
        self.last_salt = next_salt;
        Ok(encrypted)
    }
}

async fn encrypted_challenge(
    endpoints: &SessionEndpoints,
    command: &GetKey2Command,
    permitted_algorithm: HashAlgorithm,
    application_spki: &[u8],
    cancellation: &Cancellation,
    deadline: Instant,
) -> Result<(EncryptedSocket, LoginChallenge), LoopbackJwtError> {
    use LoopbackJwtError as Error;

    let public_key = parse_application_key(application_spki)?;
    let mut rng = StdRng::from_rng(OsRng).map_err(|_| Error::KeyPreparationFailed)?;
    let channel = Channel::random(&mut rng);
    let key_command = channel.key_command(&public_key, &mut rng)?;
    check_active(cancellation, deadline)?;

    let mut connection = SessionWebSocket::connect_auth(endpoints, cancellation, deadline)
        .await
        .map_err(|error| match error {
            SessionWebSocketError::Cancelled => Error::Cancelled,
            SessionWebSocketError::Timeout => Error::Timeout,
            error => Error::Connect(error),
        })?;
    if !connection.has_verified_tls() {
        return Err(Error::DirectLoopbackWssRequired);
    }
    check_active(cancellation, deadline)?;
    let accepted = connection
        .exchange_auth_reply(key_command.clone(), |reply| {
            admit_value(reply, |control| control == key_command).is_ok_and(|value| value == b"\"\"")
        })
        .await
        .map_err(|error| Error::Wire(LoginStage::KeyExchange, error))?;
    if !accepted {
        return Err(Error::KeyExchangeReply);
    }

    let initial_salt = salt(&mut rng);
    let challenge_command =
        channel.encrypt_command(&format!("salt/{initial_salt}/{}", command.as_str()))?;
    check_active(cancellation, deadline)?;
    let challenge = connection
        .exchange_auth_reply(challenge_command, |reply| {
            admit_matching_getkey2(reply, |wire| channel.echoes(wire, command.as_str()))
        })
        .await
        .map_err(|error| Error::Wire(LoginStage::Challenge, error))?
        .map_err(Error::Challenge)?;
    check_active(cancellation, deadline)?;
    if challenge.algorithm() != permitted_algorithm {
        return Err(Error::AlgorithmMismatch);
    }
    Ok((
        EncryptedSocket {
            connection,
            channel,
            rng,
            last_salt: initial_salt,
        },
        challenge,
    ))
}

async fn within_login_budget<T, E: From<LoopbackJwtError>>(
    attempt: impl Future<Output = Result<T, E>>,
    cancellation: &Cancellation,
    deadline: Instant,
) -> Result<T, E> {
    let result = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(LoopbackJwtError::Cancelled.into()),
        _ = sleep_until(deadline) => return Err(LoopbackJwtError::Timeout.into()),
        result = attempt => result,
    };
    check_active(cancellation, deadline)?;
    result
}

fn check_active(cancellation: &Cancellation, deadline: Instant) -> Result<(), LoopbackJwtError> {
    if cancellation.is_cancelled() {
        return Err(LoopbackJwtError::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(LoopbackJwtError::Timeout);
    }
    Ok(())
}

fn parse_application_key(spki: &[u8]) -> Result<RsaPublicKey, LoopbackJwtError> {
    use LoopbackJwtError as Error;

    if spki.len() > MAX_SPKI_BYTES {
        return Err(Error::InvalidApplicationKey);
    }
    let key = RsaPublicKey::from_public_key_der(spki).map_err(|_| Error::InvalidApplicationKey)?;
    if !(2048..=4096).contains(&key.n().bits()) {
        return Err(Error::InvalidApplicationKey);
    }
    Ok(key)
}

fn salt(rng: &mut StdRng) -> String {
    let mut bytes = [0; 16];
    rng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

struct Channel {
    key: [u8; 32],
    iv: [u8; 16],
}

impl Channel {
    fn random(rng: &mut StdRng) -> Self {
        let mut channel = Self {
            key: [0; 32],
            iv: [0; 16],
        };
        rng.fill_bytes(&mut channel.key);
        rng.fill_bytes(&mut channel.iv);
        channel
    }

    fn key_command(
        &self,
        key: &RsaPublicKey,
        rng: &mut StdRng,
    ) -> Result<String, LoopbackJwtError> {
        let material = format!("{}:{}", hex::encode(self.key), hex::encode(self.iv));
        let wrapped = key
            .encrypt(rng, Pkcs1v15Encrypt, material.as_bytes())
            .map_err(|_| LoopbackJwtError::KeyPreparationFailed)?;
        Ok(format!("jdev/sys/keyexchange/{}", STANDARD.encode(wrapped)))
    }

    fn encrypt_command(&self, salted: &str) -> Result<String, LoopbackJwtError> {
        // The pinned implementation uses a NUL terminator before PKCS7 padding.
        let mut text = salted.as_bytes().to_vec();
        text.push(0);
        let ciphertext = cbc::Encryptor::<aes::Aes256>::new((&self.key).into(), (&self.iv).into())
            .encrypt_padded_vec_mut::<block_padding::Pkcs7>(&text);
        let component = AuthPathComponent::encode(&STANDARD.encode(ciphertext))
            .map_err(|_| LoopbackJwtError::CommandPreparationFailed)?;
        Ok(format!("{ENC_PREFIX}{}", component.as_encoded_str()))
    }

    fn echoes(&self, wire: &str, expected: &str) -> bool {
        let Some(ciphertext) = wire
            .strip_prefix(ENC_PREFIX)
            .and_then(|b64| STANDARD.decode(b64).ok())
        else {
            return false;
        };
        let Ok(text) = cbc::Decryptor::<aes::Aes256>::new((&self.key).into(), (&self.iv).into())
            .decrypt_padded_vec_mut::<block_padding::Pkcs7>(&ciphertext)
        else {
            return false;
        };
        let Some(control) = text.strip_suffix(&[0]) else {
            return false;
        };
        let Ok(text) = std::str::from_utf8(control) else {
            return false;
        };
        let Some(rest) = text.strip_prefix("nextSalt/") else {
            return false;
        };
        let mut parts = rest.splitn(3, '/');
        matches!((parts.next(), parts.next(), parts.next()),
            (Some(old), Some(new), Some(control))
                if old != new && reply_salt(old) && reply_salt(new) && control == expected)
    }
}

fn reply_salt(salt: &str) -> bool {
    salt.len() == 8
        && salt
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}
