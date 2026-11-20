//! Read-only HTTP clock/control observations and provisional session components.
//!
//! HTTP observations require a Tokio executor with I/O and time enabled, or the
//! explicitly created `ClockDriver` / `ControlReadDriver` (`runtime-blocking`).
//! There are no subscriptions. Ordinary diagnostic callbacks are opt-in and
//! payload-free. The explicit CLI legacy policy separately permits raw body
//! observation. All observers are synchronous and must not block or panic.

mod clock;
mod control_read;
mod http;
mod model;

// Provisional Rust session components; no foreign ABI.
#[doc(hidden)]
pub mod auth_path_component;
#[doc(hidden)]
pub mod login_challenge;
#[doc(hidden)]
pub mod session_endpoints;
#[doc(hidden)]
pub mod session_http;
#[doc(hidden)]
pub mod session_token;
#[doc(hidden)]
pub mod token_command_profile;

#[cfg(feature = "runtime-blocking")]
mod blocking;
#[cfg(feature = "runtime-blocking")]
pub use blocking::ClockDriver;
#[cfg(feature = "runtime-blocking")]
pub use blocking::ControlReadDriver;
pub use clock::ClockClient;
pub use control_read::*;
pub use model::*;

use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;

/// A sticky, cloneable cancellation handle. Cancelling before submission works too.
#[derive(Clone, Debug)]
pub struct Cancellation(watch::Sender<bool>);

impl Cancellation {
    pub fn new() -> Self {
        Self(watch::channel(false).0)
    }

    pub fn cancel(&self) {
        self.0.send_replace(true);
    }

    pub fn is_cancelled(&self) -> bool {
        *self.0.borrow()
    }

    pub(crate) async fn cancelled(&self) {
        let mut receiver = self.0.subscribe();
        loop {
            if *receiver.borrow_and_update() {
                return;
            }
            if receiver.changed().await.is_err() {
                return;
            }
        }
    }
}

impl Default for Cancellation {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockEndpoint {
    Date,
    Time,
}

impl ClockEndpoint {
    pub fn path(self) -> &'static str {
        match self {
            Self::Date => "/jdev/sys/date",
            Self::Time => "/jdev/sys/time",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Diagnostic {
    Request(ClockEndpoint),
    Response {
        endpoint: ClockEndpoint,
        status: u16,
    },
    Body {
        endpoint: ClockEndpoint,
        bytes: usize,
    },
}

pub type DiagnosticObserver = Arc<dyn Fn(Diagnostic) + Send + Sync>;

/// Raw decoded text is available only through the explicitly selected legacy
/// CLI verbose policy, not through ordinary diagnostics or errors. The observer
/// runs synchronously outside state locks and must not block or panic.
pub type LegacyBodyObserver = Arc<dyn Fn(ClockEndpoint, &str) + Send + Sync>;

/// Response decoding is strict by default, independently of field validation.
#[derive(Clone, Default)]
pub enum ClockResponsePolicy {
    #[default]
    StrictJson,
    /// Reproduce the CLI's explicit `-vv` response.text()/JSON behavior,
    /// including replacement decoding and caller-owned raw-body diagnostics.
    /// Never select this automatically for library or foreign callers.
    LegacyCliVerbose { body_observer: LegacyBodyObserver },
}

impl fmt::Debug for ClockResponsePolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::StrictJson => "StrictJson",
            Self::LegacyCliVerbose { .. } => "LegacyCliVerbose { body_observer: [redacted] }",
        })
    }
}

/// Explicit connection settings; nothing is loaded from a home directory/context.
///
/// Defaults preserve the CLI's 10s per-attempt timeout, four attempts with
/// 200/400/800ms delays, and self-signed TLS acceptance. Empty credentials are
/// permitted. HTTP uses Basic credentials, never a WebSocket token.
#[derive(Clone)]
pub struct ClockOptions {
    pub host: String,
    pub user: String,
    pub password: String,
    pub verify_tls: bool,
    /// Honor ambient/system proxy configuration, as the CLI does. Disable for
    /// isolated loopback transports.
    pub use_system_proxy: bool,
    pub timeout: Duration,
    pub retry_delays: Vec<Duration>,
    /// Maximum async drain / owned-driver close wait. Default: one second.
    pub shutdown_timeout: Duration,
    pub user_agent: String,
    pub diagnostics: Option<DiagnosticObserver>,
    pub response_policy: ClockResponsePolicy,
}

impl ClockOptions {
    pub fn new(
        host: impl Into<String>,
        user: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        Self {
            host: host.into(),
            user: user.into(),
            password: password.into(),
            verify_tls: false,
            use_system_proxy: true,
            timeout: Duration::from_secs(10),
            retry_delays: [200, 400, 800].map(Duration::from_millis).into(),
            shutdown_timeout: Duration::from_secs(1),
            user_agent: concat!("lox-core/", env!("CARGO_PKG_VERSION")).into(),
            diagnostics: None,
            response_policy: ClockResponsePolicy::StrictJson,
        }
    }
}

impl fmt::Debug for ClockOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClockOptions")
            .field("connection", &"[redacted]")
            .field("verify_tls", &self.verify_tls)
            .field("use_system_proxy", &self.use_system_proxy)
            .field("timeout", &self.timeout)
            .field("retry_delays", &self.retry_delays)
            .field("shutdown_timeout", &self.shutdown_timeout)
            .field("diagnostics", &self.diagnostics.is_some())
            .field("response_policy", &self.response_policy)
            .finish()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CancelReason {
    Requested,
    Shutdown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportFailure {
    Timeout,
    Connect,
    Request,
    Body,
}

/// Sanitized, structured errors: no URLs, credentials, response bodies or
/// backend error chains are retained. The head owns rendering/categories.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClockError {
    InvalidHost,
    ClientBuild,
    Http {
        endpoint: ClockEndpoint,
        status: u16,
    },
    Transport {
        endpoint: ClockEndpoint,
        kind: TransportFailure,
    },
    Decode {
        endpoint: ClockEndpoint,
    },
    Cancelled(CancelReason),
    Closed,
    Busy,
    DriverStopped,
    ShutdownTimeout {
        active: usize,
    },
}

impl fmt::Display for ClockError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHost => f.write_str("invalid HTTP(S) clock host"),
            Self::ClientBuild => f.write_str("failed to build HTTP client"),
            Self::Http { endpoint, status } => write!(f, "HTTP {status}: {}", endpoint.path()),
            Self::Transport { kind, .. } => f.write_str(match kind {
                TransportFailure::Timeout => "error sending request (operation timed out)",
                TransportFailure::Connect => "error sending request (connection failure)",
                TransportFailure::Request => "error sending request",
                TransportFailure::Body => "error decoding response body",
            }),
            Self::Decode { .. } => f.write_str("error decoding response body"),
            Self::Cancelled(reason) => write!(f, "clock operation cancelled ({reason:?})"),
            Self::Closed => f.write_str("clock runtime is closed"),
            Self::Busy => f.write_str("clock driver already has an active operation"),
            Self::DriverStopped => f.write_str("clock driver stopped"),
            Self::ShutdownTimeout { active } => {
                write!(f, "clock shutdown timeout ({active} active)")
            }
        }
    }
}

impl std::error::Error for ClockError {}
