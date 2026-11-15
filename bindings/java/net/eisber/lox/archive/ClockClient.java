package net.eisber.lox.archive;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;

/** Bounded host wait adapter; all I/O, retries and validation remain in Rust. */
public final class ClockClient implements AutoCloseable {
    private final ClockRuntime runtime;
    private final ThreadPoolExecutor executor;
    private final long drainMillis;
    private final Object state = new Object();
    private final Object closeLock = new Object();
    private boolean reading;
    private boolean closing;
    private boolean disposed;

    public ClockClient(String host, String user, String password, ClockSettings settings)
            throws ClockRuntimeException {
        drainMillis = settings.shutdownTimeoutMillis();
        if (drainMillis < 0) throw new IllegalArgumentException("Host drain timeout exceeds signed milliseconds");
        runtime = new ClockRuntime(host, user, password, settings);
        executor = new ThreadPoolExecutor(1, 1, 0, TimeUnit.MILLISECONDS,
                new ArrayBlockingQueue<>(1), task -> {
                    Thread thread = new Thread(task, "lox-clock-ffi");
                    thread.setDaemon(false);
                    return thread;
                });
    }

    public CompletableFuture<ClockReading> readClock() {
        synchronized (state) {
            if (closing) return CompletableFuture.failedFuture(new ClockRuntimeException.Closed());
            if (reading) return CompletableFuture.failedFuture(new ClockRuntimeException.Busy());
            ClockCancellation cancellation = new ClockCancellation();
            AtomicBoolean nativeFinished = new AtomicBoolean(false);
            CompletableFuture<ClockReading> result = new CompletableFuture<>() {
                @Override
                public boolean cancel(boolean mayInterruptIfRunning) {
                    synchronized (cancellation) {
                        if (isDone()) return false;
                        // Interrupting/abandoning a Java wait does not cancel native I/O.
                        if (!nativeFinished.get()) cancellation.cancel();
                        return super.cancel(false);
                    }
                }
            };
            reading = true;
            try {
                executor.execute(() -> {
                    ClockReading value = null;
                    Throwable failure = null;
                    try {
                        value = runtime.readClock(cancellation);
                    } catch (Throwable error) {
                        failure = error;
                    }
                    try {
                        synchronized (cancellation) {
                            nativeFinished.set(true);
                            cancellation.close();
                        }
                    } catch (Throwable error) {
                        failure = error;
                    }
                    synchronized (state) {
                        reading = false;
                    }
                    // Release admission before publishing completion/inline caller continuations.
                    if (failure == null) result.complete(value);
                    else result.completeExceptionally(failure);
                });
            } catch (RuntimeException error) {
                reading = false;
                synchronized (cancellation) {
                    nativeFinished.set(true);
                    cancellation.close();
                }
                result.completeExceptionally(error);
            }
            return result;
        }
    }

    public long activeOperations() {
        synchronized (state) {
            return disposed ? 0 : runtime.activeOperations();
        }
    }

    public boolean isClosed() {
        synchronized (state) {
            return closing;
        }
    }

    @Override
    public void close() throws ClockRuntimeException, InterruptedException, ClockDispatchTimeout {
        synchronized (closeLock) {
            synchronized (state) {
                if (disposed) return;
                closing = true;
            }
            // Cancel and join the production Rust worker before releasing its native handle.
            runtime.shutdown();
            executor.shutdown();
            if (!executor.awaitTermination(drainMillis, TimeUnit.MILLISECONDS)) {
                throw new ClockDispatchTimeout();
            }
            synchronized (state) {
                runtime.close();
                disposed = true;
            }
        }
    }
}
