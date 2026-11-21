import java.nio.file.Path
import java.util.concurrent.CompletableFuture
import net.eisber.lox.archive.ControlReadClient
import net.eisber.lox.archive.ControlReading
import net.eisber.lox.archive.NativeControlReadProbe

fun main(args: Array<String>) {
    ControlReadTestSupport.run("kotlin", args[0], Path.of(args[1]), args[2]) { host, settings ->
        val client = ControlReadClient(
            host, ControlReadTestSupport.USER, ControlReadTestSupport.PASSWORD, settings,
        )
        object : ControlReadTestSupport.Client {
            override fun read(identifier: String): CompletableFuture<ControlReading> =
                client.readControl(identifier)
            override fun active(): Long = client.activeOperations()
            override fun closed(): Boolean = client.isClosed
            override fun close() = client.close()
        }
    }
    NativeControlReadProbe.verify("Kotlin", args[0], args[2], Path.of(args[1]))
    println("Kotlin uses the same Java22 control-read API: PASS")
}
