import net.eisber.lox.archive.*

fun main() {
    val input = listOf(StatEntry("opaque", "RAW\u0000uuid", -1, "month", "Heat", "label"))
    val owned: String = LoxArchive.resolveStatisticsIdentity(input, "HEAT")
    check(owned == "RAW\u0000uuid")
    try {
        LoxArchive.resolveStatisticsIdentity(input, "missing")
        error("Typed not-found required")
    } catch (failure: StatisticResolutionException.NotFound) {
        check(failure.names() == listOf("Heat"))
    }
    val operation = NativeStatisticsIdentityProbe.Operation { entries, query ->
        LoxArchive.resolveStatisticsIdentity(entries, query)
    }
    StatisticsIdentitySmoke.values(operation)
    NativeStatisticsIdentityProbe.verify("Kotlin", operation)
    check(owned == "RAW\u0000uuid")
    println("Kotlin SAME generated Java statistics-identity API typed/raw/owned PASS")
}
