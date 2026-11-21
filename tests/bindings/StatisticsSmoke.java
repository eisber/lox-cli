import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;
import net.eisber.lox.archive.*;

public final class StatisticsSmoke {
    public static void require(boolean value) {
        if (!value) throw new AssertionError("Monthly statistics contract");
    }

    public static void verify(StatSeries series) {
        require(series.name().equals("Café & 気") && series.group() == 4294967295L);
        require(series.outputs().equals(List.of("Energy & power", "V2", "Energy & power")));
        require(series.rows().equals(List.of(
                new StatRow(" arbitrary & timestamp ", List.of(-0.0, 20.0, 3.0)),
                new StatRow("earlier", List.of(1.0, 2.0, 3.0)),
                new StatRow("earlier", List.of(4.0, 5.0, 6.0)))));
        require(Double.doubleToRawLongBits(series.rows().get(0).values().get(0)) == Long.MIN_VALUE);
    }

    public static void errorsAndRecovery(String xml) throws StatisticsException {
        String[] bad = {"<Statistics></secret>", "<Statistics broken/>",
                "<Statistics Name=\"&secret;\"/>", "<Statistics>", "<Statistics/><Statistics/>",
                "", "<S/>", "<Statistics StatsGroup=\"secret\"/>",
                "<Statistics StatsGroup=\"4294967296\"/>", "<Statistics NumOutputs=\"-1\"/>",
                "<Statistics><S/>", "<Statistics><S T=\"secret\"/>",
                "<Statistics><S T=\"secret\" V=\"secret\"/>",
                "<Statistics><S T=\"secret\" V=\"NaN\"/>"};
        Class<?>[] variants = {StatisticsException.Xml.class, StatisticsException.MalformedAttribute.class,
                StatisticsException.AttributeValue.class, StatisticsException.TruncatedXml.class,
                StatisticsException.MultipleStatistics.class, StatisticsException.MissingStatistics.class,
                StatisticsException.RowBeforeStatistics.class, StatisticsException.InvalidStatsGroup.class,
                StatisticsException.InvalidStatsGroup.class, StatisticsException.InvalidNumOutputs.class,
                StatisticsException.MissingTimestamp.class, StatisticsException.MissingValue.class,
                StatisticsException.InvalidValue.class, StatisticsException.NonFiniteValue.class};
        for (int i = 0; i < bad.length; i++) {
            try {
                LoxArchive.parseStatisticsSeries(bad[i]);
                throw new AssertionError("Expected atomic statistics error");
            } catch (StatisticsException failure) {
                require(failure.getClass() == variants[i] && !failure.toString().contains("secret"));
            }
            verify(LoxArchive.parseStatisticsSeries(xml));
        }
    }

    public static void main(String[] args) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null);
        String xml = Files.readString(Path.of(args[0]));
        verify(LoxArchive.parseStatisticsSeries(xml));
        StatSeries empty = LoxArchive.parseStatisticsSeries("<Statistics NumOutputs=\"0\"/>");
        require(empty.name().isEmpty() && empty.group() == null
                && empty.outputs().isEmpty() && empty.rows().isEmpty());
        StatSeries zero = LoxArchive.parseStatisticsSeries(
                "<Statistics StatsGroup=\"0\" NumOutputs=\"0\"><S T=\"\"/></Statistics>");
        require(zero.group() == 0L && zero.rows().equals(List.of(new StatRow("", List.of()))));
        require(LoxArchive.parseStatisticsSeries("<Statistics/>").outputs().equals(List.of("V")));
        errorsAndRecovery(xml);
        NativeStatisticsProbe.verify("Java", LoxArchive::parseStatisticsSeries);
        System.out.println("Java monthly statistics nested/opaque/group/empty/errors/recovery PASS");
    }
}
