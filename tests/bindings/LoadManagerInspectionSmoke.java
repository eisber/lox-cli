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

public final class LoadManagerInspectionSmoke {
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
    private static final String FIRST = "<Load nm=\"\" maxOff=\"\" pwr=\"unknown\" Note=\"at 7am=ok\"/>";
    private static final String ONE = "<Load ioIdx=\"1\"/>";
    private static final String TWIN = "load-manager inventory twin";

    @FunctionalInterface
    public interface Operation {
        LoadManagerInspection run(LoadManagerInspectionRequest request) throws LoadManagerInspectionException;
    }

    private enum Branch { INVENTORY, NOT_FOUND, AMBIGUOUS, WRONG_TYPE, INVALID_INDEX }
    private record Trial(Branch branch, LoadManagerInspectionRequest request, byte[] prefix, String expected) {}
    private record Extent(MemorySegment pointer, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static LoadManagerInspectionRequest request(String xml, String selector) {
        return new LoadManagerInspectionRequest(bytes(xml), selector);
    }

    private static void require(boolean passed, String message) {
        if (!passed) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, LoadManagerInspectionRequest supplied) {
        try {
            return operation.run(supplied);
        } catch (LoadManagerInspectionException failure) {
            return failure;
        }
    }

    private static LoadManagerAttribute attr(String key, String value) {
        return new LoadManagerAttribute(key, value);
    }

    private static void checkInventory(LoadManagerInspection value, String uuid) {
        require(value.uuid().equals(uuid) && value.title().equals("Loads & 灯"), "Stored identity");
        int[] order = {5, 0, 3, 1, 2, 4, 6, 7, 8, 9, 10, 11};
        require(value.loads().size() == order.length, "Direct record count");
        for (int i = 0; i < order.length; i++) {
            var row = value.loads().get(i);
            require(row.recordOrder() == i + 1 && row.ioIndex() == order[i], "Stored order and resolved ioIdx");
            var expected = switch (i) {
                case 0 -> List.of(attr("Future", ""), attr("ioIdx", "05"), attr("minOn", "+0012"),
                        attr("nm", " A & B "), attr("pwr", "02.5000e0"));
                case 1 -> List.of(attr("Note", "at _7am=ok"), attr("maxOff", ""), attr("nm", ""),
                        attr("pwr", "unknown"));
                default -> List.of(attr("ioIdx", Integer.toString(order[i])));
            };
            require(row.attributes().equals(expected), "Exact loader attribute entries in Core key order");
        }
    }

    private static void checkCount(Operation operation, String xml, long found) {
        require(outcome(operation, request(xml, "uuid:load-manager"))
                instanceof LoadManagerInspectionException.RecordCount failure && failure.found() == found,
                "Direct record count refusal before index validation");
    }

    private static void checkValues(Operation operation, String fixture) throws LoadManagerInspectionException {
        for (String selector : List.of("uuid:load-manager", "Loads [work]", "gid:LOAD-BANK", "Type:loadshed")) {
            checkInventory(operation.run(request(fixture, selector)), "load-manager");
        }
        var unnamed = operation.run(request(
                fixture.replace(" U=\"load-manager\" Title=\"Loads &amp; 灯\"", "").replace(FIRST, "<Load/>"),
                "Type:LoadShed"));
        require(unnamed.uuid().isEmpty() && unnamed.title().isEmpty() && unnamed.loads().size() == 12,
                "Absent block identity uses empty strings");
        var empty = unnamed.loads().get(1);
        require(empty.ioIndex() == 0 && empty.attributes().isEmpty(), "Omitted ioIdx and no fabricated attributes");
        require(outcome(operation, request("<Root><C attr='1' attr='2'/></Root>", "uuid:missing"))
                instanceof LoadManagerInspectionException.Load failure && !failure.sourceMessage().isEmpty(),
                "Load refusal before selection");
        require(outcome(operation, request("<C Type='loadshed'/>", "Type:LoadShed"))
                instanceof LoadManagerInspectionException.NotLoadManager failure
                && failure.selector().equals("Type:LoadShed"), "Exact type check after selection");
        checkCount(operation, fixture.replace(ONE, "<Load ioIdx=\"+1\"/>").replace("<Load ioIdx=\"11\"/>", ""), 11);
        checkCount(operation, fixture.replace(ONE, ONE + "<Load/>"), 13);
        for (String raw : List.of("", "+1", " 1", "12", "opaque")) {
            String xml = fixture.replace(ONE, "<Load ioIdx=\"" + raw + "\"/>");
            require(outcome(operation, request(xml, "uuid:load-manager"))
                    instanceof LoadManagerInspectionException.InvalidIoIndex failure && failure.raw().equals(raw),
                    "Invalid stored ioIdx text remains exact");
        }
        for (String replacement : List.of("<Load ioIdx=\"000\"/>", "<Load/>")) {
            require(outcome(operation, request(fixture.replace(ONE, replacement), "uuid:load-manager"))
                    instanceof LoadManagerInspectionException.DuplicateIoIndex failure && failure.ioIndex() == 0,
                    "Duplicate resolved zero");
        }
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
        require(prefix.length == 128 && length > 65536, "Small native buffer");
        byte[] observed = new byte[prefix.length];
        RustBuffer.asByteBuffer(buffer).get(observed);
        require(Arrays.equals(observed, prefix), "Not the named wire payload");
        MemorySegment pointer = RustBuffer.getData(buffer);
        require(live(pointer), "No live allocation at the returned address");
        require(matching(prefix, length) == 1, "Expected one live allocation with the named payload");
        return new Extent(pointer, length);
    }

    private static void verifyFreed(String language, String label, Extent extent, byte[] prefix) throws Throwable {
        boolean addressLive = live(extent.pointer());
        long count = matching(prefix, extent.length());
        System.out.printf("%s Load Manager %s: bytes=%d live_address=true live_payload_count=1 "
                + "post_free_address_live=%s post_free_payload_count=%d%n",
                language, label, extent.length(), addressLive, count);
        require(count == 0, label + ": named payload remains live");
        require(!addressLive, label + ": released address still live; allocation identity unresolved");
    }

    private static long calibrate(String language, Trial trial) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_inspect_load_manager(
                    arena, FfiConverterTypeLoadManagerInspectionRequest.INSTANCE.lower(trial.request()), status);
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
        byte[] xml = bytes("<!--load-manager-invalid-selector-v1:" + "q".repeat(65536) + "--><Root/>");
        byte[] prefix = ByteBuffer.allocate(128).putInt(xml.length).put(xml, 0, 124).array();
        MemorySegment control = FfiConverterTypeLoadManagerInspectionRequest.INSTANCE.lower(
                new LoadManagerInspectionRequest(xml, "uuid:valid"));
        Extent extent;
        try {
            extent = verifyLive(control, prefix);
        } finally {
            RustBuffer.free(control);
        }
        verifyFreed(language, "marked input control", extent, prefix);

        var bad = new LoadManagerInspectionRequest(xml, "\ud800");
        for (int i = 0; i < 64; i++) {
            try {
                operation.run(bad);
                throw new AssertionError("Expected strict pre-call UTF-8 refusal");
            } catch (RuntimeException failure) {
                if (!(failure.getCause() instanceof MalformedInputException cause)
                        || cause.getInputLength() != 1) throw failure;
            }
        }
        require(matching(prefix, 4L + xml.length) == 0, "UTF-8 refusal stranded the marked input");
        System.out.println(language
                + " Load Manager selector rollback: rejections=64 input_native_retained=0 GC_requested=false");
    }

    private static ByteBuffer putString(ByteBuffer buffer, String value) {
        byte[] encoded = bytes(value);
        int room = buffer.remaining() - Integer.BYTES;
        return buffer.putInt(encoded.length).put(encoded, 0, Math.min(room, encoded.length));
    }

    private static void checkOwned(Object value, Trial trial) {
        switch (trial.branch()) {
            case INVENTORY -> checkInventory((LoadManagerInspection) value, trial.expected());
            case NOT_FOUND -> require(value instanceof LoadManagerInspectionException.NotFound failure
                    && failure.selector().equals(trial.expected()) && failure.titles().equals(List.of("Alpha", "Zulu")),
                    "NotFound selector and sorted unique titles");
            case AMBIGUOUS -> require(value instanceof LoadManagerInspectionException.Ambiguous failure
                    && failure.selector().equals(TWIN) && failure.matches().equals(List.of(
                        new LoadManagerSelectionIdentity(trial.expected(), "Switch", "first"),
                        new LoadManagerSelectionIdentity(TWIN + " other", "LoadShed", "second"))),
                    "Ambiguous selector and ordered identities");
            case WRONG_TYPE -> require(value instanceof LoadManagerInspectionException.NotLoadManager failure
                    && failure.selector().equals(trial.expected()), "NotLoadManager selector");
            case INVALID_INDEX -> require(value instanceof LoadManagerInspectionException.InvalidIoIndex failure
                    && failure.raw().equals(trial.expected()), "InvalidIoIndex raw field");
        }
    }

    public static void verify(String language, Path fixturePath, Operation operation) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null, "Unexpected Rust stack override");
        require(Runtime.version().feature() == 22 && System.getProperty("os.name").startsWith("Windows")
                && System.getProperty("os.arch").equals("amd64"), "Expected Windows AMD64 Java 22");
        String fixture = Files.readString(fixturePath);
        checkValues(operation, fixture);
        String uuid = "load-manager-inventory-v1-灯-" + "u".repeat(65536);
        String missing = "uuid:load-manager-missing-v1-灯-" + "m".repeat(65536);
        String twin = TWIN + " load-manager-ambiguity-v1-灯-" + "a".repeat(65536);
        String wrong = "load-manager-wrong-type-v1-灯-" + "w".repeat(65536);
        String invalid = "load-manager-invalid-index-v1-灯-" + "i".repeat(65536);
        // Exact wire prefixes: first field of the success record, or the one-based error tag and fields.
        var trials = List.of(
                new Trial(Branch.INVENTORY, request(fixture.replace("U=\"load-manager\"", "U=\"" + uuid + "\""),
                        "gid:LOAD-BANK"), putString(ByteBuffer.allocate(128), uuid).array(), uuid),
                new Trial(Branch.NOT_FOUND, request("<Root><C Title='Zulu'/><C Title='Alpha'/><C Title='Zulu'/></Root>",
                        missing), putString(ByteBuffer.allocate(128).putInt(2), missing).array(), missing),
                new Trial(Branch.AMBIGUOUS, request("<Root><C Type='Switch' Title='" + twin + "' U='first'/>"
                        + "<C Type='LoadShed' Title='" + TWIN + " other' U='second'/></Root>", TWIN),
                        putString(putString(ByteBuffer.allocate(128).putInt(3), TWIN).putInt(2), twin).array(), twin),
                new Trial(Branch.WRONG_TYPE, request("<C Type='loadshed' U='" + wrong + "'/>", "uuid:" + wrong),
                        putString(ByteBuffer.allocate(128).putInt(4), "uuid:" + wrong).array(), "uuid:" + wrong),
                new Trial(Branch.INVALID_INDEX, request(fixture.replace(ONE, "<Load ioIdx=\"" + invalid + "\"/>"),
                        "uuid:load-manager"), putString(ByteBuffer.allocate(128).putInt(6), invalid).array(), invalid));
        for (var trial : trials) {
            long length = calibrate(language, trial);
            byte[] caller = trial.request().bytes().clone();
            var supplied = new LoadManagerInspectionRequest(caller, trial.request().selector());
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
            System.out.printf("%s Load Manager %s: host_retained=64 native_payload_retained=0 GC_requested=false%n",
                    language, trial.branch());
        }
        checkRollback(language, operation);
        System.out.println(language
                + " Load Manager: Load, RecordCount, DuplicateIoIndex, and empty fields are values only; Other unobserved");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", Path.of(args[0]), LoxArchive::inspectLoadManager);
    }
}
