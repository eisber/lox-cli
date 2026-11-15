package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;

/** Test-only calibrated Windows heap proof for new structured returns and XML errors. */
public final class NativeConfigInspectionProbe {
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        Object run(boolean inventory, byte[] input) throws ConfigurationException;
    }

    @FunctionalInterface
    public interface ControlsOperation {
        java.util.List<XmlControlInfo> run(byte[] input, String type, String room) throws ConfigurationException;
    }

    public static void verifyControls(String language, ControlsOperation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        String marker = "xml-controls-allocation-" + "x".repeat(65536);
        for (boolean error : new boolean[] {false, true}) {
            byte[] input = (error ? "<" + marker + "></other>"
                    : "<C Type=\"Switch\" U=\"c\" Title=\"" + marker + "\"/><IoData Pr=\"r\" Cr=\"cat\"/>")
                    .getBytes(StandardCharsets.UTF_8);
            byte[] prefix;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_parse_controls(arena,
                        FfiConverterByteArray.INSTANCE.lower(input),
                        FfiConverterOptionalString.INSTANCE.lower("SWITCH"),
                        FfiConverterOptionalString.INSTANCE.lower("r"), status);
                if (UniffiRustCallStatus.isError(status) != error) throw new AssertionError("Calibration outcome");
                MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
                if (RustBuffer.getLen(buffer) < 64000) throw new AssertionError("Small calibration");
                prefix = new byte[128];
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (matching(prefix) != 1) throw new AssertionError("Expected live control calibration");
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix) != 0) throw new AssertionError("Control calibration did not release");
            }
            for (int i = 0; i < 64; i++) {
                try {
                    var result = operation.run(input, "SWITCH", "r");
                    if (error || !result.equals(java.util.List.of(
                            new XmlControlInfo("Switch", marker, "c", "r", "cat")))) {
                        throw new AssertionError("Control structured result lost");
                    }
                } catch (ConfigurationException.Xml failure) {
                    if (!error || failure.sourceMessage().length() < 65536) throw failure;
                }
            }
            long retained = matching(prefix);
            System.out.println(language + " XML controls error=" + error
                    + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained
                    + " GC_requested=false");
            if (retained != 0) throw new AssertionError("Native control RustBuffer leak");
        }
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

    private static byte[] calibrate(boolean inventory, byte[] input, boolean error) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = inventory
                    ? UniffiLib.uniffi_lox_archive_fn_func_parse_rooms(
                            arena, FfiConverterByteArray.INSTANCE.lower(input), status)
                    : UniffiLib.uniffi_lox_archive_fn_func_parse_config_summary(
                            arena, FfiConverterByteArray.INSTANCE.lower(input), status);
            if (UniffiRustCallStatus.isError(status) != error) {
                throw new AssertionError("Unexpected calibration outcome");
            }
            MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
            if (RustBuffer.getLen(buffer) < 64000) throw new AssertionError("Small calibration buffer");
            byte[] prefix = new byte[128];
            RustBuffer.asByteBuffer(buffer).get(prefix);
            if (matching(prefix) != 1) throw new AssertionError("Expected one live calibration");
            if (error) RustBuffer.free(buffer);
            RustBuffer.free(result);
            if (matching(prefix) != 0) throw new AssertionError("Calibration did not release");
            return prefix;
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        String marker = "config-inspection-allocation-" + "x".repeat(65536);
        for (boolean inventory : new boolean[] {false, true}) {
            for (boolean error : new boolean[] {false, true}) {
                String xml = error ? "<" + marker + "></other>"
                        : inventory ? "<C Type=\"Place\" U=\"r\" Title=\"" + marker + "\"/>"
                        : "<ControlList Version=\"" + marker + "\"><C Type=\"And\" U=\"c\" Title=\"n\"/>"
                                + "<C Type=\"Place\" U=\"r\" Title=\"room\"/></ControlList>";
                byte[] input = xml.getBytes(StandardCharsets.UTF_8);
                byte[] prefix = calibrate(inventory, input, error);
                for (int i = 0; i < 64; i++) {
                    try {
                        Object result = operation.run(inventory, input);
                        if (error) throw new AssertionError("Expected typed XML error");
                        if (inventory) {
                            RoomInfo room = (RoomInfo) ((java.util.List<?>) result).get(0);
                            if (!room.name().equals(marker) || room.itemCount() != 0L) {
                                throw new AssertionError("Inventory data lost");
                            }
                        } else {
                            ConfigSummary summary = (ConfigSummary) result;
                            if (!summary.version().equals(marker)
                                    || !summary.controls().get("c").equals(new ConfigControlEntry("n", "And", ""))
                                    || !summary.rooms().equals(java.util.Map.of("r", "room"))) {
                                throw new AssertionError("Structured map data lost");
                            }
                        }
                    } catch (ConfigurationException.Xml failure) {
                        if (!error || failure.sourceMessage().length() < 65536) throw failure;
                    }
                }
                long retained = matching(prefix);
                System.out.println(language + " configuration inventory=" + inventory + " error=" + error
                        + " calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=" + retained
                        + " GC_requested=false");
                if (retained != 0) throw new AssertionError("Native configuration RustBuffer leak");
            }
        }
    }
}
