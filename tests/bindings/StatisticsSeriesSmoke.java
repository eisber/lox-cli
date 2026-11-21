import net.eisber.lox.archive.*;
import java.nio.file.*;
import java.util.*;

public final class StatisticsSeriesSmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("Statistics series values");
    }

    static void refuses(NativeStatisticsSeriesProbe.Operation operation, String xml,
                        Class<? extends StatisticsException> variant) throws Throwable {
        try {
            operation.run(xml);
        } catch (StatisticsException failure) {
            require(variant.isInstance(failure));
            if (failure instanceof StatisticsException.Xml value) {
                require(!value.sourceMessage().isEmpty());
            } else if (failure instanceof StatisticsException.MalformedAttribute value) {
                require(value.attribute().equals("Name") && !value.sourceMessage().isEmpty());
            } else if (failure instanceof StatisticsException.AttributeValue value) {
                require(value.attribute().equals("Name") && !value.sourceMessage().isEmpty());
            } else if (failure instanceof StatisticsException.InvalidStatsGroup value) {
                require(value.value().equals("4294967296") && !value.sourceMessage().isEmpty());
            } else if (failure instanceof StatisticsException.InvalidNumOutputs value) {
                require(value.value().equals("-1") && !value.sourceMessage().isEmpty());
            } else if (failure instanceof StatisticsException.MissingValue value) {
                require(value.timestamp().equals("later & raw") && value.attribute().equals("V2"));
            } else if (failure instanceof StatisticsException.InvalidValue value) {
                require(value.timestamp().equals("later") && value.attribute().equals("V")
                        && value.value().equals(" warm ")
                        && value.sourceMessage().equals("invalid float literal"));
            }
            return;
        }
        throw new AssertionError("Expected atomic typed " + variant);
    }

    public static void values(NativeStatisticsSeriesProbe.Operation operation) throws Throwable {
        String input = Files.readString(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "statistics_series.xml"));
        StatSeries value = operation.run(input);
        require(value.name().equals("Café & 気") && value.group() != null
                && Integer.toUnsignedLong(value.group()) == 4294967295L);
        require(value.outputs().equals(List.of("Energy & power", "V2", "A", "A")));
        List<StatRow> rows = value.rows();
        require(rows.size() == 3 && rows.getFirst().timestamp().equals(" opaque & timestamp ")
                && rows.get(1).timestamp().equals("duplicate") && rows.get(2).timestamp().equals("duplicate"));
        long[] bits = {Long.MIN_VALUE, 1L, Long.MIN_VALUE | 1L, 0x7fefffffffffffffL};
        List<Double> numbers = rows.getFirst().values();
        for (int i = 0; i < bits.length; i++) {
            require(Double.doubleToRawLongBits(numbers.get(i)) == bits[i]);
        }
        require(rows.get(1).values().subList(0, 2).equals(List.of(1.0, 20.0)));
        require(Double.doubleToRawLongBits(rows.get(1).values().get(2)) == 0);
        require(Double.doubleToRawLongBits(rows.get(1).values().get(3)) == Long.MIN_VALUE);
        require(rows.get(2).values().equals(List.of(4.0, 5.0, 6.0, 7.0)));
        input = "replaced";
        operation.run("<Statistics/>");
        require(value.name().equals("Café & 気") && numbers.getFirst().equals(-0.0));
        require(operation.run("<Statistics/>").outputs().equals(List.of("V")));
        require(operation.run("<Statistics Outputs=\"\"/>").outputs().equals(List.of("V")));
        StatSeries zero = operation.run(
                "<Statistics NumOutputs=\"0\" Outputs=\"ignored\"><S T=\"\"/></Statistics>");
        require(zero.group() == null && zero.outputs().isEmpty()
                && zero.rows().getFirst().timestamp().isEmpty() && zero.rows().getFirst().values().isEmpty());
        StatSeries lazy = operation.run(
                "<Statistics Name=\"n\" StatsGroup=\"0\" Outputs=\"A\" NumOutputs=\"1\" broken/>"
                + "<S T=\"outside\" V=\"2\" broken/>");
        require(lazy.rows().getFirst().timestamp().equals("outside")
                && lazy.rows().getFirst().values().equals(List.of(2.0)));
        require(operation.run("<Statistics Other=\"&unknown;\"/><Ignored broken/>").rows().isEmpty());
        require(operation.run("<Statistics StatsGroup=\"0\"/>").group() == 0);
        refuses(operation, "", StatisticsException.MissingStatistics.class);
        refuses(operation, "<S/>", StatisticsException.RowBeforeStatistics.class);
        refuses(operation, "<Statistics/><Statistics StatsGroup=\"bad\"/>", StatisticsException.MultipleStatistics.class);
        refuses(operation, "<Statistics><S T=\"valid\" V=\"1\"/>", StatisticsException.TruncatedXml.class);
        refuses(operation, "<Statistics><S/>", StatisticsException.MissingTimestamp.class);
        refuses(operation, "<Statistics></Other>", StatisticsException.Xml.class);
        refuses(operation, "<Statistics broken/>", StatisticsException.MalformedAttribute.class);
        refuses(operation, "<Statistics Name=\"&unknown;\"/>", StatisticsException.AttributeValue.class);
        refuses(operation, "<Statistics StatsGroup=\"4294967296\"/>", StatisticsException.InvalidStatsGroup.class);
        refuses(operation, "<Statistics NumOutputs=\"-1\"/>", StatisticsException.InvalidNumOutputs.class);
        refuses(operation, "<Statistics NumOutputs=\"2\"><S T=\"first\" V=\"1\" V2=\"2\"/>"
                + "<S T=\"later &amp; raw\" V=\"3\"/></Statistics>", StatisticsException.MissingValue.class);
        refuses(operation, "<Statistics><S T=\"first\" V=\"1\"/>"
                + "<S T=\"later\" V=\" warm \"/></Statistics>", StatisticsException.InvalidValue.class);
        for (String raw : List.of("NaN", "inf", "-inf", "1e9999")) {
            try {
                operation.run("<Statistics><S T=\"first\" V=\"1\"/><S T=\"later\" V=\"" + raw + "\"/></Statistics>");
                throw new AssertionError("Expected Core nonfinite refusal");
            } catch (StatisticsException.NonFiniteValue failure) {
                require(failure.timestamp().equals("later") && failure.attribute().equals("V")
                        && failure.value().equals(raw));
            }
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::parseStatisticsSeries);
        NativeStatisticsSeriesProbe.verify("Java", LoxArchive::parseStatisticsSeries);
        System.out.println("Java monthly XML typed rows/finite bits/opaque time/atomic errors/causes/ownership PASS");
    }
}
