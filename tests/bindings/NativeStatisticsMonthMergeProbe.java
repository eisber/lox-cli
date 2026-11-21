package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.ArrayList;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Set;

/** Calibrate actual results/errors with the unchanged generic HeapWalk matcher. */
public final class NativeStatisticsMonthMergeProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        StatSeries run(List<StatMonthPart> parts) throws StatisticsMergeException;
    }

    private static long matching(byte[] prefix) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, 64000L, count);
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        List<String> outputs = List.of(" A ", "");
        List<String> different = List.of("", " A ");
        for (boolean error : new boolean[] {false, true}) {
            String marker = "month-merge-owned-" + error + "-" + "x".repeat(65536);
            List<StatMonthPart> parts = new ArrayList<>();
            parts.add(new StatMonthPart("first", new StatSeries(marker, null, outputs,
                    List.of(new StatRow("raw", new double[] {-0.0})))));
            if (error) {
                parts.add(new StatMonthPart("empty", new StatSeries("ignored", 0, outputs, List.of())));
                parts.add(new StatMonthPart("bad", new StatSeries("ignored", null, different,
                        List.of(new StatRow("raw", new double[0])))));
            }
            byte[] prefix = new byte[128];
            long length;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_merge_statistics_months(
                        arena, FfiConverterSequenceTypeStatMonthPart.INSTANCE.lower(parts), status);
                if (UniffiRustCallStatus.getCode(status) != (error ? 1 : 0)) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                length = RustBuffer.getLen(buffer);
                if (length < 64000) throw new AssertionError("Small actual result");
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix) != 1) throw new AssertionError("Live result calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix) != 0) throw new AssertionError("Released result calibration");
            }
            Set<Object> retained = Collections.newSetFromMap(new IdentityHashMap<>());
            for (int i = 0; i < 64; i++) {
                try {
                    StatSeries value = operation.run(parts);
                    if (error) throw new AssertionError("Unexpected successful merge");
                    retained.add(value);
                } catch (StatisticsMergeException.OutputMismatch failure) {
                    if (!error) throw failure;
                    retained.add(failure);
                }
            }
            parts.clear();
            if (retained.size() != 64) throw new AssertionError("Expected 64 distinct retained host objects");
            long nativeCount = matching(prefix);
            if (nativeCount != 0) throw new AssertionError("Native statistics month merge RustBuffer leak");
            for (Object value : retained) {
                if (error) {
                    var failure = (StatisticsMergeException.OutputMismatch) value;
                    if (!failure.name().equals(marker) || !failure.previousMonth().equals("empty")
                            || !failure.month().equals("bad") || !failure.previousOutputs().equals(outputs)
                            || !failure.outputs().equals(different)) throw new AssertionError("Owned mismatch");
                } else {
                    var series = (StatSeries) value;
                    if (!series.name().equals(marker) || series.group() != null || !series.outputs().equals(outputs)
                            || series.rows().size() != 1 || !series.rows().getFirst().timestamp().equals("raw")
                            || series.rows().getFirst().values().length != 1
                            || Double.doubleToRawLongBits(series.rows().getFirst().values()[0]) != Long.MIN_VALUE) {
                        throw new AssertionError("Owned series");
                    }
                }
            }
            System.out.println(language + " statistics-month-merge error=" + error
                    + " actual_result_bytes=" + length + " live=1 free=0 calls=64 host_retained=64"
                    + " native_retained=0 GC_requested=false");
        }
    }
}
