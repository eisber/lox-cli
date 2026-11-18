package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.List;

/** Unchanged HeapWalk primitive; exact live1/free0, then64 calls without GC. */
public final class NativeDaytimerStateProbe {
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
        List<DaytimerState> run(byte[] input) throws DaytimerStatePayloadException;
    }

    private static long matching(byte[] prefix, boolean error) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code;
            if (error) {
                code = (int) ERROR_COUNT.invokeExact(28L, 17L, count);
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
            ByteBuffer data = ByteBuffer.allocate(error ? 45 : 52 * 4096)
                    .order(ByteOrder.LITTLE_ENDIAN);
            for (int i = 0; i < (error ? 1 : 4096); i++) {
                for (int j = 0; j < 16; j++) data.put((byte) (0xa5 + j));
                data.putLong(0xfff8123456789abcL).putInt(error ? 0 : 1);
                if (!error) data.putInt(Integer.MIN_VALUE).putInt(-83).putInt(Integer.MAX_VALUE)
                        .putInt(-7).putLong(0x7ff0000000001234L);
            }
            while (data.hasRemaining()) data.put((byte) 0xa5);
            byte[] input = data.array();
            byte[] prefix;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_decode_daytimer_state_payload(
                        arena, FfiConverterByteArray.INSTANCE.lower(input), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                long length = RustBuffer.getLen(buffer);
                if (error ? length != 20 : length < 64000) throw new AssertionError("Buffer size");
                prefix = new byte[error ? 0 : 128];
                if (!error) RustBuffer.asByteBuffer(buffer).get(prefix);
                long live = matching(prefix, error);
                System.out.println(language + " Daytimer calibration error=" + error + " live=" + live);
                if (live != 1) throw new AssertionError("Live calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix, error) != 0) throw new AssertionError("Released calibration");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    List<DaytimerState> values = operation.run(input);
                    if (error || values.size() != 4096
                            || values.getLast().entries().getFirst().valueBits() != 0x7ff0000000001234L) {
                        throw new AssertionError("Values");
                    }
                } catch (DaytimerStatePayloadException.TruncatedHeader failure) {
                    if (!error || failure.recordOffset() != 28 || failure.available() != 17) throw failure;
                }
            }
            long retained = matching(prefix, error);
            System.out.println(language + " Daytimer error=" + error
                    + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained
                    + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Daytimer RustBuffer leak");
        }
    }
}
