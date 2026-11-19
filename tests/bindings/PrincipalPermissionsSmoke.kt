import net.eisber.lox.archive.LoxArchive
import net.eisber.lox.archive.PrincipalPermissionsSmoke

fun main() {
    PrincipalPermissionsSmoke.verify("Kotlin", LoxArchive::principalPermissions)
}
