import net.eisber.lox.archive.DaytimerRemovalSmoke
import net.eisber.lox.archive.LoxArchive

fun main() {
    DaytimerRemovalSmoke.verify("Kotlin") { LoxArchive.removeDaytimerEntry(it) }
}
