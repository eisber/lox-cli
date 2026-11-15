import net.eisber.lox.archive.*

fun main() {
    LoxArchive.hashAppCredentialsHex("us:ér", "päss:word", "001122aaccee").use {
        check(it.toHex() == "88bfb35cc1a3ce183b6fcd66b3b5fe3acbdaba620c3ce041d866ce6d055e2bb2")
        check(!it.toString().contains(it.toHex()))
        it.dispose()
        it.dispose()
        try {
            it.toHex()
            error("Disposed hash readable")
        } catch (failure: AuthenticationHashException.Closed) {
            check(failure.message() == "authentication hash is closed")
        }
    }
    AuthHashSmoke.verify()
    println("Kotlin same generated Java API auth PASS")
}
