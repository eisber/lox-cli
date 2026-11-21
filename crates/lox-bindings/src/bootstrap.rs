use lox_core::runtime::TransportFailure;
use lox_core::runtime::{session_endpoints as endpoints, session_http as http};

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum BootstrapProxyPolicy {
    Direct,
    System,
}

/// Explicit non-secret routes; the WebSocket route is admitted but never contacted.
#[derive(uniffi::Record)]
pub struct BootstrapRequest {
    pub http_url: String,
    pub websocket_url: String,
    pub proxy_policy: BootstrapProxyPolicy,
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum BootstrapEventSlots {
    Missing,
    Unknown,
    ReportedAvailable,
    ReportedUnavailable,
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum BootstrapTlsStatus {
    Missing,
    Unknown,
    ReportedAvailable,
    ReportedCertificateExpired,
}

/// Advisory reported data, not a local TLS result or an event-slot reservation.
#[derive(uniffi::Record)]
pub struct BootstrapCapabilities {
    pub event_slots: BootstrapEventSlots,
    pub tls_status: BootstrapTlsStatus,
}

impl From<http::BootstrapCapabilities> for BootstrapCapabilities {
    fn from(observed: http::BootstrapCapabilities) -> Self {
        use http::Capability::{Missing, Reported, Unknown};

        Self {
            event_slots: match observed.event_slots {
                Missing => BootstrapEventSlots::Missing,
                Unknown => BootstrapEventSlots::Unknown,
                Reported(true) => BootstrapEventSlots::ReportedAvailable,
                Reported(false) => BootstrapEventSlots::ReportedUnavailable,
            },
            tls_status: match observed.tls_status {
                Missing => BootstrapTlsStatus::Missing,
                Unknown => BootstrapTlsStatus::Unknown,
                Reported(http::TlsStatus::Available) => BootstrapTlsStatus::ReportedAvailable,
                Reported(http::TlsStatus::CertificateExpired) => {
                    BootstrapTlsStatus::ReportedCertificateExpired
                }
            },
        }
    }
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum BootstrapEndpointKind {
    Http,
    WebSocket,
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum BootstrapEndpointIssue {
    Scheme,
    Authority,
    Path,
    UserInfoQueryOrFragment,
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum BootstrapTransportFailure {
    Timeout,
    Connect,
    Request,
    Body,
}

#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum BootstrapPayloadIssue {
    TooLarge,
    InvalidEnvelope,
    UnexpectedControl,
    UnsupportedValueEncoding,
}

/// Route-free typed refusals. No backend exception chain crosses the FFI.
#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum BootstrapError {
    #[error("invalid bootstrap endpoint")]
    InvalidEndpoint {
        endpoint: BootstrapEndpointKind,
        issue: BootstrapEndpointIssue,
    },
    #[error("unsupported bootstrap HTTP proxy policy")]
    UnsupportedProxyPolicy,
    #[error("failed to start bootstrap HTTP worker")]
    WorkerStart,
    #[error("failed to build bootstrap HTTP executor")]
    ExecutorBuild,
    #[error("bootstrap HTTP worker panicked")]
    WorkerPanicked,
    #[error("failed to build bootstrap HTTP client")]
    ClientBuild,
    #[error("HTTP route is not the bootstrap command")]
    WrongRoute,
    #[error("bootstrap HTTP transport failure")]
    Transport { kind: BootstrapTransportFailure },
    #[error("bootstrap HTTP status: {status}")]
    HttpStatus { status: u16 },
    #[error("bootstrap command status: {status}")]
    CommandStatus { status: u16 },
    #[error("bootstrap response payload refused")]
    Payload { issue: BootstrapPayloadIssue },
}

impl From<endpoints::EndpointError> for BootstrapError {
    fn from(error: endpoints::EndpointError) -> Self {
        Self::InvalidEndpoint {
            endpoint: match error.endpoint {
                endpoints::EndpointKind::Http => BootstrapEndpointKind::Http,
                endpoints::EndpointKind::WebSocket => BootstrapEndpointKind::WebSocket,
            },
            issue: match error.issue {
                endpoints::EndpointIssue::Scheme => BootstrapEndpointIssue::Scheme,
                endpoints::EndpointIssue::Authority => BootstrapEndpointIssue::Authority,
                endpoints::EndpointIssue::Path => BootstrapEndpointIssue::Path,
                endpoints::EndpointIssue::UserInfoQueryOrFragment => {
                    BootstrapEndpointIssue::UserInfoQueryOrFragment
                }
            },
        }
    }
}

impl From<TransportFailure> for BootstrapError {
    fn from(kind: TransportFailure) -> Self {
        Self::Transport {
            kind: match kind {
                TransportFailure::Timeout => BootstrapTransportFailure::Timeout,
                TransportFailure::Connect => BootstrapTransportFailure::Connect,
                TransportFailure::Request => BootstrapTransportFailure::Request,
                TransportFailure::Body => BootstrapTransportFailure::Body,
            },
        }
    }
}

impl From<http::BlockingBootstrapError> for BootstrapError {
    fn from(error: http::BlockingBootstrapError) -> Self {
        use http::{BlockingBootstrapError as Blocking, BootstrapError as Preflight};
        use http::{BootstrapPayloadIssue as Payload, SessionHttpError as Client};

        match error {
            Blocking::WorkerStart => Self::WorkerStart,
            Blocking::ExecutorBuild => Self::ExecutorBuild,
            Blocking::WorkerPanicked => Self::WorkerPanicked,
            Blocking::Client(Client::UnsupportedProxyPolicy(_)) => Self::UnsupportedProxyPolicy,
            Blocking::Client(Client::ClientBuild) => Self::ClientBuild,
            Blocking::Client(Client::Transport(kind))
            | Blocking::Preflight(Preflight::Transport(kind)) => kind.into(),
            Blocking::Preflight(Preflight::WrongRoute) => Self::WrongRoute,
            Blocking::Preflight(Preflight::HttpStatus(status)) => Self::HttpStatus {
                status: status.as_u16(),
            },
            Blocking::Preflight(Preflight::Payload(issue)) => Self::Payload {
                issue: match issue {
                    Payload::TooLarge => BootstrapPayloadIssue::TooLarge,
                    Payload::InvalidEnvelope => BootstrapPayloadIssue::InvalidEnvelope,
                    Payload::UnexpectedControl => BootstrapPayloadIssue::UnexpectedControl,
                    Payload::UnsupportedValueEncoding => {
                        BootstrapPayloadIssue::UnsupportedValueEncoding
                    }
                    Payload::CommandStatus(status) => return Self::CommandStatus { status },
                },
            },
        }
    }
}

/// One synchronous, unauthenticated observation. No handle or cancellation is returned.
#[uniffi::export]
pub fn bootstrap_preflight(
    request: BootstrapRequest,
) -> Result<BootstrapCapabilities, BootstrapError> {
    let policy = match request.proxy_policy {
        BootstrapProxyPolicy::Direct => endpoints::ProxyPolicy::Direct,
        BootstrapProxyPolicy::System => endpoints::ProxyPolicy::System,
    };
    let endpoints =
        endpoints::SessionEndpoints::parse(&request.http_url, &request.websocket_url, policy)?;
    Ok(http::bootstrap_preflight_blocking(endpoints)?.into())
}
