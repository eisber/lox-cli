import net.eisber.lox.archive.*

fun main() {
    check(System.getenv("RUST_MIN_STACK") == null)
    DaytimerStateSmoke.functional()
    val states = LoxArchive.decodeDaytimerStatePayload(DaytimerStateSmoke.payload(1))
    DaytimerStateSmoke.verify(states)
    check(states[1].defaultValueBits().toULong() == 0x8000000000000000uL)
    check(states[5].entries()[1].valueBits().toULong() == 0xfff8123456789abcuL)
    try {
        LoxArchive.decodeDaytimerStatePayload(byteArrayOf(1))
        error("Expected typed atomic failure")
    } catch (failure: DaytimerStatePayloadException.TruncatedHeader) {
        check(failure.recordOffset() == 0L && failure.available() == 1L)
    }
    NativeDaytimerStateProbe.verify("Kotlin", LoxArchive::decodeDaytimerStatePayload)
    println("Kotlin SAME Java API Daytimer signed/IEEEbits/errors/ownership PASS")
}
