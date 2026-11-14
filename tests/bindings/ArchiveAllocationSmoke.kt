import java.nio.file.Files
import java.nio.file.Path
import net.eisber.lox.archive.LoxArchive
import net.eisber.lox.archive.NativeArchiveReleaseProbe

fun main(args: Array<String>) {
    val invalidZip = Files.readAllBytes(Path.of(args[0], "long-error.zip"))
    NativeArchiveReleaseProbe.verifyPublicErrors("Kotlin", invalidZip, 64) {
        LoxArchive.extractAndDecompress(it)
    }
}
