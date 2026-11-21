package net.eisber.lox.archive;

import java.lang.foreign.*;
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
import java.util.Set;

public final class StatusMonitorSmoke {
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
    private static final String TITLE = "Monitor & 灯";
    private static final String TITLE_XML = "Monitor &amp; 灯";

    @FunctionalInterface
    public interface Operation {
        StatusMonitorInspection run(byte[] input, String selector) throws StatusMonitorException;
    }

    private record Trial(String branch, byte[] input, String selector, boolean error, String marker, String expected) {}
    private record Witness(byte[] prefix, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static byte[] replace(String input, String needle, String value) {
        int at = input.indexOf(needle);
        require(at >= 0, "Missing fixture replacement target: " + needle);
        return bytes(input.substring(0, at) + value + input.substring(at + needle.length()));
    }

    private static void require(boolean value, String message) {
        if (!value) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, byte[] data, String selector) {
        try {
            return operation.run(data, selector);
        } catch (StatusMonitorException failure) {
            return failure;
        }
    }

    private static StatusMonitorAttribute attribute(String name, String value) {
        return new StatusMonitorAttribute(name, value);
    }

    private static void checkInventory(Object value, String title, String uuid) {
        require(value instanceof StatusMonitorInspection, "Expected stored inventory");
        var inventory = (StatusMonitorInspection) value;
        require(inventory.title().equals(title) && inventory.uuid().equals(uuid), "Block identity text");
        require(inventory.states().size() == 11, "State count");
        var ids = new ArrayList<String>();
        ids.add("A0000000-0000-0000-0000000000000001");
        for (int i = 2; i <= 11; i++) ids.add("a0000000-0000-0000-%016x".formatted(i));
        int[] outputs = {2, 0, 1, 3, 4, 5, 6, 7, 8, 9, 10};
        for (int i = 0; i < outputs.length; i++) {
            var state = inventory.states().get(i);
            require(state.priority() == i + 1 && state.outputIndex() == outputs[i]
                    && state.id().equals(ids.get(i)) && state.configured() == (i == 0),
                    "State identity, order or configured flag at " + i);
            var expected = switch (i) {
                case 0 -> List.of(attribute("Future", ""), attribute("Id", ids.get(i)),
                        attribute("Na", "  Küche e\u0301 & 灯  "), attribute("OIdx", "02"), attribute("Val", "+01.2500"));
                case 1 -> List.of(attribute("Id", ids.get(i)), attribute("Na", ""),
                        attribute("OIdx", "0"), attribute("Val", "-002.0"));
                case 2 -> List.of(attribute("Id", ids.get(i)), attribute("Na", "No value"), attribute("OIdx", "1"));
                case 3 -> List.of(attribute("Id", ids.get(i)), attribute("OIdx", "3"), attribute("Val", "3e0"));
                case 4 -> List.of(attribute("Id", ids.get(i)), attribute("Legacy", "at _7am=ok"), attribute("OIdx", "4"));
                case 10 -> List.of(attribute("Id", ids.get(i)), attribute("Na", " Explicit Rest "), attribute("OIdx", "10"));
                default -> List.of(attribute("Id", ids.get(i)), attribute("OIdx", Integer.toString(outputs[i])));
            };
            require(state.attributes().equals(expected), "Raw attribute order/text or an injected default at " + i);
        }
    }

    private static void checkAmbiguity(Object value, String title, String selector) {
        require(value instanceof StatusMonitorException.Ambiguous failure && failure.selector().equals(selector)
                && failure.matches().equals(List.of(new StatusMonitorElementIdentity(title, "StatusMonitor", "mon"),
                        new StatusMonitorElementIdentity(title, "statusmonitor", "wrong"))),
                "Ordered selection identities before type admission");
    }

    private static void checkSemantics(Operation operation, String xml) throws StatusMonitorException {
        byte[] source = bytes(xml);
        byte[] original = source.clone();
        for (String selector : List.of("uuid:mon", "gid:PRIMARY", TITLE + " [study]")) {
            checkInventory(operation.run(source, selector), TITLE, "mon");
        }
        checkInventory(operation.run(replace(xml, " U=\"mon\" Title=\"" + TITLE_XML + "\"", ""),
                "gid:primary"), "", "");
        var load = outcome(operation, bytes("<C A='1' A='2'/>"), "missing");
        require(load instanceof StatusMonitorException.Load failure && !failure.sourceMessage().isEmpty(),
                "Expected parser cause before selection");
        var missing = outcome(operation, source, "missing");
        require(missing instanceof StatusMonitorException.NotFound failure && failure.selector().equals("missing")
                && failure.titles().equals(List.of(TITLE, "Study")), "Missing selection payload");
        checkAmbiguity(outcome(operation, source, TITLE), TITLE, TITLE);
        byte[] inherited = replace(xml, "<Root>", "<Root><Other iObjS=\"\"/>");
        var wrong = outcome(operation, inherited, "uuid:wrong");
        require(wrong instanceof StatusMonitorException.NotStatusMonitor failure
                && failure.selector().equals("uuid:wrong"), "Exact type before inheritance admission");
        require(outcome(operation, inherited, "uuid:mon") instanceof StatusMonitorException.InheritedConfiguration,
                "Document-wide inheritance refusal");
        var count = outcome(operation, bytes("<C Type=\"StatusMonitor\" U=\"short\"><Other><State/></Other></C>"),
                "uuid:short");
        require(count instanceof StatusMonitorException.StateCount failure && failure.found() == 0,
                "Only direct State records count");
        String duplicateId = "a0000000-0000-0000-0000000000000001";
        var duplicate = outcome(operation, replace(xml, "a0000000-0000-0000-0000000000000002", duplicateId),
                "uuid:mon");
        require(duplicate instanceof StatusMonitorException.DuplicateStateId failure && failure.id().equals(duplicateId),
                "Duplicate ID reports original spelling");
        var output = outcome(operation, replace(xml, "OIdx=\"0\"", "OIdx=\"002\""), "uuid:mon");
        require(output instanceof StatusMonitorException.DuplicateOutputIndex failure && failure.outputIndex() == 2,
                "Duplicate output index reports the numeric index");
        var collision = outcome(operation, replace(xml, "-002.0", "1.25e0"), "uuid:mon");
        require(collision instanceof StatusMonitorException.DuplicateValue failure && failure.value() == 1.25,
                "Duplicate matching value reports the numeric value");
        require(Arrays.equals(source, original), "Caller bytes changed");
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
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_inspect_status_monitor(
                    arena, FfiConverterTypeStatusMonitorRequest.INSTANCE.lower(
                            new StatusMonitorRequest(trial.input(), trial.selector())), status);
            var code = UniffiRustCallStatus.getCode(status);
            MemorySegment buffer = code == 0 ? result : UniffiRustCallStatus.getErrorBuf(status);
            byte[] prefix = new byte[128];
            long length = RustBuffer.getLen(buffer);
            MemorySegment pointer = RustBuffer.getData(buffer);
            try {
                require(code == (trial.error() ? 1 : 0), "Unexpected native status");
                require(length >= 64000, "Small actual native result");
                RustBuffer.asByteBuffer(buffer).get(prefix);
                require(new String(prefix, StandardCharsets.ISO_8859_1).contains(trial.marker()),
                        "Expected marked payload in the actual native output");
                require(live(pointer) && matching(prefix, 64000) == 1, "Live result calibration");
            } finally {
                if (code != 0) RustBuffer.free(buffer);
                RustBuffer.free(result);
            }
            require(!live(pointer) && matching(prefix, 64000) == 0, "Released result calibration");
            return new Witness(prefix, length);
        }
    }

    private static void rejectedSelectorReleasesInput(String language, Operation operation, String source)
            throws Throwable {
        byte[] xml = bytes("<!--status-monitor-rejected-selector-input:" + "q".repeat(65536) + "-->\n" + source);
        checkInventory(operation.run(xml, "uuid:mon"), TITLE, "mon");
        long minimum = 4L + xml.length;
        var control = FfiConverterTypeStatusMonitorRequest.INSTANCE.lower(new StatusMonitorRequest(xml, "uuid:mon"));
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
                operation.run(xml, "\ud800");
                throw new AssertionError("Expected strict host selector-encoding refusal");
            } catch (RuntimeException failure) {
                if (!(failure.getCause() instanceof MalformedInputException cause)
                        || cause.getInputLength() != 1) throw failure;
            }
        }
        require(matching(prefix, minimum) == 0, "Rejected selector retained a native input buffer");
        System.out.println(language + " status-monitor invalid selector: input_live=1 input_free=0 "
                + "rejections=64 input_native_retained=0 GC_requested=false");
    }

    public static void verify(String language, Operation operation) throws Throwable {
        require(Runtime.version().feature() == 22, "Java 22 artifact qualification required");
        require(System.getenv("RUST_MIN_STACK") == null, "Stock native stack required");
        String xml = Files.readString(Path.of("crates", "lox-bindings", "tests", "fixtures", "status_monitor.xml"));
        checkSemantics(operation, xml);
        String titleMarker = "status-monitor-owned-title-";
        String valueMarker = "status-monitor-invalid-value-";
        String title = titleMarker + "x".repeat(65536);
        String raw = valueMarker + "z".repeat(65536);
        String ambiguousSelector = "owned-ambiguous";
        String ambiguousMarker = "status-monitor-" + ambiguousSelector + "-";
        String ambiguousTitle = ambiguousMarker + "a".repeat(65536);
        for (Trial trial : List.of(
                new Trial("inventory", replace(xml, TITLE_XML, title), "uuid:mon", false, titleMarker, title),
                new Trial("invalid-value", replace(xml, "+01.2500", raw), "uuid:mon", true, valueMarker, raw),
                new Trial("ambiguous", bytes(xml.replace(TITLE_XML, ambiguousTitle)), ambiguousSelector,
                        true, ambiguousMarker, ambiguousTitle))) {
            Witness witness = calibrate(trial);
            Set<Object> retained = Collections.newSetFromMap(new IdentityHashMap<>());
            for (int i = 0; i < 64; i++) retained.add(outcome(operation, trial.input(), trial.selector()));
            require(retained.size() == 64, "Expected 64 distinct host-owned values/errors");
            Arrays.fill(trial.input(), (byte) 0);
            require(matching(witness.prefix(), 64000) == 0, trial.branch() + " native RustBuffer leak");
            for (Object value : retained) {
                switch (trial.branch()) {
                    case "inventory" -> checkInventory(value, trial.expected(), "mon");
                    case "invalid-value" -> require(value instanceof StatusMonitorException.InvalidValue failure
                            && failure.raw().equals(trial.expected()), "Exact stored Val in the owned error");
                    case "ambiguous" -> checkAmbiguity(value, trial.expected(), trial.selector());
                    default -> throw new AssertionError("Unexpected branch: " + trial.branch());
                }
            }
            System.out.println(language + " status-monitor branch=" + trial.branch() + " actual_bytes="
                    + witness.length() + " live=1 free=0 calls=64 host_retained=64 native_retained=0 GC_requested=false");
        }
        rejectedSelectorReleasesInput(language, operation, xml);
        System.out.println(language + " status-monitor stored order/text/defaults/admission/typed-errors PASS");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", (input, selector) -> LoxArchive.inspectStatusMonitor(new StatusMonitorRequest(input, selector)));
    }
}
