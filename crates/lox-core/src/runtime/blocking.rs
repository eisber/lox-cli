use super::*;
use std::sync::{Condvar, Mutex, mpsc};
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc as queue;

struct Request {
    cancellation: Cancellation,
    reply: mpsc::SyncSender<Result<ClockObservation, ClockError>>,
}

#[derive(Default)]
struct Completion {
    done: Mutex<bool>,
    changed: Condvar,
}

struct Finished(Arc<Completion>);

struct CloseOnDrop(ClockClient);

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
pub struct ClockDriver {
    client: ClockClient,
    sender: queue::Sender<Request>,
    completion: Arc<Completion>,
    worker: Mutex<Option<JoinHandle<()>>>,
    admission: Mutex<()>,
}

impl ClockDriver {
    pub fn new(options: ClockOptions) -> Result<Self, ClockError> {
        let (sender, mut receiver) = queue::channel::<Request>(1);
        let (ready, result) = mpsc::sync_channel(1);
        let completion = Arc::new(Completion::default());
        let finished = Finished(completion.clone());
        let worker = thread::Builder::new()
            .name("lox-clock".into())
            .spawn(move || {
                let _finished = finished;
                let runtime = match tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                {
                    Ok(runtime) => runtime,
                    Err(_) => {
                        let _ = ready.send(Err(ClockError::DriverStopped));
                        return;
                    }
                };
                runtime.block_on(async move {
                    let client = match ClockClient::new(options) {
                        Ok(client) => client,
                        Err(error) => {
                            let _ = ready.send(Err(error));
                            return;
                        }
                    };
                    let _close = CloseOnDrop(client.clone());
                    if ready.send(Ok(client.clone())).is_err() {
                        return;
                    }
                    loop {
                        tokio::select! {
                            biased;
                            _ = client.shutdown_signal().cancelled() => break,
                            request = receiver.recv() => match request {
                                Some(request) => {
                                    let result = client.read_clock(&request.cancellation).await;
                                    let _ = request.reply.send(result);
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
            .map_err(|_| ClockError::DriverStopped)?;
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
                    .unwrap_or(ClockError::DriverStopped))
            }
        }
    }

    pub fn is_closed(&self) -> bool {
        self.client.is_closed()
    }

    pub fn active_operations(&self) -> usize {
        self.client.active_operations()
    }

    pub fn read_clock(&self, cancellation: &Cancellation) -> Result<ClockObservation, ClockError> {
        if self.is_closed() {
            return Err(ClockError::Closed);
        }
        let _admission = match self.admission.try_lock() {
            Ok(guard) => guard,
            Err(std::sync::TryLockError::WouldBlock) => return Err(ClockError::Busy),
            Err(std::sync::TryLockError::Poisoned(_)) => return Err(ClockError::DriverStopped),
        };
        let (reply, result) = mpsc::sync_channel(1);
        self.sender
            .try_send(Request {
                cancellation: cancellation.clone(),
                reply,
            })
            .map_err(|error| match error {
                queue::error::TrySendError::Full(_) => ClockError::Busy,
                queue::error::TrySendError::Closed(_) => ClockError::Closed,
            })?;
        result.recv().map_err(|_| ClockError::Closed)?
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

impl Drop for ClockDriver {
    fn drop(&mut self) {
        let _ = self.close();
    }
}
