import net.eisber.lox.archive.*

fun main() {
    val inputs = RuntimeOverlaySmoke.witness("c")
    try {
        LoxArchive.threeWay(inputs[0], inputs[1], inputs[2])
        error("Typed merged consistency cause required")
    } catch (failure: OverlayException.Refused) {
        check(failure.frames().size == 2)
        check(failure.frames()[0] is OverlayErrorFrame.MergedConsistency)
        val cause = failure.frames()[1] as OverlayErrorFrame.DuplicateUuid
        check(cause.id() == "c")
    }
    val operation = NativeRuntimeOverlayProbe.Operation { base, local, live ->
        LoxArchive.threeWay(base, local, live)
    }
    RuntimeOverlaySmoke.values(operation)
    NativeRuntimeOverlayProbe.verify("Kotlin", operation)
    println("Kotlin SAME Java runtime-overlay selected-bytes/typed-two-frame-causes/owned PASS")
}
