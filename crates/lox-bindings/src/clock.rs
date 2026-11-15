use lox_core::runtime as core;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum ClockEndpoint {
    Date,
    Time,
}

impl From<core::ClockEndpoint> for ClockEndpoint {
    fn from(value: core::ClockEndpoint) -> Self {
        match value {
            core::ClockEndpoint::Date => Self::Date,
            core::ClockEndpoint::Time => Self::Time,
        }
    }
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum ClockTransportFailure {
    Timeout,
    Connect,
    Request,
    Body,
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum ClockValidationIssue {
    Envelope,
    MissingCode,
    UnsupportedCode,
    RejectedCode,
    MissingValue,
    EmptyValue,
    UnsupportedValue,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum ClockRuntimeError {
    #[error("invalid HTTP(S) clock host")]
    InvalidHost,
    #[error("failed to build clock client")]
    ClientBuild,
    #[error("clock HTTP failure")]
    Http {
        endpoint: ClockEndpoint,
        status: u16,
    },
    #[error("clock transport failure")]
    Transport {
        endpoint: ClockEndpoint,
        kind: ClockTransportFailure,
    },
    #[error("clock response is not strict JSON")]
    Decode { endpoint: ClockEndpoint },
    #[error("clock read cancelled by caller")]
    CancelledRequested,
    #[error("clock read cancelled by shutdown")]
    CancelledShutdown,
    #[error("clock client is closed")]
    Closed,
    #[error("clock client is busy")]
    Busy,
    #[error("clock driver stopped")]
    DriverStopped,
    #[error("clock shutdown timed out")]
    ShutdownTimeout { active: u64 },
    #[error("clock validation failed")]
    Validation {
        endpoint: ClockEndpoint,
        issue: ClockValidationIssue,
    },
}

impl From<core::ClockError> for ClockRuntimeError {
    fn from(error: core::ClockError) -> Self {
        match error {
            core::ClockError::InvalidHost => Self::InvalidHost,
            core::ClockError::ClientBuild => Self::ClientBuild,
            core::ClockError::Http { endpoint, status } => Self::Http {
                endpoint: endpoint.into(),
                status,
            },
            core::ClockError::Transport { endpoint, kind } => Self::Transport {
                endpoint: endpoint.into(),
                kind: match kind {
                    core::TransportFailure::Timeout => ClockTransportFailure::Timeout,
                    core::TransportFailure::Connect => ClockTransportFailure::Connect,
                    core::TransportFailure::Request => ClockTransportFailure::Request,
                    core::TransportFailure::Body => ClockTransportFailure::Body,
                },
            },
            core::ClockError::Decode { endpoint } => Self::Decode {
                endpoint: endpoint.into(),
            },
            core::ClockError::Cancelled(core::CancelReason::Requested) => Self::CancelledRequested,
            core::ClockError::Cancelled(core::CancelReason::Shutdown) => Self::CancelledShutdown,
            core::ClockError::Closed => Self::Closed,
            core::ClockError::Busy => Self::Busy,
            core::ClockError::DriverStopped => Self::DriverStopped,
            core::ClockError::ShutdownTimeout { active } => Self::ShutdownTimeout {
                active: active as u64,
            },
        }
    }
}

impl From<core::ClockValidationError> for ClockRuntimeError {
    fn from(error: core::ClockValidationError) -> Self {
        Self::Validation {
            endpoint: error.endpoint.into(),
            issue: match error.issue {
                core::ClockValidationIssue::Envelope => ClockValidationIssue::Envelope,
                core::ClockValidationIssue::MissingCode => ClockValidationIssue::MissingCode,
                core::ClockValidationIssue::UnsupportedCode => {
                    ClockValidationIssue::UnsupportedCode
                }
                core::ClockValidationIssue::RejectedCode => ClockValidationIssue::RejectedCode,
                core::ClockValidationIssue::MissingValue => ClockValidationIssue::MissingValue,
                core::ClockValidationIssue::EmptyValue => ClockValidationIssue::EmptyValue,
                core::ClockValidationIssue::UnsupportedValue => {
                    ClockValidationIssue::UnsupportedValue
                }
            },
        }
    }
}

/// Non-secret transport settings. Connection credentials are constructor inputs,
/// not generated record fields whose repr/toString could reveal them.
#[derive(Clone, uniffi::Record)]
pub struct ClockSettings {
    pub verify_tls: bool,
    pub use_system_proxy: bool,
    pub timeout_millis: u64,
    pub retry_delays_millis: Vec<u64>,
    pub shutdown_timeout_millis: u64,
}

#[uniffi::export]
pub fn default_clock_settings() -> ClockSettings {
    let options = core::ClockOptions::new("", "", "");
    ClockSettings {
        verify_tls: options.verify_tls,
        use_system_proxy: options.use_system_proxy,
        timeout_millis: options.timeout.as_millis() as u64,
        retry_delays_millis: options
            .retry_delays
            .iter()
            .map(|delay| delay.as_millis() as u64)
            .collect(),
        shutdown_timeout_millis: options.shutdown_timeout.as_millis() as u64,
    }
}

/// Shared-core validated text, not calendar/timezone interpretation.
#[derive(uniffi::Record)]
pub struct ClockReading {
    pub date: String,
    pub time: String,
}

#[derive(uniffi::Object)]
pub struct ClockCancellation {
    inner: core::Cancellation,
}

#[uniffi::export]
impl ClockCancellation {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            inner: core::Cancellation::new(),
        })
    }

    pub fn cancel(&self) {
        self.inner.cancel();
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.is_cancelled()
    }
}

#[derive(uniffi::Object)]
pub struct ClockRuntime {
    driver: core::ClockDriver,
}

#[uniffi::export]
impl ClockRuntime {
    #[uniffi::constructor]
    pub fn new(
        host: String,
        user: String,
        password: String,
        settings: ClockSettings,
    ) -> Result<Arc<Self>, ClockRuntimeError> {
        let mut options = core::ClockOptions::new(host, user, password);
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
            driver: core::ClockDriver::new(options)?,
        }))
    }

    pub fn read_clock(
        &self,
        cancellation: Arc<ClockCancellation>,
    ) -> Result<ClockReading, ClockRuntimeError> {
        let observation = self.driver.read_clock(&cancellation.inner)?;
        let reading = observation.validate()?;
        Ok(ClockReading {
            date: reading.date,
            time: reading.time,
        })
    }

    /// Explicit core shutdown; generated object destruction alone is fallback.
    pub fn shutdown(&self) -> Result<(), ClockRuntimeError> {
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
    fn adapter_keeps_strict_json_and_maps_decode_without_secret_material() {
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
            let body = b"{\"LL\":{\"Code\":200,\"value\":\"\xff\"}}";
            write!(
                socket,
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            )
            .unwrap();
            socket.write_all(body).unwrap();
        });
        let mut settings = default_clock_settings();
        settings.use_system_proxy = false;
        settings.retry_delays_millis.clear();
        let runtime = ClockRuntime::new(
            host.clone(),
            "secret-user".into(),
            "secret-password".into(),
            settings,
        )
        .unwrap();
        let error = match runtime.read_clock(ClockCancellation::new()) {
            Err(error) => error,
            Ok(_) => panic!("strict JSON unexpectedly accepted invalid UTF-8"),
        };
        assert!(matches!(
            error,
            ClockRuntimeError::Decode {
                endpoint: ClockEndpoint::Date
            }
        ));
        let diagnostic = format!("{error:?} {error}");
        for secret in [host.as_str(), "secret-user", "secret-password"] {
            assert!(!diagnostic.contains(secret));
        }
        runtime.shutdown().unwrap();
        runtime.shutdown().unwrap();
        assert!(matches!(
            runtime.read_clock(ClockCancellation::new()),
            Err(ClockRuntimeError::Closed)
        ));
        worker.join().unwrap();
    }
}
