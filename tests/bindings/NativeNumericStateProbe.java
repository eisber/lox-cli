package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.Arrays;
import java.util.List;

/** Test-only calibrated RustBuffer checks, without forcing JVM GC. */
public final class NativeNumericStateProbe {
    private static final SymbolLookup PROBE = SymbolLookup.libraryLookup(
            System.getProperty("lox.archive.heapProbe"), Arena.global());
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
    // Same UniFFI tag + two-u64 layout; reuse the by-value error probe.
    private static final MethodHandle ERROR_COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_catalog_error_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_LONG,
                    ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        List<NumericState> run(byte[] input) throws NumericStatePayloadException;
    }

    private static long matching(byte[] prefix, boolean error) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code;
            if (error) {
                code = (int) ERROR_COUNT.invokeExact(65537L, 17L, count);
            } else {
                MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
                code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, 64000L, count);
            }
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        for (boolean error : new boolean[] {false, true}) {
            byte[] input = new byte[error ? 65537 : 24 * 4096];
            Arrays.fill(input, (byte) 0xa5);
            if (!error) {
                ByteBuffer data = ByteBuffer.wrap(input).order(ByteOrder.LITTLE_ENDIAN);
                for (int i = 0; i < 4096; i++) data.putLong(i * 24 + 16, 0xfff8123456789abcL);
            }
            byte[] prefix;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_decode_value_state_payload(
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
                    List<NumericState> values = operation.run(input);
                    if (error || values.size() != 4096
                            || values.get(4095).valueBits() != 0xfff8123456789abcL) {
                        throw new AssertionError("Values");
                    }
                } catch (NumericStatePayloadException.InvalidLength failure) {
                    if (!error || failure.actual() != 65537L || failure.trailing() != 17L) throw failure;
                }
            }
            long retained = matching(prefix, error);
            System.out.println(language + " numeric error=" + error
                    + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained
                    + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Native numeric RustBuffer leak");
        }
    }
}
