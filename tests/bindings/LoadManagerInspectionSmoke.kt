import java.nio.file.Path
import net.eisber.lox.archive.LoadManagerInspectionSmoke
import net.eisber.lox.archive.LoxArchive

fun main(args: Array<String>) {
    val operation = LoadManagerInspectionSmoke.Operation { request ->
        LoxArchive.inspectLoadManager(request)
    }
    LoadManagerInspectionSmoke.verify("Kotlin", Path.of(args[0]), operation)
}
