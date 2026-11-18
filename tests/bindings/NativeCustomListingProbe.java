package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

/** Actual result/error RustBuffers, using the existing native HeapWalk primitive. */
public final class NativeCustomListingProbe {
    private static final SymbolLookup PROBE = SymbolLookup.libraryLookup(
            System.getProperty("lox.archive.heapProbe"), Arena.global());
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
    private static final MethodHandle ERROR_COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_custom_listing_error_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_INT,
                    ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        List<String> run(List<String> rows) throws CustomListingException;
    }

    private static long matching(byte[] prefix, int variant) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code;
            if (variant != 0) {
                code = (int) ERROR_COUNT.invokeExact(variant, 4097L, count);
            } else {
                MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
                code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, 64000L, count);
            }
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        String name = "custom-listing-owned-" + "x".repeat(65536) + ".loXcC";
        for (int variant = 0; variant < 3; variant++) {
            List<String> rows = new ArrayList<>();
            if (variant == 0) {
                rows.add("- " + name);
            } else {
                rows.addAll(Collections.nCopies(4097, "- ignored"));
                rows.add(variant == 1 ? "" : "d private.loxcc");
            }
            byte[] prefix;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_custom_listing(
                        arena, FfiConverterSequenceString.INSTANCE.lower(rows), status);
                if (UniffiRustCallStatus.isError(status) != (variant != 0)) throw new AssertionError("Outcome");
                MemorySegment buffer = variant != 0 ? UniffiRustCallStatus.getErrorBuf(status) : result;
                long length = RustBuffer.getLen(buffer);
                if (variant != 0 ? length != 12 : length < 64000) throw new AssertionError("Buffer size");
                prefix = new byte[variant != 0 ? 0 : 128];
                if (variant == 0) RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix, variant) != 1) throw new AssertionError("Live calibration");
                if (variant != 0) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix, variant) != 0) throw new AssertionError("Released calibration");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    List<String> values = operation.run(rows);
                    if (variant != 0 || !values.equals(List.of(name))) throw new AssertionError("Owned values");
                } catch (CustomListingException.Malformed failure) {
                    if (variant != 1 || failure.row() != 4097) throw failure;
                } catch (CustomListingException.Unsupported failure) {
                    if (variant != 2 || failure.row() != 4097) throw failure;
                }
            }
            long retained = matching(prefix, variant);
            System.out.println(language + " custom-listing variant=" + variant
                    + " calibrated_live=1 calibrated_released=0 calls=64 retained_buffers=" + retained
                    + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Custom listing RustBuffer leak");
        }
    }
}
