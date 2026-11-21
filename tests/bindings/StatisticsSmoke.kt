import java.nio.file.Files
import java.nio.file.Path
import net.eisber.lox.archive.*

fun main(args: Array<String>) {
    check(System.getenv("RUST_MIN_STACK") == null)
    val xml = Files.readString(Path.of(args[0]))
    val series = LoxArchive.parseStatisticsSeries(xml)
    check(series.name() == "Café & 気" && series.group() == 4294967295L)
    check(series.outputs() == listOf("Energy & power", "V2", "Energy & power"))
    check(series.rows().map { it.timestamp() } ==
        listOf(" arbitrary & timestamp ", "earlier", "earlier"))
    check(series.rows().map { it.values() } ==
        listOf(listOf(-0.0, 20.0, 3.0), listOf(1.0, 2.0, 3.0), listOf(4.0, 5.0, 6.0)))
    check(series.rows()[0].values()[0].toRawBits() == Long.MIN_VALUE)
    val empty = LoxArchive.parseStatisticsSeries("<Statistics NumOutputs=\"0\"/>")
    check(empty.name() == "" && empty.group() == null &&
        empty.outputs().isEmpty() && empty.rows().isEmpty())
    val zero = LoxArchive.parseStatisticsSeries(
        "<Statistics StatsGroup=\"0\" NumOutputs=\"0\"><S T=\"\"/></Statistics>")
    check(zero.group() == 0L && zero.rows() == listOf(StatRow("", listOf())))
    check(LoxArchive.parseStatisticsSeries("<Statistics/>").outputs() == listOf("V"))
    val failures = listOf(
        "<Statistics></secret>" to StatisticsException.Xml::class.java,
        "<Statistics broken/>" to StatisticsException.MalformedAttribute::class.java,
        "<Statistics Name=\"&secret;\"/>" to StatisticsException.AttributeValue::class.java,
        "<Statistics>" to StatisticsException.TruncatedXml::class.java,
        "<Statistics/><Statistics/>" to StatisticsException.MultipleStatistics::class.java,
        "" to StatisticsException.MissingStatistics::class.java,
        "<S/>" to StatisticsException.RowBeforeStatistics::class.java,
        "<Statistics StatsGroup=\"secret\"/>" to StatisticsException.InvalidStatsGroup::class.java,
        "<Statistics StatsGroup=\"4294967296\"/>" to StatisticsException.InvalidStatsGroup::class.java,
        "<Statistics NumOutputs=\"-1\"/>" to StatisticsException.InvalidNumOutputs::class.java,
        "<Statistics><S/>" to StatisticsException.MissingTimestamp::class.java,
        "<Statistics><S T=\"secret\"/>" to StatisticsException.MissingValue::class.java,
        "<Statistics><S T=\"secret\" V=\"secret\"/>" to StatisticsException.InvalidValue::class.java,
        "<Statistics><S T=\"secret\" V=\"NaN\"/>" to StatisticsException.NonFiniteValue::class.java)
    for ((bad, variant) in failures) {
        try {
            LoxArchive.parseStatisticsSeries(bad)
            error("Expected atomic statistics error")
        } catch (failure: StatisticsException) {
            check(failure.javaClass == variant && !failure.toString().contains("secret"))
        }
        StatisticsSmoke.verify(LoxArchive.parseStatisticsSeries(xml))
    }
    NativeStatisticsProbe.verify("Kotlin", LoxArchive::parseStatisticsSeries)
    println("Kotlin same Java API monthly statistics nested/opaque/group/empty/errors/recovery PASS")
}
