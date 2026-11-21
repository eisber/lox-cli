use super::{
    Cancellation, ControlObservation, ControlReadClient, ControlReadCompatibility,
    ControlReadError, ControlReadOptions,
};
use crate::control::{IdentifierError, validate_uuid};
use std::fmt;

/// One owned legacy HTTP command. Only the identifier is validated; command
/// text retains the CLI's literal URL parsing, not segment encoding.
#[derive(Clone)]
pub struct LegacyControlCommand {
    target: String,
    command: String,
}

impl LegacyControlCommand {
    pub fn new(
        resolved_target: impl Into<String>,
        command: impl Into<String>,
    ) -> Result<Self, IdentifierError> {
        let target = resolved_target.into();
        validate_uuid(&target)?;
        Ok(Self {
            target,
            command: command.into(),
        })
    }

    pub(super) fn target(&self) -> &str {
        &self.target
    }

    pub(super) fn command(&self) -> &str {
        &self.command
    }
}

impl fmt::Debug for LegacyControlCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("LegacyControlCommand { target: [redacted], command: [redacted] }")
    }
}

/// Explicit opt-in to the existing CLI's mutating GET compatibility policy.
///
/// Uses the existing control HTTP options, observations and typed transport
/// errors. `compatibility` is forced to `LegacyCli`: host errors are deferred
/// and header/body timeout budgets are separate. No write permission, name
/// resolution, context or persistence is inferred.
///
/// IMPORTANT: all failures except HTTP 4xx are retried according to
/// `retry_delays`, INCLUDING invalid JSON after a successful HTTP response.
/// A non-idempotent command may therefore execute multiple times. Cancellation
/// or failure does not establish that the server did not execute a command.
/// This API does not provide exactly-once delivery or a general HTTP route.
#[derive(Clone)]
pub struct LegacyControlCommandClient {
    pub(super) inner: ControlReadClient,
}

impl LegacyControlCommandClient {
    pub fn new(mut options: ControlReadOptions) -> Result<Self, ControlReadError> {
        options.compatibility = ControlReadCompatibility::LegacyCli;
        Ok(Self {
            inner: ControlReadClient::new(options)?,
        })
    }

    pub fn is_closed(&self) -> bool {
        self.inner.is_closed()
    }

    pub fn active_operations(&self) -> usize {
        self.inner.active_operations()
    }

    pub fn request_close(&self) {
        self.inner.request_close();
    }

    pub async fn close(&self) -> Result<(), ControlReadError> {
        self.inner.close().await
    }

    /// Returns an owned LL observation, not an assertion of command success.
    /// `legacy_text()` is the CLI's projection; protocol-code validation is
    /// deliberately separate and never changes retry/delivery policy.
    pub async fn send_command(
        &self,
        command: &LegacyControlCommand,
        cancellation: &Cancellation,
    ) -> Result<ControlObservation, ControlReadError> {
        self.inner
            .observe_control(command.target(), Some(command.command()), cancellation)
            .await
    }
}
