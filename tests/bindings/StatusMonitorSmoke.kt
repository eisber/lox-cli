import net.eisber.lox.archive.LoxArchive
import net.eisber.lox.archive.StatusMonitorRequest
import net.eisber.lox.archive.StatusMonitorSmoke

fun main() {
    val operation = StatusMonitorSmoke.Operation { input: ByteArray, selector: String ->
        LoxArchive.inspectStatusMonitor(StatusMonitorRequest(input, selector))
    }
    StatusMonitorSmoke.verify("Kotlin", operation)
}
