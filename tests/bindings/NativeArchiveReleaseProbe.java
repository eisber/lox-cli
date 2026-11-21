package net.eisber.lox.archive;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.lang.reflect.Field;

/** Windows AMD64 allocation regression only; not shipped in the API jar. */
public final class NativeArchiveReleaseProbe {
    private static final Linker LINKER = Linker.nativeLinker();
    private static final SymbolLookup PROBE = SymbolLookup.libraryLookup(
            System.getProperty("lox.archive.heapProbe"), Arena.global());
    private static final MethodHandle VALIDATE = function("lox_test_heap_live",
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS));
    private static final MethodHandle COUNT = function("lox_test_heap_count",
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        byte[] run(byte[] data) throws ArchiveException;
    }

    private static MethodHandle function(String name, FunctionDescriptor descriptor) {
        return LINKER.downcallHandle(PROBE.find(name).orElseThrow(), descriptor);
    }

    private static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }

    private static void platform() {
        require(System.getProperty("os.name").startsWith("Windows")
                && System.getProperty("os.arch").equals("amd64"), "Windows AMD64 probe only");
    }

    private static boolean live(MemorySegment pointer) throws Throwable {
        platform();
        return (int) VALIDATE.invokeExact(pointer) != 0;
    }

    private static long matchingAllocations(byte[] prefix, long minimumBytes) throws Throwable {
        platform();
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment nativePrefix = arena.allocate(prefix.length);
            nativePrefix.asByteBuffer().put(prefix);
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int error = (int) COUNT.invokeExact(
                    nativePrefix, (long) prefix.length, minimumBytes, count);
            require(error == 0, "Native HeapWalk probe failed: " + error);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    private static byte[] calibrate(byte[] invalidZip) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_extract_and_decompress(
                    arena, FfiConverterByteArray.INSTANCE.lower(invalidZip), status);
            require(UniffiRustCallStatus.isError(status), "expected an actual native domain error");
            MemorySegment error = UniffiRustCallStatus.getErrorBuf(status);
            MemorySegment pointer = RustBuffer.getData(error);
            long length = RustBuffer.getLen(error);
            require(length > 64000 && live(pointer), "expected a large native process-heap error buffer");
            byte[] prefix = new byte[128];
            RustBuffer.asByteBuffer(error).get(prefix);
            require(matchingAllocations(prefix, length) == 1, "heap matcher must find the live error");
            // This descriptor belongs to an independent status, never the reusable status.
            RustBuffer.free(error);
            RustBuffer.free(result);
            require(!live(pointer), "independent-descriptor calibration free failed");
            require(matchingAllocations(prefix, length) == 0, "heap matcher must observe release");
            System.out.println("Native error allocation calibration: live=1 released=0 bytes=" + length);
            return prefix;
        }
    }

    public static void verifyPublicErrors(String language, byte[] invalidZip, int repetitions,
            Operation operation) throws Throwable {
        byte[] prefix = calibrate(invalidZip);
        require(matchingAllocations(prefix, 64000) == 0, "nonempty baseline");
        for (int i = 0; i < repetitions; i++) {
            try {
                operation.run(invalidZip);
                throw new AssertionError("Expected ReadZipEntry");
            } catch (ArchiveException.ReadZipEntry error) {
                require(error.index() == 0L && error.name().startsWith("native-archive-release-")
                        && error.name().endsWith("sps0.LoxCC")
                        && !error.sourceMessage().isEmpty(), "domain error context changed");
            }
        }
        long retained = matchingAllocations(prefix, 64000);
        System.out.println(language + " public native domain errors=" + repetitions
                + " retained_error_allocations=" + retained);
        require(retained == 0, "Native domain-error allocation leak: " + retained);
    }

    public static void verifyAliasedDescriptorRelease() throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment original = RustBuffer.alloc(1024 * 1024);
            MemorySegment saved = arena.allocate(RustBuffer.LAYOUT);
            MemorySegment.copy(original, 0, saved, 0, RustBuffer.LAYOUT.byteSize());
            MemorySegment pointer = RustBuffer.getData(saved);
            require(live(pointer), "sentinel is not a process-heap allocation");
            Field field = UniffiHelpers.class.getDeclaredField("REUSABLE_STATUS");
            field.setAccessible(true);
            MemorySegment status = (MemorySegment) ((ThreadLocal<?>) field.get(null)).get();
            UniffiRustCallStatus.setErrorBuf(status, saved);
            boolean retained;
            try {
                RustBuffer.free(UniffiRustCallStatus.getErrorBuf(status));
                retained = live(pointer);
            } finally {
                // Negative-control cleanup only: never double-free corrected output.
                if (live(pointer)) RustBuffer.free(saved);
                require(!live(pointer), "sentinel cleanup failed");
            }
            System.out.println("Aliased 1MiB descriptor retained_after_free=" + retained);
            require(!retained, "Aliased descriptor lost before native free");
        }
    }
}
