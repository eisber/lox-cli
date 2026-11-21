//! Status-only HTTP carrier for a provisionally admitted session route.

use super::TransportFailure;
use super::http::transport_kind;
use super::session_endpoints::{ProxyPolicy, SessionEndpoints};
use reqwest::{Client, StatusCode, Url};
use std::fmt;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

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
        self.client
            .get(self.route.clone())
            .send()
            .await
            .map(|response| response.status())
            .map_err(|error| SessionHttpError::Transport(transport_kind(&error)))
    }
}

impl fmt::Debug for SessionHttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionHttpClient").finish_non_exhaustive()
    }
}
