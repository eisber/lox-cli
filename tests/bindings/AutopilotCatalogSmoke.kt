import net.eisber.lox.archive.*
import java.nio.file.Files
import java.nio.file.Path

fun main() {
    val rows: List<AutopilotRule> = LoxArchive.listLoxapp3AutopilotRules(
        Files.readAllBytes(Path.of("crates", "lox-bindings", "tests", "fixtures", "autopilot_catalog.json")))
    check(rows.last() == AutopilotRule("é é\r\n<xml>", "raw\u0000ID", "raw\u0000state", "<xml>\r\n"))
    check(rows.map { it.name() } == listOf("", "?", "?", "A", "Same", "Same", "a", "é é\r\n<xml>"))
    val operation = NativeAutopilotProbe.Operation { input ->
        LoxArchive.listLoxapp3AutopilotRules(input)
    }
    AutopilotCatalogSmoke.values(operation)
    NativeAutopilotProbe.verify("Kotlin", operation)
    println("Kotlin SAME Java autopilot API typed values/raw ordering/defaults/errors/input independence PASS")
}
