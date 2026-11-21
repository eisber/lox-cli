import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import net.eisber.lox.archive.*;

public final class StatisticsMonthMergeSmoke {
    private static void require(boolean value) {
        if (!value) throw new AssertionError("Core month merge contract");
    }

    private static StatMonthPart part(String month, String name, Integer group,
                                      List<String> outputs, StatRow... rows) {
        return new StatMonthPart(month, new StatSeries(name, group, outputs, List.of(rows)));
    }

    public static void values(NativeStatisticsMonthMergeProbe.Operation operation) throws StatisticsMergeException {
        StatSeries empty = operation.run(List.of());
        require(empty.name().isEmpty() && empty.group() == null && empty.outputs().isEmpty()
                && empty.rows().isEmpty());
        List<String> labels = List.of(" watts ", "", " watts ");
        String stamp = " \0e\u0301\r\n ";
        long[] witness = {Long.MIN_VALUE, 1, 0x7ff0000000000000L, 0xfff0000000000000L,
                0x7ff8000000001234L, 0xfff8000000005678L};
        double[] input = Arrays.stream(witness).mapToDouble(Double::longBitsToDouble).toArray();
        List<String> inputLabels = new ArrayList<>(labels);
        List<StatMonthPart> parts = new ArrayList<>(List.of(
                part("z\0", "first \0", -1, inputLabels, new StatRow(stamp, input),
                        new StatRow(stamp, new double[0])),
                part("", "ignored", 0, labels, new StatRow(stamp, new double[] {99.0}),
                        new StatRow("", new double[] {2.0}), new StatRow("", new double[] {3.0})),
                part("z\0", "", null, labels, new StatRow("", new double[] {99.0}),
                        new StatRow("a", new double[] {4.0, 5.0}))));
        StatSeries merged = operation.run(parts);
        input[0] = 99.0;
        inputLabels.clear();
        parts.clear();
        require(merged.name().equals("first \0") && merged.group() == -1 && merged.outputs().equals(labels));
        List<StatRow> rows = merged.rows();
        require(rows.stream().map(StatRow::timestamp).toList().equals(List.of(stamp, stamp, "", "", "a")));
        double[] output = rows.getFirst().values();
        require(Arrays.equals(Arrays.stream(output).mapToLong(Double::doubleToRawLongBits).toArray(), witness));
        require(rows.get(1).values().length == 0);
        require(Arrays.equals(rows.get(2).values(), new double[] {2.0}));
        require(Arrays.equals(rows.get(3).values(), new double[] {3.0}));
        require(Arrays.equals(rows.get(4).values(), new double[] {4.0, 5.0}));

        List<String> different = List.of("", " watts ", " watts ");
        try {
            operation.run(List.of(
                    part("first", "raw \0name", 0, labels, new StatRow("x", new double[0])),
                    part(" empty \0", "ignored", null, labels),
                    part("bad\0 ", "also ignored", null, different, new StatRow("x", new double[0]))));
            throw new AssertionError("Exact output-vector mismatch required");
        } catch (StatisticsMergeException.OutputMismatch failure) {
            require(failure.name().equals("raw \0name")
                    && failure.previousMonth().equals(" empty \0") && failure.month().equals("bad\0 ")
                    && failure.previousOutputs().equals(labels) && failure.outputs().equals(different));
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::mergeStatisticsMonths);
        NativeStatisticsMonthMergeProbe.verify("Java", LoxArchive::mergeStatisticsMonths);
        System.out.println("Java statistics-month-merge opaque order/rows/float bits/exact typed errors/ownership PASS");
    }
}
