use super::*;
use std::sync::{Condvar, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc as queue;

enum Request {
    Clock {
        cancellation: Cancellation,
        reply: mpsc::SyncSender<Result<ClockObservation, ClockError>>,
    },
    Control {
        target: String,
        cancellation: Cancellation,
        reply: mpsc::SyncSender<Result<ControlObservation, ControlReadError>>,
    },
}

enum Options {
    Clock(ClockOptions),
    Control(ControlReadOptions),
}

enum Client {
    Clock(ClockClient),
    Control(ControlReadClient),
}

enum StartError {
    Clock(ClockError),
    Control(ControlReadError),
    Driver(ClockError),
}

impl Client {
    fn new(options: Options) -> Result<Self, StartError> {
        match options {
            Options::Clock(options) => ClockClient::new(options)
                .map(Self::Clock)
                .map_err(StartError::Clock),
            Options::Control(options) => ControlReadClient::new(options)
                .map(Self::Control)
                .map_err(StartError::Control),
        }
    }

    fn http(&self) -> &http::HttpRuntime {
        match self {
            Self::Clock(client) => &client.http,
            Self::Control(client) => &client.http,
        }
    }

    async fn execute(&self, request: Request) {
        match (self, request) {
            (
                Self::Clock(client),
                Request::Clock {
                    cancellation,
                    reply,
                },
            ) => {
                let _ = reply.send(client.read_clock(&cancellation).await);
            }
            (
                Self::Control(client),
                Request::Control {
                    target,
                    cancellation,
                    reply,
                },
            ) => {
                let _ = reply.send(client.read_control(&target, &cancellation).await);
            }
            (_, Request::Clock { reply, .. }) => {
                let _ = reply.send(Err(ClockError::DriverStopped));
            }
            (_, Request::Control { reply, .. }) => {
                let _ = reply.send(Err(ControlReadError::new(
                    ControlReadFailure::DriverStopped,
                )));
            }
        }
    }
}

#[derive(Default)]
struct Completion {
    done: Mutex<bool>,
    changed: Condvar,
}

struct Finished(Arc<Completion>);

struct CloseOnDrop(http::HttpRuntime);

impl Drop for CloseOnDrop {
    fn drop(&mut self) {
        self.0.request_close();
    }
}

impl Drop for Finished {
    fn drop(&mut self) {
        *self.0.done.lock().unwrap_or_else(|e| e.into_inner()) = true;
        self.0.changed.notify_all();
    }
}

/// Explicitly owned single-thread Tokio driver, admitting one operation at a
/// time (concurrent calls report Busy). Construction starts the thread/runtime.
/// Blocking calls may be made under another executor without nested block_on
/// or dropping a Tokio runtime on that caller's thread. They still block the
/// caller: async Rust callers should use ClockClient instead.
struct OwnedDriver {
    client: http::HttpRuntime,
    sender: queue::Sender<Request>,
    completion: Arc<Completion>,
    worker: Mutex<Option<JoinHandle<()>>>,
    admission: Mutex<()>,
}

impl OwnedDriver {
    fn new(options: Options) -> Result<Self, StartError> {
        let (sender, mut receiver) = queue::channel::<Request>(1);
        let (ready, result) = mpsc::sync_channel(1);
        let completion = Arc::new(Completion::default());
        let finished = Finished(completion.clone());
        let worker = thread::Builder::new()
            .name(
                match &options {
                    Options::Clock(_) => "lox-clock",
                    Options::Control(_) => "lox-control-read",
                }
                .into(),
            )
            .spawn(move || {
                let _finished = finished;
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(_) => {
                        let _ = ready.send(Err(StartError::Driver(ClockError::DriverStopped)));
                        return;
                    }
                };
                runtime.block_on(async move {
                    let client = match Client::new(options) {
                        Ok(client) => client,
                        Err(error) => {
                            let _ = ready.send(Err(error));
                            return;
                        }
                    };
                    let _close = CloseOnDrop(client.http().clone());
                    if ready.send(Ok(client.http().clone())).is_err() {
                        return;
                    }
                    loop {
                        tokio::select! {
                            biased;
                            _ = client.http().shutdown_signal().cancelled() => break,
                            request = receiver.recv() => match request {
                                Some(request) => {
                                    client.execute(request).await;
                                }
                                None => break,
                            }
                        }
                    }
                });
                // Async requests are dropped before this point. Do not indefinitely
                // wait for platform DNS/blocking tasks during runtime teardown.
                runtime.shutdown_timeout(Duration::from_millis(100));
            })
            .map_err(|_| StartError::Driver(ClockError::DriverStopped))?;
        match result.recv() {
            Ok(Ok(client)) => Ok(Self {
                client,
                sender,
                completion,
                worker: Mutex::new(Some(worker)),
                admission: Mutex::new(()),
            }),
            result => {
                let _ = worker.join();
                Err(result
                    .ok()
                    .and_then(Result::err)
                    .unwrap_or(StartError::Driver(ClockError::DriverStopped)))
            }
        }
    }

    pub fn is_closed(&self) -> bool {
        self.client.is_closed()
    }

    pub fn active_operations(&self) -> usize {
        self.client.active_operations()
    }

    fn submit(
        &self,
        request: Request,
        receive: impl FnOnce() -> Result<(), ClockError>,
    ) -> Result<(), ClockError> {
        if self.is_closed() {
            return Err(ClockError::Closed);
        }
        let _admission = match self.admission.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::WouldBlock) => return Err(ClockError::Busy),
            Err(std::sync::TryLockError::Poisoned(_)) => return Err(ClockError::DriverStopped),
        };
        self.sender.try_send(request).map_err(|error| match error {
            queue::error::TrySendError::Full(_) => ClockError::Busy,
            queue::error::TrySendError::Closed(_) => ClockError::Closed,
        })?;
        receive()
    }

    /// Cancels pending I/O and waits at most shutdown_timeout for worker exit.
    /// Success proves the owned worker has exited and been joined. Timeout
    /// leaves the driver closed and can be retried; Drop is bounded best effort.
    pub fn close(&self) -> Result<(), ClockError> {
        self.client.request_close();
        let done = self
            .completion
            .done
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let (done, _) = self
            .completion
            .changed
            .wait_timeout_while(done, self.client.shutdown_timeout(), |done| !*done)
            .unwrap_or_else(|e| e.into_inner());
        if !*done {
            return Err(ClockError::ShutdownTimeout {
                active: self.active_operations(),
            });
        }
        drop(done);
        if let Some(worker) = self.worker.lock().unwrap_or_else(|e| e.into_inner()).take() {
            worker.join().map_err(|_| ClockError::DriverStopped)?;
        }
        Ok(())
    }
}

impl Drop for OwnedDriver {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

/// Explicitly owned clock worker. Public behavior is unchanged; private worker
/// admission/teardown is shared with the one control-read operation.
pub struct ClockDriver(OwnedDriver);

impl ClockDriver {
    pub fn new(options: ClockOptions) -> Result<Self, ClockError> {
        OwnedDriver::new(Options::Clock(options))
            .map(Self)
            .map_err(|error| match error {
                StartError::Clock(error) | StartError::Driver(error) => error,
                StartError::Control(_) => ClockError::DriverStopped,
            })
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    pub fn active_operations(&self) -> usize {
        self.0.active_operations()
    }

    pub fn read_clock(&self, cancellation: &Cancellation) -> Result<ClockObservation, ClockError> {
        let (reply, result) = mpsc::sync_channel(1);
        let mut observation = None;
        self.0.submit(
            Request::Clock {
                cancellation: cancellation.clone(),
                reply,
            },
            || {
                observation = Some(result.recv().map_err(|_| ClockError::Closed)?);
                Ok(())
            },
        )?;
        observation.ok_or(ClockError::DriverStopped)?
    }

    /// Idempotent bounded close; success proves the owned worker was joined.
    pub fn close(&self) -> Result<(), ClockError> {
        self.0.close()
    }
}

/// One-operation-at-a-time blocking facade on an explicitly owned Tokio worker.
/// Safe to construct/drop under another executor; async callers should use
/// ControlReadClient. Concurrent calls return Busy, not an unbounded queue.
pub struct ControlReadDriver(OwnedDriver);

impl ControlReadDriver {
    pub fn new(options: ControlReadOptions) -> Result<Self, ControlReadError> {
        OwnedDriver::new(Options::Control(options))
            .map(Self)
            .map_err(|error| match error {
                StartError::Control(error) => error,
                StartError::Clock(error) | StartError::Driver(error) => control_driver_error(error),
            })
    }

    pub fn is_closed(&self) -> bool {
        self.0.is_closed()
    }

    pub fn active_operations(&self) -> usize {
        self.0.active_operations()
    }

    pub fn read_control(
        &self,
        resolved_target: &str,
        cancellation: &Cancellation,
    ) -> Result<ControlObservation, ControlReadError> {
        let (reply, result) = mpsc::sync_channel(1);
        let mut observation = None;
        self.0
            .submit(
                Request::Control {
                    target: resolved_target.into(),
                    cancellation: cancellation.clone(),
                    reply,
                },
                || {
                    observation = Some(result.recv().map_err(|_| ClockError::Closed)?);
                    Ok(())
                },
            )
            .map_err(control_driver_error)?;
        observation.ok_or_else(|| ControlReadError::new(ControlReadFailure::DriverStopped))?
    }

    /// Cancels pending I/O and joins the worker within shutdown_timeout.
    /// Timeout leaves the driver closed and may be retried.
    pub fn close(&self) -> Result<(), ControlReadError> {
        self.0.close().map_err(control_driver_error)
    }
}

fn control_driver_error(error: ClockError) -> ControlReadError {
    ControlReadError::new(match error {
        ClockError::Closed => ControlReadFailure::Closed,
        ClockError::Busy => ControlReadFailure::Busy,
        ClockError::ShutdownTimeout { active } => ControlReadFailure::ShutdownTimeout { active },
        _ => ControlReadFailure::DriverStopped,
    })
}
