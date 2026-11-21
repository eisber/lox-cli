package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;

/** Test-only positive calibration and no-GC buffer release checks. */
public final class NativeColorProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    static long matching(byte[] prefix, long length) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment p = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int error = (int) COUNT.invokeExact(p, (long) prefix.length, length, count);
            if (error != 0) throw new AssertionError("HeapWalk: " + error);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    static final ColorEncodeInput INPUT = new ColorEncodeInput.Rgb(137.0, 29.0, 211.0);
    static final String BAD = "color-buffer-probe-" + "x".repeat(65536);

    static MemorySegment raw(int path, MemorySegment status, Arena arena) {
        return switch (path) {
            case 0 -> UniffiLib.uniffi_lox_archive_fn_func_encode_color(arena,
                    FfiConverterTypeColorEncodeInput.INSTANCE.lower(INPUT),
                    FfiConverterOptionalDouble.INSTANCE.lower(null), status);
            case 1 -> UniffiLib.uniffi_lox_archive_fn_func_encode_color(arena,
                    FfiConverterTypeColorEncodeInput.INSTANCE.lower(INPUT),
                    FfiConverterOptionalDouble.INSTANCE.lower(101.25), status);
            case 2, 3 -> UniffiLib.uniffi_lox_archive_fn_func_decode_color_legacy(arena,
                    FfiConverterString.INSTANCE.lower(path == 2 ? "hsv(137.25,29.5,81.75)" : BAD), status);
            case 4, 5 -> UniffiLib.uniffi_lox_archive_fn_func_parse_mood_color_legacy(arena,
                    FfiConverterString.INSTANCE.lower(path == 4 ? "rgb(137,29,211)" : BAD), status);
            case 6, 7 -> UniffiLib.uniffi_lox_archive_fn_func_observe_actor_color_legacy(
                    arena, path == 6 ? -1000000000L : -123456789L, status);
            case 8 -> UniffiLib.uniffi_lox_archive_fn_func_color_format_number(arena, 137.251937, status);
            default -> throw new AssertionError();
        };
    }

    static Object call(ColorSmoke.Api api, int path) throws ColorException {
        return switch (path) {
            case 0 -> api.encode(INPUT, null);
            case 1 -> api.encode(INPUT, 101.25);
            case 2 -> api.decode("hsv(137.25,29.5,81.75)");
            case 3 -> api.decode(BAD);
            case 4 -> api.mood("rgb(137,29,211)");
            case 5 -> api.mood(BAD);
            case 6 -> api.actor(-1000000000L);
            case 7 -> api.actor(-123456789L);
            case 8 -> api.number(137.251937);
            default -> throw new AssertionError();
        };
    }

    public static void verify(String language, ColorSmoke.Api api) throws Throwable {
        for (int path = 0; path < 9; path++) {
            boolean error = path < 8 && path % 2 == 1;
            byte[] prefix;
            long length;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = raw(path, status, arena);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("calibration outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                length = RustBuffer.getLen(buffer);
                // The native probe's own prefix allocation must be smaller than the buffer.
                prefix = new byte[(int) Math.min(length / 2, 128)];
                RustBuffer.asByteBuffer(buffer).get(prefix);
                long live = matching(prefix, length);
                if (live != 1) throw new AssertionError("calibration live path=" + path + " length=" + length + " matching=" + live);
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix, length) != 0) throw new AssertionError("calibration released path=" + path);
            }
            for (int i = 0; i < 64; i++) {
                try {
                    if (call(api, path) == null || error) throw new AssertionError("expected success/error");
                } catch (ColorException failure) {
                    if (!error) throw failure;
                    if (path == 1 && !(failure instanceof ColorException.Brightness)
                            || path == 3 && !(failure instanceof ColorException.InvalidValue)
                            || path == 5 && !(failure instanceof ColorException.InvalidValue)
                            || path == 7 && !(failure instanceof ColorException.ActorChannel)) throw failure;
                }
            }
            long retained = matching(prefix, length);
            System.out.println(language + " color path=" + path + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Color RustBuffer leak");
        }
    }
}
