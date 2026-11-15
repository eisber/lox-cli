package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;

/** Reuses the unchanged catalog error and generic prefix HeapWalk primitives. */
public final class NativeMetadataProbe {
    private static final SymbolLookup PROBE = SymbolLookup.libraryLookup(
            System.getProperty("lox.archive.heapProbe"), Arena.global());
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
    private static final MethodHandle ERROR_COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_catalog_error_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_LONG,
                    ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        StructureMetadata run(byte[] input) throws CatalogDecodeException;
    }

    private static long matching(byte[] prefix, boolean error) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code;
            if (error) {
                code = (int) ERROR_COUNT.invokeExact(258L, 65540L, count);
            } else {
                MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
                code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, 64000L, count);
            }
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        String marker = "metadata-owned-allocation-" + "x".repeat(65536);
        for (boolean error : new boolean[] {false, true}) {
            byte[] input = (error ? "\n".repeat(257) + " ".repeat(65539) + "!"
                    : "{\"cats\":{\"" + marker + "\":{\"name\":\"" + marker + "\"}}}")
                    .getBytes(StandardCharsets.UTF_8);
            byte[] prefix;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_loxapp3_structure_metadata(
                        arena, FfiConverterByteArray.INSTANCE.lower(input), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                long length = RustBuffer.getLen(buffer);
                if (error ? length != 20 : length < 64000) throw new AssertionError("Buffer size");
                prefix = new byte[error ? 0 : 128];
                if (!error) RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix, error) != 1) throw new AssertionError("Live calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix, error) != 0) throw new AssertionError("Released calibration");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    StructureMetadata value = operation.run(input);
                    if (error || !value.categories().get(0).uuid().equals(marker)
                            || !value.categories().get(0).name().equals(marker))
                        throw new AssertionError("Values");
                } catch (CatalogDecodeException.InvalidJson failure) {
                    if (!error || failure.line() != 258L || failure.column() != 65540L) throw failure;
                }
            }
            long retained = matching(prefix, error);
            System.out.println(language + " metadata error=" + error
                    + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained
                    + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Native metadata RustBuffer leak");
        }
    }
}
