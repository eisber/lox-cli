package net.eisber.lox.archive;

import java.lang.foreign.*;
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
import java.util.Locale;
import java.util.Map;
import java.util.Set;

public final class DaytimerRemovalSmoke {
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
    private static final String BLOCK = "ABCDEF00-0000-0004-FFFF000000000000";
    private static final String BLOCK_RFC = "ABCDEF00-0000-0004-FFFF-000000000000";
    private static final String MODE = "abcdef01-abcd-0004-1500-0000abcdef00";
    private static final String MODE_NATIVE = "ABCDEF01-ABCD-0004-15000000ABCDEF00";
    private static final String MODE_B = "00000000-0000-0005-1500000000000000";
    private static final DaytimerRemovalClock CLOCK = new DaytimerRemovalClock(1230768000L, 123456789, 3661);
    private static final DaytimerRemovalClock LATER = new DaytimerRemovalClock(1230940800L, 987654321, -3661);
    private static final String MALFORMED = "<Root><C attr='1' attr='2'/></Root>";
    private static final String MARKER = "entfernt & 灯";
    private static final String SELECTED = "\t\t\t\t<Entry To=\"180\" V=\"+005\" Marker=\"entfernt &amp; 灯\"/>\r\n";
    private static final String SURVIVOR = "\t\t\t\t<Entry To=\"180\" Ix=\"+01\" V=\"+004\" NeedActivate=\"TrUe\"/>\r\n";

    @FunctionalInterface
    public interface Operation {
        DaytimerRemovalOutcome run(DaytimerRemovalRequest request) throws DaytimerRemovalException;
    }

    private enum Branch { SUCCESS, LOAD, NOT_FOUND, NOT_DAYTIMER, UNRESOLVED_MODE, INVALID_INTEGER }
    private record Trial(Branch branch, DaytimerRemovalRequest request, boolean large) {}
    private record Witness(byte[] prefix, long minimum, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static DaytimerRemovalRequest request(String xml) {
        return new DaytimerRemovalRequest(bytes(xml), BLOCK, MODE, 0, 180, CLOCK);
    }

    private static void require(boolean value, String message) {
        if (!value) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, DaytimerRemovalRequest request) {
        try {
            return operation.run(request);
        } catch (DaytimerRemovalException failure) {
            return failure;
        }
    }

    private static void refuse(Operation operation, DaytimerRemovalRequest request,
            Class<? extends DaytimerRemovalException> expected) {
        byte[] before = request.bytes().clone();
        Object result = outcome(operation, request);
        require(expected.isInstance(result), "Expected " + expected.getSimpleName() + ", got " + result);
        require(Arrays.equals(before, request.bytes()), "Refusal mutated caller bytes");
    }

    private static String firstXml(String source, String selected) {
        return source.replace(selected, "").replace(" N=\"+02\"", " N=\"1\"")
                .replace("Date=\"2009-01-02 00:00:00\" DateS=\"86400\"",
                        "Date=\"2009-01-02 01:01:02\" DateS=\"86401\"");
    }

    private static void checkFirst(DaytimerRemovalOutcome value, byte[] expected, String marker) {
        require(Arrays.equals(value.bytes(), expected), "Expected full canonical bytes");
        var item = value.removal();
        require(item.blockUuid().equals(BLOCK) && item.requestedModeUuid().equals(MODE)
                && item.storedModeUuid().equals(MODE_NATIVE), "Wrong raw UUID spellings");
        require(item.fromMinutes() == 0 && item.toMinutes() == 180 && item.remainingEntries() == 1,
                "Wrong removed interval or remaining count");
        require(item.removedAttributes().equals(Map.of("To", "180", "V", "+005", "Marker", marker)),
                "Wrong raw removed attributes");
    }

    private static void checkCases(String language, Operation operation, String xml) throws DaytimerRemovalException {
        var original = request(xml);
        var first = operation.run(original);
        String firstXml = firstXml(xml, SELECTED);
        checkFirst(first, bytes(firstXml), MARKER);
        var later = operation.run(new DaytimerRemovalRequest(original.bytes(), BLOCK, MODE, 0, 180, LATER));
        checkFirst(later, bytes(firstXml.replace("2009-01-02 01:01:02\" DateS=\"86401\"",
                "2009-01-02 22:58:59\" DateS=\"172800\"")), MARKER);
        var last = operation.run(new DaytimerRemovalRequest(first.bytes(), BLOCK, MODE_B, 0, 180, CLOCK));
        require(Arrays.equals(last.bytes(), bytes(firstXml.replace(SURVIVOR, "").replace(" N=\"1\"", "")
                .replace("01:01:02\" DateS=\"86401\"", "01:01:03\" DateS=\"86402\""))),
                "Wrong last-entry canonical output");
        var item = last.removal();
        require(item.remainingEntries() == 0 && item.blockUuid().equals(BLOCK)
                && item.requestedModeUuid().equals(MODE_B) && item.storedModeUuid().equals(MODE_B)
                && item.fromMinutes() == 0 && item.toMinutes() == 180, "Wrong last-entry identity");
        require(item.removedAttributes().equals(Map.of("To", "180", "Ix", "+01", "V", "+004", "NeedActivate", "TrUe")),
                "Wrong last-entry attributes");

        String program = xml.replace(BLOCK, BLOCK_RFC).replace("<C Type=\"Page\">", "<C Type=\"Program\"><C Type=\"Page\">")
                .replace("\t\t</C>\r\n\t\t<C Type=\"Mode\"", "\t\t</C></C>\r\n\t\t<C Type=\"Mode\"");
        var admitted = operation.run(new DaytimerRemovalRequest(bytes(program), BLOCK_RFC, MODE_NATIVE, 0, 180, CLOCK));
        require(admitted.removal().blockUuid().equals(BLOCK_RFC)
                && admitted.removal().requestedModeUuid().equals(MODE_NATIVE)
                && admitted.removal().storedModeUuid().equals(MODE_NATIVE), "Wrong Program/Page or native/RFC admission");

        refuse(operation, new DaytimerRemovalRequest(last.bytes(), BLOCK, MODE, 0, 180, CLOCK),
                DaytimerRemovalException.EntryNotFound.class);
        refuse(operation, new DaytimerRemovalRequest(original.bytes(), BLOCK.toLowerCase(Locale.ROOT),
                MODE, 0, 180, CLOCK), DaytimerRemovalException.NotFound.class);
        refuse(operation, new DaytimerRemovalRequest(original.bytes(), BLOCK_RFC,
                MODE, 0, 180, CLOCK), DaytimerRemovalException.NotFound.class);
        for (String invalid : List.of("Schedule [Kitchen]", "uuid:" + BLOCK)) {
            refuse(operation, new DaytimerRemovalRequest(original.bytes(), invalid, MODE, 0, 180, CLOCK),
                    DaytimerRemovalException.InvalidBlockUuid.class);
        }
        refuse(operation, new DaytimerRemovalRequest(original.bytes(), BLOCK, "00000000-0000-0000-0000-000000000000",
                0, 180, CLOCK), DaytimerRemovalException.InvalidModeUuid.class);
        refuse(operation, new DaytimerRemovalRequest(original.bytes(), BLOCK, MODE, 180, 180, CLOCK),
                DaytimerRemovalException.InvalidInterval.class);
        refuse(operation, new DaytimerRemovalRequest(original.bytes(), BLOCK, MODE, 0, 1441, CLOCK),
                DaytimerRemovalException.InvalidInterval.class);
        refuse(operation, new DaytimerRemovalRequest(bytes(MALFORMED), BLOCK, MODE, 0, 180,
                new DaytimerRemovalClock(0L, 0, 86400)), DaytimerRemovalException.InvalidClock.class);
        var empty = request("<Root/>");
        require(outcome(operation, empty) instanceof DaytimerRemovalException.NotFound failure
                && failure.titles().isEmpty() && Arrays.equals(empty.bytes(), bytes("<Root/>")),
                "Expected untouched input and empty suggestions");
        refuse(operation, request(xml.replace("Type=\"Page\"", "Type=\"Folder\"")),
                DaytimerRemovalException.OutsidePage.class);
        refuse(operation, request(xml.replace("Analog=\"false\"", "Analog=\"true\"")),
                DaytimerRemovalException.AnalogSchedule.class);
        refuse(operation, request(xml.replace(" N=\"+02\"", " N=\"3\"")),
                DaytimerRemovalException.EntryCountMismatch.class);
        refuse(operation, request(xml.replace(MODE_NATIVE + "," + MODE_B, MODE_NATIVE + "," + MODE)),
                DaytimerRemovalException.DuplicateModes.class);
        refuse(operation, request(xml.replace(SELECTED, "<x:Entry xmlns:x=\"urn:x\" To=\"180\"/>")),
                DaytimerRemovalException.NamespacedEntry.class);
        refuse(operation, request(xml.replace(SURVIVOR, "<Entry xmlns=\"urn:x\" To=\"180\" Ix=\"1\"/>")),
                DaytimerRemovalException.NamespacedEntry.class);
        require(Arrays.equals(original.bytes(), bytes(xml)), "Original caller bytes changed");
        System.out.println(language + " DayTimer last-entry/empty suggestions/payload-free branches: value checks only");
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

    private static Witness calibrate(Trial trial) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_remove_daytimer_entry(
                    arena, FfiConverterTypeDaytimerRemovalRequest.INSTANCE.lower(trial.request()), status);
            var code = UniffiRustCallStatus.getCode(status);
            MemorySegment buffer = code == 0 ? result : UniffiRustCallStatus.getErrorBuf(status);
            byte[] prefix = new byte[trial.large() ? 128 : 12];
            long length = RustBuffer.getLen(buffer);
            long minimum = trial.large() ? 64000 : length;
            MemorySegment pointer = RustBuffer.getData(buffer);
            try {
                require(code == (trial.branch() == Branch.SUCCESS ? 0 : 1), "Unexpected native status");
                require(code == 0 || RustBuffer.getLen(result) == 0, "Typed refusal returned a result buffer");
                require(length >= Math.max(2L * prefix.length, minimum), "Small actual native result");
                RustBuffer.asByteBuffer(buffer).get(prefix);
                require(live(pointer) && matching(prefix, minimum) == 1, "Live result calibration");
            } finally {
                if (code != 0) RustBuffer.free(buffer);
                RustBuffer.free(result);
            }
            require(!live(pointer) && matching(prefix, minimum) == 0, "Released result calibration");
            return new Witness(prefix, minimum, length);
        }
    }

    private static void checkRollback(String language, Operation operation, boolean block) throws Throwable {
        String field = block ? "block_uuid" : "mode_uuid";
        String marked = "<!--daytimer-removal-" + field + "-rejected-input:" + "q".repeat(65536) + "--><Root/>";
        byte[] xml = bytes(marked);
        long minimum = 4L + xml.length;
        var control = FfiConverterTypeDaytimerRemovalRequest.INSTANCE.lower(
                new DaytimerRemovalRequest(xml, BLOCK, MODE, 0, 180, CLOCK));
        MemorySegment pointer = RustBuffer.getData(control);
        byte[] prefix = new byte[128];
        try {
            require(RustBuffer.getLen(control) >= minimum, "Small actual native input");
            RustBuffer.asByteBuffer(control).get(prefix);
            require(Arrays.equals(prefix, ByteBuffer.allocate(128).putInt(xml.length).put(xml, 0, 124).array()),
                    "Expected the actual length-prefixed marked XML in the request buffer");
            require(live(pointer) && matching(prefix, minimum) == 1, "Live marked input calibration");
        } finally {
            RustBuffer.free(control);
        }
        require(!live(pointer) && matching(prefix, minimum) == 0, "Released marked input calibration");
        var bad = new DaytimerRemovalRequest(xml, block ? "\ud800" : BLOCK, block ? MODE : "\ud800", 0, 180, CLOCK);
        for (int i = 0; i < 64; i++) {
            try {
                operation.run(bad);
                throw new AssertionError("Expected strict pre-call " + field + " refusal");
            } catch (RuntimeException failure) {
                if (!(failure.getCause() instanceof MalformedInputException cause)
                        || cause.getInputLength() != 1) throw failure;
            }
        }
        require(matching(prefix, minimum) == 0, "Rejected " + field + " retained marked native XML");
        System.out.println(language + " DayTimer " + field + " pre-call UTF-8: input_live=1 input_free=0 "
                + "rejections=64 input_native_retained=0 GC_requested=false");
    }

    private static void checkError(Object value, Branch branch, String marker) {
        boolean correct = switch (branch) {
            case LOAD -> value instanceof DaytimerRemovalException.Load failure && !failure.sourceMessage().isEmpty();
            case NOT_FOUND -> value instanceof DaytimerRemovalException.NotFound failure
                    && failure.selector().equals("uuid:" + BLOCK_RFC) && failure.titles().equals(List.of(marker));
            case NOT_DAYTIMER -> value instanceof DaytimerRemovalException.NotDayTimer failure
                    && failure.selector().equals("uuid:" + BLOCK);
            case UNRESOLVED_MODE -> value instanceof DaytimerRemovalException.UnresolvedMode failure
                    && failure.storedModeUuid().equals(MODE_NATIVE);
            case INVALID_INTEGER -> value instanceof DaytimerRemovalException.InvalidInteger failure
                    && failure.attribute().equals("N") && failure.value().equals(marker);
            case SUCCESS -> false;
        };
        require(correct, "Wrong error or payload for " + branch);
    }

    public static void verify(String language, Operation operation) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null, "Stock native stack required");
        require(Runtime.version().feature() == 22, "Java 22 runtime required");
        String xml = "\ufeff" + Files.readString(Path.of("crates", "lox-bindings", "tests", "fixtures",
                "daytimer_removal.xml")).replace("\r\n", "\n").replace("\n", "\r\n");
        checkCases(language, operation, xml);
        String marker = "daytimer-removal-Καλημέρα-" + "x".repeat(65536);
        String marked = xml.replace("KeEp", marker).replace("entfernt", marker);
        for (Trial trial : List.of(
                new Trial(Branch.SUCCESS, request(marked), true),
                new Trial(Branch.LOAD, request(MALFORMED), false),
                new Trial(Branch.NOT_FOUND, new DaytimerRemovalRequest(
                        bytes(xml.replace("Title=\"Schedule\"", "Title=\"" + marker + "\"")),
                        BLOCK_RFC, MODE, 0, 180, CLOCK), true),
                new Trial(Branch.NOT_DAYTIMER, request(xml.replace("Type=\"DayTimer\"", "Type=\"And\"")), false),
                new Trial(Branch.UNRESOLVED_MODE, request(xml.replace("U=\"" + MODE + "\"",
                        "U=\"" + BLOCK_RFC + "\"")), false),
                new Trial(Branch.INVALID_INTEGER, request(xml.replace(" N=\"+02\"", " N=\"" + marker + "\"")), true))) {
            Witness witness = calibrate(trial);
            var control = trial.request();
            byte[] caller = control.bytes().clone();
            var supplied = new DaytimerRemovalRequest(caller, control.blockUuid(), control.modeUuid(),
                    control.fromMinutes(), control.toMinutes(), control.clock());
            Set<Object> held = Collections.newSetFromMap(new IdentityHashMap<>());
            for (int i = 0; i < 64; i++) held.add(outcome(operation, supplied));
            require(held.size() == 64 && Arrays.equals(caller, control.bytes()), "Host identity or caller bytes changed");
            Arrays.fill(caller, (byte) 0);
            require(matching(witness.prefix(), witness.minimum()) == 0, "Native result retained: " + trial.branch());
            if (trial.branch() == Branch.SUCCESS) {
                byte[] expected = bytes(firstXml(marked, SELECTED.replace("entfernt", marker)));
                Set<byte[]> outputs = Collections.newSetFromMap(new IdentityHashMap<>());
                Set<Map<String, String>> attributes = Collections.newSetFromMap(new IdentityHashMap<>());
                for (Object value : held) {
                    var actual = (DaytimerRemovalOutcome) value;
                    checkFirst(actual, expected, MARKER.replace("entfernt", marker));
                    outputs.add(actual.bytes());
                    attributes.add(actual.removal().removedAttributes());
                }
                require(outputs.size() == 64 && attributes.size() == 64, "Output bytes or maps aliased");
            } else {
                for (Object value : held) checkError(value, trial.branch(), marker);
                if (trial.branch() == Branch.LOAD) {
                    require(held.stream().map(value -> ((DaytimerRemovalException.Load) value).sourceMessage())
                            .distinct().count() == 1, "Retained load cause changed");
                }
            }
            System.out.println(language + " DayTimer branch=" + trial.branch() + " actual_bytes=" + witness.length()
                    + " live=1 free=0 calls=64 host_retained=64 native_retained=0 GC_requested=false");
        }
        checkRollback(language, operation, true);
        checkRollback(language, operation, false);
        System.out.println(language + " DayTimer removal exact bytes/typed atomic errors PASS");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", LoxArchive::removeDaytimerEntry);
    }
}
