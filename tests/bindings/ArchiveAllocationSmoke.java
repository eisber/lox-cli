import java.nio.file.Files;
import java.nio.file.Path;
import net.eisber.lox.archive.LoxArchive;
import net.eisber.lox.archive.NativeArchiveReleaseProbe;

public final class ArchiveAllocationSmoke {
    public static void main(String[] args) throws Throwable {
        if (args.length < 2 || !args[1].equals("--alias-only")) {
            byte[] invalidZip = Files.readAllBytes(Path.of(args[0], "long-error.zip"));
            NativeArchiveReleaseProbe.verifyPublicErrors(
                    "Java", invalidZip, 64, LoxArchive::extractAndDecompress);
        }
        if (args.length < 2 || !args[1].equals("--domain-only")) {
            NativeArchiveReleaseProbe.verifyAliasedDescriptorRelease();
        }
    }
}
