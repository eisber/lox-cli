import net.eisber.lox.archive.LoxArchive
import net.eisber.lox.archive.UnwireSmoke

fun main() {
    UnwireSmoke.verify("Kotlin") { LoxArchive.unwire(it) }
}
