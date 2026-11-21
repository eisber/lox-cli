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

public final class MoodInventorySmoke {
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
        List<MoodInfo> run(byte[] input, String selector) throws MoodInventoryException;
    }

    private enum Load { MALFORMED }
    private record Moods(List<MoodInfo> rows) {}
    private record Missing(String selector, List<String> titles) {}
    private record Ambiguity(String selector, List<MoodElementIdentity> matches) {}
    private record WrongType(String selector, String blockType) {}
    private record Trial(String branch, byte[] input, String selector, Object expected, int tag, String marker) {}
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
        } catch (MoodInventoryException failure) {
            return failure;
        }
    }

    private static void check(Object actual, Object expected) {
        boolean matches = switch (expected) {
            case Moods moods -> moods.rows().equals(actual);
            case Load ignored -> actual instanceof MoodInventoryException.Load failure
                    && !failure.sourceMessage().isEmpty();
            case Missing missing -> actual instanceof MoodInventoryException.NotFound failure
                    && failure.selector().equals(missing.selector()) && failure.titles().equals(missing.titles());
            case Ambiguity ambiguity -> actual instanceof MoodInventoryException.Ambiguous failure
                    && failure.selector().equals(ambiguity.selector()) && failure.matches().equals(ambiguity.matches());
            case WrongType wrong -> actual instanceof MoodInventoryException.NotLightController failure
                    && failure.selector().equals(wrong.selector()) && failure.blockType().equals(wrong.blockType());
            default -> false;
        };
        require(matches, "Unexpected mood-inventory value/error: "
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
        boolean error = !(trial.expected() instanceof Moods);
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_list_moods(
                    arena, FfiConverterTypeMoodInventoryRequest.INSTANCE.lower(
                            new MoodInventoryRequest(trial.input(), trial.selector())), status);
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
                require(ByteBuffer.wrap(prefix).getInt() == trial.tag(), "Wrong actual native wire tag/count");
                require(trial.marker() == null
                                || new String(prefix, StandardCharsets.ISO_8859_1).contains(trial.marker()),
                        "Expected this branch's marker in its original native buffer");
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
        byte[] xml = bytes("<!--lox-mood-inventory-rejected-selector-input:" + "m".repeat(65536) + "--><Root/>");
        long minimum = 4L + xml.length;
        var control = FfiConverterTypeMoodInventoryRequest.INSTANCE.lower(new MoodInventoryRequest(xml, "uuid:lc"));
        MemorySegment pointer = RustBuffer.getData(control);
        byte[] prefix = new byte[128];
        try {
            require(RustBuffer.getLen(control) >= minimum, "Small actual native input");
            RustBuffer.asByteBuffer(control).get(prefix);
            require(Arrays.equals(prefix, ByteBuffer.allocate(128).putInt(xml.length).put(xml, 0, 124).array()),
                    "Expected the original length-prefixed marked XML in this request buffer");
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
        require(matching(prefix, minimum) == 0, "Rejected mood selector retained marked XML");
        System.out.println(language + " mood-inventory invalid selector: input_live=1 input_free=0 "
                + "rejections=64 input_native_retained=0 GC_requested=false");
    }

    public static void verify(String language, Operation operation) throws Throwable {
        require(Runtime.version().feature() == 22 && "amd64".equalsIgnoreCase(System.getProperty("os.arch"))
                && System.getProperty("os.name").startsWith("Windows"), "Windows AMD64 Java 22 required");
        require(System.getenv("RUST_MIN_STACK") == null, "Stock native stack required");
        byte[] xml = Files.readAllBytes(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "mood_inventory.xml"));
        byte[] original = xml.clone();
        var second = new MoodInfo("Inner", "same", 2L, null, "");
        var third = new MoodInfo("", "", null, null, null);
        var fourth = new MoodInfo("", "", null, Long.MAX_VALUE, "opaque&unparsed");
        var moods = new Moods(List.of(new MoodInfo("Outer & 灯", "same", 2L, -3L, "00037"), second, third, fourth));
        for (String selector : List.of("uuid:lc", "gid:LIGHT", "Lamps [kit]")) {
            check(outcome(operation, xml, selector), moods);
        }
        check(outcome(operation, xml, "uuid:empty"), new Moods(List.of()));
        List<String> titles = List.of("Case only", "Kitchen", "Kitchen lamps", "Mixed lights");
        check(outcome(operation, xml, "uuid:LC"), new Missing("uuid:LC", titles));
        check(outcome(operation, xml, "Mixed lights"), new Ambiguity("Mixed lights", List.of(
                new MoodElementIdentity("Mixed lights", "And", "a"),
                new MoodElementIdentity("Mixed lights", "LightController2", "b"))));
        check(outcome(operation, xml, "uuid:case"), new WrongType("uuid:case", "lightcontroller2"));
        check(outcome(operation, xml, "uuid:sparse"), new WrongType("uuid:sparse", ""));
        byte[] malformed = bytes("<Root>\n  <C Duplicate='1' Duplicate='2'/>\n</Root>");
        check(outcome(operation, malformed, "missing"), Load.MALFORMED);
        require(Arrays.equals(xml, original), "Caller bytes changed");

        String q1 = "mood-inventory-owned-Q1-Καλημέρα-" + "x".repeat(65536);
        String missing = "mood-inventory-missing-" + "y".repeat(65536);
        String longTitle = "mood-inventory-matched-title-" + "z".repeat(65536);
        String wrongType = "mood-inventory-wrong-type-" + "w".repeat(65536);
        for (Trial trial : List.of(
                new Trial("moods", bytes(new String(xml, StandardCharsets.UTF_8).replace(
                        "Q1=\"00037\"", "Q1=\"" + q1 + "\"")), "uuid:lc",
                        new Moods(List.of(new MoodInfo("Outer & 灯", "same", 2L, -3L, q1), second, third, fourth)),
                        4, "mood-inventory-owned-Q1-"),
                new Trial("load", malformed, "missing", Load.MALFORMED, 1, null),
                new Trial("not-found", xml, missing, new Missing(missing, titles), 2, "mood-inventory-missing-"),
                new Trial("ambiguous", bytes("<Root><C Type=\"And\" U=\"a\" Title=\"" + longTitle
                        + "\"/><C Type=\"LightController2\" U=\"b\" Title=\"" + longTitle + "\"/></Root>"),
                        "matched-title-", new Ambiguity("matched-title-", List.of(
                                new MoodElementIdentity(longTitle, "And", "a"),
                                new MoodElementIdentity(longTitle, "LightController2", "b"))),
                        3, "mood-inventory-matched-title-"),
                new Trial("not-light-controller", bytes("<C Type=\"" + wrongType + "\" U=\"w\"/>"),
                        "uuid:w", new WrongType("uuid:w", wrongType), 4, "mood-inventory-wrong-type-"))) {
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
                require(retained.stream().map(value -> ((MoodInventoryException.Load) value).sourceMessage())
                        .distinct().count() == 1, "Retained load cause changed");
            }
            if (trial.expected() instanceof Moods) {
                Set<Object> firstRows = Collections.newSetFromMap(new IdentityHashMap<>());
                for (Object value : retained) firstRows.add(((List<?>) value).getFirst());
                require(firstRows.size() == 64, "Expected distinct host-owned mood records");
            }
            System.out.println(language + " mood-inventory branch=" + trial.branch() + " actual_bytes="
                    + witness.length() + " live=1 free=0 calls=64 host_retained=64 native_retained=0 GC_requested=false");
        }

        rejectedSelectorReleasesInput(language, operation);
        System.out.println(language + " mood-inventory order/defaults/duplicates/empty/typed-errors PASS");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", (input, selector) -> LoxArchive.listMoods(new MoodInventoryRequest(input, selector)));
    }
}
