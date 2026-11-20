//! Direct, unauthenticated WebSocket upgrade and provisional challenge exchange.

use super::Cancellation;
use super::auth_challenge_envelope::{AuthChallengeEnvelopeError, GetKey2Command, admit_getkey2};
use super::auth_reply_framing::{
    AuthReplyFramingError, AuthReplyMessage, AuthReplyText, MAX_REPLY_BYTES,
    admit_exact_text_header, admit_exact_text_payload,
};
#[cfg(feature = "auth-hash")]
use super::login_challenge::HashAlgorithm;
use super::login_challenge::LoginChallenge;
#[cfg(feature = "auth-hash")]
use super::login_response::{self, LoginResponse};
use super::session_endpoints::{ProxyPolicy, SessionEndpoints};
use super::session_token::{TokenAccess, TokenLookup, TokenSlot, TokenSubject};
use futures_util::{SinkExt, StreamExt};
use std::fmt;
use std::net::{IpAddr, SocketAddr};
#[cfg(feature = "auth-hash")]
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::net::TcpStream;
use tokio::time::{Instant, sleep_until};
use tokio_tungstenite::tungstenite::{Error as WebSocketError, Message, protocol::WebSocketConfig};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, client_async_tls_with_config};

/// Opt-in encrypted acquisition against the local synthetic peer only.
#[cfg(feature = "runtime-websocket-encrypted-loopback")]
pub mod encrypted_loopback;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_GETKEY2_REQUEST_BYTES: usize = 1024;

/// A refusal or failure without routes, response data or backend error chains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionWebSocketError {
    UnsupportedProxyPolicy(ProxyPolicy),
    PlaintextRequiresLoopback,
    Cancelled,
    Timeout,
    Connect,
    Upgrade,
    Redirect,
}

impl fmt::Display for SessionWebSocketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProxyPolicy(policy) => {
                write!(f, "unsupported session WebSocket proxy policy: {policy:?}")
            }
            Self::PlaintextRequiresLoopback => {
                f.write_str("plaintext session WebSocket requires a numeric loopback address")
            }
            Self::Cancelled => f.write_str("session WebSocket connection cancelled"),
            Self::Timeout => f.write_str("session WebSocket connection timed out"),
            Self::Connect => f.write_str("session WebSocket TCP connection failed"),
            Self::Upgrade => f.write_str("session WebSocket TLS or upgrade failed"),
            Self::Redirect => f.write_str("session WebSocket redirect refused"),
        }
    }
}

impl std::error::Error for SessionWebSocketError {}

/// A token eligibility refusal without routes, principals, tokens or backend details.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionWebSocketTokenError {
    UnverifiedTransport,
    RouteMismatch,
    SubjectMismatch,
    Empty,
    Expired,
    UnknownExpiry,
}

impl fmt::Display for SessionWebSocketTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnverifiedTransport => "token eligibility requires verified WebSocket TLS",
            Self::RouteMismatch => "token subject route does not match the WebSocket connection",
            Self::SubjectMismatch => "token slot subject mismatch",
            Self::Empty => "token slot is empty",
            Self::Expired => "token has expired",
            Self::UnknownExpiry => "token expiry is unknown",
        })
    }
}

impl std::error::Error for SessionWebSocketTokenError {}

/// A pre-authentication refusal without principals, routes or wire data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GetKey2ExchangeError {
    VerifiedWssRequired,
    RequestTooLarge,
    Cancelled,
    Timeout,
    Connect(SessionWebSocketError),
    Send,
    Receive,
    Closed,
    ReplyTooLarge,
    Framing(AuthReplyFramingError),
    Envelope(AuthChallengeEnvelopeError),
}

impl fmt::Display for GetKey2ExchangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::VerifiedWssRequired => "getkey2 requires direct verified WebSocket TLS",
            Self::RequestTooLarge => "encoded getkey2 request is outside the local limit",
            Self::Cancelled => "getkey2 exchange cancelled",
            Self::Timeout => "getkey2 exchange timed out",
            Self::Send => "getkey2 request send failed",
            Self::Receive => "getkey2 reply receive failed",
            Self::Closed => "connection closed before getkey2 reply admission",
            Self::ReplyTooLarge => "getkey2 reply exceeds the WebSocket size limit",
            Self::Connect(error) => return write!(f, "getkey2 connection refused: {error}"),
            Self::Framing(error) => return write!(f, "getkey2 framing refused: {error}"),
            Self::Envelope(error) => return write!(f, "getkey2 reply refused: {error}"),
        };
        f.write_str(message)
    }
}

impl std::error::Error for GetKey2ExchangeError {}

mod auth_wire_error {
    use super::{AuthReplyFramingError, fmt};

    /// Payload-free failure during one bounded authentication request and reply.
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum AuthWireError {
        Send,
        Receive,
        Closed,
        ReplyTooLarge,
        Framing(AuthReplyFramingError),
    }

    impl fmt::Display for AuthWireError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                Self::Send => "authentication request send failed",
                Self::Receive => "authentication reply receive failed",
                Self::Closed => "connection closed before authentication reply admission",
                Self::ReplyTooLarge => "authentication reply exceeds the WebSocket size limit",
                Self::Framing(error) => {
                    return write!(f, "authentication reply framing refused: {error}");
                }
            })
        }
    }

    impl std::error::Error for AuthWireError {}
}

use auth_wire_error::AuthWireError;

impl GetKey2ExchangeError {
    fn from_auth_wire(error: AuthWireError) -> Self {
        match error {
            AuthWireError::Send => Self::Send,
            AuthWireError::Receive => Self::Receive,
            AuthWireError::Closed => Self::Closed,
            AuthWireError::ReplyTooLarge => Self::ReplyTooLarge,
            AuthWireError::Framing(error) => Self::Framing(error),
        }
    }
}

/// Retains the same verified connection and its admitted pre-auth challenge.
/// This provisional handle grants no frame I/O or authenticated session access.
#[must_use = "dropping the challenge handle releases the connection"]
pub struct PendingLoginChallenge {
    connection: SessionWebSocket,
    challenge: LoginChallenge,
    #[cfg(feature = "auth-hash")]
    principal: Arc<str>,
}

impl PendingLoginChallenge {
    /// Borrows the challenge while retaining ownership of its connection.
    pub fn challenge(&self) -> &LoginChallenge {
        &self.challenge
    }

    /// Consumes the verified pending exchange and computes a response for its
    /// original principal, without I/O. The caller must explicitly permit the
    /// algorithm; disagreement is refused before hashing the password.
    ///
    /// A refusal releases the connection. Cancellation is checked before and
    /// after derivation, but does not interrupt the synchronous hash. The
    /// password is not retained and no zeroization is promised.
    #[cfg(feature = "auth-hash")]
    pub fn prepare_response(
        self,
        permitted_algorithm: HashAlgorithm,
        password: &str,
        cancellation: &Cancellation,
    ) -> Result<PreparedLoginResponse, PrepareLoginResponseError> {
        use PrepareLoginResponseError as Error;

        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if self.challenge.algorithm() != permitted_algorithm {
            return Err(Error::AlgorithmMismatch);
        }
        let response = login_response::derive(&self.challenge, self.principal.as_ref(), password);
        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let response = response.map_err(|_| Error::DerivationFailed)?;
        Ok(PreparedLoginResponse {
            pending: self,
            response,
        })
    }

    /// Releases the connection immediately, without a WebSocket close handshake.
    pub fn close(self) {
        self.connection.close();
    }
}

impl fmt::Debug for PendingLoginChallenge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PendingLoginChallenge")
            .finish_non_exhaustive()
    }
}

/// Payload-free refusal to prepare a response; a refused socket is not reusable.
#[cfg(feature = "auth-hash")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrepareLoginResponseError {
    Cancelled,
    AlgorithmMismatch,
    DerivationFailed,
}

#[cfg(feature = "auth-hash")]
impl fmt::Display for PrepareLoginResponseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Cancelled => "login response preparation cancelled",
            Self::AlgorithmMismatch => {
                "login challenge algorithm does not match the permitted algorithm"
            }
            Self::DerivationFailed => "login response derivation failed",
        })
    }
}

#[cfg(feature = "auth-hash")]
impl std::error::Error for PrepareLoginResponseError {}

/// A provisional response and the same owned pre-authentication connection.
/// This grants no network or authenticated session access.
#[cfg(feature = "auth-hash")]
#[must_use = "dropping the prepared response releases the connection"]
pub struct PreparedLoginResponse {
    pending: PendingLoginChallenge,
    response: LoginResponse,
}

#[cfg(feature = "auth-hash")]
impl PreparedLoginResponse {
    /// Explicitly borrows the secret response. Do not log its contents.
    pub fn response(&self) -> &LoginResponse {
        &self.response
    }

    /// Releases the connection immediately, without a WebSocket close handshake.
    pub fn close(self) {
        self.pending.close();
    }
}

#[cfg(feature = "auth-hash")]
impl fmt::Debug for PreparedLoginResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedLoginResponse")
            .finish_non_exhaustive()
    }
}

/// The sole owner of an upgraded, unauthenticated connection. No frames are
/// exposed or driven by this handle's public methods; there is no session worker.
#[must_use = "dropping the WebSocket handle releases the connection"]
pub struct SessionWebSocket {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    endpoints: SessionEndpoints,
}

impl SessionWebSocket {
    /// Connects only to the canonical WebSocket route on a Tokio runtime with
    /// I/O and time enabled. `wss` uses the transport's default verified TLS.
    /// `ws` is permitted only for literal IPv4/IPv6 loopback addresses.
    ///
    /// Cancellation and the ten-second deadline cover dialing and the upgrade.
    /// Dropping the future releases any TCP/TLS stream it acquired. The
    /// runtime's system DNS lookup, if started, can finish independently, but
    /// cannot keep dialing after this future is dropped. Cancellation is not
    /// retained by the returned handle; the caller owns its lifetime.
    pub async fn connect(
        endpoints: &SessionEndpoints,
        cancellation: &Cancellation,
    ) -> Result<Self, SessionWebSocketError> {
        Self::connect_configured(endpoints, cancellation, None, None).await
    }

    async fn connect_configured(
        endpoints: &SessionEndpoints,
        cancellation: &Cancellation,
        config: Option<WebSocketConfig>,
        deadline: Option<Instant>,
    ) -> Result<Self, SessionWebSocketError> {
        use SessionWebSocketError as Error;

        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let policy = endpoints.proxy_policy();
        if policy != ProxyPolicy::Direct {
            return Err(Error::UnsupportedProxyPolicy(policy));
        }
        let route = endpoints.websocket_route();
        let host = route.host_str().ok_or(Error::Connect)?;
        let address = host.trim_matches(['[', ']']).parse::<IpAddr>().ok();
        if route.scheme() == "ws" && !address.is_some_and(|ip| ip.is_loopback()) {
            return Err(Error::PlaintextRequiresLoopback);
        }
        let port = route.port_or_known_default().ok_or(Error::Connect)?;
        let deadline = deadline.unwrap_or_else(|| Instant::now() + CONNECT_TIMEOUT);
        let attempt = async {
            // Dial numeric addresses without DNS, preserving IPv6 without URI brackets.
            let tcp = match address {
                Some(ip) => TcpStream::connect(SocketAddr::new(ip, port)).await,
                None => TcpStream::connect((host, port)).await,
            }
            .map_err(|_| Error::Connect)?;
            let (socket, _) = client_async_tls_with_config(route.as_str(), tcp, config, None)
                .await
                .map_err(|error| match error {
                    WebSocketError::Http(response) if response.status().is_redirection() => {
                        Error::Redirect
                    }
                    _ => Error::Upgrade,
                })?;
            Ok(Self {
                socket,
                endpoints: endpoints.clone(),
            })
        };
        let result = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(Error::Cancelled),
            _ = sleep_until(deadline) => return Err(Error::Timeout),
            result = attempt => result,
        };
        // A transport poll can complete while the deadline or cancellation becomes ready.
        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if Instant::now() >= deadline {
            return Err(Error::Timeout);
        }
        result
    }

    /// Open a fresh direct, verified `wss` connection and issue one encoded
    /// getkey2 text request. Read only the next binary-header/text pair.
    ///
    /// One ten-second deadline and sticky cancellation cover connect, send
    /// and reply on the caller's Tokio runtime. Failure or dropping this
    /// future releases the stream. No worker, retry or fallback is used,
    /// and the separate HTTP route is not contacted. TLS and WebSocket
    /// protocol control are distinct from this one application send.
    ///
    /// This is a provisional dialect for the pinned mock subset only, not a
    /// firmware compatibility or authentication claim.
    pub async fn getkey2(
        endpoints: &SessionEndpoints,
        command: &GetKey2Command,
        cancellation: &Cancellation,
    ) -> Result<PendingLoginChallenge, GetKey2ExchangeError> {
        use GetKey2ExchangeError as Error;

        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        if endpoints.websocket_route().scheme() != "wss" {
            return Err(Error::VerifiedWssRequired);
        }
        if command.as_str().len() > MAX_GETKEY2_REQUEST_BYTES {
            return Err(Error::RequestTooLarge);
        }

        let deadline = Instant::now() + CONNECT_TIMEOUT;
        let attempt = async {
            let mut connection = Self::connect_auth(endpoints, cancellation, deadline)
                .await
                .map_err(|error| match error {
                    SessionWebSocketError::Cancelled => Error::Cancelled,
                    SessionWebSocketError::Timeout => Error::Timeout,
                    error => Error::Connect(error),
                })?;
            if !connection.has_verified_tls() {
                return Err(Error::VerifiedWssRequired);
            }
            check_exchange_active(cancellation, deadline)?;

            let challenge = connection
                .exchange_auth_reply(command.as_str().to_owned(), |reply| {
                    admit_getkey2(reply, command)
                })
                .await
                .map_err(Error::from_auth_wire)?
                .map_err(Error::Envelope)?;
            Ok(PendingLoginChallenge {
                connection,
                challenge,
                #[cfg(feature = "auth-hash")]
                principal: command.clone_principal(),
            })
        };
        let result = tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(Error::Cancelled),
            _ = sleep_until(deadline) => return Err(Error::Timeout),
            result = attempt => result,
        };
        check_exchange_active(cancellation, deadline)?;
        result
    }

    async fn connect_auth(
        endpoints: &SessionEndpoints,
        cancellation: &Cancellation,
        deadline: Instant,
    ) -> Result<Self, SessionWebSocketError> {
        let config = WebSocketConfig {
            max_message_size: Some(MAX_REPLY_BYTES),
            max_frame_size: Some(MAX_REPLY_BYTES),
            ..WebSocketConfig::default()
        };
        Self::connect_configured(endpoints, cancellation, Some(config), Some(deadline)).await
    }

    async fn exchange_auth_reply<T>(
        &mut self,
        command: String,
        admit: impl FnOnce(&AuthReplyText<'_>) -> T,
    ) -> Result<T, AuthWireError> {
        use AuthWireError as Error;

        self.socket
            .send(Message::Text(command))
            .await
            .map_err(|_| Error::Send)?;
        let header = self.read_auth_message().await?;
        let size = admit_exact_text_header(reply_message(&header)).map_err(Error::Framing)?;
        let payload = self.read_auth_message().await?;
        let reply =
            admit_exact_text_payload(size, reply_message(&payload)).map_err(Error::Framing)?;
        Ok(admit(&reply))
    }

    async fn read_auth_message(&mut self) -> Result<Message, AuthWireError> {
        use AuthWireError as Error;

        match self.socket.next().await {
            Some(
                Ok(Message::Close(_))
                | Err(WebSocketError::ConnectionClosed | WebSocketError::AlreadyClosed),
            )
            | None => Err(Error::Closed),
            Some(Err(WebSocketError::Capacity(_))) => Err(Error::ReplyTooLarge),
            Some(Err(_)) => Err(Error::Receive),
            Some(Ok(message)) => Ok(message),
        }
    }

    fn has_verified_tls(&self) -> bool {
        // The private constructor only returns after the default direct
        // connector has completed the handshake; no custom connector is accepted.
        self.endpoints.websocket_route().scheme() == "wss"
            && matches!(self.socket.get_ref(), MaybeTlsStream::Rustls(_))
    }

    /// Borrows an existing token for this direct connection only. The returned
    /// access borrows both the handle and the slot. The admitted HTTP route is
    /// a scope label only; no HTTP trust is established or consulted.
    ///
    /// TLS verifies the WebSocket server name and certificate, not a device or
    /// user identity. The check does not probe liveness, authenticate, send
    /// frames or refresh the token. Expiry uses the supplied `now`, as in the
    /// slot's ordinary lookup. This is a provisional Rust-only API.
    pub fn eligible_token<'a>(
        &'a self,
        slot: &'a TokenSlot,
        subject: &TokenSubject<'_>,
        now: SystemTime,
    ) -> Result<TokenAccess<'a>, SessionWebSocketTokenError> {
        use SessionWebSocketTokenError as Error;

        if !self.has_verified_tls() {
            return Err(Error::UnverifiedTransport);
        }
        if !self.endpoints.same_scope(subject.endpoints()) {
            return Err(Error::RouteMismatch);
        }
        match slot.lookup(subject, now) {
            TokenLookup::Match(access) => Ok(access),
            TokenLookup::Empty => Err(Error::Empty),
            TokenLookup::SubjectMismatch => Err(Error::SubjectMismatch),
            TokenLookup::Expired => Err(Error::Expired),
            TokenLookup::UnknownExpiry => Err(Error::UnknownExpiry),
        }
    }

    /// Releases the owned connection immediately, as dropping the handle does.
    /// Does not send a Close frame or wait for a peer close handshake.
    pub fn close(self) {
        drop(self.socket);
    }
}

fn check_exchange_active(
    cancellation: &Cancellation,
    deadline: Instant,
) -> Result<(), GetKey2ExchangeError> {
    if cancellation.is_cancelled() {
        return Err(GetKey2ExchangeError::Cancelled);
    }
    if Instant::now() >= deadline {
        return Err(GetKey2ExchangeError::Timeout);
    }
    Ok(())
}

fn reply_message(message: &Message) -> AuthReplyMessage<'_> {
    match message {
        Message::Binary(bytes) => AuthReplyMessage::Binary(bytes),
        Message::Text(text) => AuthReplyMessage::Text(text),
        _ => AuthReplyMessage::Other,
    }
}

impl fmt::Debug for SessionWebSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionWebSocket").finish_non_exhaustive()
    }
}
