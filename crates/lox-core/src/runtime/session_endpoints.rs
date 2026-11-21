//! Provisional, I/O-free endpoint admission for a future authenticated session.

use reqwest::Url;
use std::fmt;

/// Requested routing policy, not an installed proxy or a transport guarantee.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProxyPolicy {
    /// Do not use any proxy.
    Direct,
    /// Honor ambient proxy settings only when the transport explicitly supports it.
    System,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointKind {
    Http,
    WebSocket,
}

impl EndpointKind {
    fn accepts(self, scheme: &str) -> bool {
        let schemes = match self {
            Self::Http => ["http", "https"],
            Self::WebSocket => ["ws", "wss"],
        };
        schemes.iter().any(|name| scheme.eq_ignore_ascii_case(name))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointIssue {
    Scheme,
    Authority,
    Path,
    UserInfoQueryOrFragment,
}

/// A refusal without the input string or a URL parser error chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndpointError {
    pub endpoint: EndpointKind,
    pub issue: EndpointIssue,
}

impl fmt::Display for EndpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let endpoint = match self.endpoint {
            EndpointKind::Http => "HTTP",
            EndpointKind::WebSocket => "WebSocket",
        };
        let reason = match self.issue {
            EndpointIssue::Scheme => "an explicit scheme for this protocol is required",
            EndpointIssue::Authority => "invalid host or port",
            EndpointIssue::Path => "unsupported path syntax",
            EndpointIssue::UserInfoQueryOrFragment => {
                "user information, queries and fragments are not allowed"
            }
        };
        write!(f, "invalid {endpoint} endpoint: {reason}")
    }
}

impl std::error::Error for EndpointError {}

/// Explicit HTTP and WebSocket routes. Construction neither couples their
/// authorities nor establishes that either may receive credentials.
#[derive(Clone)]
pub struct SessionEndpoints {
    http: Url,
    websocket: Url,
    proxy_policy: ProxyPolicy,
}

impl SessionEndpoints {
    pub fn parse(
        http: &str,
        websocket: &str,
        proxy_policy: ProxyPolicy,
    ) -> Result<Self, EndpointError> {
        let validate = |input, endpoint| {
            parse_route(input, endpoint).map_err(|issue| EndpointError { endpoint, issue })
        };
        Ok(Self {
            http: validate(http, EndpointKind::Http)?,
            websocket: validate(websocket, EndpointKind::WebSocket)?,
            proxy_policy,
        })
    }

    pub fn http_url(&self) -> &str {
        self.http.as_str()
    }

    pub(super) fn http_route(&self) -> &Url {
        &self.http
    }

    pub fn websocket_url(&self) -> &str {
        self.websocket.as_str()
    }

    #[cfg(feature = "runtime-websocket")]
    pub(super) fn websocket_route(&self) -> &Url {
        &self.websocket
    }

    pub fn proxy_policy(&self) -> ProxyPolicy {
        self.proxy_policy
    }

    pub(super) fn same_scope(&self, other: &Self) -> bool {
        self.http_url() == other.http_url()
            && self.websocket_url() == other.websocket_url()
            && self.proxy_policy() == other.proxy_policy()
    }
}

impl fmt::Debug for SessionEndpoints {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SessionEndpoints")
            .field("routes", &"[redacted]")
            .field("proxy_policy", &self.proxy_policy)
            .finish()
    }
}

fn parse_route(input: &str, endpoint: EndpointKind) -> Result<Url, EndpointIssue> {
    let (scheme, address) = input.split_once("://").ok_or(EndpointIssue::Scheme)?;
    if !endpoint.accepts(scheme) {
        return Err(EndpointIssue::Scheme);
    }

    let (authority, path) = address.split_at(address.find('/').unwrap_or(address.len()));
    // Check raw syntax first: Url can discard empty userinfo, recover missing
    // authority slashes and normalize dot segments before we can reject them.
    if authority.contains('@') || address.contains(['?', '#']) {
        return Err(EndpointIssue::UserInfoQueryOrFragment);
    }
    if authority.is_empty()
        || authority.ends_with(':')
        || !authority
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b':' | b'[' | b']'))
    {
        return Err(EndpointIssue::Authority);
    }
    if path.contains("//")
        || path.split('/').any(|part| matches!(part, "." | ".."))
        || !path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'/' | b'-' | b'.' | b'_' | b'~'))
    {
        return Err(EndpointIssue::Path);
    }

    let url = Url::parse(input).map_err(|_| EndpointIssue::Authority)?;
    let host = url.host_str().ok_or(EndpointIssue::Authority)?;
    if url.port() == Some(0) {
        return Err(EndpointIssue::Authority);
    }
    if !host.starts_with('[') {
        let source_host = authority
            .split_once(':')
            .map_or(authority, |(host, _)| host);
        // For unbracketed hosts, permit DNS case folding but not alternate
        // numeric IP spellings that the URL parser would silently rewrite.
        if !host.eq_ignore_ascii_case(source_host) || !valid_host_labels(host) {
            return Err(EndpointIssue::Authority);
        }
    }
    Ok(url)
}

fn valid_host_labels(host: &str) -> bool {
    let host = host.strip_suffix('.').unwrap_or(host);
    host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
        })
}
