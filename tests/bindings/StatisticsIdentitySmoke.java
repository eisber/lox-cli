import java.util.ArrayList;
import java.util.List;
import net.eisber.lox.archive.*;

public final class StatisticsIdentitySmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("Core identity contract");
    }

    static StatEntry row(String name, String uuid, Integer group) {
        return new StatEntry("not-an-identity.xml", uuid, group, "opaque-month", name, "not-an-identity-label");
    }

    public static void values(NativeStatisticsIdentityProbe.Operation operation) throws Throwable {
        List<StatEntry> rows = new ArrayList<>(List.of(row("Heat", "winner", -1),
                row("winner", "other", null), row("Heat Pump", "third", null), row("Heat", "winner", 0)));
        List<StatEntry> before = List.copyOf(rows);
        require(operation.run(rows, "WINNER").equals("winner"));
        require(operation.run(rows, "hEaT").equals("winner"));
        require(operation.run(rows, "pump").equals("third") && rows.equals(before));
        require(operation.run(List.of(row("unrelated", "", null), row("", "other", null)), "").isEmpty());
        require(operation.run(List.of(row("a", "same", null), row("b", "same", 1)), "").equals("same"));
        String raw = "RAW\0é e\u0301\r\nuuid";
        require(operation.run(List.of(row("İSTANBUL", raw, null)), "İstanbul").equals(raw));
        for (String query : List.of("ABC", "Istanbul", "cafe\u0301", "Café ", "not-an-identity.xml",
                "not-an-identity-label", "opaque-month")) {
            try {
                operation.run(List.of(row("İSTANBUL", "ABC", null), row("Café", "x", null)), query);
                throw new AssertionError("No normalization, trimming, stored-UUID folding or metadata lookup");
            } catch (StatisticResolutionException.NotFound error) {
                require(error.names().equals(List.of("Café", "İSTANBUL")));
            }
        }
        List<StatEntry> diagnosticRows = new ArrayList<>(List.of(row("Same", "z", -1),
                row("Same", "z", null), row("Same", "a", 0), row("Same", "z", -1), row("same", "a", null)));
        List<StatisticResolutionException> retained = new ArrayList<>();
        try {
            operation.run(diagnosticRows, "SAME");
            throw new AssertionError("Raw UUID ambiguity");
        } catch (StatisticResolutionException.Ambiguous error) {
            require(error.distinctUuids() == 2);
            require(error.candidates().equals(List.of(new StatisticIdentity("Same", null, "z"),
                    new StatisticIdentity("Same", 0, "a"), new StatisticIdentity("Same", -1, "z"),
                    new StatisticIdentity("same", null, "a"))));
            retained.add(error);
        }
        try {
            operation.run(diagnosticRows, "missing");
            throw new AssertionError("Typed not-found");
        } catch (StatisticResolutionException.NotFound error) {
            require(error.names().equals(List.of("Same", "same")));
            retained.add(error);
        }
        diagnosticRows.clear();
        require(((StatisticResolutionException.Ambiguous) retained.get(0)).candidates().get(2).group() == -1);
        require(((StatisticResolutionException.NotFound) retained.get(1)).names().equals(List.of("Same", "same")));
        try {
            operation.run(List.of(), "");
            throw new AssertionError("Empty inventory is not-found");
        } catch (StatisticResolutionException.NotFound error) {
            require(error.names().isEmpty());
        }
        try {
            operation.run(List.of(row("\ud800\udc00", "a", null), row("\ue000", "b", null)), "");
            throw new AssertionError("Rust scalar ordering, not Java UTF-16 ordering");
        } catch (StatisticResolutionException.Ambiguous error) {
            require(error.candidates().stream().map(StatisticIdentity::name).toList()
                    .equals(List.of("\ue000", "\ud800\udc00")));
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::resolveStatisticsIdentity);
        NativeStatisticsIdentityProbe.verify("Java", LoxArchive::resolveStatisticsIdentity);
        System.out.println("Java statistics-identity raw/precedence/typed-owned-diagnostics PASS");
    }
}
