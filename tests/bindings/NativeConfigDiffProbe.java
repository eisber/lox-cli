package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.List;
import java.util.Map;

/** Reuses the unchanged large RustBuffer probe on the actual infallible export. */
public final class NativeConfigDiffProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        ConfigDiff run(ConfigSummary old, ConfigSummary newer);
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
        String marker = "config-diff-owned-jvm-" + "x".repeat(65536);
        ConfigSummary old = new ConfigSummary(marker, "", Map.of(), Map.of(), Map.of(), List.of());
        ConfigSummary newer = new ConfigSummary("new", "", Map.of(), Map.of(), Map.of(), List.of());
        byte[] prefix = new byte[128];
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_diff_configs(
                    arena, FfiConverterTypeConfigSummary.INSTANCE.lower(old),
                    FfiConverterTypeConfigSummary.INSTANCE.lower(newer), status);
            if (UniffiRustCallStatus.isError(status)) throw new AssertionError("Infallible diff failed");
            if (RustBuffer.getLen(result) < 64000) throw new AssertionError("Small actual result");
            RustBuffer.asByteBuffer(result).get(prefix);
            if (matching(prefix) != 1) throw new AssertionError("Actual result not live");
            System.out.println(language + " actual diff export len=" + RustBuffer.getLen(result) + " live=1");
            RustBuffer.free(result);
            if (matching(prefix) != 0) throw new AssertionError("Actual result not released");
        }
        for (int i = 0; i < 64; i++) {
            ConfigDiff result = operation.run(old, newer);
            if (!result.versionOld().equals(marker) || result.hasChanges()) throw new AssertionError("Values");
        }
        long retained = matching(prefix);
        System.out.println(language + " diff calls=64 calibrated_live=1 calibrated_released=0 retained_buffers="
                + retained + " GC_requested=false");
        if (retained != 0) throw new AssertionError("Native diff RustBuffer leak");
    }
}
