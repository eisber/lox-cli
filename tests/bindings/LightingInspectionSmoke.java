package net.eisber.lox.archive;

import java.lang.foreign.Arena;
import java.lang.foreign.FunctionDescriptor;
import java.lang.foreign.Linker;
import java.lang.foreign.MemorySegment;
import java.lang.foreign.SymbolLookup;
import java.lang.foreign.ValueLayout;
import java.lang.invoke.MethodHandle;
import java.nio.ByteBuffer;
import java.nio.charset.MalformedInputException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Objects;

public final class LightingInspectionSmoke {
    private static final SymbolLookup PROBE = SymbolLookup.libraryLookup(
            System.getProperty("lox.archive.heapProbe"), Arena.global());
    private static final Linker LINKER = Linker.nativeLinker();
    private static final MethodHandle COUNT = LINKER.downcallHandle(
            PROBE.find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));
    private static final MethodHandle LIVE = LINKER.downcallHandle(
            PROBE.find("lox_test_heap_live").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS));
    private static final String AMBIGUOUS = "Lighting inventory twin";

    @FunctionalInterface
    public interface Operation {
        LightingInspection run(LightingInspectionRequest request) throws LightingInspectionException;
    }

    private enum Branch { INVENTORY, NOT_FOUND, AMBIGUITY, WRONG_TYPE }
    private record Trial(Branch branch, LightingInspectionRequest request, byte[] prefix, String expected) {}
    private record Extent(MemorySegment pointer, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static LightingInspectionRequest request(String xml, String selector) {
        return new LightingInspectionRequest(bytes(xml), selector);
    }

    private static void require(boolean passed, String message) {
        if (!passed) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, LightingInspectionRequest request) {
        try {
            return operation.run(request);
        } catch (LightingInspectionException failure) {
            return failure;
        }
    }

    private static LightingAttribute attr(String key, String value) {
        return new LightingAttribute(key, value);
    }

    private static void checkInventory(LightingInspection value, String uuid) {
        require(uuid.equals(value.uuid()) && "Lamps & 灯".equals(value.title()), "Controller identity");
        require(value.attributes().equals(List.of(attr("CapAI0", "+0007"), attr("Title", "Lamps & 灯"),
                attr("Type", "LightController2"), attr("U", uuid), attr("gid", "primary"))), "Stored attributes");
        require(value.documentVersions().equals(List.of(
                new LightingDocumentVersion(List.of(attr("Version", "+0267"))),
                new LightingDocumentVersion(List.of(attr("ConfigVersion", "17.2"), attr("V", "opaque"))))),
                "Document version order");
        require(value.directSceneCollectionCount() == 1 && Objects.equals(value.soleSceneCollection(), 1L),
                "Sole scene collection index");
        require(value.nestedStorage().stream().map(LightingStorage::element).toList()
                .equals(List.of("IoData", "LightscenesC", "HCL")), "Nested storage order");
        var collection = value.nestedStorage().get(1);
        require(collection.attributes().equals(List.of(attr("Num", "99"), attr("Outputs", "0002"))),
                "Raw collection fields");
        var sequence = collection.children().getFirst().children().getFirst();
        require(sequence.element().equals("SeqConf") && sequence.attributes().equals(List.of(attr("Flag", "0")))
                && sequence.children().getFirst().element().equals("LightsceneC"), "Nested storage depth");

        require(value.moods().size() == 2, "Depth-first duplicate moods");
        var direct = value.moods().getFirst();
        var nested = value.moods().getLast();
        require(direct.summary().equals(new MoodInfo("Direct", "same", null, 9L, ""))
                && nested.summary().equals(new MoodInfo("Nested", "same", null, null, "-1"))
                && direct.inSoleDirectCollection() && !nested.inSoleDirectCollection(), "Mood summary and membership");
        require(direct.attributes().containsAll(List.of(attr("SID", "bad"), attr("Q01", "unprojected")))
                && direct.outputs().equals(List.of(new StoredLightingOutput("Q10", 10L, "opaque"),
                    new StoredLightingOutput("Q2", 2L, "+0006"), new StoredLightingOutput("Q1", 1L, ""))),
                "Raw loader-ordered mood outputs");

        require(value.channels().size() == 2, "Direct indexed channels");
        var connected = value.channels().getFirst();
        var missing = value.channels().getLast();
        require("AQ10".equals(connected.key()) && connected.index() == 10 && "src".equals(connected.uuid())
                && Objects.equals(connected.uuidOccurrences(), 2L)
                && connected.attributes().contains(attr("Future", "keep & stay")), "Stored channel and duplicate UUID");
        require("AQ2".equals(missing.key()) && missing.index() == 2 && missing.uuid() == null
                && missing.uuidOccurrences() == null && missing.downstream().isEmpty(), "Missing channel identity");
        require(connected.downstream().equals(List.of(
                new StoredLightingWire("wrong", "And", "Lamps & 灯", "in", "I1",
                    List.of(attr("FLG", "02"), attr("Input", "src"))),
                new StoredLightingWire("wrong", "And", "Lamps & 灯", null, null,
                    List.of(attr("Future", "unresolved"), attr("Input", "src"))))), "Exact downstream references");
    }

    private static void checkValues(Operation operation, String fixture) throws LightingInspectionException {
        checkInventory(operation.run(request(fixture, "Lamps [study]")), "light");
        var multiple = operation.run(request(
                "<C Type='LightController2'><LightscenesC><LightsceneC/></LightscenesC><LightscenesC/></C>",
                "Type:LightController2"));
        require(multiple.uuid() == null && multiple.title() == null
                && multiple.directSceneCollectionCount() == 2 && multiple.soleSceneCollection() == null
                && !multiple.moods().getFirst().inSoleDirectCollection(), "Nonunique scene collection");
        var empty = operation.run(request("<C Type='LightController2' U='empty'/>", "uuid:empty"));
        require(empty.moods().isEmpty() && empty.channels().isEmpty() && empty.nestedStorage().isEmpty(),
                "Empty inventory");
        require(outcome(operation, request("<Root><C attr='1' attr='2'/></Root>", "uuid:missing"))
                instanceof LightingInspectionException.Load failure && !failure.sourceMessage().isEmpty(),
                "Load refusal before selection");
        require(outcome(operation, request("<C U='sparse'/>", "uuid:sparse"))
                instanceof LightingInspectionException.NotLightController failure && failure.blockType().isEmpty(),
                "Absent type represented as empty text");
    }

    private static long matching(byte[] prefix, long minimum) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, minimum, count);
            require(code == 0, "HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    private static boolean live(MemorySegment pointer) throws Throwable {
        return (int) LIVE.invokeExact(pointer) != 0;
    }

    private static Extent verifyLive(MemorySegment buffer, byte[] prefix) throws Throwable {
        long length = RustBuffer.getLen(buffer);
        require(prefix.length == 128 && length > 2L * prefix.length, "Small native buffer");
        byte[] observed = new byte[prefix.length];
        RustBuffer.asByteBuffer(buffer).get(observed);
        require(Arrays.equals(observed, prefix), "Not the named wire payload");
        MemorySegment pointer = RustBuffer.getData(buffer);
        require(live(pointer), "No live allocation at the returned address");
        require(matching(prefix, length) == 1, "Named payload did not identify one live allocation");
        return new Extent(pointer, length);
    }

    private static void verifyFreed(String language, String label, Extent extent, byte[] prefix) throws Throwable {
        boolean addressLive = live(extent.pointer());
        long count = matching(prefix, extent.length());
        System.out.printf("%s Lighting %s: bytes=%d live_address=true live_payload_count=1 "
                + "post_free_address_live=%s post_free_payload_count=%d%n",
                language, label, extent.length(), addressLive, count);
        require(count == 0, label + ": named payload still present in a live extent");
        require(!addressLive, label + ": post-free address remains live; allocation identity unresolved");
    }

    private static long calibrate(String language, Trial trial) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_inspect_lighting(
                    arena, FfiConverterTypeLightingInspectionRequest.INSTANCE.lower(trial.request()), status);
            int code = UniffiRustCallStatus.getCode(status);
            MemorySegment buffer = code == 0 ? result : UniffiRustCallStatus.getErrorBuf(status);
            Extent extent;
            try {
                require(code == (trial.branch() == Branch.INVENTORY ? 0 : 1), "Unexpected native status");
                require(code == 0 || RustBuffer.getLen(result) == 0, "Typed refusal returned a result payload");
                extent = verifyLive(buffer, trial.prefix());
            } finally {
                if (code != 0) RustBuffer.free(buffer);
                RustBuffer.free(result);
            }
            verifyFreed(language, trial.branch().name(), extent, trial.prefix());
            return extent.length();
        }
    }

    private static void checkRollback(String language, Operation operation) throws Throwable {
        byte[] xml = bytes("<!--lighting-inspection-invalid-selector-v1:" + "q".repeat(65536) + "--><Root/>");
        byte[] prefix = ByteBuffer.allocate(128).putInt(xml.length).put(xml, 0, 124).array();
        MemorySegment buffer = FfiConverterTypeLightingInspectionRequest.INSTANCE.lower(
                new LightingInspectionRequest(xml, "uuid:valid"));
        Extent extent;
        try {
            extent = verifyLive(buffer, prefix);
        } finally {
            RustBuffer.free(buffer);
        }
        verifyFreed(language, "marked input control", extent, prefix);
        var bad = new LightingInspectionRequest(xml, "\ud800");
        for (int i = 0; i < 64; i++) {
            try {
                operation.run(bad);
                throw new AssertionError("Expected strict pre-call UTF-8 refusal");
            } catch (RuntimeException failure) {
                if (!(failure.getCause() instanceof MalformedInputException cause)
                        || cause.getInputLength() != 1) throw failure;
            }
        }
        require(matching(prefix, 4L + xml.length) == 0, "UTF-8 refusal retained the marked request");
        System.out.println(language + " Lighting selector rollback: rejections=64 input_native_retained=0 GC_requested=false");
    }

    private static ByteBuffer putString(ByteBuffer buffer, String value) {
        byte[] encoded = bytes(value);
        int room = buffer.remaining() - Integer.BYTES;
        return buffer.putInt(encoded.length).put(encoded, 0, Math.min(room, encoded.length));
    }

    private static void checkOwned(Object value, Trial trial) {
        switch (trial.branch()) {
            case INVENTORY -> checkInventory((LightingInspection) value, trial.expected());
            case NOT_FOUND -> require(value instanceof LightingInspectionException.NotFound failure
                    && failure.selector().equals(trial.expected()) && failure.titles().equals(List.of("Alpha", "Zulu")),
                    "NotFound selector and sorted unique titles");
            case AMBIGUITY -> require(value instanceof LightingInspectionException.Ambiguous failure
                    && failure.selector().equals(AMBIGUOUS) && failure.matches().equals(List.of(
                        new LightingSelectionIdentity(trial.expected(), "And", "first"),
                        new LightingSelectionIdentity(AMBIGUOUS + " other", "LightController2", "second"))),
                    "Ambiguous selector and ordered identities");
            case WRONG_TYPE -> require(value instanceof LightingInspectionException.NotLightController failure
                    && failure.selector().equals("uuid:wrong") && failure.blockType().equals(trial.expected()),
                    "NotLightController selector and stored type");
        }
    }

    public static void verify(String language, Path fixturePath, Operation operation) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null, "Unexpected Rust stack override");
        require(Runtime.version().feature() == 22 && System.getProperty("os.name").startsWith("Windows")
                && System.getProperty("os.arch").equals("amd64"), "Expected Windows AMD64 Java 22");
        String fixture = Files.readString(fixturePath);
        checkValues(operation, fixture);
        String uuid = "lighting-inspection-inventory-v1-灯-" + "u".repeat(65536);
        String missing = "uuid:lighting-inspection-missing-v1-灯-" + "m".repeat(65536);
        String twin = AMBIGUOUS + " lighting-inspection-ambiguity-v1-灯-" + "a".repeat(65536);
        String wrongType = "lighting-inspection-wrong-type-v1-灯-" + "w".repeat(65536);
        // Exact UniFFI prefixes: a present UUID, or a typed error tag followed by its fields.
        var trials = List.of(
                new Trial(Branch.INVENTORY, request(fixture.replace("U=\"light\"", "U=\"" + uuid + "\""), "gid:PRIMARY"),
                    putString(ByteBuffer.allocate(128).put((byte) 1), uuid).array(), uuid),
                new Trial(Branch.NOT_FOUND, request("<Root><C Title='Zulu'/><C Title='Alpha'/><C Title='Zulu'/></Root>", missing),
                    putString(ByteBuffer.allocate(128).putInt(2), missing).array(), missing),
                new Trial(Branch.AMBIGUITY, request("<Root><C Type='And' Title='" + twin + "' U='first'/>"
                        + "<C Type='LightController2' Title='" + AMBIGUOUS + " other' U='second'/></Root>", AMBIGUOUS),
                    putString(putString(ByteBuffer.allocate(128).putInt(3), AMBIGUOUS).putInt(2), twin).array(), twin),
                new Trial(Branch.WRONG_TYPE, request("<C Type='" + wrongType + "' U='wrong'/>", "uuid:wrong"),
                    putString(putString(ByteBuffer.allocate(128).putInt(4), "uuid:wrong"), wrongType).array(), wrongType));
        for (var trial : trials) {
            long length = calibrate(language, trial);
            byte[] caller = trial.request().bytes().clone();
            var supplied = new LightingInspectionRequest(caller, trial.request().selector());
            var held = new ArrayList<Object>(64);
            var identities = Collections.newSetFromMap(new IdentityHashMap<Object, Boolean>());
            for (int i = 0; i < 64; i++) {
                var value = outcome(operation, supplied);
                held.add(value);
                identities.add(value);
            }
            require(Arrays.equals(caller, trial.request().bytes()), "Caller bytes changed");
            Arrays.fill(caller, (byte) 0);
            require(identities.size() == 64, "Expected 64 distinct held host objects");
            require(matching(trial.prefix(), length) == 0, "Public calls retained a named native payload");
            for (var value : held) checkOwned(value, trial);
            System.out.printf("%s Lighting %s: host_retained=64 native_payload_retained=0 GC_requested=false%n",
                    language, trial.branch());
        }
        checkRollback(language, operation);
        System.out.println(language + " Lighting Load, small/empty inventories, and absent fields: values only");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", Path.of(args[0]), LoxArchive::inspectLighting);
    }
}
