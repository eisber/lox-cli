package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.util.List;

/** Reuses the published large-buffer calibration on actual device exports. */
public final class NativeDeviceProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        List<XmlDeviceInfo> run(byte[] input) throws ConfigurationException;
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
        String marker = "xml-devices-allocation-" + "x".repeat(65536);
        for (boolean error : new boolean[] {false, true}) {
            byte[] input = (error ? "<" + marker + "></other>"
                    : "<C Type=\"TreeDevice\" Title=\"" + marker + "\" Serial=\"\" SubType=\"4294967295\"/>")
                    .getBytes(StandardCharsets.UTF_8);
            byte[] prefix;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_devices(
                        arena, FfiConverterByteArray.INSTANCE.lower(input), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Calibration outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                if (RustBuffer.getLen(buffer) < 64000) throw new AssertionError("Small calibration");
                prefix = new byte[128];
                RustBuffer.asByteBuffer(buffer).get(prefix);
                long live = matching(prefix);
                System.out.println(language + " actual device export error=" + error
                        + " len=" + RustBuffer.getLen(buffer) + " live=" + live);
                if (live != 1) throw new AssertionError("Expected live device calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix) != 0) throw new AssertionError("Device calibration did not release");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    var result = operation.run(input);
                    if (error || !result.equals(List.of(new XmlDeviceInfo(
                            marker, XmlDeviceBus.TREE, "", 4294967295L,
                            "Unknown (4294967295)", null, null)))) {
                        throw new AssertionError("Device structured result lost");
                    }
                } catch (ConfigurationException.Xml failure) {
                    if (!error || failure.sourceMessage().length() < 65536) throw failure;
                }
            }
            long retained = matching(prefix);
            System.out.println(language + " devices error=" + error
                    + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained
                    + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Native device RustBuffer leak");
        }
    }
}
