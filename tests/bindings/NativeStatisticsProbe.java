package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.List;

/** Test-only exact statistics error tag and calibrated owned result checks. */
public final class NativeStatisticsProbe {
    private static final SymbolLookup PROBE = SymbolLookup.libraryLookup(
            System.getProperty("lox.archive.heapProbe"), Arena.global());
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
    private static final MethodHandle ERROR_COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_statistics_error_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        StatSeries run(String input) throws StatisticsException;
    }

    private static long matching(byte[] prefix, boolean error) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code;
            if (error) {
                code = (int) ERROR_COUNT.invokeExact(count);
            } else {
                MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
                code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, 64000L, count);
            }
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        String marker = "monthly-statistics-owned-" + "x".repeat(65536);
        for (boolean error : new boolean[] {false, true}) {
            String input = error ? "<Statistics><S T=\"secret\" V=\"NaN\"/></Statistics>"
                    : "<Statistics Name=\"" + marker + "\" StatsGroup=\"4294967295\" Outputs=\"a,b\">"
                    + "<S T=\"opaque &amp; duplicate\" V=\"-0\" V2=\"2\"/>"
                    + "<S T=\"opaque &amp; duplicate\" V=\"3\" V2=\"4\"/></Statistics>";
            byte[] prefix;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_statistics_series(
                        arena, FfiConverterString.INSTANCE.lower(input), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                long length = RustBuffer.getLen(buffer);
                if (error ? length != 4 : length < 64000) throw new AssertionError("Buffer size");
                prefix = new byte[error ? 0 : 128];
                if (!error) RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix, error) != 1) throw new AssertionError("Live calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix, error) != 0) throw new AssertionError("Released calibration");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    StatSeries series = operation.run(input);
                    if (error || !series.name().equals(marker) || series.group() != 4294967295L
                            || !series.outputs().equals(List.of("a", "b"))
                            || !series.rows().equals(List.of(
                                    new StatRow("opaque & duplicate", List.of(-0.0, 2.0)),
                                    new StatRow("opaque & duplicate", List.of(3.0, 4.0))))
                            || Double.doubleToRawLongBits(series.rows().get(0).values().get(0))
                                    != Long.MIN_VALUE) throw new AssertionError("Values");
                } catch (StatisticsException.NonFiniteValue failure) {
                    if (!error || failure.toString().contains("secret")) throw failure;
                }
            }
            long retained = matching(prefix, error);
            System.out.println(language + " statistics error=" + error
                    + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained
                    + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Native statistics RustBuffer leak");
        }
    }
}
