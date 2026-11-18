import net.eisber.lox.archive.*

fun main() {
    val input = doubleArrayOf(Double.fromBits(0x7ff8000000001234L), -0.0)
    val series = StatSeries("opaque", null, emptyList(), listOf(StatRow("", input)))
    val merged: StatSeries = LoxArchive.mergeStatisticsMonths(listOf(StatMonthPart("", series)))
    val owned: DoubleArray = merged.rows().single().values()
    check(owned[0].toRawBits() == 0x7ff8000000001234L && owned[1].toRawBits() == Long.MIN_VALUE)
    val operation = NativeStatisticsMonthMergeProbe.Operation { parts ->
        LoxArchive.mergeStatisticsMonths(parts)
    }
    StatisticsMonthMergeSmoke.values(operation)
    NativeStatisticsMonthMergeProbe.verify("Kotlin", operation)
    println("Kotlin SAME generated Java statistics-month-merge API float bits/exact typed errors/ownership PASS")
}
