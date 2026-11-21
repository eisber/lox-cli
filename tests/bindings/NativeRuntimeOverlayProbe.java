package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

/** Actual success/error RustBuffers; unchanged generic Windows HeapWalk primitive. */
public final class NativeRuntimeOverlayProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        byte[] run(byte[] base, byte[] local, byte[] live) throws OverlayException;
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

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    public static void chain(OverlayException.Refused failure, String id) {
        List<OverlayErrorFrame> frames = failure.frames();
        if (frames.size() != 2 || !(frames.get(0) instanceof OverlayErrorFrame.MergedConsistency)
                || !(frames.get(1) instanceof OverlayErrorFrame.DuplicateUuid duplicate)
                || !duplicate.id().equals(id)) throw new AssertionError("Typed cause chain");
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        String marker = "overlay-owned-" + "x".repeat(65536);
        String a = "<C U=\"a\" Type=\"And\">%s</C>";
        String b = "<C U=\"b\" Type=\"And\">%s</C>";
        String child = "<C U=\"" + marker + "\" Type=\"And\"/>";
        List<Object> retained = new ArrayList<>();
        for (boolean error : new boolean[] {false, true}) {
            byte[] base = bytes(error ? "<Root>" + a.formatted("") + b.formatted("") + "</Root>"
                    : "<Root Marker=\"" + marker + "\"/>");
            byte[] local = error ? bytes("<Root>" + a.formatted(child) + b.formatted("") + "</Root>") : base.clone();
            byte[] live = error ? bytes("<Root>" + a.formatted("") + b.formatted(child) + "</Root>") : base.clone();
            byte[] prefix;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_three_way(
                        arena, FfiConverterByteArray.INSTANCE.lower(base),
                        FfiConverterByteArray.INSTANCE.lower(local),
                        FfiConverterByteArray.INSTANCE.lower(live), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                if (RustBuffer.getLen(buffer) < 64000) throw new AssertionError("Small calibration");
                prefix = new byte[128];
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix) != 1) throw new AssertionError("Live calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix) != 0) throw new AssertionError("Released calibration");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    byte[] value = operation.run(base, local, live);
                    if (error || !Arrays.equals(value, base)) throw new AssertionError("Owned bytes");
                    retained.add(value);
                } catch (OverlayException.Refused failure) {
                    if (!error) throw failure;
                    chain(failure, marker);
                    retained.add(failure);
                }
            }
            long count = matching(prefix);
            System.out.println(language + " runtime-overlay error=" + error
                    + " calibrated_live=1 calibrated_released=0 calls=64 retained_buffers=" + count
                    + " GC_requested=false");
            if (count != 0) throw new AssertionError("Runtime overlay RustBuffer leak");
        }
        byte[] expected = bytes("<Root Marker=\"" + marker + "\"/>");
        for (Object value : retained) {
            if (value instanceof byte[] data) {
                if (!Arrays.equals(data, expected)) throw new AssertionError("Retained output");
            } else {
                chain((OverlayException.Refused) value, marker);
            }
        }
    }
}
