import net.eisber.lox.archive.LoxArchive
import net.eisber.lox.archive.SetAttributeSmoke

fun main() {
    SetAttributeSmoke.verify("Kotlin") { LoxArchive.setAttribute(it) }
}
