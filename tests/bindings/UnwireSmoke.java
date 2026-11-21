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

public final class UnwireSmoke {
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
    private static final UnwireClock CLOCK = new UnwireClock(1230768000L, 123456789, 3661);
    private static final UnwireClock LATER = new UnwireClock(1230854400L, 0, -3661);
    private static final String ORIGINAL = "\ufeff<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n"
            + "<C Type=\"Document\" DateS=\"30\" Note=\"first\nsecond\" 12hTF=\"true\">"
            + "<C Type=\"And\" U=\"a\" Title=\"Gâte &amp; 灯\" WF=\"2\">"
            + "<Co K=\"I1\" U=\"raw35-not-a-validated-uuid\" Nc=\"2\" Other=\"keep\">"
            + "<In Input=\"same\" FLG=\"2\"/><In Input=\"same\"/><Future><In Input=\"nested\"/></Future>"
            + "</Co><Co K=\"I1\" U=\"duplicate\"/><Co K=\"Q\" U=\"other\"/></C>"
            + "<C U=\"a\" Title=\"mirror\"><Co K=\"I1\" U=\"mirror\"/></C></C>\r\n";
    private static final String MALFORMED = "<Root><C attr='1' attr='2'/></Root>";

    @FunctionalInterface
    public interface Operation {
        UnwireOutcome run(UnwireRequest request) throws UnwireException;
    }

    private enum Branch { CHANGED, LOAD, NOT_FOUND, AMBIGUOUS, CONNECTOR_MISSING }
    private record Trial(Branch branch, UnwireRequest request, String expected) {}
    private record Witness(byte[] prefix, long minimum, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static String text(UnwireOutcome value) {
        return new String(value.bytes(), StandardCharsets.UTF_8);
    }

    private static UnwireRequest request(String data, String selector, String connector) {
        return new UnwireRequest(bytes(data), selector, connector, CLOCK);
    }

    private static void require(boolean value, String message) {
        if (!value) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, UnwireRequest request) {
        try {
            return operation.run(request);
        } catch (UnwireException failure) {
            return failure;
        }
    }

    private static void checkChange(UnwireOutcome value, UnwireElementIdentity element,
            String connector, String previous, long count, boolean changed, boolean stamped) {
        var actual = value.change();
        require(actual.element().equals(element), "Wrong identity");
        require(actual.connector().equals(connector) && Objects.equals(actual.previousUuid(), previous)
                && actual.removedReferences() == count, "Wrong connector observation");
        require(value.contentChanged() == changed && value.documentStamped() == stamped, "Wrong flags");
    }

    private static void checkCases(String language, Operation operation) throws UnwireException {
        var element = new UnwireElementIdentity("Gâte & 灯", "And", "a");
        var input = request(ORIGINAL, "uuid:a", "I1");
        var first = operation.run(input);
        require(Arrays.equals(input.bytes(), bytes(ORIGINAL)), "Caller bytes changed");
        checkChange(first, element, "I1", "raw35-not-a-validated-uuid", 2, true, true);
        String text = text(first);
        require(text.startsWith("\ufeff<?xml version=\"1.0\" encoding=\"utf-8\"?>\r\n")
                && text.endsWith("\r\n"), "BOM or EOL changed");
        for (String token : List.of(
                "Date=\"2009-01-01 01:01:32\"", "DateS=\"31\"", "Note=\"first\nsecond\" 12hTF=\"true\"",
                "WF=\"2\"", "U=\"raw35-not-a-validated-uuid\" Other=\"keep\"", "<In Input=\"nested\"/>",
                "<Co K=\"I1\" U=\"duplicate\"/>", "<Co K=\"Q\" U=\"other\"/>", "<Co K=\"I1\" U=\"mirror\"/>")) {
            require(text.contains(token), "Missing preserved token: " + token);
        }
        require(!text.contains("Nc=") && !text.contains("Input=\"same\""), "Direct wiring survived");

        var second = operation.run(new UnwireRequest(first.bytes(), "uuid:a", "I1", CLOCK));
        checkChange(second, element, "I1", "raw35-not-a-validated-uuid", 0, true, true);
        require(text(second).contains("Date=\"2009-01-01 01:01:33\"")
                && text(second).contains("DateS=\"32\"") && !text(second).contains("raw35-not-a-validated-uuid"),
                "Second pass did not clear legacy U and advance the stamp");
        var third = operation.run(new UnwireRequest(second.bytes(), "uuid:a", "I1", LATER));
        checkChange(third, element, "I1", null, 0, false, false);
        require(Arrays.equals(third.bytes(), second.bytes()), "No-op returned different canonical bytes");

        var legacy = operation.run(request("<C U=\"a\" Title=\"old 12hTF=value\"><Co K=\"I1\" U=\"\"/></C>",
                "uuid:a", "I1"));
        checkChange(legacy, new UnwireElementIdentity("old _12hTF=value", "", "a"), "I1", "", 0, true, false);
        var shortInput = request("<C U=\"a\"><Co K=\"\"/></C>", "uuid:a", "");
        var shortNoop = operation.run(shortInput);
        checkChange(shortNoop, new UnwireElementIdentity("", "", "a"), "", null, 0, false, false);
        require(!Arrays.equals(shortInput.bytes(), shortNoop.bytes()) && !text(shortNoop).contains("Date"),
                "Canonical no-op may normalize without stamping");
        require(outcome(operation, request(ORIGINAL, "uuid:a", "i1"))
                instanceof UnwireException.ConnectorMissing failure
                && failure.element().equals(element) && failure.connector().equals("i1"), "Case mismatch");
        require(outcome(operation, request("<Root/>", "missing", "I1"))
                instanceof UnwireException.NotFound failure && failure.titles().isEmpty(), "Empty suggestions");
        var badClock = new UnwireClock(0L, 0, 86400);
        require(outcome(operation, new UnwireRequest(bytes(MALFORMED), "uuid:a", "I1", badClock))
                instanceof UnwireException.InvalidClock, "Bad clock representation");
        System.out.println(language + " unwire shared-loader/short/no-op/empty-fields/payload-free cases: value checks only");
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
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_unwire(
                    arena, FfiConverterTypeUnwireRequest.INSTANCE.lower(trial.request()), status);
            var code = UniffiRustCallStatus.getCode(status);
            MemorySegment buffer = code == 0 ? result : UniffiRustCallStatus.getErrorBuf(status);
            byte[] prefix = new byte[trial.branch() == Branch.LOAD ? 12 : 128];
            long length = RustBuffer.getLen(buffer);
            long minimum = trial.branch() == Branch.LOAD ? length : 64000;
            MemorySegment pointer = RustBuffer.getData(buffer);
            try {
                require(code == (trial.branch() == Branch.CHANGED ? 0 : 1), "Unexpected native status");
                require(code == 0 || RustBuffer.getLen(result) == 0, "Refusal returned a byte result");
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

    private static void rejectedStringReleasesInput(String language, Operation operation, String field)
            throws Throwable {
        byte[] xml = bytes("<!--unwire-" + field + "-rejected-input:" + "q".repeat(65536) + "--><Root/>");
        long minimum = 4L + xml.length;
        var control = FfiConverterTypeUnwireRequest.INSTANCE.lower(new UnwireRequest(xml, "uuid:a", "I1", CLOCK));
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
        for (int i = 0; i < 64; i++) {
            try {
                operation.run(field.equals("selector") ? new UnwireRequest(xml, "\ud800", "I1", CLOCK)
                        : new UnwireRequest(xml, "uuid:a", "\ud800", CLOCK));
                throw new AssertionError("Expected strict host " + field + " encoding refusal");
            } catch (RuntimeException failure) {
                if (!(failure.getCause() instanceof MalformedInputException cause)
                        || cause.getInputLength() != 1) throw failure;
            }
        }
        require(matching(prefix, minimum) == 0, "Rejected " + field + " retained a marked native input");
        System.out.println(language + " unwire invalid " + field + ": input_live=1 input_free=0 "
                + "rejections=64 input_native_retained=0 GC_requested=false");
    }

    private static void checkOwned(Object value, Trial trial) {
        switch (trial.branch()) {
            case CHANGED -> {
                var result = (UnwireOutcome) value;
                checkChange(result, new UnwireElementIdentity("", "", "a"), "I1", "raw", 1, true, true);
                require(text(result).contains(trial.expected()) && text(result).contains("DateS=\"31\"")
                        && !text(result).contains("<In "), "Wrong byte result");
            }
            case LOAD -> require(value instanceof UnwireException.Load failure && !failure.sourceMessage().isEmpty(),
                    "Missing Core load cause");
            case NOT_FOUND -> require(value instanceof UnwireException.NotFound failure
                    && failure.selector().equals(trial.expected())
                    && failure.titles().equals(List.of("Gâte & 灯", "mirror")), "Wrong missing selection");
            case AMBIGUOUS -> require(value instanceof UnwireException.Ambiguous failure
                    && failure.selector().equals("unwire-match-")
                    && failure.matches().equals(List.of(new UnwireElementIdentity(trial.expected(), "And", "a"),
                            new UnwireElementIdentity("unwire-match-other", "Or", "b"))), "Wrong ambiguity");
            case CONNECTOR_MISSING -> require(value instanceof UnwireException.ConnectorMissing failure
                    && failure.element().equals(new UnwireElementIdentity(trial.expected(), "And", "a"))
                    && failure.connector().equals("i1"), "Wrong missing connector identity");
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null, "Stock native stack required");
        require(Runtime.version().feature() == 22, "Java 22 runtime required");
        checkCases(language, operation);
        String mark = "unwire-result-Καλημέρα-" + "x".repeat(65536);
        String missing = "unwire-missing-" + "y".repeat(65536);
        String title = "unwire-match-" + "z".repeat(65536);
        for (Trial trial : List.of(
                new Trial(Branch.CHANGED, request("<C Type=\"Document\" DateS=\"30\" Marker=\"" + mark
                        + "\"><C U=\"a\"><Co K=\"I1\" U=\"raw\" Nc=\"1\"><In Input=\"x\"/></Co></C></C>",
                        "uuid:a", "I1"), mark),
                new Trial(Branch.LOAD, request(MALFORMED, "uuid:a", "I1"), null),
                new Trial(Branch.NOT_FOUND, request(ORIGINAL, missing, "I1"), missing),
                new Trial(Branch.AMBIGUOUS, request("<Root><C U=\"a\" Type=\"And\" Title=\"" + title + "\"/>"
                        + "<C U=\"b\" Type=\"Or\" Title=\"unwire-match-other\"/></Root>", "unwire-match-", "I1"), title),
                new Trial(Branch.CONNECTOR_MISSING, request("<C U=\"a\" Type=\"And\" Title=\"" + title + "\"/>",
                        "uuid:a", "i1"), title))) {
            Witness witness = calibrate(trial);
            Set<Object> held = Collections.newSetFromMap(new IdentityHashMap<>());
            var calibration = trial.request();
            byte[] caller = calibration.bytes().clone();
            var supplied = new UnwireRequest(caller, calibration.selector(), calibration.connector(), calibration.clock());
            for (int i = 0; i < 64; i++) held.add(outcome(operation, supplied));
            require(held.size() == 64, "Expected 64 distinct host-owned values/errors");
            require(Arrays.equals(caller, calibration.bytes()), "Caller bytes changed");
            Arrays.fill(caller, (byte) 0);
            require(matching(witness.prefix(), witness.minimum()) == 0, "Native result retained: " + trial.branch());
            for (Object value : held) checkOwned(value, trial);
            if (trial.branch() == Branch.CHANGED) {
                Set<byte[]> outputs = Collections.newSetFromMap(new IdentityHashMap<>());
                for (Object value : held) outputs.add(((UnwireOutcome) value).bytes());
                require(outputs.size() == 64, "Output byte arrays were aliased");
                byte[] sample = outputs.iterator().next();
                require(outputs.stream().allMatch(output -> Arrays.equals(sample, output)), "Outputs differ");
            } else if (trial.branch() == Branch.LOAD) {
                require(held.stream().map(value -> ((UnwireException.Load) value).sourceMessage())
                        .distinct().count() == 1, "Retained load cause changed");
            }
            System.out.println(language + " unwire branch=" + trial.branch() + " actual_bytes=" + witness.length()
                    + " live=1 free=0 calls=64 host_retained=64 native_retained=0 GC_requested=false");
        }
        for (String field : List.of("selector", "connector")) rejectedStringReleasesInput(language, operation, field);
        System.out.println(language + " unwire raw bytes/clock/typed errors PASS");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", LoxArchive::unwire);
    }
}
