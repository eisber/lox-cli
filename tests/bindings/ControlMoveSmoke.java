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
import java.util.Arrays;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;

public final class ControlMoveSmoke {
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
    private static final ControlMoveClock CLOCK = new ControlMoveClock(1230768000L, 123456789, 3661);
    private static final ControlMoveClock LATER = new ControlMoveClock(1230854400L, 0, -3661);
    private static final String ROOM = "KÜCHE 室";
    private static final String TWIN = "move-element-";
    private static final String ROOM_QUERY = "move-room-";
    private static final byte[] MALFORMED = bytes("<C A='1' A='2'/>");
    private static final List<String> TITLES = List.of("Dry", "Küche 室", "Küche 室 Annex", "Lamp Ω", "Mirror");

    @FunctionalInterface
    public interface Operation {
        ControlMoveOutcome run(ControlMoveRequest request) throws ControlMoveException;
    }

    private enum Branch { CHANGED, NOT_FOUND, AMBIGUOUS, ROOM_NOT_FOUND, ROOM_AMBIGUOUS }
    private record Trial(Branch branch, ControlMoveRequest request, byte[] prefix, String expected) {}
    private record Extent(MemorySegment pointer, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static String text(byte[] value) {
        return new String(value, StandardCharsets.UTF_8);
    }

    private static byte[] inputBytes(String fixture) {
        return bytes("\ufeff" + fixture.replace("\r\n", "\n").replace("\n", "\r\n"));
    }

    private static ControlMoveRequest request(byte[] data) {
        return new ControlMoveRequest(data, null, "uuid:a", ROOM, List.of(), CLOCK);
    }

    private static String changedText(byte[] data, boolean mirror, boolean later) {
        String output = text(data).replace("Pr=\"old\"", "Pr=\"Raw-r\"")
                .replace("Pr=\"\"", "Pr=\"Raw-r\"").replace("<IoData/>", "<IoData Pr=\"Raw-r\"/>");
        if (mirror) output = output.replace("Pr=\"mirror\"", "Pr=\"Raw-r\"");
        return output.replace("Date=\"old\"", later ? "Date=\"2009-01-01 22:58:59\"" : "Date=\"2009-01-01 01:01:32\"")
                .replace("DateS=\"30\"", later ? "DateS=\"86400\"" : "DateS=\"31\"");
    }

    private static void require(boolean passed, String message) {
        if (!passed) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, ControlMoveRequest request) {
        try {
            return operation.run(request);
        } catch (ControlMoveException failure) {
            return failure;
        }
    }

    private static void checkMove(ControlMoveOutcome value, byte[] expected, ControlMoveSelector selector,
            long count, boolean changed, boolean stamped) {
        require(Arrays.equals(value.bytes(), expected), "Expected exact raw bytes");
        require(value.change().equals(new ControlMoveChange(selector, ROOM, "Raw-r", count, changed))
                && value.contentChanged() == changed && value.documentStamped() == stamped, "Unexpected move outcome");
    }

    private static <T extends ControlMoveException> T refuse(Operation operation, ControlMoveRequest request,
            Class<T> expected) {
        byte[] before = request.bytes().clone();
        Object failure = outcome(operation, request);
        require(expected.isInstance(failure) && Arrays.equals(request.bytes(), before), "Unexpected refusal");
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = rawCall(arena, status, request);
            int code = UniffiRustCallStatus.getCode(status);
            try {
                require(code == 1 && RustBuffer.getLen(result) == 0, "Refusal returned revised bytes");
            } finally {
                if (code != 0) RustBuffer.free(UniffiRustCallStatus.getErrorBuf(status));
                RustBuffer.free(result);
            }
        }
        return expected.cast(failure);
    }

    private static void checkValues(String language, Operation operation, byte[] data) throws ControlMoveException {
        byte[] original = data.clone();
        var title = new ControlMoveSelector.Title("uuid:a");
        var first = operation.run(new ControlMoveRequest(data, null, "uuid:a", ROOM, List.of("future"), CLOCK));
        checkMove(first, bytes(changedText(data, false, false)), title, 1, true, true);
        checkMove(operation.run(new ControlMoveRequest(first.bytes(), null, "uuid:a", ROOM, List.of(), LATER)),
                first.bytes(), title, 1, false, false);
        checkMove(operation.run(new ControlMoveRequest(data, "fUtUrE", "absent", ROOM, List.of(), LATER)),
                bytes(changedText(data, true, true)), new ControlMoveSelector.Type("fUtUrE"), 2, true, true);
        byte[] canonicalNoop = bytes(text(data).replace("<IoData/>", "<IoData></IoData>"));
        checkMove(operation.run(new ControlMoveRequest(data, "Future", "uuid:a", ROOM, List.of("FUTURE"), CLOCK)),
                canonicalNoop, new ControlMoveSelector.Type("Future"), 0, false, false);
        checkMove(operation.run(new ControlMoveRequest(data, null, "Dry", ROOM, List.of(), CLOCK)),
                canonicalNoop, new ControlMoveSelector.Title("Dry"), 1, false, false);
        String place = "<C Type=\"Place\" U=\"Raw-r\" Title=\"Küche 室\"/>";
        String canonical = "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<Root>\n\t" + place + "\n</Root>\n";
        checkMove(operation.run(new ControlMoveRequest(bytes("<Root>" + place + "</Root>"), "", "uuid:a", ROOM,
                List.of(), CLOCK)), bytes(canonical), new ControlMoveSelector.Type(""), 0, false, false);
        checkMove(operation.run(request(bytes("<Root>" + place + "<C U=\"a\"><IoData/></C></Root>"))),
                bytes(canonical.replace("</Root>", "\t<C U=\"a\">\n\t\t<IoData Pr=\"Raw-r\"/>\n\t</C>\n</Root>")),
                title, 1, true, false);
        require(Arrays.equals(data, original), "Caller bytes changed");
        require(!refuse(operation, new ControlMoveRequest(MALFORMED, null, null, ROOM, List.of(), CLOCK),
                ControlMoveException.Load.class).sourceMessage().isEmpty(), "Missing load cause");
        refuse(operation, new ControlMoveRequest(MALFORMED, null, null, ROOM, List.of(),
                new ControlMoveClock(0L, 0, 86400)), ControlMoveException.InvalidClock.class);
        refuse(operation, new ControlMoveRequest(data, null, null, "absent", List.of(), CLOCK),
                ControlMoveException.SelectorMissing.class);
        require(refuse(operation, new ControlMoveRequest(data, null, "absent", "absent", List.of(), CLOCK),
                ControlMoveException.NotFound.class).titles().equals(TITLES), "Wrong title suggestions");
        require(refuse(operation, new ControlMoveRequest(data, "Future", "absent", "absent", List.of(), CLOCK),
                ControlMoveException.RoomNotFound.class).room().equals("absent"), "Type must resolve room first");
        System.out.println(language + " control-move positional records and selector variants, exact bytes and small refusals: values only");
    }

    private static MemorySegment rawCall(Arena arena, MemorySegment status, ControlMoveRequest request) {
        return UniffiLib.uniffi_lox_archive_fn_func_move_control(
                arena, FfiConverterTypeControlMoveRequest.INSTANCE.lower(request), status);
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

    private static Extent liveExtent(MemorySegment buffer, byte[] prefix) throws Throwable {
        long length = RustBuffer.getLen(buffer);
        MemorySegment pointer = RustBuffer.getData(buffer);
        require(pointer.address() != 0 && prefix.length == 128 && length > 2L * prefix.length, "Small native buffer");
        byte[] observed = new byte[prefix.length];
        RustBuffer.asByteBuffer(buffer).get(observed);
        require(Arrays.equals(observed, prefix), "Not the named wire payload");
        require(live(pointer), "Named buffer address not live");
        require(matching(prefix, length) == 1, "Named payload did not identify one live allocation");
        return new Extent(pointer, length);
    }

    private static long freed(String language, String label, Extent extent, byte[] prefix) throws Throwable {
        boolean addressLive = live(extent.pointer());
        long count = matching(prefix, extent.length());
        System.out.printf("%s control-move %s: bytes=%d live_address=true live_payload_count=1 "
                + "post_free_address_live=%s post_free_payload_count=%d%n",
                language, label, extent.length(), addressLive, count);
        require(count == 0, label + ": named payload still present in a live extent");
        require(!addressLive, label + ": post-free address remains live; allocation identity unresolved");
        return extent.length();
    }

    private static long calibrate(String language, Trial trial) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = rawCall(arena, status, trial.request());
            int code = UniffiRustCallStatus.getCode(status);
            MemorySegment buffer = code == 0 ? result : UniffiRustCallStatus.getErrorBuf(status);
            Extent extent;
            try {
                require(code == (trial.branch() == Branch.CHANGED ? 0 : 1), "Unexpected native status");
                require(code == 0 || RustBuffer.getLen(result) == 0, "Refusal returned revised bytes");
                extent = liveExtent(buffer, trial.prefix());
            } finally {
                if (code != 0) RustBuffer.free(buffer);
                RustBuffer.free(result);
            }
            return freed(language, trial.branch().name(), extent, trial.prefix());
        }
    }

    private static ByteBuffer putValue(ByteBuffer buffer, byte[] value) {
        buffer.putInt(value.length);
        return buffer.put(value, 0, Math.min(buffer.remaining(), value.length));
    }

    private static ByteBuffer putString(ByteBuffer buffer, String value) {
        return putValue(buffer, bytes(value));
    }

    private static void checkRollback(String language, Operation operation) throws Throwable {
        String bad = "\ud800";
        for (String field : List.of("type-filter", "title", "target-room", "exclude-item")) {
            byte[] data = bytes("<!--control-move-" + field + "-rejected-input:" + "q".repeat(65536) + "--><Root/>");
            byte[] prefix = putValue(ByteBuffer.allocate(128), data).array();
            MemorySegment input = FfiConverterTypeControlMoveRequest.INSTANCE.lower(request(data));
            Extent extent;
            try {
                extent = liveExtent(input, prefix);
            } finally {
                RustBuffer.free(input);
            }
            freed(language, field + " marked input", extent, prefix);
            var rejected = new ControlMoveRequest(data, field.equals("type-filter") ? bad : null,
                    field.equals("title") ? bad : "uuid:a", field.equals("target-room") ? bad : ROOM,
                    field.equals("exclude-item") ? List.of("valid", bad) : List.of(), CLOCK);
            for (int i = 0; i < 64; i++) {
                try {
                    operation.run(rejected);
                    throw new AssertionError("Expected strict pre-call UTF-8 refusal: " + field);
                } catch (RuntimeException failure) {
                    if (!(failure.getCause() instanceof MalformedInputException cause)
                            || cause.getInputLength() != 1) throw failure;
                }
            }
            require(matching(prefix, 4L + data.length) == 0, field + ": marked native input retained");
            System.out.printf("%s control-move rollback %s: rejections=64 input_native_retained=0 GC_requested=false%n",
                    language, field);
        }
    }

    private static void checkOwned(Object value, Trial trial) {
        switch (trial.branch()) {
            case CHANGED -> checkMove((ControlMoveOutcome) value, bytes(trial.expected()),
                    new ControlMoveSelector.Title("uuid:a"), 1, true, true);
            case NOT_FOUND -> require(value instanceof ControlMoveException.NotFound failure
                    && failure.selector().equals(trial.expected()) && failure.titles().equals(TITLES), "NotFound");
            case AMBIGUOUS -> require(value instanceof ControlMoveException.Ambiguous failure
                    && failure.selector().equals(TWIN) && failure.matches().equals(List.of(
                        new ControlMoveElementIdentity(trial.expected(), "Future", "first"),
                        new ControlMoveElementIdentity(TWIN + "other", "And", "second"))), "Ambiguous order");
            case ROOM_NOT_FOUND -> require(value instanceof ControlMoveException.RoomNotFound failure
                    && failure.room().equals(trial.expected()), "RoomNotFound");
            case ROOM_AMBIGUOUS -> require(value instanceof ControlMoveException.RoomAmbiguous failure
                    && failure.room().equals(ROOM_QUERY)
                    && failure.matches().equals(List.of(trial.expected(), ROOM_QUERY + "other", trial.expected())),
                    "RoomAmbiguous order and duplicate");
        }
    }

    public static void verify(String language, Path fixturePath, Operation operation) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null, "Stock native stack required");
        require(Runtime.version().feature() == 22 && System.getProperty("os.name").startsWith("Windows")
                && System.getProperty("os.arch").equals("amd64"), "Expected Windows AMD64 Java 22");
        String fixture = Files.readString(fixturePath);
        byte[] data = inputBytes(fixture);
        checkValues(language, operation, data);
        String mark = "control-move-wire-result-灯-" + "x".repeat(65536);
        byte[] marked = inputBytes(fixture.replace("<ControlList>", "<ControlList Marker=\"" + mark + "\">"));
        String expectedBytes = changedText(marked, false, false);
        String missing = "control-move-wire-missing-" + "y".repeat(65536);
        String twin = TWIN + "control-move-wire-ambiguous-" + "z".repeat(65536);
        String roomMissing = "control-move-wire-room-missing-" + "m".repeat(65536);
        String roomTwin = ROOM_QUERY + "control-move-wire-room-ambiguous-" + "a".repeat(65536);
        byte[] rooms = bytes("<Root><C Type=\"Place\" U=\"r\" Title=\"" + roomTwin + "\"/>"
                + "<C Type=\"Place\" U=\"other\" Title=\"" + ROOM_QUERY + "other\"/>"
                + "<C Type=\"Place\" U=\"r\" Title=\"" + roomTwin + "\"/></Root>");
        // Exact UniFFI prefixes: byte length and raw output, or error tag and leading fields.
        var trials = List.of(
                new Trial(Branch.CHANGED, request(marked),
                        putString(ByteBuffer.allocate(128), expectedBytes).array(), expectedBytes),
                new Trial(Branch.NOT_FOUND, new ControlMoveRequest(data, null, missing, "absent", List.of(), CLOCK),
                        putString(ByteBuffer.allocate(128).putInt(4), missing).array(), missing),
                new Trial(Branch.AMBIGUOUS, new ControlMoveRequest(bytes(
                        "<Root><C U=\"first\" Type=\"Future\" Title=\"" + twin + "\"/>"
                        + "<C U=\"second\" Type=\"And\" Title=\"" + TWIN + "other\"/></Root>"),
                        null, TWIN, "absent", List.of(), CLOCK),
                        putString(putString(ByteBuffer.allocate(128).putInt(5), TWIN).putInt(2), twin).array(), twin),
                new Trial(Branch.ROOM_NOT_FOUND,
                        new ControlMoveRequest(data, "Future", missing, roomMissing, List.of(), CLOCK),
                        putString(ByteBuffer.allocate(128).putInt(6), roomMissing).array(), roomMissing),
                new Trial(Branch.ROOM_AMBIGUOUS, new ControlMoveRequest(rooms, "", null, ROOM_QUERY, List.of(), CLOCK),
                        putString(putString(ByteBuffer.allocate(128).putInt(7), ROOM_QUERY).putInt(3), roomTwin).array(),
                        roomTwin));
        for (Trial trial : trials) {
            long length = calibrate(language, trial);
            var control = trial.request();
            byte[] caller = control.bytes().clone();
            var supplied = new ControlMoveRequest(caller, control.typeFilter(), control.title(),
                    control.targetRoom(), control.exclude(), control.clock());
            var held = Collections.newSetFromMap(new IdentityHashMap<Object, Boolean>());
            for (int i = 0; i < 64; i++) held.add(outcome(operation, supplied));
            require(Arrays.equals(caller, control.bytes()), "Caller bytes changed");
            Arrays.fill(caller, (byte) 0);
            require(held.size() == 64, "Expected 64 distinct held host objects");
            require(matching(trial.prefix(), length) == 0, "Public calls retained a named native payload");
            for (Object value : held) checkOwned(value, trial);
            if (trial.branch() == Branch.CHANGED) {
                var arrays = Collections.newSetFromMap(new IdentityHashMap<byte[], Boolean>());
                for (Object value : held) arrays.add(((ControlMoveOutcome) value).bytes());
                require(arrays.size() == 64 && !arrays.contains(caller), "Expected separately owned results");
            }
            System.out.printf("%s control-move %s: host_retained=64 native_payload_retained=0 GC_requested=false%n",
                    language, trial.branch());
        }
        checkRollback(language, operation);
        System.out.println(language + " control-move Load, short/no-op results and payload-free errors: values only");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", Path.of(args[0]), LoxArchive::moveControl);
    }
}
