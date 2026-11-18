import net.eisber.lox.archive.*

fun main() {
    val value: StatSeries = LoxArchive.parseStatisticsSeries(
        "<Statistics StatsGroup=\"4294967295\"><S T=\"opaque\" V=\"-0\"/></Statistics>")
    val row: StatRow = value.rows().single()
    val numbers: List<Double> = row.values()
    check(value.group()!!.toUInt() == UInt.MAX_VALUE && row.timestamp() == "opaque")
    check(java.lang.Double.doubleToRawLongBits(numbers.single()) == Long.MIN_VALUE)
    try {
        LoxArchive.parseStatisticsSeries("<Statistics><S T=\"x\" V=\"inf\"/></Statistics>")
        error("Expected Core nonfinite refusal")
    } catch (failure: StatisticsException.NonFiniteValue) {
        check(failure.timestamp() == "x" && failure.attribute() == "V" && failure.value() == "inf")
    }
    val operation = NativeStatisticsSeriesProbe.Operation { xml -> LoxArchive.parseStatisticsSeries(xml) }
    StatisticsSeriesSmoke.values(operation)
    NativeStatisticsSeriesProbe.verify("Kotlin", operation)
    println("Kotlin SAME generated Java monthly XML typed rows/errors/causes/ownership PASS")
}
