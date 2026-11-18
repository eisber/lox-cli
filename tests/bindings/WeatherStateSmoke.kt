import net.eisber.lox.archive.*

fun main() {
    check(System.getenv("RUST_MIN_STACK") == null)
    WeatherStateSmoke.functional()
    val states = LoxArchive.decodeWeatherStatePayload(WeatherStateSmoke.payload(1))
    WeatherStateSmoke.verify(states)
    check(states[0].lastUpdate().toUInt() == 0xffffffffu)
    check(states[0].entries()[0].perceivedTemperatureBits().toULong() == 0x8000000000000000uL)
    check(states[0].entries()[0].barometricPressureBits().toULong() == 0xfff8123456789abcuL)
    try {
        LoxArchive.decodeWeatherStatePayload(byteArrayOf(1))
        error("Expected typed atomic failure")
    } catch (failure: WeatherStatePayloadException.TruncatedHeader) {
        check(failure.recordOffset() == 0L && failure.available() == 1L)
    }
    NativeWeatherStateProbe.verify("Kotlin", LoxArchive::decodeWeatherStatePayload)
    println("Kotlin SAME Java API Weather unsigned/signed/sixIEEEbits/errors/ownership PASS")
}
