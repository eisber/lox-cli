package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.List;

/** Selects the actual infallible String export with the unchanged large-buffer matcher. */
public final class NativeStatisticsIndexProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        List<StatEntry> run(String input);
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
        String marker = "index-owned-allocation-" + "x".repeat(65536);
        String input = "<li><a href=\"01234567-89ab-cdef-0123456789abcdef.202610.xml\">"
                + marker + "</a>";
        byte[] prefix;
        long length;
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_statistics_index(
                    arena, FfiConverterString.INSTANCE.lower(input), status);
            if (UniffiRustCallStatus.isError(status)) throw new AssertionError("Unexpected FFI status");
            length = RustBuffer.getLen(result);
            if (length < 64000) throw new AssertionError("Small actual result");
            prefix = new byte[128];
            RustBuffer.asByteBuffer(result).get(prefix);
            if (matching(prefix) != 1) throw new AssertionError("Live result calibration");
            RustBuffer.free(result);
            if (matching(prefix) != 0) throw new AssertionError("Released result calibration");
        }
        for (int i = 0; i < 64; i++) {
            List<StatEntry> rows = operation.run(input);
            if (rows.size() != 1 || !rows.getFirst().name().equals(marker)
                    || !rows.getFirst().label().equals(marker) || rows.getFirst().group() != null) {
                throw new AssertionError("Owned values");
            }
        }
        long retained = matching(prefix);
        System.out.println(language + " statistics-index actual_result_bytes=" + length
                + " live=1 free=0 calls=64 retained=" + retained
                + " GC_requested=false String_lowering=generated");
        if (retained != 0) throw new AssertionError("Native statistics index RustBuffer leak");
    }
}
