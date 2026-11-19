import net.eisber.lox.archive.CentralLightLinksSmoke
import net.eisber.lox.archive.LoxArchive

fun main() {
    CentralLightLinksSmoke.verify("Kotlin", LoxArchive::inspectCentralLightLinks)
}
