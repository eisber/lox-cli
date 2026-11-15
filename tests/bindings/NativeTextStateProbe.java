package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.List;

/** Test-only calibrated text RustBuffer checks; no requested JVM GC. */
public final class NativeTextStateProbe {
    private static final SymbolLookup PROBE = SymbolLookup.libraryLookup(
            System.getProperty("lox.archive.heapProbe"), Arena.global());
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
    private static final MethodHandle ERROR_COUNT = Linker.nativeLinker().downcallHandle(
            PROBE.find("lox_test_heap_text_error_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.JAVA_INT,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG,
                    ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        List<TextState> run(byte[] input) throws TextStatePayloadException;
    }

    private static long matching(byte[] prefix, int tag, long first, long second) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code;
            if (tag != 0) {
                code = (int) ERROR_COUNT.invokeExact(tag, 65536L, first, second, count);
            } else {
                MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
                code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, 64000L, count);
            }
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verifyPath(String language, Operation operation, byte[] input,
            int tag, long first, long second, int serializedLength) throws Throwable {
        byte[] prefix;
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_decode_text_state_payload(
                    arena, FfiConverterByteArray.INSTANCE.lower(input), status);
            if (UniffiRustCallStatus.isError(status) != (tag != 0)) throw new AssertionError("Outcome");
            MemorySegment buffer = tag == 0 ? result : UniffiRustCallStatus.getErrorBuf(status);
            long length = RustBuffer.getLen(buffer);
            if (tag == 0 ? length < 64000 : length != serializedLength) throw new AssertionError("Buffer size");
            prefix = new byte[tag == 0 ? 128 : 0];
            if (tag == 0) RustBuffer.asByteBuffer(buffer).get(prefix);
            if (matching(prefix, tag, first, second) != 1) throw new AssertionError("Live calibration");
            if (tag != 0) RustBuffer.free(buffer);
            RustBuffer.free(result);
            if (matching(prefix, tag, first, second) != 0) throw new AssertionError("Released calibration");
        }
        for (int i = 0; i < 64; i++) {
            try {
                List<TextState> values = operation.run(input);
                if (tag != 0 || values.size() != 3072
                        || !values.get(3071).text().equals("é")
                        || values.get(3071).textByteLen() != 2) throw new AssertionError("Values");
            } catch (TextStatePayloadException failure) {
                // Caller checks every variant/field too; keep the resource probe package-local to generated API.
                boolean valid = switch (tag) {
                    case 1 -> failure instanceof TextStatePayloadException.TruncatedHeader f
                            && f.recordOffset() == 65536 && f.available() == first;
                    case 3 -> failure instanceof TextStatePayloadException.TruncatedText f
                            && f.recordOffset() == 65536 && Integer.toUnsignedLong(f.declared()) == first
                            && f.available() == second;
                    case 4 -> failure instanceof TextStatePayloadException.TruncatedPadding f
                            && f.recordOffset() == 65536 && f.expected() == first && f.available() == second;
                    case 5 -> failure instanceof TextStatePayloadException.InvalidUtf8 f
                            && f.recordOffset() == 65536 && f.validUpTo() == first
                            && (second == -1 ? f.errorLen() == null : Long.valueOf(second).equals(f.errorLen()));
                    default -> false;
                };
                if (!valid) throw failure;
            }
        }
        long retained = matching(prefix, tag, first, second);
        System.out.println(language + " text path=tag" + tag + "-option" + second
                + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained
                + " GC_requested=false");
        if (retained != 0) throw new AssertionError("Native text RustBuffer leak");
    }
}
