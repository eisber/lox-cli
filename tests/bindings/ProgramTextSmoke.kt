import net.eisber.lox.archive.LoxArchive
import net.eisber.lox.archive.ProgramTextRequest
import net.eisber.lox.archive.ProgramTextSmoke

fun main() {
    val operation = ProgramTextSmoke.Operation { input: ByteArray, selector: String ->
        LoxArchive.getProgramText(ProgramTextRequest(input, selector))
    }
    ProgramTextSmoke.verify("Kotlin", operation)
}
