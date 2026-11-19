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
import java.util.Objects;
import java.util.Set;

public final class CentralLightLinksSmoke {
    private static final String OWNER = "ABCDEF01-0002-0003-0000000000000001";
    private static final String FIRST = "F00BABE1-0002-0003-000000000000000A";
    private static final String FIRST_LOWER = "f00babe1-0002-0003-000000000000000a";
    private static final String SECOND = "f00babe2-0002-0003-000000000000000b";
    private static final String MISSING = "f00babe3-0002-0003-000000000000000c";
    private static final String TITLE = "Zentrale & 灯";
    private static final String SELECT = "gid:primary";
    private static final List<String> TITLES = List.of("Den", "Empty", "Küche & status", TITLE);
    private static final List<CentralLightLink> LINKS = List.of(
            new CentralLightLink(1L, FIRST, "", ""),
            new CentralLightLink(2L, SECOND, "Küche & status", "Not"));
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
        CentralLightLinks run(CentralLightLinksRequest request) throws CentralLightLinksException;
    }

    private enum Load { MALFORMED }
    private record Missing(String selector, List<String> titles) {}
    private record Ambiguity(String selector, List<CentralLightElementIdentity> matches) {}
    private record WrongType(String selector) {}
    private record InvalidIdentity(String uuid) {}
    private record Duplicate(String uuid) {}
    private record Unresolved(String uuid) {}
    private record Flags(String value, CentralLightFlagsCause kind) {}
    private record Marker(String identity) {}
    private record Trial(byte[] data, String selector, Object expected, boolean large) {}
    private record Witness(byte[] prefix, long minimum, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static void require(boolean value, String message) {
        if (!value) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, byte[] data, String selector) {
        try {
            return operation.run(new CentralLightLinksRequest(data, selector));
        } catch (CentralLightLinksException failure) {
            return failure;
        }
    }

    private static void check(Object actual, Object expected) {
        boolean matches = switch (expected) {
            case CentralLightLinks result -> result.equals(actual);
            case Load ignored -> actual instanceof CentralLightLinksException.Load failure
                    && !failure.sourceMessage().isEmpty();
            case Missing missing -> actual instanceof CentralLightLinksException.NotFound failure
                    && failure.selector().equals(missing.selector()) && failure.titles().equals(missing.titles());
            case Ambiguity ambiguity -> actual instanceof CentralLightLinksException.Ambiguous failure
                    && failure.selector().equals(ambiguity.selector()) && failure.matches().equals(ambiguity.matches());
            case WrongType wrong -> actual instanceof CentralLightLinksException.NotPlainCentralLight failure
                    && failure.selector().equals(wrong.selector());
            case InvalidIdentity invalid -> actual instanceof CentralLightLinksException.InvalidIdentity failure
                    && failure.uuid().equals(invalid.uuid());
            case Duplicate duplicate -> actual instanceof CentralLightLinksException.DuplicateReference failure
                    && failure.uuid().equals(duplicate.uuid());
            case Unresolved unresolved -> actual instanceof CentralLightLinksException.UnresolvedControl failure
                    && failure.uuid().equals(unresolved.uuid());
            case Flags flags -> actual instanceof CentralLightLinksException.InvalidFlags failure
                    && failure.value().equals(flags.value()) && failure.kind() == flags.kind()
                    && !failure.sourceMessage().isEmpty();
            case Marker marker -> actual instanceof CentralLightLinksException.InconsistentMarker failure
                    && Objects.equals(failure.identity(), marker.identity());
            case Class<?> error -> error.isInstance(actual);
            default -> false;
        };
        require(matches, "Unexpected CentralLight result/error: "
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
        boolean error = !(trial.expected() instanceof CentralLightLinks);
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_inspect_central_light_links(
                    arena, FfiConverterTypeCentralLightLinksRequest.INSTANCE.lower(
                            new CentralLightLinksRequest(trial.data(), trial.selector())), status);
            int code = UniffiRustCallStatus.getCode(status);
            MemorySegment buffer = code != 0 ? UniffiRustCallStatus.getErrorBuf(status) : result;
            long length = RustBuffer.getLen(buffer);
            byte[] prefix = new byte[trial.large() ? 128 : 16];
            long minimum = trial.large() ? 64000 : length;
            MemorySegment pointer = RustBuffer.getData(buffer);
            try {
                require(code == (error ? 1 : 0), "Unexpected native status " + code);
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

    private static void rejectedSelectorReleasesInput(String language, Operation operation, String xml)
            throws Throwable {
        byte[] marked = bytes("<!--central-light-links-" + language + "-rejected-input:"
                + "q".repeat(65536) + "-->" + xml);
        long minimum = 4L + marked.length;
        var control = FfiConverterTypeCentralLightLinksRequest.INSTANCE.lower(
                new CentralLightLinksRequest(marked, SELECT));
        MemorySegment pointer = RustBuffer.getData(control);
        byte[] prefix = new byte[128];
        try {
            require(RustBuffer.getLen(control) >= minimum, "Small actual native input");
            RustBuffer.asByteBuffer(control).get(prefix);
            require(Arrays.equals(prefix, ByteBuffer.allocate(128)
                            .putInt(marked.length).put(marked, 0, 124).array()),
                    "Expected the actual length-prefixed marked XML in the request buffer");
            require(live(pointer) && matching(prefix, minimum) == 1, "Live marked input calibration");
        } finally {
            RustBuffer.free(control);
        }
        require(!live(pointer) && matching(prefix, minimum) == 0, "Released marked input calibration");
        var request = new CentralLightLinksRequest(marked, "\ud800");
        for (int i = 0; i < 64; i++) {
            try {
                operation.run(request);
                throw new AssertionError("Expected strict host selector-encoding refusal");
            } catch (RuntimeException failure) {
                if (!(failure.getCause() instanceof MalformedInputException cause)
                        || cause.getInputLength() != 1) throw failure;
            }
        }
        require(matching(prefix, minimum) == 0, "Rejected selector retained a marked native input");
        System.out.println(language + " central-light-links invalid selector: input_live=1 input_free=0 "
                + "rejections=64 input_native_retained=0 GC_requested=false");
    }

    public static void verify(String language, Operation operation) throws Throwable {
        require(Runtime.version().feature() == 22 && System.getProperty("os.name").startsWith("Windows")
                && System.getProperty("os.arch").equalsIgnoreCase("amd64"), "Windows x64 Java 22 required");
        require(System.getenv("RUST_MIN_STACK") == null, "Stock native stack required");
        String xml = Files.readString(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "central_light_links.xml"));
        byte[] original = bytes(xml);
        byte[] data = original.clone();
        for (String selector : List.of("uuid:" + OWNER, SELECT, "Zentrale & 灯 [dEn]")) {
            check(outcome(operation, data, selector), new CentralLightLinks(OWNER, TITLE, LINKS));
        }
        check(outcome(operation, data, "Empty"), new CentralLightLinks(
                "abcdef01-0002-0003-0000000000000002", "Empty", List.of()));
        String lowerSelector = "uuid:abcdef01-0002-0003-0000000000000001";
        check(outcome(operation, data, lowerSelector), new Missing(lowerSelector, TITLES));
        check(outcome(operation, data, "uuid:ordinary"), new WrongType("uuid:ordinary"));
        require(Arrays.equals(original, data), "Caller bytes changed");

        check(outcome(operation, bytes(xml.replace("</ControlList>", "<C Type=\"Document\"/></ControlList>")),
                SELECT), CentralLightLinksException.NoSingleOwningDocument.class);
        check(outcome(operation, bytes(xml.replace("<C Type=\"Document\">",
                        "<x:C Type=\"Document\" xmlns:x=\"urn:test\">")
                .replace("</C></ControlList>", "</x:C></ControlList>")), SELECT),
                CentralLightLinksException.NotPlainDocument.class);
        check(outcome(operation, bytes(xml.replace("Type=\"Document\">", "Type=\"Document\"><Template/>")),
                SELECT), CentralLightLinksException.DocumentTemplate.class);
        check(outcome(operation, bytes(xml.replace("Visu=\"true\" Pr=\"den\"", "Visu=\"True\" Pr=\"den\"")),
                SELECT), CentralLightLinksException.NotDirectVisibleOwner.class);
        check(outcome(operation, bytes(xml.replace(" U=\"" + OWNER + "\"", "")), SELECT),
                CentralLightLinksException.MissingOwnerIdentity.class);
        check(outcome(operation, bytes(xml.replace("</C></ControlList>",
                "<C WF=\"268435456\"/></C></ControlList>")), SELECT), new Marker(null));
        check(outcome(operation, bytes(xml.replace("268451840", "")), SELECT),
                new Flags("", CentralLightFlagsCause.EMPTY));
        check(outcome(operation, bytes(xml.replace("268451840", "4294967296")), SELECT),
                new Flags("4294967296", CentralLightFlagsCause.POSITIVE_OVERFLOW));

        String marker = "links-marker-Καλημέρα-" + "x".repeat(65536);
        String query = "links-absent-" + "y".repeat(65536);
        String invalidId = "links-invalid-id-" + "z".repeat(65536);
        String invalidFlags = "links-invalid-flags-" + "f".repeat(65536);
        String wrongSelector = "uuid:" + marker;
        String longXml = xml.replace("Zentrale &amp; 灯", marker);
        String ambiguousXml = longXml.replace("U=\"ordinary\"", "U=\"ordinary\" Title=\"" + marker + "\"");
        String reference = "," + SECOND + "\"";
        for (Trial trial : List.of(
                new Trial(bytes(longXml), SELECT, new CentralLightLinks(OWNER, marker, LINKS), true),
                new Trial(bytes("<C A='1' A='2'/>"), SELECT, Load.MALFORMED, false),
                new Trial(original.clone(), query, new Missing(query, TITLES), true),
                new Trial(bytes(ambiguousXml), "links-marker-", new Ambiguity("links-marker-", List.of(
                        new CentralLightElementIdentity(marker, "CentralLight", OWNER),
                        new CentralLightElementIdentity(marker, "Other", "ordinary"))), true),
                new Trial(bytes("<C Type=\"Other\" U=\"" + marker + "\"/>"), wrongSelector,
                        new WrongType(wrongSelector), true),
                new Trial(bytes(xml.replace(reference, "," + invalidId + "\"")), SELECT,
                        new InvalidIdentity(invalidId), true),
                new Trial(bytes(xml.replace(reference, "," + FIRST_LOWER + "\"")), SELECT,
                        new Duplicate(FIRST_LOWER), false),
                new Trial(bytes(xml.replace(reference, ",F00BABE3-0002-0003-000000000000000C\"")), SELECT,
                        new Unresolved(MISSING), false),
                new Trial(bytes(xml.replace("268451840", invalidFlags)), SELECT,
                        new Flags(invalidFlags, CentralLightFlagsCause.INVALID_DIGIT), true),
                new Trial(bytes(xml.replace("268451840", "16384")), SELECT, new Marker(SECOND), false))) {
            Witness witness = calibrate(trial);
            Set<Object> retained = Collections.newSetFromMap(new IdentityHashMap<>());
            for (int i = 0; i < 64; i++) {
                retained.add(outcome(operation, trial.data(), trial.selector()));
            }
            require(retained.size() == 64, "Expected 64 distinct host-owned values/errors");
            String branch = retained.iterator().next().getClass().getSimpleName();
            Arrays.fill(trial.data(), (byte) 0);
            require(matching(witness.prefix(), witness.minimum()) == 0, branch + " native RustBuffer leak");
            for (Object value : retained) check(value, trial.expected());
            if (trial.expected() == Load.MALFORMED) {
                require(retained.stream().map(value -> ((CentralLightLinksException.Load) value).sourceMessage())
                        .distinct().count() == 1, "Retained load cause changed");
            } else if (trial.expected() instanceof Flags) {
                require(retained.stream().map(value -> ((CentralLightLinksException.InvalidFlags) value).sourceMessage())
                        .distinct().count() == 1, "Retained parse cause changed");
            }
            System.out.println(language + " central-light-links branch=" + branch + " actual_bytes="
                    + witness.length() + " live=1 free=0 calls=64 host_retained=64 native_retained=0 GC_requested=false");
        }
        rejectedSelectorReleasesInput(language, operation, xml);
        System.out.println(language + " central-light-links stored order/empty/typed errors PASS");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", LoxArchive::inspectCentralLightLinks);
    }
}
