package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;

/** Reuses only the existing large-buffer HeapWalk prefix primitive. */
public final class NativeFirmwareMetadataProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        UpdateMetadata run(String input) throws FirmwareMetadataException;
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
        String marker = "firmware-owned-allocation-" + "x".repeat(65536);
        for (boolean error : new boolean[] {false, true}) {
            String input = "<Miniserversoftware><Release Version=\"" + (error ? marker : "1.2")
                    + "\" Path=\"" + (error ? "unused" : marker) + "\"/></Miniserversoftware>";
            byte[] prefix;
            long length;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_firmware_metadata(
                        arena, FfiConverterString.INSTANCE.lower(input), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                length = RustBuffer.getLen(buffer);
                if (length < 64000) throw new AssertionError("Small actual result");
                prefix = new byte[128];
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix) != 1) throw new AssertionError("Live calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix) != 0) throw new AssertionError("Released calibration");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    UpdateMetadata value = operation.run(input);
                    if (error || !value.release().configDownload().equals(marker))
                        throw new AssertionError("Owned values");
                } catch (FirmwareMetadataException.InvalidVersion failure) {
                    if (!error || !failure.value().equals(marker)
                            || !failure.message().equals("invalid numeric firmware version \"" + marker + "\""))
                        throw failure;
                }
            }
            long retained = matching(prefix);
            System.out.println(language + " firmware error=" + error + " actual_result_bytes=" + length
                    + " live=1 free=0 calls=64 retained=" + retained + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Native firmware result RustBuffer leak");
        }
    }
}
