use super::*;
use reqwest::Client;
use std::sync::{Mutex, MutexGuard};

struct State {
    client: Option<Client>,
    active: usize,
}

struct Shared {
    state: Mutex<State>,
    active: watch::Sender<usize>,
    shutdown: Cancellation,
    options: ClockOptions,
}

impl Shared {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

/// Async-first client. Clones share close state. No worker/runtime is started.
/// Dropping an operation future aborts its in-flight request. Call `close` to
/// cancel and drain all operations; the last handle's drop is only a fallback.
#[derive(Clone)]
pub struct ClockClient(Arc<Shared>);

struct Operation(Arc<Shared>);

impl Drop for Operation {
    fn drop(&mut self) {
        let mut state = self.0.state();
        state.active -= 1;
        self.0.active.send_replace(state.active);
    }
}

impl ClockClient {
    pub fn new(options: ClockOptions) -> Result<Self, ClockError> {
        let host = reqwest::Url::parse(&options.host).map_err(|_| ClockError::InvalidHost)?;
        if !matches!(host.scheme(), "http" | "https") || host.host_str().is_none() {
            return Err(ClockError::InvalidHost);
        }
        let configured_host = host.host_str().unwrap_or_default().to_owned();
        // Deliberately retain the legacy host-only redirect check (not a new
        // scheme/port restriction). reqwest still strips auth across origins.
        let redirects = reqwest::redirect::Policy::custom(move |attempt| {
            if attempt.url().host_str().unwrap_or_default() != configured_host {
                attempt.error("blocked cross-origin redirect")
            } else {
                attempt.follow()
            }
        });
        let mut builder = Client::builder()
            .user_agent(&options.user_agent)
            .danger_accept_invalid_certs(!options.verify_tls)
            .redirect(redirects)
            .timeout(options.timeout);
        if !options.use_system_proxy {
            builder = builder.no_proxy();
        }
        let client = builder.build().map_err(|_| ClockError::ClientBuild)?;
        Ok(Self(Arc::new(Shared {
            state: Mutex::new(State {
                client: Some(client),
                active: 0,
            }),
            active: watch::channel(0).0,
            shutdown: Cancellation::new(),
            options,
        })))
    }

    pub fn is_closed(&self) -> bool {
        self.0.state().client.is_none()
    }

    pub fn active_operations(&self) -> usize {
        self.0.state().active
    }

    #[cfg(feature = "runtime-blocking")]
    pub(crate) fn shutdown_signal(&self) -> &Cancellation {
        &self.0.shutdown
    }

    pub(crate) fn shutdown_timeout(&self) -> Duration {
        self.0.options.shutdown_timeout
    }

    /// Marks closed synchronously and wakes pending operations. Does not wait.
    pub fn request_close(&self) {
        let mut state = self.0.state();
        state.client.take();
        self.0.shutdown.cancel();
    }

    /// Idempotent, bounded drain. A future retained without being polled/dropped
    /// by its caller can prevent draining; this reports ShutdownTimeout rather
    /// than claiming that caller-owned resources have been released.
    pub async fn close(&self) -> Result<(), ClockError> {
        self.request_close();
        let mut active = self.0.active.subscribe();
        let drain = async {
            while *active.borrow_and_update() != 0 {
                if active.changed().await.is_err() {
                    break;
                }
            }
        };
        tokio::time::timeout(self.shutdown_timeout(), drain)
            .await
            .map_err(|_| ClockError::ShutdownTimeout {
                active: self.active_operations(),
            })
    }

    /// Reads date then time, stopping on the first transport/decode failure.
    /// Invalid/missing LL fields remain explicit in the returned observation.
    /// `ClockObservation::validate` and `legacy_text` are separate policies.
    pub async fn read_clock(
        &self,
        cancellation: &Cancellation,
    ) -> Result<ClockObservation, ClockError> {
        let (client, _operation) = {
            let mut state = self.0.state();
            let client = state.client.clone().ok_or(ClockError::Closed)?;
            state.active += 1;
            self.0.active.send_replace(state.active);
            (client, Operation(self.0.clone()))
        };
        tokio::select! {
            biased;
            _ = self.0.shutdown.cancelled() => Err(ClockError::Cancelled(CancelReason::Shutdown)),
            _ = cancellation.cancelled() => Err(ClockError::Cancelled(CancelReason::Requested)),
            result = async {
                let date = self.get_field(&client, ClockEndpoint::Date).await?;
                let time = self.get_field(&client, ClockEndpoint::Time).await?;
                Ok(ClockObservation { date, time })
            } => result,
        }
    }

    fn emit(&self, event: Diagnostic) {
        if let Some(observer) = &self.0.options.diagnostics {
            observer(event);
        }
    }

    async fn get_field(
        &self,
        client: &Client,
        endpoint: ClockEndpoint,
    ) -> Result<ClockField, ClockError> {
        let options = &self.0.options;
        let url = format!(
            "{}/{}",
            options.host,
            endpoint.path().trim_start_matches('/')
        );
        self.emit(Diagnostic::Request(endpoint));
        let mut delays = options.retry_delays.iter();
        loop {
            match self.attempt(client, &url, endpoint).await {
                Ok(value) => return Ok(value),
                Err(error) => {
                    if matches!(
                        error,
                        ClockError::Http {
                            status: 400..500,
                            ..
                        }
                    ) {
                        return Err(error);
                    }
                    match delays.next() {
                        Some(delay) => tokio::time::sleep(*delay).await,
                        None => return Err(error),
                    }
                }
            }
        }
    }

    async fn attempt(
        &self,
        client: &Client,
        url: &str,
        endpoint: ClockEndpoint,
    ) -> Result<ClockField, ClockError> {
        let options = &self.0.options;
        let response = client
            .get(url)
            .basic_auth(&options.user, Some(&options.password))
            .send()
            .await
            .map_err(|error| transport(error, endpoint))?;
        let status = response.status();
        self.emit(Diagnostic::Response {
            endpoint,
            status: status.as_u16(),
        });
        if !status.is_success() {
            return Err(ClockError::Http {
                endpoint,
                status: status.as_u16(),
            });
        }
        let value = match &options.response_policy {
            ClockResponsePolicy::StrictJson => {
                let body = response
                    .bytes()
                    .await
                    .map_err(|error| transport(error, endpoint))?;
                self.emit(Diagnostic::Body {
                    endpoint,
                    bytes: body.len(),
                });
                serde_json::from_slice(&body)
            }
            ClockResponsePolicy::LegacyCliVerbose { body_observer } => {
                let body = response
                    .text()
                    .await
                    .map_err(|error| transport(error, endpoint))?;
                self.emit(Diagnostic::Body {
                    endpoint,
                    bytes: body.len(),
                });
                body_observer(endpoint, &body);
                serde_json::from_str(&body)
            }
        }
        .map_err(|_| ClockError::Decode { endpoint })?;
        Ok(ClockField::decode(value))
    }
}

fn transport(error: reqwest::Error, endpoint: ClockEndpoint) -> ClockError {
    let kind = if error.is_timeout() {
        TransportFailure::Timeout
    } else if error.is_connect() {
        TransportFailure::Connect
    } else if error.is_body() || error.is_decode() {
        TransportFailure::Body
    } else {
        TransportFailure::Request
    };
    ClockError::Transport { endpoint, kind }
}
