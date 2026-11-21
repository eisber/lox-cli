package net.eisber.lox.archive;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;

/** Bounded host wait adapter; transport, validation and cancellation remain in Core. */
public final class ControlReadClient implements AutoCloseable {
    private final ControlReadRuntime runtime;
    private final ThreadPoolExecutor executor;
    private final long drainMillis;
    private final Object state = new Object();
    private final Object closeLock = new Object();
    private boolean reading;
    private boolean closing;
    private boolean disposed;

    public ControlReadClient(String host, String user, String password, ClockSettings settings)
            throws ControlReadException {
        drainMillis = settings.shutdownTimeoutMillis();
        if (drainMillis < 0) throw new IllegalArgumentException("Host drain timeout exceeds signed milliseconds");
        runtime = new ControlReadRuntime(host, user, password, settings);
        executor = new ThreadPoolExecutor(1, 1, 0, TimeUnit.MILLISECONDS,
                new ArrayBlockingQueue<>(1), task -> {
                    Thread thread = new Thread(task, "lox-control-read-ffi");
                    thread.setDaemon(false);
                    return thread;
                });
    }

    public CompletableFuture<ControlReading> readControl(String identifier) {
        synchronized (state) {
            if (closing) return CompletableFuture.failedFuture(new ControlReadException.Closed());
            if (reading) return CompletableFuture.failedFuture(new ControlReadException.Busy());
            ClockCancellation cancellation = new ClockCancellation();
            AtomicBoolean nativeFinished = new AtomicBoolean(false);
            CompletableFuture<ControlReading> result = new CompletableFuture<>() {
                @Override
                public boolean cancel(boolean mayInterruptIfRunning) {
                    synchronized (cancellation) {
                        if (isDone()) return false;
                        if (!nativeFinished.get()) cancellation.cancel();
                        return super.cancel(false);
                    }
                }
            };
            reading = true;
            try {
                executor.execute(() -> UniffiHelpers.uniffiRunWithOwnedStatus(() -> {
                    ControlReading value = null;
                    Throwable failure = null;
                    try {
                        value = runtime.readControl(identifier, cancellation);
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
                    if (failure == null) result.complete(value);
                    else result.completeExceptionally(failure);
                }));
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
    public void close() throws ControlReadException, InterruptedException, ControlReadDispatchTimeout {
        synchronized (closeLock) {
            synchronized (state) {
                if (disposed) return;
                closing = true;
            }
            runtime.shutdown();
            executor.shutdown();
            if (!executor.awaitTermination(drainMillis, TimeUnit.MILLISECONDS)) {
                throw new ControlReadDispatchTimeout();
            }
            synchronized (state) {
                runtime.close();
                disposed = true;
            }
        }
    }
}
