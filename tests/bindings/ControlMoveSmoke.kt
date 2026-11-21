import java.nio.file.Path
import net.eisber.lox.archive.ControlMoveClock
import net.eisber.lox.archive.ControlMoveRequest
import net.eisber.lox.archive.ControlMoveSmoke
import net.eisber.lox.archive.LoxArchive

fun main(args: Array<String>) {
    ControlMoveSmoke.verify("Kotlin", Path.of(args[0])) { request ->
        val clock = request.clock()
        LoxArchive.moveControl(
            ControlMoveRequest(
                request.bytes(), request.typeFilter(), request.title(), request.targetRoom(),
                request.exclude(), ControlMoveClock(clock.unixSeconds(), clock.nanoseconds(), clock.offsetSeconds())
            )
        )
    }
}
