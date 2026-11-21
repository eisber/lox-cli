package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;

/** Actual buffers use the established native HeapWalk prefix primitive. */
public final class NativeWiringCatalogProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        List<ConnectorWiring> run(byte[] input, String selector) throws WiringCatalogException;
    }

    private static long matching(byte[] prefix, long minimum) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment p = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code = (int) COUNT.invokeExact(p, (long) prefix.length, minimum, count);
            if (code != 0) throw new AssertionError("HeapWalk: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack");
        byte[] fixture = Files.readAllBytes(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "wiring_catalog.xml"));
        var rows = operation.run(fixture, "uuid:raw-id");
        if (!rows.equals(List.of(
                new ConnectorWiring("I1", WiringDirection.INPUT, "00000000-0000-0000-0000000000000000", false),
                new ConnectorWiring("AQ1", WiringDirection.OUTPUT, "00000000-0000-0000-0000-000000000000", true),
                new ConnectorWiring("input", WiringDirection.PARAMETER, "", false),
                new ConnectorWiring("OutputSuffix", WiringDirection.OUTPUT, "", true),
                new ConnectorWiring("Qx", WiringDirection.OUTPUT, " raw & 灯 ", true),
                new ConnectorWiring("Qx", WiringDirection.OUTPUT, " raw & 灯 ", true),
                new ConnectorWiring("", WiringDirection.PARAMETER, "", false))))
            throw new AssertionError("Direct values/order/defaults/raw sentinel");
        String marker = "wiring-owned-allocation-" + "x".repeat(65536);
        byte[][] inputs = {
                ("<C Type=\"And\" Title=\"Selected\" U=\"raw-id\"><Co K=\"" + marker + "\"/></C>")
                        .getBytes(StandardCharsets.UTF_8),
                "<C>".getBytes(StandardCharsets.UTF_8), fixture, fixture};
        String[] selectors = {"uuid:raw-id", "uuid:raw-id", marker, "Twin"};
        for (int branch = 0; branch < 4; branch++) {
            boolean error = branch != 0;
            byte[] prefix;
            long length;
            long minimum;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_list_connector_wiring(
                        arena, FfiConverterByteArray.INSTANCE.lower(inputs[branch]),
                        FfiConverterString.INSTANCE.lower(selectors[branch]), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                length = RustBuffer.getLen(buffer);
                if (length < 16) throw new AssertionError("Actual result too small");
                prefix = new byte[error ? 16 : 128];
                minimum = error ? length : 64000L;
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix, minimum) != 1) throw new AssertionError("Live calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix, minimum) != 0) throw new AssertionError("Freed calibration");
            }
            Object saved = null;
            for (int call = 0; call < 64; call++) {
                try {
                    var value = operation.run(inputs[branch], selectors[branch]);
                    if (error || !value.get(0).connector().equals(marker)) throw new AssertionError("Values");
                    saved = value;
                } catch (WiringCatalogException failure) {
                    if (branch == 1 && failure instanceof WiringCatalogException.Load load) {
                        if (load.sourceMessage().isEmpty()) throw new AssertionError("Load source");
                    } else if (branch == 2 && failure instanceof WiringCatalogException.NotFound missing) {
                        if (!missing.selector().equals(marker) || !missing.titles().contains("Selected"))
                            throw new AssertionError("NotFound context");
                    } else if (branch == 3 && failure instanceof WiringCatalogException.Ambiguous ambiguous) {
                        if (!ambiguous.selector().equals("Twin") || !ambiguous.matches().equals(List.of(
                                new WiringElementIdentity("Twin one", "And", "one"),
                                new WiringElementIdentity("Twin two", "Or", "two"))))
                            throw new AssertionError("Ambiguous context/order");
                    } else throw failure;
                    if (saved == null) saved = failure;
                }
            }
            if (saved == null || matching(prefix, minimum) != 0) throw new AssertionError("Retained buffer");
            System.out.println(language + " wiring branch=" + branch + " actual_result_bytes=" + length
                    + " live=1 free=0 calls=64 retained=0 GC_requested=false");
        }
    }
}
