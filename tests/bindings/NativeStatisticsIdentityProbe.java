package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.util.List;

/** Actual String result and both structured error RustBuffers; unchanged HeapWalk matcher. */
public final class NativeStatisticsIdentityProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        String run(List<StatEntry> entries, String query) throws StatisticResolutionException;
    }

    private static StatEntry row(String name, String uuid, Integer group) {
        return new StatEntry("not-an-identity.xml", uuid, group, "opaque-month", name, "label");
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

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        for (int variant = 0; variant < 3; variant++) {
            String marker = "identity-owned-" + variant + "-" + "x".repeat(65536);
            List<StatEntry> entries = switch (variant) {
                case 0 -> List.of(row("winner", marker, null));
                case 1 -> List.of(row(marker, "a", null));
                default -> List.of(row(marker, "a", null), row(marker, "b", -1));
            };
            String query = variant == 0 ? "winner" : variant == 1 ? "missing" : marker;
            byte[] prefix = new byte[128];
            long length;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_resolve_statistics_identity(
                        arena, FfiConverterSequenceTypeStatEntry.INSTANCE.lower(entries),
                        FfiConverterString.INSTANCE.lower(query), status);
                if (UniffiRustCallStatus.isError(status) != (variant != 0)) throw new AssertionError("Outcome");
                MemorySegment buffer = variant == 0 ? result : UniffiRustCallStatus.getErrorBuf(status);
                length = RustBuffer.getLen(buffer);
                if (length < 64000) throw new AssertionError("Small actual buffer");
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix) != 1) throw new AssertionError("Live calibration");
                if (variant != 0) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix) != 0) throw new AssertionError("Released calibration");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    String value = operation.run(entries, query);
                    if (variant != 0 || !value.equals(marker)) throw new AssertionError("Owned raw UUID");
                } catch (StatisticResolutionException.NotFound error) {
                    if (variant != 1 || !error.names().equals(List.of(marker))) throw error;
                } catch (StatisticResolutionException.Ambiguous error) {
                    if (variant != 2 || error.distinctUuids() != 2
                            || !error.candidates().stream().map(StatisticIdentity::uuid).toList().equals(List.of("a", "b"))
                            || error.candidates().get(1).group() != -1) throw error;
                }
            }
            long count = matching(prefix);
            System.out.println(language + " statistics-identity variant=" + variant + " actual_bytes=" + length
                    + " live=1 free=0 calls=64 retained=" + count + " GC_requested=false");
            if (count != 0) throw new AssertionError("Native identity RustBuffer leak");
        }
    }
}
