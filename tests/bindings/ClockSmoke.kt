import java.nio.file.Path
import java.util.concurrent.TimeUnit
import net.eisber.lox.archive.ClockClient
import net.eisber.lox.archive.ClockEndpoint
import net.eisber.lox.archive.ClockRuntimeException
import net.eisber.lox.archive.ClockValidationIssue
import net.eisber.lox.archive.NativeClockStatusProbe
import java.nio.file.Files

fun main(args: Array<String>) {
    val host = args[0]
    val signals = Path.of(args[1])
    fun client(mode: String) = ClockClient(
        "$host/kotlin/$mode", ClockTestSupport.USER, ClockTestSupport.PASSWORD,
        ClockTestSupport.settings(mode == "retry"),
    )
    for (mode in listOf("normal", "number")) {
        val connection = client(mode)
        connection.use {
            val reading = it.readClock().get(5, TimeUnit.SECONDS)
            check(reading.date() == if (mode == "normal") "2026-11-15" else "42")
            check(reading.time() == if (mode == "normal") "09:10:11" else "12.5")
            if (mode == "normal") check(it.readClock().get(5, TimeUnit.SECONDS) == reading)
        }
        connection.close()
        ClockTestSupport.expect(ClockRuntimeException.Closed::class.java, connection.readClock())
    }
    for (mode in listOf("json", "utf8", "missing", "status", "http")) {
        client(mode).use {
            when (mode) {
                "json", "utf8" -> check(ClockTestSupport.expect(
                    ClockRuntimeException.Decode::class.java, it.readClock(),
                ).endpoint() == ClockEndpoint.DATE)
                "http" -> check(ClockTestSupport.expect(
                    ClockRuntimeException.Http::class.java, it.readClock(),
                ).status().toInt() == 401)
                else -> check(ClockTestSupport.expect(
                    ClockRuntimeException.Validation::class.java, it.readClock(),
                ).issue() == if (mode == "missing") {
                    ClockValidationIssue.MISSING_VALUE
                } else {
                    ClockValidationIssue.REJECTED_CODE
                })
            }
        }
        println("Kotlin typed clock error: $mode")
    }
    for (mode in listOf("blocked-date", "blocked-time", "retry")) {
        client(mode).use {
            val future = it.readClock()
            ClockTestSupport.waitSignal(
                signals, "kotlin/$mode", if (mode == "retry") "retry-response" else "entered",
            )
            ClockTestSupport.expect(ClockRuntimeException.Busy::class.java, it.readClock())
            check(future.cancel(false) && future.isCancelled)
            if (mode != "retry") ClockTestSupport.waitSignal(signals, "kotlin/$mode", "eof")
            ClockTestSupport.waitFor { it.activeOperations() == 0L }
            check(!it.isClosed())
            println("Kotlin future cancellation -> Rust drain: $mode")
        }
    }
    for (mode in listOf("close-date", "close-time")) {
        val connection = client(mode)
        val future = connection.readClock()
        ClockTestSupport.waitSignal(signals, "kotlin/$mode", "entered")
        connection.close()
        ClockTestSupport.expect(ClockRuntimeException.CancelledShutdown::class.java, future)
        ClockTestSupport.waitSignal(signals, "kotlin/$mode", "eof")
        connection.close()
        ClockTestSupport.expect(ClockRuntimeException.Closed::class.java, connection.readClock())
        println("Kotlin shared shutdown/socket EOF/idempotence: $mode")
    }
    val allocations = NativeClockStatusProbe()
    for (i in 0 until 16) {
        val scenario = "kotlin-status/$i/callback"
        ClockClient(
            "$host/$scenario", ClockTestSupport.USER, ClockTestSupport.PASSWORD,
            ClockTestSupport.settings(false),
        ).use {
            val read = it.readClock()
            ClockTestSupport.waitSignal(signals, scenario, "entered")
            val captured = read.thenRun { allocations.capture() }
            Files.writeString(ClockTestSupport.signal(signals, scenario, "release"), "release")
            captured.get(5, TimeUnit.SECONDS)
        }
        allocations.afterClose()
    }
    allocations.verify("Kotlin")
    println("Kotlin same Java clock API/futures/explicit disposal: PASS")
}
