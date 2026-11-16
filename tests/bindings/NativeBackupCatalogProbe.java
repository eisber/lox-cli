package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;

/** Actual optional success descriptor, using the established unchanged heap matcher. */
public final class NativeBackupCatalogProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        BackupInfo run(String name, long size);
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
        String timestamp = "20261399259999" + "7".repeat(65536);
        String input = "sps_4294967295_" + timestamp + ".zip";
        byte[] prefix;
        long length;
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_backup_name(
                    arena, FfiConverterString.INSTANCE.lower(input), -1L, status);
            if (UniffiRustCallStatus.isError(status)) throw new AssertionError("Unexpected FFI status");
            length = RustBuffer.getLen(result);
            if (length < 64000) throw new AssertionError("Small actual result");
            prefix = new byte[128];
            RustBuffer.asByteBuffer(result).get(prefix);
            if (matching(prefix) != 1) throw new AssertionError("Live result calibration");
            RustBuffer.free(result);
            if (matching(prefix) != 0) throw new AssertionError("Released result calibration");
        }
        for (int i = 0; i < 64; i++) {
            BackupInfo value = operation.run(input, -1L);
            if (!value.filename().equals(input) || !value.timestamp().equals(timestamp)
                    || value.version() != -1 || value.size() != -1L) {
                throw new AssertionError("Owned values");
            }
        }
        long retained = matching(prefix);
        System.out.println(language + " backup-catalog actual_Some_result_bytes=" + length
                + " live=1 free=0 calls=64 retained=" + retained
                + " GC_requested=false None=functional_only Vec=inherited_generated_ownership no_error_branch");
        if (retained != 0) throw new AssertionError("Native backup catalog RustBuffer leak");
    }
}
