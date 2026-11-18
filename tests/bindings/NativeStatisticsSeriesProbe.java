package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;

/** Actual monthly-series success/error buffers, using the unchanged HeapWalk primitive. */
public final class NativeStatisticsSeriesProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        StatSeries run(String input) throws StatisticsException;
    }

    private static long matching(byte[] prefix) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int error = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, 64000L, count);
            if (error != 0) throw new AssertionError("HeapWalk error: " + error);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        String marker = "series-owned-allocation-" + "x".repeat(65536);
        for (boolean error : new boolean[] {false, true}) {
            String input = error
                    ? "<Statistics><S T=\"first\" V=\"1\"/><S T=\"later\" V=\"" + marker + "\"/></Statistics>"
                    : "<Statistics Name=\"" + marker + "\"><S T=\"raw\" V=\"-0\"/></Statistics>";
            byte[] prefix = new byte[128];
            long length;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_statistics_series(
                        arena, FfiConverterString.INSTANCE.lower(input), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                length = RustBuffer.getLen(buffer);
                if (length < 64000) throw new AssertionError("Small actual result");
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix) != 1) throw new AssertionError("Live result calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix) != 0) throw new AssertionError("Released result calibration");
            }
            StatSeries retainedValue = null;
            StatisticsException.InvalidValue retainedFailure = null;
            for (int i = 0; i < 64; i++) {
                try {
                    StatSeries value = operation.run(input);
                    if (error || !value.name().equals(marker) || value.rows().size() != 1
                            || !value.rows().getFirst().timestamp().equals("raw")
                            || Double.doubleToRawLongBits(value.rows().getFirst().values()[0])
                            != Long.MIN_VALUE) throw new AssertionError("Owned values");
                    if (i == 0) retainedValue = value;
                } catch (StatisticsException.InvalidValue failure) {
                    if (!error || !failure.timestamp().equals("later")
                            || !failure.attribute().equals("V") || !failure.value().equals(marker)
                            || !failure.sourceMessage().equals("invalid float literal")) throw failure;
                    if (i == 0) retainedFailure = failure;
                }
            }
            if (error ? retainedFailure == null || !retainedFailure.value().equals(marker)
                    : retainedValue == null || !retainedValue.name().equals(marker)) {
                throw new AssertionError("Retained host values");
            }
            long retained = matching(prefix);
            System.out.println(language + " statistics-series error=" + error
                    + " actual_result_bytes=" + length + " live=1 free=0 calls=64 retained=" + retained
                    + " GC_requested=false String_lowering=generated");
            if (retained != 0) throw new AssertionError("Native statistics series RustBuffer leak");
        }
    }
}
