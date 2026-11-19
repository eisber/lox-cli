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

public final class PrincipalPermissionsSmoke {
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
    private static final PrincipalKind USER = PrincipalKind.USER;
    private static final PrincipalKind GROUP = PrincipalKind.USER_GROUP;
    private static final byte[] MALFORMED = bytes("<C A='1' A='2'/>");

    @FunctionalInterface
    public interface Operation {
        PrincipalPermissions run(PrincipalPermissionsRequest request) throws PrincipalPermissionsException;
    }

    private enum Load { MALFORMED }
    private record Missing(PrincipalKind kind, String selector, List<String> titles) {}
    private record Ambiguity(PrincipalKind kind, String selector, List<String> uuids) {}
    private record Trial(String branch, byte[] input, String selector, PrincipalKind kind, Object expected) {}
    private record Witness(byte[] prefix, long minimum, long length) {}

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static void require(boolean value, String message) {
        if (!value) throw new AssertionError(message);
    }

    private static Object outcome(Operation operation, byte[] data, String selector, PrincipalKind kind) {
        try {
            return operation.run(new PrincipalPermissionsRequest(data, selector, kind));
        } catch (PrincipalPermissionsException failure) {
            return failure;
        }
    }

    private static PrincipalPermissions expected(
            PrincipalKind kind, String title, String uuid, DirectPermission... permissions) {
        return new PrincipalPermissions(new PermissionPrincipal(kind, title, uuid), List.of(permissions));
    }

    private static void check(Object actual, Object expected) {
        boolean matches = switch (expected) {
            case PrincipalPermissions value -> value.equals(actual);
            case Load ignored -> actual instanceof PrincipalPermissionsException.Load failure
                    && "Configuration load failed".equals(failure.message())
                    && failure.sourceMessage() != null && !failure.sourceMessage().isEmpty();
            case Missing missing -> actual instanceof PrincipalPermissionsException.NotFound failure
                    && failure.kind() == missing.kind() && failure.selector().equals(missing.selector())
                    && failure.titles().equals(missing.titles());
            case Ambiguity ambiguity -> actual instanceof PrincipalPermissionsException.Ambiguous failure
                    && failure.kind() == ambiguity.kind() && failure.selector().equals(ambiguity.selector())
                    && failure.uuids().equals(ambiguity.uuids());
            default -> false;
        };
        require(matches, "Unexpected direct-permissions value/error: "
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
        boolean error = !(trial.expected() instanceof PrincipalPermissions);
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment status = UniffiRustCallStatus.allocate(arena);
            MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_principal_permissions(
                    arena, FfiConverterTypePrincipalPermissionsRequest.INSTANCE.lower(
                            new PrincipalPermissionsRequest(trial.input(), trial.selector(), trial.kind())), status);
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
        byte[] xml = bytes("<!--principal-permissions-rejected-selector-input:" + "q".repeat(65536)
                + "--><Root/>");
        long minimum = 4L + xml.length;
        var control = FfiConverterTypePrincipalPermissionsRequest.INSTANCE.lower(
                new PrincipalPermissionsRequest(xml, "valid", USER));
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
                operation.run(new PrincipalPermissionsRequest(xml, "\ud800", USER));
                throw new AssertionError("Expected strict host selector-encoding refusal before calling Core");
            } catch (RuntimeException failure) {
                if (!(failure.getCause() instanceof MalformedInputException cause)
                        || cause.getInputLength() != 1) throw failure;
            }
        }
        require(matching(prefix, minimum) == 0, "Rejected selector retained a marked XML request buffer");
        System.out.println(language + " principal-permissions invalid selector: input_live=1 input_free=0 "
                + "rejections=64 input_native_retained=0 GC_requested=false");
    }

    public static void verify(String language, Operation operation) throws Throwable {
        require(Runtime.version().feature() == 22 && "amd64".equalsIgnoreCase(System.getProperty("os.arch"))
                && System.getProperty("os.name").startsWith("Windows"), "Windows AMD64 Java 22 required");
        require(System.getenv("RUST_MIN_STACK") == null, "Stock native stack required");
        byte[] xml = Files.readAllBytes(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "principal_permissions.xml"));
        byte[] original = xml.clone();
        var user = expected(USER, "Sämple & 灯", "user-01",
                new DirectPermission("target", "Erste & Küche", "Button", "0007", "opaque & 01"),
                new DirectPermission("target", "Erste & Küche", "Button", "", ""),
                new DirectPermission("Target", null, null, "text", null),
                new DirectPermission("", "", "", null, null),
                new DirectPermission(null, "", "", null, null));
        for (String selector : List.of("SÄMPLE & 灯", "uuid:user-01")) {
            check(outcome(operation, xml, selector, USER), user);
        }
        check(outcome(operation, xml, "sämple & 灯", GROUP), expected(GROUP, "Sämple & 灯", "group-01",
                new DirectPermission("target", "Erste & Küche", "Button", "opaque", "01")));
        check(outcome(operation, xml, "Empty", USER), expected(USER, "Empty", ""));
        check(outcome(operation, xml, "uuid:titleless", USER), expected(USER, "", "titleless"));
        List<String> titles = List.of("Duplicate", "Empty", "Sämple & 灯", "duplicate");
        check(outcome(operation, xml, "uuid:USER-01", USER), new Missing(USER, "uuid:USER-01", titles));
        check(outcome(operation, xml, "Sämple", USER), new Missing(USER, "Sämple", titles));
        check(outcome(operation, bytes("<Root/>"), "Sämple", GROUP), new Missing(GROUP, "Sämple", List.of()));
        check(outcome(operation, xml, "dUpLiCaTe", USER), new Ambiguity(USER, "dUpLiCaTe", List.of("z", "", "z")));
        check(outcome(operation, xml, "uuid:z", USER), new Ambiguity(USER, "uuid:z", List.of("z", "z")));
        check(outcome(operation, MALFORMED, "missing", GROUP), Load.MALFORMED);
        require(Arrays.equals(xml, original), "Caller bytes changed");

        String marker = "principal-direct-success-Καλημέρα-" + "x".repeat(65536);
        String missing = "principal-missing-selector-" + "y".repeat(65536);
        String longUuid = "principal-ambiguous-uuid-" + "z".repeat(65536);
        for (Trial trial : List.of(
                new Trial("success", bytes("<C Type=\"User\" U=\"owner\" Title=\"" + marker + "\">"
                        + "<PH><PRM REF=\"unresolved\" ACT=\"0002\" TIM=\"\"/></PH></C>"), "uuid:owner",
                        USER, expected(USER, marker, "owner",
                                new DirectPermission("unresolved", null, null, "0002", ""))),
                new Trial("load", MALFORMED, "missing", GROUP, Load.MALFORMED),
                new Trial("not-found", xml, missing, GROUP, new Missing(GROUP, missing, List.of("Sämple & 灯"))),
                new Trial("ambiguous", bytes("<Root><C Type=\"User\" Title=\"Same\" U=\"" + longUuid
                        + "\"/><C Type=\"User\" Title=\"Same\"/></Root>"),
                        "sAmE", USER, new Ambiguity(USER, "sAmE", List.of(longUuid, ""))))) {
            Witness witness = calibrate(trial);
            byte[] input = trial.input().clone();
            Set<Object> retained = Collections.newSetFromMap(new IdentityHashMap<>());
            for (int i = 0; i < 64; i++) {
                retained.add(outcome(operation, input, trial.selector(), trial.kind()));
            }
            require(retained.size() == 64, "Expected 64 distinct host-owned values/errors");
            require(Arrays.equals(input, trial.input()), "Caller bytes changed");
            Arrays.fill(input, (byte) 0);
            require(matching(witness.prefix(), witness.minimum()) == 0, trial.branch() + " native RustBuffer leak");
            for (Object value : retained) check(value, trial.expected());
            if (trial.expected() == Load.MALFORMED) {
                require(retained.stream().map(value -> ((PrincipalPermissionsException.Load) value).sourceMessage())
                        .distinct().count() == 1, "Retained load cause changed");
            }
            System.out.println(language + " principal-permissions branch=" + trial.branch() + " actual_bytes="
                    + witness.length() + " live=1 free=0 calls=64 host_retained=64 native_retained=0 GC_requested=false");
        }
        rejectedSelectorReleasesInput(language, operation);
        System.out.println(language + " principal-permissions raw/order/optional/typed-errors PASS");
    }

    public static void main(String[] args) throws Throwable {
        verify("Java", LoxArchive::principalPermissions);
    }
}
