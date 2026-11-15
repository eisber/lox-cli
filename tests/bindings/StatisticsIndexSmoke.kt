import net.eisber.lox.archive.*

fun main() {
    val input = "<li><a href=\"01234567-89ab-cdef-0123456789abcdef_4294967295.202613.xml\">é é</a>"
    val rows: List<StatEntry> = LoxArchive.parseStatisticsIndex(input)
    check(rows.single().group()!!.toUInt() == UInt.MAX_VALUE)
    check(rows.single().month() == "2026-13")
    check(rows.single().name() == "é é")
    val operation = NativeStatisticsIndexProbe.Operation { html -> LoxArchive.parseStatisticsIndex(html) }
    StatisticsIndexSmoke.values(operation)
    NativeStatisticsIndexProbe.verify("Kotlin", operation)
    println("Kotlin SAME generated Java statistics index API typed values/unsigned/ownership PASS")
}
