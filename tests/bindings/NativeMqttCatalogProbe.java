package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.util.List;

/** Reuses the established native HeapWalk prefix primitive for actual buffers. */
public final class NativeMqttCatalogProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        List<MqttTopic> run(byte[] input) throws MqttCatalogException;
    }

    private static long matching(byte[] prefix, long minimum) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, minimum, count);
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        String marker = "mqtt-owned-allocation-" + "x".repeat(65536);
        for (boolean error : new boolean[] {false, true}) {
            byte[] input = (error ? "<C>" : "<C Type=\"GenTSensor\"><SET><mqtt_topic v=\""
                    + marker + "\"/></SET></C>").getBytes(StandardCharsets.UTF_8);
            byte[] prefix;
            long length;
            long minimum;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_list_mqtt_topics(
                        arena, FfiConverterByteArray.INSTANCE.lower(input), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                length = RustBuffer.getLen(buffer);
                if (length < 16) throw new AssertionError("Small actual result");
                prefix = new byte[error ? 16 : 128];
                minimum = error ? length : 64000L;
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix, minimum) != 1) throw new AssertionError("Live calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix, minimum) != 0) throw new AssertionError("Released calibration");
            }
            MqttCatalogException.Load savedFailure = null;
            List<MqttTopic> savedValue = null;
            for (int i = 0; i < 64; i++) {
                try {
                    List<MqttTopic> values = operation.run(input);
                    if (error || !values.get(0).topic().equals(marker)) throw new AssertionError("Values");
                    savedValue = values;
                } catch (MqttCatalogException.Load failure) {
                    if (!error || failure.sourceMessage().isEmpty()) throw failure;
                    if (savedFailure == null) savedFailure = failure;
                    if (!savedFailure.sourceMessage().equals(failure.sourceMessage()))
                        throw new AssertionError("Retained exception changed");
                }
            }
            long retained = matching(prefix, minimum);
            if (retained != 0) throw new AssertionError("Native MQTT RustBuffer leak");
            if (error ? savedFailure == null : savedValue == null) throw new AssertionError("Owned values");
            System.out.println(language + " MQTT error=" + error + " actual_result_bytes=" + length
                    + " live=1 free=0 calls=64 retained=0 GC_requested=false");
        }
    }
}
