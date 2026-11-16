use super::http::{
    HostPolicy, HttpError, HttpEvent, HttpOptions, HttpRuntime, JsonPolicy, transport_kind,
};
use super::{
    CancelReason, Cancellation, ClockField, ClockOptions, EnvelopeState, ProtocolCode,
    TransportFailure,
};
use crate::control::{IdentifierError, validate_uuid};
use std::error::Error;
use std::{fmt, sync::Arc, time::Duration};

/// Shared LL wire primitive, not the clock's nonempty-value validation policy.
pub use super::ClockValue as ControlValue;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ControlReadCompatibility {
    #[default]
    Strict,
    /// Preserve opaque paths, deferred host errors and separate header/body
    /// timeout budgets. This does not validate/encode identifiers or harden URLs.
    LegacyCli,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlReadPhase {
    Headers,
    Body,
}

/// An actual legacy phase-budget expiry, not a fabricated reqwest error.
/// Available only through the explicit `ControlReadError::into_source` opt-in.
pub struct ControlReadPhaseTimeout {
    phase: ControlReadPhase,
    url: reqwest::Url,
    _elapsed: tokio::time::error::Elapsed,
}

impl ControlReadPhaseTimeout {
    pub(super) fn new(
        phase: ControlReadPhase,
        url: reqwest::Url,
        elapsed: tokio::time::error::Elapsed,
    ) -> Self {
        Self {
            phase,
            url,
            _elapsed: elapsed,
        }
    }

    pub fn phase(&self) -> ControlReadPhase {
        self.phase
    }

    /// Explicit legacy adapter disclosure; the URL can contain private data.
    pub fn request_url(&self) -> &reqwest::Url {
        &self.url
    }
}

impl fmt::Debug for ControlReadPhaseTimeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlReadPhaseTimeout")
            .field("phase", &self.phase)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for ControlReadPhaseTimeout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("operation timed out")
    }
}

impl std::error::Error for ControlReadPhaseTimeout {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlReadDiagnostic {
    Request,
    Response { status: u16 },
    Body { bytes: usize },
}

pub type ControlReadObserver = Arc<dyn Fn(ControlReadDiagnostic) + Send + Sync>;
pub type ControlReadBodyObserver = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Clone, Default)]
pub enum ControlReadResponsePolicy {
    #[default]
    StrictJson,
    /// Explicit legacy replacement-decoded response.text() and raw body access.
    LegacyCliVerbose {
        body_observer: ControlReadBodyObserver,
    },
}

impl fmt::Debug for ControlReadResponsePolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::StrictJson => "StrictJson",
            Self::LegacyCliVerbose { .. } => "LegacyCliVerbose { body_observer: [redacted] }",
        })
    }
}

/// Explicit primitive connection options. No resolution, context or persistence.
/// Defaults match ClockOptions' HTTP/timeouts/retry and shutdown settings.
#[derive(Clone)]
pub struct ControlReadOptions {
    pub host: String,
    pub user: String,
    pub password: String,
    pub verify_tls: bool,
    pub use_system_proxy: bool,
    pub timeout: Duration,
    pub retry_delays: Vec<Duration>,
    pub shutdown_timeout: Duration,
    pub user_agent: String,
    pub compatibility: ControlReadCompatibility,
    pub diagnostics: Option<ControlReadObserver>,
    pub response_policy: ControlReadResponsePolicy,
}

impl ControlReadOptions {
    pub fn new(
        host: impl Into<String>,
        user: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        let defaults = ClockOptions::new(host, user, password);
        Self {
            host: defaults.host,
            user: defaults.user,
            password: defaults.password,
            verify_tls: defaults.verify_tls,
            use_system_proxy: defaults.use_system_proxy,
            timeout: defaults.timeout,
            retry_delays: defaults.retry_delays,
            shutdown_timeout: defaults.shutdown_timeout,
            user_agent: defaults.user_agent,
            compatibility: ControlReadCompatibility::Strict,
            diagnostics: None,
            response_policy: ControlReadResponsePolicy::StrictJson,
        }
    }
}

impl fmt::Debug for ControlReadOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlReadOptions")
            .field("connection", &"[redacted]")
            .field("verify_tls", &self.verify_tls)
            .field("use_system_proxy", &self.use_system_proxy)
            .field("timeout", &self.timeout)
            .field("retry_delays", &self.retry_delays)
            .field("shutdown_timeout", &self.shutdown_timeout)
            .field("compatibility", &self.compatibility)
            .field("diagnostics", &self.diagnostics.is_some())
            .field("response_policy", &self.response_policy)
            .finish()
    }
}

#[derive(Clone, PartialEq)]
pub struct ControlObservation {
    pub envelope: EnvelopeState,
    pub value: ControlValue,
    pub code: ProtocolCode,
    pub lowercase_code: ProtocolCode,
}

impl fmt::Debug for ControlObservation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlObservation")
            .field("envelope", &self.envelope)
            .field("value", &"[redacted]")
            .field("code", &"[redacted]")
            .field("lowercase_code", &"[redacted]")
            .finish()
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ControlState {
    pub value: String,
    pub code: Option<String>,
}

impl fmt::Debug for ControlState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ControlState { value: [redacted], code: [redacted] }")
    }
}

impl ControlObservation {
    fn decode(value: serde_json::Value) -> Self {
        let fields = ClockField::decode(value);
        Self {
            envelope: fields.envelope,
            value: fields.value,
            code: fields.code,
            lowercase_code: fields.lowercase_code,
        }
    }

    /// Explicit compatibility projection, not validation. All valid JSON shapes
    /// are observable. Codes prefer uppercase text, lowercase text, uppercase i64.
    pub fn legacy_text(&self) -> ControlState {
        let value = match &self.value {
            ControlValue::Text(text) => text.clone(),
            ControlValue::Number(number) => number.to_string(),
            ControlValue::Missing | ControlValue::Unsupported(_) => String::new(),
        };
        let code = match (&self.code, &self.lowercase_code) {
            (ProtocolCode::Text(code), _) | (_, ProtocolCode::Text(code)) => Some(code.clone()),
            (ProtocolCode::Integer(code), _) => Some(code.to_string()),
            _ => None,
        };
        ControlState { value, code }
    }

    /// Requires an LL object, successful code and text/finite numeric value.
    /// Unlike clock validation, an empty text state is legitimate.
    pub fn validate(&self) -> Result<ControlState, ControlValidationIssue> {
        if self.envelope != EnvelopeState::Present {
            return Err(ControlValidationIssue::Envelope);
        }
        let code = if self.code == ProtocolCode::Missing {
            &self.lowercase_code
        } else {
            &self.code
        };
        match code {
            ProtocolCode::Text(code) if code == "200" => {}
            ProtocolCode::Integer(200) => {}
            ProtocolCode::Missing => return Err(ControlValidationIssue::MissingCode),
            ProtocolCode::Unsupported(_) => return Err(ControlValidationIssue::UnsupportedCode),
            _ => return Err(ControlValidationIssue::RejectedCode),
        }
        match &self.value {
            ControlValue::Text(_) => {}
            ControlValue::Number(value) if value.is_finite() => {}
            ControlValue::Missing => return Err(ControlValidationIssue::MissingValue),
            _ => return Err(ControlValidationIssue::UnsupportedValue),
        }
        Ok(ControlState {
            value: self.legacy_text().value,
            code: Some("200".into()),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlValidationIssue {
    Envelope,
    MissingCode,
    UnsupportedCode,
    RejectedCode,
    MissingValue,
    UnsupportedValue,
}

impl fmt::Display for ControlValidationIssue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid control-state envelope: {self:?}")
    }
}

impl std::error::Error for ControlValidationIssue {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ControlReadFailure {
    InvalidHost,
    InvalidTarget(IdentifierError),
    ClientBuild,
    Http { status: u16 },
    Transport { kind: TransportFailure },
    Decode,
    Cancelled(CancelReason),
    Closed,
    Busy,
    DriverStopped,
    ShutdownTimeout { active: usize },
}

/// Typed, sanitized presentation. Backend details are not part of Debug,
/// Display or the automatic source chain. They can be explicitly taken by a
/// compatibility adapter, which then owns their disclosure.
pub struct ControlReadError {
    pub kind: ControlReadFailure,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ControlReadError {
    pub(super) fn new(kind: ControlReadFailure) -> Self {
        Self { kind, source: None }
    }

    /// Opt-in original reqwest/serde source or genuine `ControlReadPhaseTimeout`,
    /// for an existing head's error adapter.
    /// May contain request URLs/credentials or backend details; do not log it
    /// unconditionally. This does not claim global secret sanitization.
    pub fn into_source(self) -> Option<Box<dyn std::error::Error + Send + Sync>> {
        self.source
    }
}

impl fmt::Debug for ControlReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ControlReadError")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for ControlReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ControlReadFailure::InvalidHost => f.write_str("invalid HTTP(S) control-read host"),
            ControlReadFailure::InvalidTarget(_) => f.write_str("invalid control-read target"),
            ControlReadFailure::ClientBuild => f.write_str("failed to build HTTP client"),
            ControlReadFailure::Http { status } => write!(f, "HTTP {status}: control state"),
            ControlReadFailure::Transport { kind } => f.write_str(match kind {
                TransportFailure::Timeout => "error sending request (operation timed out)",
                TransportFailure::Connect => "error sending request (connection failure)",
                TransportFailure::Request => "error sending request",
                TransportFailure::Body => "error decoding response body",
            }),
            ControlReadFailure::Decode => f.write_str("error decoding response body"),
            ControlReadFailure::Cancelled(reason) => {
                write!(f, "control read cancelled ({reason:?})")
            }
            ControlReadFailure::Closed => f.write_str("control-read runtime is closed"),
            ControlReadFailure::Busy => {
                f.write_str("control-read driver already has an active operation")
            }
            ControlReadFailure::DriverStopped => f.write_str("control-read driver stopped"),
            ControlReadFailure::ShutdownTimeout { active } => {
                write!(f, "control-read shutdown timeout ({active} active)")
            }
        }
    }
}

impl std::error::Error for ControlReadError {}

fn control_error(error: HttpError) -> ControlReadError {
    let (kind, source): (_, Option<Box<dyn std::error::Error + Send + Sync>>) = match error {
        HttpError::InvalidHost => (ControlReadFailure::InvalidHost, None),
        HttpError::Build(error) => (ControlReadFailure::ClientBuild, Some(Box::new(error))),
        HttpError::Status(status) => (ControlReadFailure::Http { status }, None),
        HttpError::Request(error) => (
            if error
                .source()
                .is_some_and(|source| source.is::<serde_json::Error>())
            {
                ControlReadFailure::Decode
            } else {
                ControlReadFailure::Transport {
                    kind: transport_kind(&error),
                }
            },
            Some(Box::new(error)),
        ),
        HttpError::PhaseTimeout(error) => (
            ControlReadFailure::Transport {
                kind: TransportFailure::Timeout,
            },
            Some(Box::new(error)),
        ),
        HttpError::Json(error) => (ControlReadFailure::Decode, Some(Box::new(error))),
        HttpError::Cancelled(reason) => (ControlReadFailure::Cancelled(reason), None),
        HttpError::Closed => (ControlReadFailure::Closed, None),
        HttpError::ShutdownTimeout { active } => {
            (ControlReadFailure::ShutdownTimeout { active }, None)
        }
    };
    ControlReadError { kind, source }
}

/// Async-first actual GET /jdev/sps/io/{resolved_target}. Clones share close state.
/// Requires the caller's Tokio executor; construction starts no thread/runtime.
#[derive(Clone)]
pub struct ControlReadClient {
    pub(super) http: HttpRuntime,
    compatibility: ControlReadCompatibility,
    diagnostics: Option<ControlReadObserver>,
    response_policy: ControlReadResponsePolicy,
}

impl ControlReadClient {
    pub fn new(options: ControlReadOptions) -> Result<Self, ControlReadError> {
        let http = HttpRuntime::new(
            HttpOptions {
                host: options.host,
                user: options.user,
                password: options.password,
                verify_tls: options.verify_tls,
                use_system_proxy: options.use_system_proxy,
                timeout: options.timeout,
                retry_delays: options.retry_delays,
                shutdown_timeout: options.shutdown_timeout,
                user_agent: options.user_agent,
            },
            match options.compatibility {
                ControlReadCompatibility::Strict => HostPolicy::ControlStrict,
                ControlReadCompatibility::LegacyCli => HostPolicy::ControlLegacy,
            },
        )
        .map_err(control_error)?;
        Ok(Self {
            http,
            compatibility: options.compatibility,
            diagnostics: options.diagnostics,
            response_policy: options.response_policy,
        })
    }

    pub fn is_closed(&self) -> bool {
        self.http.is_closed()
    }

    pub fn active_operations(&self) -> usize {
        self.http.active_operations()
    }

    /// Synchronously marks closed and cancels pending HTTP/retry waits.
    pub fn request_close(&self) {
        self.http.request_close();
    }

    /// Idempotent bounded drain, with ClockClient's caller-owned-future caveat.
    pub async fn close(&self) -> Result<(), ControlReadError> {
        self.http.close().await.map_err(control_error)
    }

    pub async fn read_control(
        &self,
        resolved_target: &str,
        cancellation: &Cancellation,
    ) -> Result<ControlObservation, ControlReadError> {
        self.observe_control(resolved_target, None, cancellation)
            .await
    }

    pub(super) async fn observe_control(
        &self,
        resolved_target: &str,
        command: Option<&str>,
        cancellation: &Cancellation,
    ) -> Result<ControlObservation, ControlReadError> {
        self.http
            .run(cancellation, |client| async move {
                if command.is_some() || self.compatibility == ControlReadCompatibility::Strict {
                    validate_uuid(resolved_target).map_err(|error| {
                        ControlReadError::new(ControlReadFailure::InvalidTarget(error))
                    })?;
                }
                let path = match command {
                    Some(command) => format!("/jdev/sps/io/{resolved_target}/{command}"),
                    None => format!("/jdev/sps/io/{resolved_target}"),
                };
                let observer = |body: &str| {
                    if let ControlReadResponsePolicy::LegacyCliVerbose { body_observer } =
                        &self.response_policy
                    {
                        body_observer(body);
                    }
                };
                let policy = match &self.response_policy {
                    ControlReadResponsePolicy::StrictJson => JsonPolicy::ResponseJson,
                    ControlReadResponsePolicy::LegacyCliVerbose { .. } => {
                        JsonPolicy::LegacyText(&observer)
                    }
                };
                self.http
                    .get_json(&client, &path, policy, |event| {
                        if let Some(observer) = &self.diagnostics {
                            observer(match event {
                                HttpEvent::Request => ControlReadDiagnostic::Request,
                                HttpEvent::Response(status) => {
                                    ControlReadDiagnostic::Response { status }
                                }
                                HttpEvent::Body(bytes) => ControlReadDiagnostic::Body { bytes },
                            });
                        }
                    })
                    .await
                    .map(ControlObservation::decode)
                    .map_err(control_error)
            })
            .await
            .map_err(control_error)?
    }
}
