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
import java.util.Set;

public final class ProgramTextSmoke {
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

    @FunctionalInterface
    public interface Operation {
        String run(byte[] input, String selector) throws ProgramTextException;
    }

    private enum Load { MALFORMED }
    private record Missing(String selector, List<String> titles) {}
    private record Ambiguity(String selector, List<ProgramElementIdentity> matches) {}
    private record Trial(String branch, byte[] input, String selector, Object expected) {}
    private record Witness(byte[] prefix, long minimum, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static void require(boolean value, String message) {
        if (!value) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, byte[] data, String selector) {
        try {
            return operation.run(data, selector);
        } catch (ProgramTextException failure) {
            return failure;
        }
    }

    private static void check(Object actual, Object expected) {
        boolean matches = switch (expected) {
            case String text -> text.equals(actual);
            case Load ignored -> actual instanceof ProgramTextException.Load failure
                    && !failure.sourceMessage().isEmpty();
            case Missing missing -> actual instanceof ProgramTextException.NotFound failure
                    && failure.selector().equals(missing.selector()) && failure.titles().equals(missing.titles());
            case Ambiguity ambiguity -> actual instanceof ProgramTextException.Ambiguous failure
                    && failure.selector().equals(ambiguity.selector()) && failure.matches().equals(ambiguity.matches());
            case ProgramElementIdentity element -> actual instanceof ProgramTextException.Unsupported failure
                    && failure.element().equals(element);
            default -> false;
        };
        require(matches, "Unexpected program-text value/error: "
                + (actual == null ? "null" : actual.getClass().getName()));
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
        boolean error = !(trial.expected() instanceof String);
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_get_program_text(
                    arena, FfiConverterTypeProgramTextRequest.INSTANCE.lower(
                            new ProgramTextRequest(trial.input(), trial.selector())), status);
            require(UniffiRustCallStatus.getCode(status) == (error ? 1 : 0), "Unexpected native status");
            MemorySegment buffer = error ? UniffiRustCallStatus.getErrorBuf(status) : result;
            long length = RustBuffer.getLen(buffer);
            boolean small = trial.expected() == Load.MALFORMED;
            byte[] prefix = new byte[small ? 16 : 128];
            long minimum = small ? length : 64000;
            MemorySegment pointer = RustBuffer.getData(buffer);
            try {
                require(length >= Math.max(2L * prefix.length, minimum), "Small actual native result");
                RustBuffer.asByteBuffer(buffer).get(prefix);
                require(live(pointer) && matching(prefix, minimum) == 1, "Live result calibration");
            } finally {
                if (error) RustBuffer.free(buffer);
                RustBuffer.free(result);
            }
            require(!live(pointer) && matching(prefix, minimum) == 0, "Released result calibration");
            return new Witness(prefix, minimum, length);
        }
    }

    private static void rejectedSelectorReleasesInput(String language, Operation operation) throws Throwable {
        byte[] xml = bytes("<!--program-text-rejected-selector-input:" + "q".repeat(65536) + "--><Root/>");
        long minimum = 4L + xml.length;
        var control = FfiConverterTypeProgramTextRequest.INSTANCE.lower(new ProgramTextRequest(xml, "uuid:s"));
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
        System.out.println(language + " program-text invalid selector: input_live=1 input_free=0 "
                + "rejections=64 input_native_retained=0 GC_requested=false");
    }

    public static void verify(String language, Operation operation) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null, "Stock native stack required");
        byte[] xml = Files.readAllBytes(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "program_text.xml"));
        byte[] original = xml.clone();
        String expectedText = "first & Küche\ne\u0301secondthird 灯  ";
        for (String selector : List.of("uuid:sequence", "gid:cook", "Cook & 灯 [Kitchen]")) {
            check(outcome(operation, xml, selector), expectedText);
        }
        for (String selector : List.of("Empty", "Blank")) {
            check(outcome(operation, xml, selector), "");
        }
        check(outcome(operation, xml, "uuid:legacy"), "text _12hTF=value");
        List<String> titles = List.of("Blank", "Cook & 灯", "Empty", "Kitchen", "Legacy _12hTF=x");
        check(outcome(operation, xml, "missing"), new Missing("missing", titles));
        var wrong = new ProgramElementIdentity("Cook & 灯", "sequencecontroller", "wrong");
        check(outcome(operation, xml, "Cook & 灯"), new Ambiguity("Cook & 灯", List.of(
                new ProgramElementIdentity("Cook & 灯", "SequenceController", "sequence"), wrong)));
        check(outcome(operation, xml, "uuid:wrong"), wrong);
        check(outcome(operation, xml, "uuid:sparse"), new ProgramElementIdentity("", "", "sparse"));
        byte[] malformed = bytes("<C A='1' A='2'/>");
        check(outcome(operation, malformed, "missing"), Load.MALFORMED);
        require(Arrays.equals(xml, original), "Caller bytes changed");

        String marker = "program-result-Καλημέρα-" + "x".repeat(65536);
        String query = "missing-program-" + "y".repeat(65536);
        String longTitle = "program-element-" + "z".repeat(65536);
        var longWrong = new ProgramElementIdentity(longTitle, "sequencecontroller", "w");
        for (Trial trial : List.of(
                new Trial("text", bytes("<C Type=\"SequenceController\" U=\"s\"><Field Name=\"Configuration\">"
                        + marker + "</Field></C>"), "uuid:s", marker),
                new Trial("load", malformed, "missing", Load.MALFORMED),
                new Trial("not-found", xml, query, new Missing(query, titles)),
                new Trial("ambiguous", bytes("<Root><C Type=\"sequencecontroller\" U=\"w\" Title=\"" + longTitle
                        + "\"/><C Type=\"SequenceController\" U=\"s\" Title=\"" + longTitle + "\"/></Root>"),
                        "program-element-", new Ambiguity("program-element-",
                                List.of(longWrong, new ProgramElementIdentity(longTitle, "SequenceController", "s")))),
                new Trial("unsupported", bytes("<C Type=\"sequencecontroller\" U=\"w\" Title=\""
                        + longTitle + "\"/>"), "uuid:w", longWrong))) {
            Witness witness = calibrate(trial);
            Set<Object> retained = Collections.newSetFromMap(new IdentityHashMap<>());
            for (int i = 0; i < 64; i++) {
                retained.add(outcome(operation, trial.input(), trial.selector()));
            }
            require(retained.size() == 64, "Expected 64 distinct host-owned values/errors");
            Arrays.fill(trial.input(), (byte) 0);
            require(matching(witness.prefix(), witness.minimum()) == 0, trial.branch() + " native RustBuffer leak");
            for (Object value : retained) check(value, trial.expected());
            if (trial.expected() == Load.MALFORMED) {
                require(retained.stream().map(value -> ((ProgramTextException.Load) value).sourceMessage())
                        .distinct().count() == 1, "Retained load cause changed");
            }
            System.out.println(language + " program-text branch=" + trial.branch() + " actual_bytes="
                    + witness.length() + " live=1 free=0 calls=64 host_retained=64 native_retained=0 GC_requested=false");
        }
        rejectedSelectorReleasesInput(language, operation);
        System.out.println(language + " program-text exact/empty/shared-loader/typed-errors PASS");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", (input, selector) -> LoxArchive.getProgramText(new ProgramTextRequest(input, selector)));
    }
}
