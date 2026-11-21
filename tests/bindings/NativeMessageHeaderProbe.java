package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.Arrays;

/** Reuses the unchanged HeapWalk primitive for actual header RustBuffers. */
public final class NativeMessageHeaderProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        MessageHeader run(byte[] input) throws MessageHeaderException;
    }

    private static long matching(byte[] prefix, long length) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code = (int) COUNT.invokeExact(nativePrefix, (long)prefix.length, length, count);
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock stack required");
        for (String branch : new String[] {"success", "length", "magic"}) {
            boolean error = !branch.equals("success");
            byte[] input = branch.equals("success")
                    ? new byte[] {3, (byte)0xe7, (byte)0xff, (byte)0x91,
                            0x76, 0x54, 0x32, (byte)0xf0}
                    : new byte[branch.equals("length") ? 65537 : 8];
            if (error) Arrays.fill(input, (byte)0x83);
            byte[] prefix;
            long length;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_decode_message_header(
                        arena, FfiConverterByteArray.INSTANCE.lower(input), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                length = RustBuffer.getLen(buffer);
                prefix = new byte[Math.toIntExact(length)];
                RustBuffer.asByteBuffer(buffer).get(prefix);
                long live = matching(prefix, length);
                System.out.println(language + " header branch=" + branch
                        + " actual_result_bytes=" + length + " live=" + live);
                if (live != 1) throw new AssertionError("Unresolved actual header live calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix, length) != 0) throw new AssertionError("Released calibration");
            }
            MessageHeader saved = null;
            MessageHeaderException savedFailure = null;
            for (int i = 0; i < 64; i++) {
                try {
                    saved = operation.run(input);
                    if (error || !Arrays.equals(saved.wireBytes(), input))
                        throw new AssertionError("Owned observation");
                } catch (MessageHeaderException.InvalidLength failure) {
                    if (!branch.equals("length") || failure.actual() != 65537L) throw failure;
                    savedFailure = failure;
                } catch (MessageHeaderException.InvalidMagic failure) {
                    if (!branch.equals("magic") || Byte.toUnsignedInt(failure.actual()) != 0x83)
                        throw failure;
                    savedFailure = failure;
                }
            }
            if (error ? savedFailure == null : saved == null) throw new AssertionError("Retained values");
            if (matching(prefix, length) != 0) throw new AssertionError("Retained RustBuffer");
            System.out.println(language + " header branch=" + branch
                    + " live=1 free=0 calls=64 retained=0 GC_requested=false");
        }
    }
}
