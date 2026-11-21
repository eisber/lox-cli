package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;

/** Test-only positive calibration of actual return/error RustBuffers; no GC. */
public final class NativeControlReadProbe {
    private static final String ID = "1fbc668c-005c-7471-ffffed57184a04d2";
    private static final String USER = "control-user";
    private static final String PASSWORD = "control-secret";
    private static final AtomicInteger SERIAL = new AtomicInteger();
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_control_error_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_LONG,
                    ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    private static ClockSettings settings(String mode) {
        ClockSettings result = LoxArchive.defaultClockSettings();
        result.setUseSystemProxy(false);
        result.setTimeoutMillis(mode.equals("timeout") ? 150 : 60000);
        result.setRetryDelaysMillis(new long[0]);
        result.setShutdownTimeoutMillis(2000);
        return result;
    }

    private static long matching(long marker, long length) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int error = (int) COUNT.invokeExact(marker, length, count);
            if (error != 0) throw new AssertionError("HeapWalk: " + error);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    private static void waitFile(Path path) {
        long deadline = System.nanoTime() + TimeUnit.SECONDS.toNanos(5);
        while (!Files.exists(path)) {
            if (System.nanoTime() > deadline) throw new AssertionError("Control buffer fixture deadline");
            try { Thread.sleep(5); }
            catch (InterruptedException error) { throw new AssertionError(error); }
        }
    }

    private static final class Operation implements AutoCloseable {
        final ControlReadRuntime runtime;
        final ClockCancellation cancellation = new ClockCancellation();
        final String identifier;
        final ExecutorService executor;
        final Future<?> pending;
        final Path eof;

        Operation(String language, String host, String refused, Path signals, String mode) throws Exception {
            String caseName = language + "-control-buffers/" + SERIAL.incrementAndGet() + "/" + mode;
            boolean busy = mode.equals("busy");
            boolean shutdown = mode.equals("shutdown");
            if (busy) caseName += "/blocked";
            if (shutdown) caseName += "/close";
            String address = mode.equals("connect") ? refused : host + "/" + caseName;
            runtime = new ControlReadRuntime(address, USER, PASSWORD, settings(mode));
            identifier = switch (mode) {
                case "length" -> "";
                case "characters" -> "../private";
                case "nonhex" -> "secret";
                default -> ID;
            };
            if (mode.equals("closed")) runtime.shutdown();
            if (mode.equals("precancel")) cancellation.cancel();
            eof = busy || shutdown || mode.equals("timeout")
                    ? signals.resolve(caseName.replace("/", "__") + "__eof") : null;
            if (busy || shutdown) {
                executor = Executors.newSingleThreadExecutor();
                Path entered = signals.resolve(caseName.replace("/", "__") + "__entered");
                pending = executor.submit(() -> {
                    try {
                        if (busy) {
                            try (ClockCancellation token = new ClockCancellation()) {
                                runtime.readControl(ID, token);
                                throw new AssertionError("blocked native request returned normally");
                            } catch (ControlReadException.CancelledShutdown expected) {}
                        } else {
                            waitFile(entered);
                            runtime.shutdown();
                        }
                    } catch (ControlReadException error) {
                        throw new AssertionError(error);
                    }
                });
                if (busy) waitFile(entered);
            } else {
                executor = null;
                pending = null;
            }
        }

        MemorySegment raw(MemorySegment status, Arena arena) {
            return UniffiLib.uniffi_lox_archive_fn_method_controlreadruntime_read_control(
                    arena, runtime.uniffiCloneHandle(), FfiConverterString.INSTANCE.lower(identifier),
                    FfiConverterTypeClockCancellation.INSTANCE.lower(cancellation), status);
        }

        ControlReading call() throws ControlReadException {
            return runtime.readControl(identifier, cancellation);
        }

        public void close() throws Exception {
            runtime.shutdown();
            if (pending != null) pending.get(5, TimeUnit.SECONDS);
            if (executor != null) {
                executor.shutdown();
                if (!executor.awaitTermination(5, TimeUnit.SECONDS)) throw new AssertionError("probe wait lane");
            }
            if (eof != null) waitFile(eof);
            cancellation.close();
            runtime.close();
        }
    }

    public static void verify(String language, String host, String refused, Path signals) throws Throwable {
        String[] modes = {"normal", "length", "characters", "nonhex", "envelope", "missing-code",
                "unsupported-code", "status", "conflict", "missing", "nontext", "json", "utf8",
                "body", "http", "connect", "timeout", "precancel", "closed", "busy", "shutdown"};
        for (String mode : modes) {
            boolean errorPath = !mode.equals("normal");
            byte[] prefix = null;
            long marker = 0;
            long length;
            try (Operation operation = new Operation(language, host, refused, signals, mode);
                    Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = operation.raw(status, arena);
                if (UniffiRustCallStatus.isError(status) != errorPath) {
                    throw new AssertionError("calibration outcome " + mode);
                }
                MemorySegment buffer = errorPath ? UniffiRustCallStatus.getErrorBuf(status) : result;
                length = RustBuffer.getLen(buffer);
                if (errorPath) {
                    if (length > 8) throw new AssertionError("unexpected small error layout");
                    var bytes = RustBuffer.asByteBuffer(buffer);
                    for (int i = 0; i < length; i++) marker |= (bytes.get() & 255L) << (i * 8);
                } else {
                    prefix = new byte[128];
                    RustBuffer.asByteBuffer(buffer).get(prefix);
                }
                long live = errorPath ? matching(marker, length) : NativeColorProbe.matching(prefix, length);
                if (live != 1) throw new AssertionError("calibration live " + mode + ": " + live);
                if (errorPath) RustBuffer.free(buffer);
                RustBuffer.free(result);
                long freed = errorPath ? matching(marker, length) : NativeColorProbe.matching(prefix, length);
                if (freed != 0) throw new AssertionError("calibration freed " + mode + ": " + freed);
            }
            for (int i = 0; i < 64; i++) {
                try (Operation operation = new Operation(language, host, refused, signals, mode)) {
                    try {
                        ControlReading reading = operation.call();
                        if (errorPath || !reading.code().equals("200") || reading.value().length() < 65536) {
                            throw new AssertionError("expected success/error " + mode);
                        }
                    } catch (ControlReadException failure) {
                        if (!errorPath) throw failure;
                    }
                }
            }
            long retained = errorPath ? matching(marker, length) : NativeColorProbe.matching(prefix, length);
            System.out.println(language + " control path=" + mode
                    + " calls=64 calibrated_live=1 calibrated_freed=0 retained=" + retained + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Control RustBuffer leak " + mode);
        }
        long marker = 0;
        long length;
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            long handle = UniffiLib.uniffi_lox_archive_fn_constructor_controlreadruntime_new(
                    FfiConverterString.INSTANCE.lower("bad host control-secret"),
                    FfiConverterString.INSTANCE.lower(USER), FfiConverterString.INSTANCE.lower(PASSWORD),
                    FfiConverterTypeClockSettings.INSTANCE.lower(settings("")), status);
            if (handle != 0 || !UniffiRustCallStatus.isError(status)) throw new AssertionError("constructor calibration");
            MemorySegment buffer = UniffiRustCallStatus.getErrorBuf(status);
            length = RustBuffer.getLen(buffer);
            var bytes = RustBuffer.asByteBuffer(buffer);
            for (int i = 0; i < length; i++) marker |= (bytes.get() & 255L) << (i * 8);
            if (matching(marker, length) != 1) throw new AssertionError("constructor live calibration");
            RustBuffer.free(buffer);
            if (matching(marker, length) != 0) throw new AssertionError("constructor freed calibration");
        }
        for (int i = 0; i < 64; i++) {
            try {
                new ControlReadRuntime("bad host control-secret", USER, PASSWORD, settings(""));
                throw new AssertionError("invalid host accepted");
            } catch (ControlReadException.InvalidHost expected) {}
        }
        if (matching(marker, length) != 0) throw new AssertionError("constructor error buffer retained");
        System.out.println(language + " control path=constructor-error calls=64 calibrated_live=1 calibrated_freed=0 retained=0 GC_requested=false");
    }
}
