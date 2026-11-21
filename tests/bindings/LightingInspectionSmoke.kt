import java.nio.file.Path
import net.eisber.lox.archive.LightingInspectionSmoke
import net.eisber.lox.archive.LoxArchive

fun main(args: Array<String>) {
    LightingInspectionSmoke.verify("Kotlin", Path.of(args[0])) { LoxArchive.inspectLighting(it) }
}
