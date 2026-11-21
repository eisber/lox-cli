import net.eisber.lox.archive.*

fun main() {
    val operation = NativeWiringCatalogProbe.Operation { input, selector ->
        LoxArchive.listConnectorWiring(input, selector)
    }
    NativeWiringCatalogProbe.verify("Kotlin", operation)
    println("Kotlin SAME Java selected wiring catalog API PASS")
}
