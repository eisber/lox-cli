//! Unauthenticated HTTP carrier for a provisionally admitted session route.

use super::TransportFailure;
use super::http::transport_kind;
use super::session_endpoints::{ProxyPolicy, SessionEndpoints};
use reqwest::{Client, Response, StatusCode, Url};
use std::fmt;
use std::time::Duration;

mod bootstrap;
pub use bootstrap::{
    BootstrapCapabilities, BootstrapError, BootstrapPayloadIssue, Capability, TlsStatus,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const BOOTSTRAP_PATH: &str = "/jdev/cfg/api";
const MAX_BOOTSTRAP_BODY: usize = 8 * 1024;

/// A failure with no endpoint, headers, body or backend error chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionHttpError {
    UnsupportedProxyPolicy(ProxyPolicy),
    ClientBuild,
    Transport(TransportFailure),
}

impl fmt::Display for SessionHttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProxyPolicy(policy) => {
                write!(f, "unsupported session HTTP proxy policy: {policy:?}")
            }
            Self::ClientBuild => f.write_str("failed to build session HTTP client"),
            Self::Transport(kind) => write!(f, "session HTTP transport failure: {kind:?}"),
        }
    }
}

impl std::error::Error for SessionHttpError {}

/// An async client bound to one admitted HTTP URL. It never uses the WebSocket
/// route and does not establish that either route is eligible for credentials.
pub struct SessionHttpClient {
    client: Client,
    route: Url,
}

impl SessionHttpClient {
    pub fn new(endpoints: &SessionEndpoints) -> Result<Self, SessionHttpError> {
        let policy = endpoints.proxy_policy();
        if policy != ProxyPolicy::Direct {
            return Err(SessionHttpError::UnsupportedProxyPolicy(policy));
        }

        let client = Client::builder()
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|_| SessionHttpError::ClientBuild)?;
        Ok(Self {
            client,
            route: endpoints.http_route().clone(),
        })
    }

    /// Performs an unauthenticated GET on the caller's Tokio runtime, returning
    /// the status (including valid 3xx) without reading the response body.
    ///
    /// If reqwest rejects a malformed `Location` before applying the no-follow
    /// policy, returns `SessionHttpError::Transport(TransportFailure::Request)`
    /// instead. Neither case attempts a redirect.
    pub async fn get_status(&self) -> Result<StatusCode, SessionHttpError> {
        self.send_get()
            .await
            .map(|response| response.status())
            .map_err(SessionHttpError::Transport)
    }

    /// Reads only the unauthenticated capabilities reported by `GET /jdev/cfg/api`.
    ///
    /// The admitted HTTP route must already have this exact path. No URL or
    /// credentials are derived from the WebSocket route or the response. The
    /// transport deadline covers the body read as well as the request.
    pub async fn bootstrap_preflight(&self) -> Result<BootstrapCapabilities, BootstrapError> {
        if self.route.path() != BOOTSTRAP_PATH {
            return Err(BootstrapError::WrongRoute);
        }

        let mut response = self.send_get().await.map_err(BootstrapError::Transport)?;
        let status = response.status();
        if status != StatusCode::OK {
            return Err(BootstrapError::HttpStatus(status));
        }

        let too_large = BootstrapError::Payload(BootstrapPayloadIssue::TooLarge);
        if response
            .content_length()
            .is_some_and(|len| len > MAX_BOOTSTRAP_BODY as u64)
        {
            return Err(too_large);
        }
        let mut body = Vec::with_capacity(MAX_BOOTSTRAP_BODY);
        while let Some(chunk) = response.chunk().await.map_err(|error| {
            BootstrapError::Transport(if error.is_timeout() {
                TransportFailure::Timeout
            } else {
                TransportFailure::Body
            })
        })? {
            if chunk.len() > MAX_BOOTSTRAP_BODY - body.len() {
                return Err(too_large);
            }
            body.extend_from_slice(&chunk);
        }

        bootstrap::decode(&body).map_err(BootstrapError::Payload)
    }

    async fn send_get(&self) -> Result<Response, TransportFailure> {
        self.client
            .get(self.route.clone())
            .send()
            .await
            .map_err(|error| transport_kind(&error))
    }
}

impl fmt::Debug for SessionHttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionHttpClient").finish_non_exhaustive()
    }
}

#[cfg(feature = "runtime-blocking")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BlockingBootstrapError {
    WorkerStart,
    ExecutorBuild,
    WorkerPanicked,
    Client(SessionHttpError),
    Preflight(BootstrapError),
}

#[cfg(feature = "runtime-blocking")]
impl fmt::Display for BlockingBootstrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkerStart => f.write_str("failed to start bootstrap HTTP worker"),
            Self::ExecutorBuild => f.write_str("failed to build bootstrap HTTP executor"),
            Self::WorkerPanicked => f.write_str("bootstrap HTTP worker panicked"),
            Self::Client(error) => fmt::Display::fmt(error, f),
            Self::Preflight(error) => fmt::Display::fmt(error, f),
        }
    }
}

#[cfg(feature = "runtime-blocking")]
impl std::error::Error for BlockingBootstrapError {}

/// Runs one preflight on an owned worker and joins it before returning.
///
/// The request retains the async carrier's ten-second timeout. There is no
/// caller cancellation or wall-clock bound on startup and cleanup: dropping
/// the runtime waits for blocking tasks such as an in-progress system DNS
/// lookup. Async callers should use the async client or offload this call.
#[cfg(feature = "runtime-blocking")]
pub fn bootstrap_preflight_blocking(
    endpoints: SessionEndpoints,
) -> Result<BootstrapCapabilities, BlockingBootstrapError> {
    use BlockingBootstrapError as Error;

    std::thread::Builder::new()
        .name("lox-bootstrap-preflight".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|_| Error::ExecutorBuild)?;
            runtime.block_on(async move {
                let client = SessionHttpClient::new(&endpoints).map_err(Error::Client)?;
                client.bootstrap_preflight().await.map_err(Error::Preflight)
            })
        })
        .map_err(|_| Error::WorkerStart)?
        .join()
        .map_err(|_| Error::WorkerPanicked)?
}
