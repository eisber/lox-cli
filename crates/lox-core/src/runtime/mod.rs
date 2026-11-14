//! Ordered HTTP clock observations, with explicit validation and resource lifetime.
//!
//! Requires a Tokio executor with I/O and time enabled, or the explicitly created
//! `ClockDriver` (`runtime-blocking`). There are no subscriptions or callbacks
//! for clock values. Diagnostic callbacks are opt-in, synchronous, payload-free,
//! and must not block or panic.

mod clock;
mod model;

#[cfg(feature = "runtime-blocking")]
mod blocking;
#[cfg(feature = "runtime-blocking")]
pub use blocking::ClockDriver;
pub use clock::ClockClient;
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
