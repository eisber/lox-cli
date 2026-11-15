package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.util.List;

/** Reuses the published large-buffer guard on actual user inventory exports. */
public final class NativeUserProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        List<XmlUserInfo> run(byte[] input) throws ConfigurationException;
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
        String marker = "xml-users-allocation-" + "x".repeat(65536);
        for (boolean error : new boolean[] {false, true}) {
            byte[] input = (error ? "<" + marker + "></other>"
                    : "<C Type=\"User\" Title=\"" + marker + "\" U=\"raw_uuid\" Desc=\"raw_desc\" NFCArr=\"0\"/>")
                    .getBytes(StandardCharsets.UTF_8);
            byte[] prefix;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_users(
                        arena, FfiConverterByteArray.INSTANCE.lower(input), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Calibration outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                if (RustBuffer.getLen(buffer) < 64000) throw new AssertionError("Small calibration");
                prefix = new byte[128];
                RustBuffer.asByteBuffer(buffer).get(prefix);
                long live = matching(prefix);
                System.out.println(language + " actual user export error=" + error
                        + " len=" + RustBuffer.getLen(buffer) + " live=" + live);
                if (live != 1) throw new AssertionError("Expected live user calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix) != 0) throw new AssertionError("User calibration did not release");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    var result = operation.run(input);
                    if (error || !result.equals(List.of(new XmlUserInfo(
                            marker, "raw_uuid", true, "raw desc")))) {
                        throw new AssertionError("User structured result lost");
                    }
                } catch (ConfigurationException.Xml failure) {
                    if (!error || failure.sourceMessage().length() < 65536) throw failure;
                }
            }
            long retained = matching(prefix);
            System.out.println(language + " users error=" + error
                    + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained
                    + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Native user RustBuffer leak");
        }
    }
}
