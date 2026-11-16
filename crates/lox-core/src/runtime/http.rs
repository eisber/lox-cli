//! Private HTTP/lifetime shared by clock, control reads and explicit legacy commands.
use super::{
    CancelReason, Cancellation, ClockOptions, ControlReadPhase, ControlReadPhaseTimeout,
    TransportFailure,
};
use reqwest::Client;
use serde_json::Value;
use std::future::Future;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;
use tokio::sync::watch;

pub(super) struct HttpOptions {
    pub host: String,
    pub user: String,
    pub password: String,
    pub verify_tls: bool,
    pub use_system_proxy: bool,
    pub timeout: Duration,
    pub retry_delays: Vec<Duration>,
    pub shutdown_timeout: Duration,
    pub user_agent: String,
}

impl From<ClockOptions> for HttpOptions {
    fn from(options: ClockOptions) -> Self {
        Self {
            host: options.host,
            user: options.user,
            password: options.password,
            verify_tls: options.verify_tls,
            use_system_proxy: options.use_system_proxy,
            timeout: options.timeout,
            retry_delays: options.retry_delays,
            shutdown_timeout: options.shutdown_timeout,
            user_agent: options.user_agent,
        }
    }
}

pub(super) enum HostPolicy {
    Clock,
    ControlStrict,
    ControlLegacy,
}

pub(super) enum HttpError {
    InvalidHost,
    Build(reqwest::Error),
    Status(u16),
    Request(reqwest::Error),
    PhaseTimeout(ControlReadPhaseTimeout),
    Json(serde_json::Error),
    Cancelled(CancelReason),
    Closed,
    ShutdownTimeout { active: usize },
}

pub(super) fn transport_kind(error: &reqwest::Error) -> TransportFailure {
    if error.is_timeout() {
        TransportFailure::Timeout
    } else if error.is_connect() {
        TransportFailure::Connect
    } else if error.is_body() || error.is_decode() {
        TransportFailure::Body
    } else {
        TransportFailure::Request
    }
}

pub(super) enum HttpEvent {
    Request,
    Response(u16),
    Body(usize),
}

pub(super) enum JsonPolicy<'a> {
    Bytes,
    ResponseJson,
    LegacyText(&'a (dyn Fn(&str) + Send + Sync)),
}

struct State {
    client: Option<Client>,
    active: usize,
}

struct Shared {
    state: Mutex<State>,
    active: watch::Sender<usize>,
    shutdown: Cancellation,
    options: HttpOptions,
    legacy_phase_timeout: bool,
}

impl Shared {
    fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[derive(Clone)]
pub(super) struct HttpRuntime(Arc<Shared>);

struct Operation(Arc<Shared>);

impl Drop for Operation {
    fn drop(&mut self) {
        let mut state = self.0.state();
        state.active -= 1;
        self.0.active.send_replace(state.active);
    }
}

impl HttpRuntime {
    pub fn new(options: HttpOptions, policy: HostPolicy) -> Result<Self, HttpError> {
        let legacy_phase_timeout = matches!(policy, HostPolicy::ControlLegacy);
        let parsed = reqwest::Url::parse(&options.host);
        if !matches!(policy, HostPolicy::ControlLegacy)
            && !parsed.as_ref().is_ok_and(|host| {
                matches!(host.scheme(), "http" | "https") && host.host_str().is_some()
            })
        {
            return Err(HttpError::InvalidHost);
        }
        let configured_host = parsed
            .or_else(|_| reqwest::Url::parse(&format!("http://{}", options.host)))
            .ok()
            .and_then(|host| host.host_str().map(str::to_owned))
            .unwrap_or_default();
        let redirects = reqwest::redirect::Policy::custom(move |attempt| {
            let target = attempt.url().host_str().unwrap_or_default().to_owned();
            if !configured_host.is_empty() && target != configured_host {
                if matches!(policy, HostPolicy::Clock) {
                    attempt.error("blocked cross-origin redirect")
                } else {
                    attempt.error(std::io::Error::other(format!(
                        "Blocked cross-origin redirect from {configured_host} to {target}. \
                         If using a local IP, ensure your config uses https:// \
                         or check your Miniserver's network settings."
                    )))
                }
            } else {
                attempt.follow()
            }
        });
        let mut builder = Client::builder()
            .user_agent(&options.user_agent)
            .danger_accept_invalid_certs(!options.verify_tls)
            .redirect(redirects);
        if !legacy_phase_timeout {
            builder = builder.timeout(options.timeout);
        }
        if !options.use_system_proxy {
            builder = builder.no_proxy();
        }
        let client = builder
            .build()
            .map_err(|error| HttpError::Build(error.without_url()))?;
        Ok(Self(Arc::new(Shared {
            state: Mutex::new(State {
                client: Some(client),
                active: 0,
            }),
            active: watch::channel(0).0,
            shutdown: Cancellation::new(),
            options,
            legacy_phase_timeout,
        })))
    }

    async fn phase<T>(
        &self,
        phase: ControlReadPhase,
        url: &reqwest::Url,
        operation: impl Future<Output = Result<T, reqwest::Error>>,
    ) -> Result<T, HttpError> {
        if self.0.legacy_phase_timeout {
            tokio::time::timeout(self.0.options.timeout, operation)
                .await
                .map_err(|elapsed| {
                    HttpError::PhaseTimeout(ControlReadPhaseTimeout::new(
                        phase,
                        url.clone(),
                        elapsed,
                    ))
                })?
                .map_err(HttpError::Request)
        } else {
            operation.await.map_err(HttpError::Request)
        }
    }

    pub fn is_closed(&self) -> bool {
        self.0.state().client.is_none()
    }

    pub fn active_operations(&self) -> usize {
        self.0.state().active
    }

    #[cfg(feature = "runtime-blocking")]
    pub fn shutdown_signal(&self) -> &Cancellation {
        &self.0.shutdown
    }

    pub fn shutdown_timeout(&self) -> Duration {
        self.0.options.shutdown_timeout
    }

    pub fn request_close(&self) {
        let mut state = self.0.state();
        state.client.take();
        self.0.shutdown.cancel();
    }

    pub async fn close(&self) -> Result<(), HttpError> {
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
            .map_err(|_| HttpError::ShutdownTimeout {
                active: self.active_operations(),
            })
    }

    pub async fn run<T>(
        &self,
        cancellation: &Cancellation,
        operation: impl FnOnce(Client) -> T,
    ) -> Result<T::Output, HttpError>
    where
        T: Future,
    {
        let (client, _operation) = {
            let mut state = self.0.state();
            let client = state.client.clone().ok_or(HttpError::Closed)?;
            state.active += 1;
            self.0.active.send_replace(state.active);
            (client, Operation(self.0.clone()))
        };
        tokio::select! {
            biased;
            _ = self.0.shutdown.cancelled() => Err(HttpError::Cancelled(CancelReason::Shutdown)),
            _ = cancellation.cancelled() => Err(HttpError::Cancelled(CancelReason::Requested)),
            result = operation(client) => Ok(result),
        }
    }

    pub async fn get_json(
        &self,
        client: &Client,
        path: &str,
        policy: JsonPolicy<'_>,
        emit: impl Fn(HttpEvent),
    ) -> Result<Value, HttpError> {
        let options = &self.0.options;
        let url = format!("{}/{}", options.host, path.trim_start_matches('/'));
        emit(HttpEvent::Request);
        let mut delays = options.retry_delays.iter();
        loop {
            let result = async {
                let request = client
                    .get(&url)
                    .basic_auth(&options.user, Some(&options.password))
                    .build()
                    .map_err(HttpError::Request)?;
                let request_url = request.url().clone();
                let response = self
                    .phase(
                        ControlReadPhase::Headers,
                        &request_url,
                        client.execute(request),
                    )
                    .await?;
                let status = response.status();
                emit(HttpEvent::Response(status.as_u16()));
                if !status.is_success() {
                    return Err(HttpError::Status(status.as_u16()));
                }
                match &policy {
                    JsonPolicy::ResponseJson => {
                        self.phase(ControlReadPhase::Body, &request_url, response.json())
                            .await
                    }
                    JsonPolicy::Bytes => {
                        let body = response.bytes().await.map_err(HttpError::Request)?;
                        emit(HttpEvent::Body(body.len()));
                        serde_json::from_slice(&body).map_err(HttpError::Json)
                    }
                    JsonPolicy::LegacyText(observer) => {
                        let body = self
                            .phase(ControlReadPhase::Body, &request_url, response.text())
                            .await?;
                        emit(HttpEvent::Body(body.len()));
                        observer(&body);
                        serde_json::from_str(&body).map_err(HttpError::Json)
                    }
                }
            }
            .await;
            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    if matches!(error, HttpError::Status(400..500)) {
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
}
