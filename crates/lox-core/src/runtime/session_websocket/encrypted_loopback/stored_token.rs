use super::{
    AuthWireError, EncryptedSocket, LoopbackJwtError, check_active, check_login_request,
    encrypted_challenge, within_login_budget,
};
use crate::authentication::HashKey;
use crate::runtime::Cancellation;
use crate::runtime::auth_challenge_envelope::GetKey2Command;
use crate::runtime::auth_reply_envelope::{EnvelopeError, admit_value};
use crate::runtime::auth_reply_framing::AuthReplyText;
use crate::runtime::login_challenge::{HashAlgorithm, LoginChallenge};
use crate::runtime::session_endpoints::SessionEndpoints;
use crate::runtime::session_token::{TokenExpiry, TokenSlot, TokenSubject};
use crate::runtime::session_websocket::{
    CONNECT_TIMEOUT, SessionWebSocketTokenError, scoped_token_access,
};
use hmac::Hmac;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use sha1::Sha1;
use sha2::Sha256;
use std::{fmt, time::SystemTime};
use tokio::time::Instant;

const TOKEN_CONTROL: &str = "authwithtoken";
const MAX_TOKEN_BYTES: usize = 4096;
const MAX_REPLY_VALUE_BYTES: usize = 8 * 1024;

/// A refused acknowledgment, without response data or decode details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoredTokenReplyError {
    InvalidEnvelope,
    EchoMismatch,
    StatusNotAccepted,
    InvalidValue,
    InvalidExpiry,
    Expired,
}

impl fmt::Display for StoredTokenReplyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidEnvelope => "invalid stored-token reply envelope",
            Self::EchoMismatch => "stored-token reply control mismatch",
            Self::StatusNotAccepted => "stored-token reply status was not accepted",
            Self::InvalidValue => "invalid stored-token reply value",
            Self::InvalidExpiry => "stored-token reply expiry is out of range",
            Self::Expired => "stored-token reply expiry has passed",
        })
    }
}

impl std::error::Error for StoredTokenReplyError {}

/// A loopback stored-token refusal without secrets, routes or backend diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoopbackStoredTokenError {
    Handshake(LoopbackJwtError),
    IneligibleToken(SessionWebSocketTokenError),
    TokenTooLarge,
    ResponseDerivationFailed,
    Wire(AuthWireError),
    Reply(StoredTokenReplyError),
}

impl From<LoopbackJwtError> for LoopbackStoredTokenError {
    fn from(error: LoopbackJwtError) -> Self {
        Self::Handshake(error)
    }
}

impl fmt::Display for LoopbackStoredTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Handshake(error) => fmt::Display::fmt(error, f),
            Self::IneligibleToken(error) => fmt::Display::fmt(error, f),
            Self::TokenTooLarge => f.write_str("stored loopback token exceeds the local limit"),
            Self::ResponseDerivationFailed => {
                f.write_str("stored-token response derivation failed")
            }
            Self::Wire(error) => write!(f, "stored-token exchange failed: {error}"),
            Self::Reply(error) => write!(f, "stored-token acknowledgment refused: {error}"),
        }
    }
}

impl std::error::Error for LoopbackStoredTokenError {}

/// The owned connection after a successful synthetic `authwithtoken` exchange.
///
/// There is no worker, refresh, automatic reauthentication or public frame I/O.
/// The session does not retain the token, its HMAC or the challenge value.
#[must_use = "dropping the ready loopback session releases the connection"]
pub struct ReadyLoopbackSession {
    session: EncryptedSocket,
    reported_expiry: SystemTime,
}

impl ReadyLoopbackSession {
    /// Peer-reported expiry, not a local token-slot update or a liveness check.
    pub fn reported_expiry(&self) -> SystemTime {
        self.reported_expiry
    }

    /// Release the socket without sending a WebSocket close frame.
    pub fn close(self) {
        self.session.connection.close();
    }
}

impl fmt::Debug for ReadyLoopbackSession {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ReadyLoopbackSession")
            .finish_non_exhaustive()
    }
}

/// Authenticate a stored token against the direct verified-WSS loopback dialect.
///
/// The route and original account must match the slot, whose expiry must be in
/// the future before dialing, before sending, and when accepting the reply.
/// Each attempt uses a new encrypted challenge and an explicitly permitted
/// algorithm; only the token HMAC is sent. No plaintext or password fallback
/// is available. The separate RSA SPKI is caller-supplied, not discovered.
///
/// One ten-second deadline and sticky cancellation cover the handshake until
/// the handle is returned. Future drop or any refusal releases the connection.
/// This admits only the synthetic profile, not a firmware authentication claim.
pub async fn authenticate_loopback_with_token(
    endpoints: &SessionEndpoints,
    command: &GetKey2Command,
    permitted_algorithm: HashAlgorithm,
    slot: &TokenSlot,
    application_spki: &[u8],
    cancellation: &Cancellation,
) -> Result<ReadyLoopbackSession, LoopbackStoredTokenError> {
    use LoopbackStoredTokenError as Error;

    let deadline = Instant::now() + CONNECT_TIMEOUT;
    check_active(cancellation, deadline)?;
    check_login_request(endpoints, command)?;
    let principal = command.clone_principal();
    let subject = TokenSubject::new(endpoints, &principal)
        .map_err(|_| Error::IneligibleToken(SessionWebSocketTokenError::SubjectMismatch))?;
    let access =
        scoped_token_access(slot, &subject, SystemTime::now()).map_err(Error::IneligibleToken)?;
    if access.expose_token().len() > MAX_TOKEN_BYTES {
        return Err(Error::TokenTooLarge);
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
        let token = session
            .connection
            .eligible_token(slot, &subject, SystemTime::now())
            .map_err(Error::IneligibleToken)?;
        let mac = token_mac(&challenge, token.expose_token())?;
        let request = session.encrypt_next(&format!(
            "{TOKEN_CONTROL}/{mac}/{}",
            command.encoded_principal()
        ))?;
        check_active(cancellation, deadline)?;
        // Expensive preparation must not grant an expired slot permission to send.
        session
            .connection
            .eligible_token(slot, &subject, SystemTime::now())
            .map_err(Error::IneligibleToken)?;
        let reported_expiry = session
            .connection
            .exchange_auth_reply(request, |reply| {
                admit_ack(reply, |control| {
                    session.channel.echoes(control, TOKEN_CONTROL)
                })
            })
            .await
            .map_err(Error::Wire)?
            .map_err(Error::Reply)?;
        session
            .connection
            .eligible_token(slot, &subject, SystemTime::now())
            .map_err(Error::IneligibleToken)?;
        if reported_expiry <= SystemTime::now() {
            return Err(Error::Reply(StoredTokenReplyError::Expired));
        }
        Ok(ReadyLoopbackSession {
            session,
            reported_expiry,
        })
    };
    within_login_budget(attempt, cancellation, deadline).await
}

fn token_mac(challenge: &LoginChallenge, token: &str) -> Result<String, LoopbackStoredTokenError> {
    let key = HashKey::from_hex(challenge.key_hex())
        .map_err(|_| LoopbackStoredTokenError::ResponseDerivationFailed)?;
    let parts = &[token.as_bytes()];
    Ok(match challenge.algorithm() {
        HashAlgorithm::Sha1 => hex::encode(key.hmac_bytes::<Hmac<Sha1>>(parts)),
        HashAlgorithm::Sha256 => hex::encode(key.hmac_bytes::<Hmac<Sha256>>(parts)),
    })
}

fn admit_ack(
    reply: &AuthReplyText<'_>,
    matches_control: impl FnOnce(&str) -> bool,
) -> Result<SystemTime, StoredTokenReplyError> {
    use StoredTokenReplyError as Error;

    let value = admit_value(reply, matches_control).map_err(|error| match error {
        EnvelopeError::InvalidEnvelope => Error::InvalidEnvelope,
        EnvelopeError::EchoMismatch => Error::EchoMismatch,
        EnvelopeError::StatusNotAccepted => Error::StatusNotAccepted,
    })?;
    if value.len() > MAX_REPLY_VALUE_BYTES {
        return Err(Error::InvalidValue);
    }
    let mut json = serde_json::Deserializer::from_slice(value);
    let seconds = json
        .deserialize_map(ExpiryVisitor)
        .and_then(|seconds| json.end().map(|()| seconds))
        .map_err(|_| Error::InvalidValue)?;
    match TokenExpiry::from_loxone_seconds(seconds) {
        Ok(TokenExpiry::At(at)) => Ok(at),
        Ok(TokenExpiry::Unknown) | Err(_) => Err(Error::InvalidExpiry),
    }
}

struct ExpiryVisitor;

impl<'de> Visitor<'de> for ExpiryVisitor {
    type Value = u64;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("a stored-token acknowledgment value")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<u64, A::Error> {
        let (mut expiry, mut rights, mut unsecure) = (None, false, false);
        while let Some(name) = map.next_key::<String>()? {
            match name.as_str() {
                "validUntil" if expiry.is_none() => expiry = Some(map.next_value()?),
                "tokenRights" if !rights => {
                    map.next_value::<u32>()?;
                    rights = true;
                }
                "unsecurePass" if !unsecure => {
                    map.next_value::<bool>()?;
                    unsecure = true;
                }
                _ => {
                    return Err(de::Error::custom(
                        "unexpected or repeated acknowledgment field",
                    ));
                }
            }
        }
        expiry.ok_or_else(|| de::Error::missing_field("validUntil"))
    }
}
