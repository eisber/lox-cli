use super::clock::{ClockCancellation, ClockSettings, ClockTransportFailure};
use lox_core::runtime as core;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum ControlIdentifierIssue {
    InvalidLength,
    InvalidCharacters,
    NonHex,
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum ControlValidationIssue {
    Envelope,
    MissingCode,
    UnsupportedCode,
    RejectedCode,
    MissingValue,
    UnsupportedValue,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ControlReadError {
    #[error("invalid HTTP(S) control host")]
    InvalidHost,
    #[error("invalid control identifier")]
    InvalidTarget { issue: ControlIdentifierIssue },
    #[error("failed to build control client")]
    ClientBuild,
    #[error("control HTTP failure")]
    Http { status: u16 },
    #[error("control transport failure")]
    Transport { kind: ClockTransportFailure },
    #[error("control response is not strict JSON")]
    Decode,
    #[error("control read cancelled by caller")]
    CancelledRequested,
    #[error("control read cancelled by shutdown")]
    CancelledShutdown,
    #[error("control client is closed")]
    Closed,
    #[error("control client is busy")]
    Busy,
    #[error("control driver stopped")]
    DriverStopped,
    #[error("control shutdown timed out")]
    ShutdownTimeout { active: u64 },
    #[error("control validation failed")]
    Validation { issue: ControlValidationIssue },
}

impl From<core::ControlReadError> for ControlReadError {
    fn from(error: core::ControlReadError) -> Self {
        use lox_core::control::IdentifierError;
        match error.kind {
            core::ControlReadFailure::InvalidHost => Self::InvalidHost,
            core::ControlReadFailure::InvalidTarget(issue) => Self::InvalidTarget {
                issue: match issue {
                    IdentifierError::InvalidLength => ControlIdentifierIssue::InvalidLength,
                    IdentifierError::InvalidCharacters => ControlIdentifierIssue::InvalidCharacters,
                    IdentifierError::NonHex => ControlIdentifierIssue::NonHex,
                },
            },
            core::ControlReadFailure::ClientBuild => Self::ClientBuild,
            core::ControlReadFailure::Http { status } => Self::Http { status },
            core::ControlReadFailure::Transport { kind } => Self::Transport {
                kind: match kind {
                    core::TransportFailure::Timeout => ClockTransportFailure::Timeout,
                    core::TransportFailure::Connect => ClockTransportFailure::Connect,
                    core::TransportFailure::Request => ClockTransportFailure::Request,
                    core::TransportFailure::Body => ClockTransportFailure::Body,
                },
            },
            core::ControlReadFailure::Decode => Self::Decode,
            core::ControlReadFailure::Cancelled(core::CancelReason::Requested) => {
                Self::CancelledRequested
            }
            core::ControlReadFailure::Cancelled(core::CancelReason::Shutdown) => {
                Self::CancelledShutdown
            }
            core::ControlReadFailure::Closed => Self::Closed,
            core::ControlReadFailure::Busy => Self::Busy,
            core::ControlReadFailure::DriverStopped => Self::DriverStopped,
            core::ControlReadFailure::ShutdownTimeout { active } => Self::ShutdownTimeout {
                active: active as u64,
            },
        }
    }
}

impl From<core::ControlValidationIssue> for ControlReadError {
    fn from(issue: core::ControlValidationIssue) -> Self {
        Self::Validation {
            issue: match issue {
                core::ControlValidationIssue::Envelope => ControlValidationIssue::Envelope,
                core::ControlValidationIssue::MissingCode => ControlValidationIssue::MissingCode,
                core::ControlValidationIssue::UnsupportedCode => {
                    ControlValidationIssue::UnsupportedCode
                }
                core::ControlValidationIssue::RejectedCode => ControlValidationIssue::RejectedCode,
                core::ControlValidationIssue::MissingValue => ControlValidationIssue::MissingValue,
                core::ControlValidationIssue::UnsupportedValue => {
                    ControlValidationIssue::UnsupportedValue
                }
            },
        }
    }
}

/// Core-validated text/finite numeric state, not the non-validating legacy projection.
#[derive(uniffi::Record)]
pub struct ControlReading {
    pub value: String,
    pub code: Option<String>,
}

#[derive(uniffi::Object)]
pub struct ControlReadRuntime {
    driver: core::ControlReadDriver,
}

#[uniffi::export]
impl ControlReadRuntime {
    #[uniffi::constructor]
    pub fn new(
        host: String,
        user: String,
        password: String,
        settings: ClockSettings,
    ) -> Result<Arc<Self>, ControlReadError> {
        let mut options = core::ControlReadOptions::new(host, user, password);
        options.verify_tls = settings.verify_tls;
        options.use_system_proxy = settings.use_system_proxy;
        options.timeout = Duration::from_millis(settings.timeout_millis);
        options.retry_delays = settings
            .retry_delays_millis
            .into_iter()
            .map(Duration::from_millis)
            .collect();
        options.shutdown_timeout = Duration::from_millis(settings.shutdown_timeout_millis);
        Ok(Arc::new(Self {
            driver: core::ControlReadDriver::new(options)?,
        }))
    }

    /// Explicit path-safe identifier; no name, context or alias resolution.
    pub fn read_control(
        &self,
        identifier: String,
        cancellation: Arc<ClockCancellation>,
    ) -> Result<ControlReading, ControlReadError> {
        let observation = self.driver.read_control(&identifier, &cancellation.inner)?;
        let state = observation.validate()?;
        Ok(ControlReading {
            value: state.value,
            code: state.code,
        })
    }

    /// Success proves cancellation/drain and joining the owned Core worker.
    pub fn shutdown(&self) -> Result<(), ControlReadError> {
        self.driver.close().map_err(Into::into)
    }

    pub fn is_closed(&self) -> bool {
        self.driver.is_closed()
    }

    pub fn active_operations(&self) -> u64 {
        self.driver.active_operations() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn adapter_uses_existing_validation_not_legacy_projection_and_redacts_errors() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let host = format!("http://{}", listener.local_addr().unwrap());
        let worker = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0; 4096];
            let mut received = Vec::new();
            while !received.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
                let count = socket.read(&mut request).unwrap();
                assert_ne!(count, 0);
                received.extend_from_slice(&request[..count]);
            }
            let body = r#"{"LL":{"Code":403,"code":"200","value":"private-body"}}"#;
            write!(
                socket,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
        });
        let mut settings = super::super::clock::default_clock_settings();
        settings.use_system_proxy = false;
        settings.retry_delays_millis.clear();
        let runtime = ControlReadRuntime::new(
            host.clone(),
            "private-user".into(),
            "private-password".into(),
            settings,
        )
        .unwrap();
        let error = runtime
            .read_control("abc".into(), ClockCancellation::new())
            .err()
            .unwrap();
        assert!(matches!(
            error,
            ControlReadError::Validation {
                issue: ControlValidationIssue::RejectedCode
            }
        ));
        let diagnostic = format!("{error:?} {error}");
        for secret in [
            host.as_str(),
            "private-user",
            "private-password",
            "private-body",
        ] {
            assert!(!diagnostic.contains(secret));
        }
        runtime.shutdown().unwrap();
        runtime.shutdown().unwrap();
        assert!(runtime.is_closed());
        assert!(matches!(
            runtime.read_control("abc".into(), ClockCancellation::new()),
            Err(ControlReadError::Closed)
        ));
        worker.join().unwrap();
    }
}
