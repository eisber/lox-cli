import java.util.concurrent.Callable
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import net.eisber.lox.archive.*

private const val path = "/jdev/cfg/api"
private const val marker = "private-bootstrap-marker"

private inline fun <reified T : BootstrapException> fails(request: BootstrapRequest): T {
    try {
        LoxArchive.bootstrapPreflight(request)
    } catch (failure: BootstrapException) {
        if (failure !is T) throw failure
        check(listOf(marker, "127.0.0.1", path).none { it in failure.toString() })
        check(failure.cause == null)
        return failure
    }
    error("Expected ${T::class.java.simpleName}")
}

private fun verifyPreflight(http: String, sink: String, ws: String) {
    val direct = BootstrapProxyPolicy.DIRECT
    val request = BootstrapRequest(http + path, ws, direct)
    for ((slots, tls) in listOf(
        BootstrapEventSlots.REPORTED_AVAILABLE to BootstrapTlsStatus.REPORTED_AVAILABLE,
        BootstrapEventSlots.REPORTED_UNAVAILABLE to BootstrapTlsStatus.REPORTED_CERTIFICATE_EXPIRED,
        BootstrapEventSlots.MISSING to BootstrapTlsStatus.UNKNOWN,
        BootstrapEventSlots.UNKNOWN to BootstrapTlsStatus.MISSING,
    )) {
        val observed = LoxArchive.bootstrapPreflight(request)
        check(observed.eventSlots() == slots && observed.tlsStatus() == tls)
        check(marker !in observed.toString())
    }
    check(fails<BootstrapException.HttpStatus>(request).status().toInt() == 503)
    check(fails<BootstrapException.CommandStatus>(request).status().toInt() == 403)
    check(fails<BootstrapException.Payload>(request).issue() == BootstrapPayloadIssue.INVALID_ENVELOPE)
    fails<BootstrapException.WrongRoute>(BootstrapRequest("$sink/$marker", ws, direct))
    fails<BootstrapException.UnsupportedProxyPolicy>(BootstrapRequest(sink + path, ws, BootstrapProxyPolicy.SYSTEM))
    val invalid = fails<BootstrapException.InvalidEndpoint>(BootstrapRequest(sink + path, marker, direct))
    check(invalid.endpoint() == BootstrapEndpointKind.WEB_SOCKET && invalid.issue() == BootstrapEndpointIssue.SCHEME)
}

fun main(args: Array<String>) {
    Executors.newSingleThreadExecutor().use { worker ->
        worker.submit(Callable { verifyPreflight(args[0], args[1], args[2]) }).get(30, TimeUnit.SECONDS)
    }
    println("Kotlin same generated Java preflight PASS")
}
