//! Direct, unauthenticated WebSocket upgrade and connection ownership.

use super::Cancellation;
use super::session_endpoints::{ProxyPolicy, SessionEndpoints};
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::{Instant, sleep_until};
use tokio_tungstenite::tungstenite::Error as WebSocketError;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, client_async_tls};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

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

/// The sole owner of an upgraded, unauthenticated connection. No frames are
/// exposed or driven after the upgrade, and there is no session worker.
#[must_use = "dropping the WebSocket handle releases the connection"]
pub struct SessionWebSocket {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
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
        let deadline = Instant::now() + CONNECT_TIMEOUT;
        let attempt = async {
            // Dial numeric addresses without DNS, preserving IPv6 without URI brackets.
            let tcp = match address {
                Some(ip) => TcpStream::connect(SocketAddr::new(ip, port)).await,
                None => TcpStream::connect((host, port)).await,
            }
            .map_err(|_| Error::Connect)?;
            let (socket, _) = client_async_tls(route.as_str(), tcp)
                .await
                .map_err(|error| match error {
                    WebSocketError::Http(response) if response.status().is_redirection() => {
                        Error::Redirect
                    }
                    _ => Error::Upgrade,
                })?;
            Ok(Self { socket })
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

    /// Releases the owned connection immediately, as dropping the handle does.
    /// Does not send a Close frame or wait for a peer close handshake.
    pub fn close(self) {
        drop(self.socket);
    }
}

impl fmt::Debug for SessionWebSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionWebSocket").finish_non_exhaustive()
    }
}
