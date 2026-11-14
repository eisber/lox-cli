import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.Arrays;
import java.util.HexFormat;
import java.util.function.Consumer;
import net.eisber.lox.archive.ArchiveException;
import net.eisber.lox.archive.LoxArchive;

public final class ArchiveSmoke {
    @FunctionalInterface
    interface Operation {
        byte[] run() throws ArchiveException;
    }

    static void require(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }

    static <T extends ArchiveException> void expect(
            Class<T> type, Operation operation, Consumer<T> context) throws ArchiveException {
        try {
            operation.run();
        } catch (ArchiveException error) {
            if (!type.isInstance(error)) throw error;
            context.accept(type.cast(error));
            System.out.println("Java typed error: " + error.getClass().getSimpleName()
                    + "(" + error.getMessage() + ")");
            return;
        }
        throw new AssertionError("Expected " + type.getName());
    }

    public static void main(String[] args) throws Exception {
        Path fixtures = Path.of(args[0]);
        byte[] raw = Files.readAllBytes(fixtures.resolve("raw.bin"));
        byte[] encoded = Files.readAllBytes(fixtures.resolve("encoded.bin"));
        require(Arrays.equals(LoxArchive.compressLoxcc(raw), encoded), "cross-language encoding");
        require(Arrays.equals(LoxArchive.decompressLoxcc(encoded), raw), "raw roundtrip");
        require(Arrays.equals(LoxArchive.extractAndDecompress(encoded), raw), "raw auto-detection");
        require(Arrays.equals(LoxArchive.extractAndDecompress(
                Files.readAllBytes(fixtures.resolve("backup.zip"))), raw), "ZIP selection");
        require(Arrays.equals(LoxArchive.decompressLoxcc(
                Files.readAllBytes(fixtures.resolve("permissive.bin"))), raw), "permissive header");
        byte[] large = new byte[256 * 1024];
        for (int i = 0; i < large.length; i++) large[i] = (byte) i;
        for (byte[] input : new byte[][] {new byte[0], large}) {
            require(Arrays.equals(LoxArchive.decompressLoxcc(LoxArchive.compressLoxcc(input)),
                    input), "empty/large byte roundtrip");
        }
        System.out.println("Java byte[] roundtrip: " + raw.length + " bytes sha256="
                + HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(raw)));
        expect(ArchiveException.HeaderTooShort.class, () -> LoxArchive.decompressLoxcc(new byte[3]),
                error -> require(error.actual() == 3L, "header actual"));
        expect(ArchiveException.InvalidMagic.class, () -> LoxArchive.decompressLoxcc(new byte[16]),
                error -> require(error.actual() == 0, "magic actual"));
        byte[] zero = Files.readAllBytes(fixtures.resolve("zero-offset.bin"));
        expect(ArchiveException.ZeroBackReference.class, () -> LoxArchive.decompressLoxcc(zero),
                error -> require(error.position() == 1L, "back-reference position"));
        byte[] offset = Files.readAllBytes(fixtures.resolve("bad-offset.bin"));
        expect(ArchiveException.BackReferenceOutOfBounds.class,
                () -> LoxArchive.decompressLoxcc(offset),
                error -> require(error.offset() == 1L && error.outputLen() == 0L, "offset context"));
        expect(ArchiveException.InvalidZip.class, () -> LoxArchive.extractAndDecompress(new byte[3]),
                error -> require(!error.sourceMessage().isEmpty(), "ZIP source text"));
        byte[] empty = Files.readAllBytes(fixtures.resolve("empty.zip"));
        expect(ArchiveException.MissingConfigEntry.class,
                () -> LoxArchive.extractAndDecompress(empty), error -> {});
        byte[] badCrc = Files.readAllBytes(fixtures.resolve("badcrc.zip"));
        expect(ArchiveException.ReadZipEntry.class, () -> LoxArchive.extractAndDecompress(badCrc),
                error -> require(error.index() == 0L && error.name().equals("sps0.LoxCC")
                        && !error.sourceMessage().isEmpty(), "ZIP read context"));
    }
}
