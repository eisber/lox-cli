use super::http::{HostPolicy, HttpError, HttpEvent, HttpRuntime, JsonPolicy, transport_kind};
use super::*;

/// Async-first client. Clones share close state. No worker/runtime is started.
/// Dropping an operation future aborts its in-flight request. Call `close` to
/// cancel and drain all operations; the last handle's drop is only a fallback.
#[derive(Clone)]
pub struct ClockClient {
    pub(super) http: HttpRuntime,
    diagnostics: Option<DiagnosticObserver>,
    response_policy: ClockResponsePolicy,
}

impl ClockClient {
    pub fn new(options: ClockOptions) -> Result<Self, ClockError> {
        let diagnostics = options.diagnostics.clone();
        let response_policy = options.response_policy.clone();
        let http = HttpRuntime::new(options.into(), HostPolicy::Clock)
            .map_err(|error| clock_error(error, ClockEndpoint::Date))?;
        Ok(Self {
            http,
            diagnostics,
            response_policy,
        })
    }

    pub fn is_closed(&self) -> bool {
        self.http.is_closed()
    }

    pub fn active_operations(&self) -> usize {
        self.http.active_operations()
    }

    /// Marks closed synchronously and wakes pending operations. Does not wait.
    pub fn request_close(&self) {
        self.http.request_close();
    }

    /// Idempotent, bounded drain. A future retained without being polled/dropped
    /// by its caller can prevent draining; this reports ShutdownTimeout rather
    /// than claiming that caller-owned resources have been released.
    pub async fn close(&self) -> Result<(), ClockError> {
        self.http
            .close()
            .await
            .map_err(|error| clock_error(error, ClockEndpoint::Date))
    }

    /// Reads date then time, stopping on the first transport/decode failure.
    /// Invalid/missing LL fields remain explicit in the returned observation.
    /// `ClockObservation::validate` and `legacy_text` are separate policies.
    pub async fn read_clock(
        &self,
        cancellation: &Cancellation,
    ) -> Result<ClockObservation, ClockError> {
        self.http
            .run(cancellation, |client| async move {
                let date = self.get_field(&client, ClockEndpoint::Date).await?;
                let time = self.get_field(&client, ClockEndpoint::Time).await?;
                Ok(ClockObservation { date, time })
            })
            .await
            .map_err(|error| clock_error(error, ClockEndpoint::Date))?
    }

    async fn get_field(
        &self,
        client: &reqwest::Client,
        endpoint: ClockEndpoint,
    ) -> Result<ClockField, ClockError> {
        let observer = |body: &str| {
            if let ClockResponsePolicy::LegacyCliVerbose { body_observer } = &self.response_policy {
                body_observer(endpoint, body);
            }
        };
        let policy = match &self.response_policy {
            ClockResponsePolicy::StrictJson => JsonPolicy::Bytes,
            ClockResponsePolicy::LegacyCliVerbose { .. } => JsonPolicy::LegacyText(&observer),
        };
        let value = self
            .http
            .get_json(client, endpoint.path(), policy, |event| {
                if let Some(observer) = &self.diagnostics {
                    observer(match event {
                        HttpEvent::Request => Diagnostic::Request(endpoint),
                        HttpEvent::Response(status) => Diagnostic::Response { endpoint, status },
                        HttpEvent::Body(bytes) => Diagnostic::Body { endpoint, bytes },
                    });
                }
            })
            .await
            .map_err(|error| clock_error(error, endpoint))?;
        Ok(ClockField::decode(value))
    }
}

fn clock_error(error: HttpError, endpoint: ClockEndpoint) -> ClockError {
    match error {
        HttpError::InvalidHost => ClockError::InvalidHost,
        HttpError::Build(_) => ClockError::ClientBuild,
        HttpError::Status(status) => ClockError::Http { endpoint, status },
        HttpError::Request(error) => ClockError::Transport {
            endpoint,
            kind: transport_kind(&error),
        },
        HttpError::PhaseTimeout(_) => ClockError::Transport {
            endpoint,
            kind: TransportFailure::Timeout,
        },
        HttpError::Json(_) => ClockError::Decode { endpoint },
        HttpError::Cancelled(reason) => ClockError::Cancelled(reason),
        HttpError::Closed => ClockError::Closed,
        HttpError::ShutdownTimeout { active } => ClockError::ShutdownTimeout { active },
    }
}
