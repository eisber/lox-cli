package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.List;

/** Actual export buffers, using the existing prefix HeapWalk primitive. */
public final class NativeAutopilotRuleProbe {
    private static final SymbolLookup PROBE = SymbolLookup.libraryLookup(
            System.getProperty("lox.archive.heapProbe"), Arena.global());
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        AutopilotRule run(List<AutopilotRule> rows, String query) throws RuleResolutionException;
    }

    private static long matching(byte[] prefix) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
            int code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, 64000L, count);
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        for (int branch = 0; branch < 4; branch++) {
            String marker = "autopilot-selector-owned-" + "x".repeat(65536);
            String query = branch == 1 ? "a".repeat(65536) + "-" : marker;
            List<AutopilotRule> rows = switch (branch) {
                case 0 -> List.of(new AutopilotRule(marker, "raw\0ID", "raw/\0", "<history>\r\n"));
                case 3 -> List.of(new AutopilotRule(marker + " extra", "same", "raw/\0", "<history>\r\n"),
                        new AutopilotRule(marker, "same", "raw/\0", "<history>\r\n"));
                default -> List.of();
            };
            byte[] prefix = new byte[128];
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_resolve_autopilot_rule(
                        arena, FfiConverterSequenceTypeAutopilotRule.INSTANCE.lower(rows),
                        FfiConverterString.INSTANCE.lower(query), status);
                if (UniffiRustCallStatus.isError(status) != (branch != 0)) throw new AssertionError("Outcome");
                MemorySegment buffer = branch == 0 ? result : UniffiRustCallStatus.getErrorBuf(status);
                if (RustBuffer.getLen(buffer) < 64000) throw new AssertionError("Buffer size");
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix) != 1) throw new AssertionError("Live calibration");
                if (branch != 0) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix) != 0) throw new AssertionError("Released calibration");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    AutopilotRule value = operation.run(rows, query);
                    if (branch != 0 || !value.equals(rows.get(0))) throw new AssertionError("Owned values");
                } catch (RuleResolutionException.UuidNotFound failure) {
                    if (branch != 1 || !failure.query().equals(query)) throw failure;
                } catch (RuleResolutionException.NotFound failure) {
                    if (branch != 2 || !failure.query().equals(query)) throw failure;
                } catch (RuleResolutionException.Ambiguous failure) {
                    if (branch != 3 || !failure.query().equals(query)
                            || !failure.candidates().equals(rows)) throw failure;
                }
            }
            long retained = matching(prefix);
            System.out.println(language + " autopilot-selector branch=" + branch
                    + " calibrated_live=1 calibrated_released=0 calls=64 retained_buffers=" + retained
                    + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Autopilot selector RustBuffer leak");
        }
    }
}
