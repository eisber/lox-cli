package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.ByteBuffer;
import java.nio.charset.MalformedInputException;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Collections;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Objects;
import java.util.Set;

public final class SetAttributeSmoke {
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
    private static final SetAttributeClock CLOCK = new SetAttributeClock(1230768000L, 123456789, 3661);
    private static final SetAttributeClock LATER = new SetAttributeClock(1230854400L, 0, -3661);
    private static final String HEADER = "<?xml version=\"1.0\" encoding=\"utf-8\"?>";
    private static final String ORIGINAL = "\ufeff<C Type=\"Document\" DateS=\"30\" Note=\"one\ntwo\" 12hTF=\"true\">\r\n"
            + "<C U=\"a\" Type=\"And\" Title=\"Gâte &amp; 灯\" gid=\"synthetic\" Empty=\"\"/>"
            + "<C U=\"a\" Title=\"Mirror\"/>\r\n</C>\r\n";
    private static final String CANONICAL = "\ufeff" + HEADER + "\r\n"
            + "<C Type=\"Document\" DateS=\"30\" Note=\"one\ntwo\" 12hTF=\"true\">\r\n"
            + "\t<C U=\"a\" Type=\"And\" Title=\"Gâte &amp; 灯\" gid=\"synthetic\" Empty=\"\"/>\r\n"
            + "\t<C U=\"a\" Title=\"Mirror\"/>\r\n</C>\r\n";
    private static final String MALFORMED = "<Root><C attr='1' attr='2'/></Root>";

    @FunctionalInterface
    public interface Operation {
        SetAttributeOutcome run(SetAttributeRequest request) throws SetAttributeException;
    }

    private enum Branch { CHANGED, LOAD, NOT_FOUND, AMBIGUOUS }
    private enum Field { selector, attribute, value }
    private record Trial(Branch branch, SetAttributeRequest request, String expected) {}
    private record Witness(byte[] prefix, long minimum, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static SetAttributeRequest request(String data, String selector, String attribute, String value) {
        return new SetAttributeRequest(bytes(data), selector, attribute, value, CLOCK);
    }

    private static void require(boolean value, String message) {
        if (!value) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, SetAttributeRequest request) {
        try {
            return operation.run(request);
        } catch (SetAttributeException failure) {
            return failure;
        }
    }

    private static void checkChange(SetAttributeOutcome value, String expectedBytes,
            SetAttributeElementIdentity element, String attribute, String old, String next,
            boolean changed, boolean stamped) {
        require(Arrays.equals(value.bytes(), bytes(expectedBytes)), "Unexpected canonical bytes");
        var actual = value.change();
        require(actual.element().equals(element), "Wrong pre-edit identity");
        require(actual.attribute().equals(attribute) && Objects.equals(actual.oldValue(), old)
                && actual.newValue().equals(next), "Wrong raw attribute observation");
        require(value.contentChanged() == changed && value.documentStamped() == stamped, "Wrong flags");
    }

    private static void checkCases(String language, Operation operation) throws SetAttributeException {
        var element = new SetAttributeElementIdentity("Gâte & 灯", "And", "a");
        var caller = bytes(ORIGINAL);
        var noop = operation.run(new SetAttributeRequest(caller, "uuid:a", "Empty", "", LATER));
        checkChange(noop, CANONICAL, element, "Empty", "", "", false, false);
        require(Arrays.equals(caller, bytes(ORIGINAL)) && !Arrays.equals(noop.bytes(), caller),
                "Canonical no-op overwrote the original or skipped normalization");

        String renamed = CANONICAL.replace("12hTF=\"true\">", "12hTF=\"true\" Date=\"2009-01-01 01:01:32\">")
                .replace("DateS=\"30\"", "DateS=\"31\"").replace("Title=\"Gâte &amp; 灯\"", "Title=\"New\"");
        var rename = operation.run(request(ORIGINAL, "gid:SYNTHETIC", "Title", "New"));
        checkChange(rename, renamed, element, "Title", "Gâte & 灯", "New", true, true);

        String raw = "<&\"\n室";
        String updated = renamed.replace("DateS=\"31\"", "DateS=\"32\"").replace("01:01:32", "01:01:33")
                .replace("Empty=\"\"/>", "Empty=\"\" title=\"&lt;&amp;&quot;\n室\"/>");
        var newIdentity = new SetAttributeElementIdentity("New", "And", "a");
        var added = operation.run(new SetAttributeRequest(rename.bytes(), "Type:AND", "title", raw, CLOCK));
        checkChange(added, updated, newIdentity, "title", null, raw, true, true);
        var repeat = operation.run(new SetAttributeRequest(added.bytes(), "uuid:a", "Empty", "", LATER));
        checkChange(repeat, updated, newIdentity, "Empty", "", "", false, false);

        var sparse = operation.run(request("<C U=\"a\"/>", "uuid:a", "Future", ""));
        checkChange(sparse, HEADER + "\n<C U=\"a\" Future=\"\"/>\n",
                new SetAttributeElementIdentity("", "", "a"), "Future", null, "", true, false);
        var legacy = operation.run(request("<C U=\"a\" Title=\"old 12hTF=value\"/>", "uuid:a", "Future", "new"));
        checkChange(legacy, HEADER + "\n<C U=\"a\" Title=\"old 12hTF=value\" Future=\"new\"/>\n",
                new SetAttributeElementIdentity("old _12hTF=value", "", "a"), "Future", null, "new", true, false);
        require(outcome(operation, request("<Root/>", "missing", "", ""))
                instanceof SetAttributeException.NotFound failure && failure.titles().isEmpty(), "Empty suggestions");
        var badClock = new SetAttributeClock(0L, 0, 86400);
        require(outcome(operation, new SetAttributeRequest(bytes(MALFORMED), "missing", "", "", badClock))
                instanceof SetAttributeException.InvalidClock, "Bad clock representation must precede load");
        System.out.println(language + " set-attribute normalized/no-op/short/empty/InvalidClock: value checks only");
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

    private static byte[] lengthPrefixedStart(byte[] payload) {
        return ByteBuffer.allocate(128).putInt(payload.length).put(payload, 0, 124).array();
    }

    private static Witness calibrate(Trial trial) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_set_attribute(
                    arena, FfiConverterTypeSetAttributeRequest.INSTANCE.lower(trial.request()), status);
            var code = UniffiRustCallStatus.getCode(status);
            MemorySegment buffer = code == 0 ? result : UniffiRustCallStatus.getErrorBuf(status);
            byte[] prefix = new byte[trial.branch() == Branch.LOAD ? 12 : 128];
            long length = RustBuffer.getLen(buffer);
            long minimum = trial.branch() == Branch.LOAD ? length : 64000;
            MemorySegment pointer = RustBuffer.getData(buffer);
            try {
                require(code == (trial.branch() == Branch.CHANGED ? 0 : 1), "Unexpected native status");
                require(code == 0 || RustBuffer.getLen(result) == 0, "Refusal returned a result buffer");
                require(length >= Math.max(2L * prefix.length, minimum), "Small actual native result");
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (trial.branch() == Branch.CHANGED) {
                    require(Arrays.equals(prefix, lengthPrefixedStart(bytes(trial.expected()))),
                            "Expected length-prefixed canonical bytes in actual success buffer");
                }
                require(live(pointer), "Actual result address not live");
                require(matching(prefix, minimum) == 1, "Expected one live result prefix");
            } finally {
                if (code != 0) RustBuffer.free(buffer);
                RustBuffer.free(result);
            }
            require(!live(pointer), "Specific result address still live after free");
            require(matching(prefix, minimum) == 0, "Result prefix still present after free");
            return new Witness(prefix, minimum, length);
        }
    }

    private static void rejectedStringReleasesInput(String language, Operation operation, Field field)
            throws Throwable {
        String marker = "set-attribute-rejected-" + field + "-";
        byte[] xml = bytes("<!--" + marker + "q".repeat(65536) + "--><Root/>");
        long minimum = 4L + xml.length;
        var control = FfiConverterTypeSetAttributeRequest.INSTANCE.lower(
                new SetAttributeRequest(xml, "uuid:a", "Future", "new", CLOCK));
        MemorySegment pointer = RustBuffer.getData(control);
        byte[] prefix = new byte[128];
        try {
            require(RustBuffer.getLen(control) >= minimum, "Small actual native input");
            RustBuffer.asByteBuffer(control).get(prefix);
            require(Arrays.equals(prefix, lengthPrefixedStart(xml)), "Wrong length-prefixed marked input");
            require(live(pointer), "Actual input address not live");
            require(matching(prefix, minimum) == 1, "Expected one live input prefix");
        } finally {
            RustBuffer.free(control);
        }
        require(!live(pointer), "Specific input address still live after free");
        require(matching(prefix, minimum) == 0, "Input prefix still present after free");

        String bad = marker + "\ud800";
        var supplied = switch (field) {
            case selector -> new SetAttributeRequest(xml, bad, "Future", "new", CLOCK);
            case attribute -> new SetAttributeRequest(xml, "uuid:a", bad, "new", CLOCK);
            case value -> new SetAttributeRequest(xml, "uuid:a", "Future", bad, CLOCK);
        };
        for (int i = 0; i < 64; i++) {
            try {
                operation.run(supplied);
                throw new AssertionError("Expected strict host " + field + " encoding refusal");
            } catch (RuntimeException failure) {
                if (!(failure.getCause() instanceof MalformedInputException cause)
                        || cause.getInputLength() != 1) throw failure;
            }
        }
        require(matching(prefix, minimum) == 0, "Rejected " + field + " retained a marked native input");
        System.out.println(language + " set-attribute invalid " + field
                + ": input_address_live=1 input_address_post_free=0 input_prefix_live_count=1"
                + " input_prefix_post_free_count=0 rejections=64 input_matching_native_retained=0 GC_requested=false");
    }

    private static void checkOwned(Object value, Trial trial) {
        switch (trial.branch()) {
            case CHANGED -> checkChange((SetAttributeOutcome) value, trial.expected(),
                    new SetAttributeElementIdentity("", "", "a"), "Future", null, "new", true, true);
            case LOAD -> require(value instanceof SetAttributeException.Load failure
                    && !failure.sourceMessage().isEmpty(), "Missing Core load cause");
            case NOT_FOUND -> require(value instanceof SetAttributeException.NotFound failure
                    && failure.selector().equals(trial.expected())
                    && failure.titles().equals(List.of("Gâte & 灯", "Mirror")), "Wrong missing selection");
            case AMBIGUOUS -> require(value instanceof SetAttributeException.Ambiguous failure
                    && failure.selector().equals("set-attr-match-")
                    && failure.matches().equals(List.of(new SetAttributeElementIdentity(trial.expected(), "And", "a"),
                            new SetAttributeElementIdentity("set-attr-match-a", "Or", "b"))), "Wrong ambiguity");
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null, "Stock native stack required");
        require(Runtime.version().feature() == 22, "Java 22 runtime required");
        checkCases(language, operation);
        String mark = "set-attribute-result-Καλημέρα-" + "x".repeat(65536);
        String expectedOutput = HEADER + "\n<C Type=\"Document\" DateS=\"31\" Marker=\"" + mark
                + "\" Date=\"2009-01-01 01:01:32\">\n\t<C U=\"a\" Future=\"new\"/>\n</C>\n";
        String missing = "set-attr-missing-" + "y".repeat(65536);
        String title = "set-attr-match-z-" + "z".repeat(65536);
        for (Trial trial : List.of(
                new Trial(Branch.CHANGED, request("<C Type=\"Document\" DateS=\"30\" Marker=\"" + mark
                        + "\"><C U=\"a\"/></C>", "uuid:a", "Future", "new"), expectedOutput),
                new Trial(Branch.LOAD, request(MALFORMED, "missing", "", ""), null),
                new Trial(Branch.NOT_FOUND, request(ORIGINAL, missing, "", ""), missing),
                new Trial(Branch.AMBIGUOUS, request("<Root><C U=\"a\" Type=\"And\" Title=\"" + title + "\"/>"
                        + "<C U=\"b\" Type=\"Or\" Title=\"set-attr-match-a\"/>"
                        + "<C U=\"a\" Title=\"set-attr-match-mirror\"/></Root>", "set-attr-match-", "", ""), title))) {
            Witness witness = calibrate(trial);
            Set<Object> held = Collections.newSetFromMap(new IdentityHashMap<>());
            var calibration = trial.request();
            byte[] caller = calibration.bytes().clone();
            var supplied = new SetAttributeRequest(caller, calibration.selector(), calibration.attribute(),
                    calibration.value(), calibration.clock());
            for (int i = 0; i < 64; i++) held.add(outcome(operation, supplied));
            require(held.size() == 64, "Expected 64 distinct host-owned values/errors");
            require(Arrays.equals(caller, calibration.bytes()), "Caller bytes changed");
            Arrays.fill(caller, (byte) 0);
            require(matching(witness.prefix(), witness.minimum()) == 0, "Matching native result retained: " + trial.branch());
            for (Object value : held) checkOwned(value, trial);
            if (trial.branch() == Branch.CHANGED) {
                Set<byte[]> outputs = Collections.newSetFromMap(new IdentityHashMap<>());
                for (Object value : held) outputs.add(((SetAttributeOutcome) value).bytes());
                require(outputs.size() == 64, "Output byte arrays were aliased");
            } else if (trial.branch() == Branch.LOAD) {
                require(held.stream().map(value -> ((SetAttributeException.Load) value).sourceMessage())
                        .distinct().count() == 1, "Retained load cause changed");
            }
            System.out.println(language + " set-attribute branch=" + trial.branch() + " actual_bytes=" + witness.length()
                    + " address_live=1 address_post_free=0 prefix_live_count=1 prefix_post_free_count=0"
                    + " calls=64 host_retained=64 matching_native_retained=0 GC_requested=false");
        }
        for (Field field : Field.values()) rejectedStringReleasesInput(language, operation, field);
        System.out.println(language + " set-attribute exact Core bytes, explicit clock and typed atomic refusals PASS");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", LoxArchive::setAttribute);
    }
}
