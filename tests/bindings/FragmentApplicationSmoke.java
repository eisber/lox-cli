package net.eisber.lox.archive;

import java.lang.foreign.*;
import java.lang.invoke.MethodHandle;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.IdentityHashMap;
import java.util.List;
import java.util.Locale;

public final class FragmentApplicationSmoke {
    private static final String A = "abcdef00-0000-0000-ffff000000000001";
    private static final String B = "fedcba00-0000-0000-ffff000000000002";
    private static final String A_NAME = A + ".LoxCC";
    private static final MethodHandle COUNT = Linker.nativeLinker().downcallHandle(
            SymbolLookup.libraryLookup(System.getProperty("lox.archive.heapProbe"), Arena.global())
                    .find("lox_test_heap_count").orElseThrow(),
            FunctionDescriptor.of(ValueLayout.JAVA_INT, ValueLayout.ADDRESS,
                    ValueLayout.JAVA_LONG, ValueLayout.JAVA_LONG, ValueLayout.ADDRESS));

    @FunctionalInterface
    public interface Operation {
        byte[] run(byte[] base, List<LoxccFragment> fragments) throws FragmentApplicationException;
    }

    private record Case(String label, byte[] base, List<LoxccFragment> fragments, byte[] expected,
                        List<FragmentApplicationErrorFrame> frames) {
        boolean error() {
            return frames != null;
        }
    }

    private static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    private static String control(String id, String kind, String extra) {
        return "<C U=\"" + id + "\" Type=\"" + kind + "\"" + extra + "/>";
    }

    private static LoxccFragment fragment(String id, String xml) {
        return new LoxccFragment(id + ".LoxCC", LoxArchive.compressLoxcc(bytes(xml)));
    }

    private static void equal(byte[] actual, byte[] expected) {
        if (!Arrays.equals(actual, expected)) throw new AssertionError("Exact bytes");
    }

    private static FragmentApplicationException.Refused refused(
            Operation operation, byte[] base, List<LoxccFragment> fragments)
            throws FragmentApplicationException {
        try {
            operation.run(base, fragments);
        } catch (FragmentApplicationException.Refused failure) {
            return failure;
        }
        throw new AssertionError("Expected atomic typed refusal");
    }

    private static void chain(FragmentApplicationException.Refused failure,
                              List<FragmentApplicationErrorFrame> expected) {
        if (!failure.frames().equals(expected)) throw new AssertionError("Exact typed frames: " + failure);
    }

    public static void values(Operation operation) throws FragmentApplicationException {
        byte[] invalid = {(byte) 0xff, '<', 0};
        equal(operation.run(invalid, List.of()), invalid);
        String page = control("page", "Page", " SpNo=\"1\"");
        String base = "\ufeff<Root>\r\n<!--keep-->" + page + "\r\n</Root>";
        String aRule = control(A, "AutopilotRule", " V=\"line\r\nbreak\"");
        String bRule = control(B, "AutopilotRule", "");
        equal(operation.run(bytes(base), List.of(fragment(B, bRule), fragment(A, aRule))),
                bytes(base.replace(page, page.substring(0, page.length() - 2)
                        + ">" + aRule + bRule + "</C>")));
        var a = fragment(A, control(A, "And", ""));
        var b = fragment(B, bRule);
        try {
            operation.run(bytes(base), List.of(b, b, a, new LoxccFragment(A_NAME, new byte[0])));
            throw new AssertionError("Duplicate exact names were collapsed");
        } catch (FragmentApplicationException.DuplicateFilename failure) {
            if (!failure.name().equals(A_NAME)) throw failure;
        }
        var alias = fragment("ABCDEF00-0000-0000-FFFF-000000000001", control(A, "And", ""));
        chain(refused(operation, bytes(base), List.of(a, alias)),
                List.of(new FragmentApplicationErrorFrame.DuplicateOverlay(A.replace("-", ""))));
        chain(refused(operation, bytes("<Root/>"), List.of(fragment(B, bRule))),
                List.of(new FragmentApplicationErrorFrame.RulePlacement(B + ".LoxCC")));
        chain(refused(operation, bytes("<Root/>"), List.of(a)),
                List.of(new FragmentApplicationErrorFrame.UnknownControl(A_NAME, "And")));
        chain(refused(operation, bytes("<Root/>"), List.of(new LoxccFragment("bad", a.data()))),
                List.of(new FragmentApplicationErrorFrame.FilenameSuffix()));
    }

    private static List<Case> ownershipCases() {
        String before = control(A, "And", " V=\"before\"");
        String after = control(A, "And", " V=\"after\"");
        var a = fragment(A, after);
        StringBuilder all = new StringBuilder("<Root>\r\n").append(before).append("\r\n");
        for (int i = 0; i < 1100; i++) {
            all.append(control(String.format(Locale.ROOT, "%08x-0000-0000-0000000000000000", i),
                    "And", " V=\"" + i + "\""));
        }
        String document = all.append("\r\n</Root>").toString();
        List<Case> cases = new ArrayList<>();
        cases.add(new Case("success", bytes(document), List.of(a),
                bytes(document.replace(before, after)), null));

        byte[] zero = Arrays.copyOf(LoxArchive.compressLoxcc(new byte[0]), 20);
        zero[16] = 0x10;
        zero[17] = 'x';
        zero[18] = zero[19] = 0;
        byte[] outOfBounds = zero.clone();
        outOfBounds[18] = 2;
        byte[][] malformed = {new byte[15], new byte[16], zero, outOfBounds};
        List<FragmentApplicationErrorFrame> causes = List.of(
                new FragmentApplicationErrorFrame.ArchiveHeaderTooShort(15L),
                new FragmentApplicationErrorFrame.ArchiveInvalidMagic(0),
                new FragmentApplicationErrorFrame.ArchiveZeroBackReference(2L),
                new FragmentApplicationErrorFrame.ArchiveBackReferenceOutOfBounds(2L, 1L));
        byte[] small = bytes("<Root>" + before + "</Root>");
        for (int i = 0; i < causes.size(); i++) {
            String name = String.format(Locale.ROOT, "%08x", 0xfedcba00L + i) + B.substring(8) + ".LoxCC";
            cases.add(new Case("archive-" + i, small, List.of(new LoxccFragment(name, malformed[i]), a),
                    null, List.of(new FragmentApplicationErrorFrame.FragmentRead(name),
                            new FragmentApplicationErrorFrame.Archive(), causes.get(i))));
        }
        byte[] corrupt = a.data().clone();
        corrupt[12] ^= 1;
        cases.add(new Case("integrity", small, List.of(new LoxccFragment(A_NAME, corrupt)), null,
                List.of(new FragmentApplicationErrorFrame.FragmentRead(A_NAME),
                        new FragmentApplicationErrorFrame.FragmentIntegrity())));
        byte[] collisionBase = bytes("<Root>" + before + control("Keep-Me", "Or", "") + "</Root>");
        var collision = fragment(A, before.substring(0, before.length() - 2)
                + ">" + control("keep-me", "Or", "") + "</C>");
        cases.add(new Case("consistency", collisionBase, List.of(collision), null,
                List.of(new FragmentApplicationErrorFrame.UuidConsistency(A_NAME),
                        new FragmentApplicationErrorFrame.Overlay(new OverlayErrorFrame.DuplicateUuid("Keep-Me")))));
        return cases;
    }

    private static long matching(byte[] prefix, long minimum) throws Throwable {
        try (Arena arena = Arena.ofConfined()) {
            MemorySegment nativePrefix = arena.allocateFrom(ValueLayout.JAVA_BYTE, prefix);
            MemorySegment count = arena.allocate(ValueLayout.JAVA_LONG);
            int code = (int) COUNT.invokeExact(nativePrefix, (long) prefix.length, minimum, count);
            if (code != 0) throw new AssertionError("HeapWalk error: " + code);
            return count.get(ValueLayout.JAVA_LONG, 0);
        }
    }

    public static void verify(String language, Operation operation) throws Throwable {
        if (System.getenv("RUST_MIN_STACK") != null) throw new AssertionError("Stock native stack required");
        for (Case test : ownershipCases()) {
            byte[] baseline = test.base().clone();
            List<byte[]> supplied = test.fragments().stream().map(part -> part.data().clone()).toList();
            byte[] prefix;
            long minimum;
            try (Arena arena = Arena.ofConfined()) {
                MemorySegment status = UniffiRustCallStatus.allocate(arena);
                MemorySegment result = UniffiLib.uniffi_lox_archive_fn_func_merge_overlays(
                        arena, FfiConverterByteArray.INSTANCE.lower(test.base()),
                        FfiConverterSequenceTypeLoxccFragment.INSTANCE.lower(test.fragments()), status);
                if (UniffiRustCallStatus.isError(status) != test.error()) throw new AssertionError("Outcome");
                MemorySegment buffer = test.error() ? UniffiRustCallStatus.getErrorBuf(status) : result;
                long length = RustBuffer.getLen(buffer);
                if (length < (test.error() ? 60 : 64000)) throw new AssertionError("Calibration length");
                // MQTT's short-buffer shape: prefix copy shorter than the actual buffer.
                // The prefix also includes 16 bytes of the admitted filename.
                minimum = test.error() ? length : 64000;
                prefix = new byte[test.error() ? 32 : 128];
                RustBuffer.asByteBuffer(buffer).get(prefix);
                if (test.error()) {
                    String name = switch (test.frames().getFirst()) {
                        case FragmentApplicationErrorFrame.FragmentRead frame -> frame.name();
                        case FragmentApplicationErrorFrame.UuidConsistency frame -> frame.name();
                        default -> throw new AssertionError("Expected filename-bearing frame");
                    };
                    equal(Arrays.copyOfRange(prefix, 16, 32), Arrays.copyOf(bytes(name), 16));
                }
                if (matching(prefix, minimum) != 1) throw new AssertionError("Live calibration");
                if (test.error()) RustBuffer.free(buffer);
                RustBuffer.free(result);
                if (matching(prefix, minimum) != 0) throw new AssertionError("Released calibration");
            }
            var retained = new IdentityHashMap<Object, Boolean>();
            for (int i = 0; i < 64; i++) {
                Object owned = test.error() ? refused(operation, test.base(), test.fragments())
                        : operation.run(test.base(), test.fragments());
                if (retained.put(owned, true) != null) throw new AssertionError("Repeated host identity");
            }
            if (matching(prefix, minimum) != 0) throw new AssertionError("Fragment RustBuffer leak");
            for (Object owned : retained.keySet()) {
                if (test.error()) chain((FragmentApplicationException.Refused) owned, test.frames());
                else equal((byte[]) owned, test.expected());
            }
            equal(test.base(), baseline);
            for (int i = 0; i < supplied.size(); i++) equal(test.fragments().get(i).data(), supplied.get(i));
            System.out.println(language + " fragment-application " + test.label()
                    + " calibrated_live=1 calibrated_released=0 calls=64 retained_buffers=0 GC_requested=false");
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::mergeOverlays);
        verify("Java", LoxArchive::mergeOverlays);
    }
}
