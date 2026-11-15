import net.eisber.lox.archive.*;
import java.nio.file.*;
import java.util.*;

public final class StatisticsIndexSmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("Statistics index values");
    }

    public static void values(NativeStatisticsIndexProbe.Operation operation) throws Throwable {
        String uuid = "01234567-89ab-cdef-0123456789abcdef";
        String input = Files.readString(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "statistics_index.html"));
        List<StatEntry> rows = operation.run(input);
        String roomZero = "Zero (Statistic Group: 0) (Room)";
        String roomMax = "Max (Statistic Group: 4294967295) (Room)";
        StatEntry duplicate = new StatEntry(uuid + ".202610.xml", uuid, null, "2026-10", "Dup", "Dup (Room)");
        require(rows.equals(List.of(
                new StatEntry(uuid + ".202613.xml", uuid, null, "2026-13", "Minimal", "Minimal"),
                new StatEntry(uuid + "_0.000000.xml", uuid, 0, "0000-00", "Zero", roomZero),
                new StatEntry(uuid + "_4294967295.999999.xml", uuid, -1, "9999-99", "Max", roomMax),
                duplicate, duplicate,
                new StatEntry(uuid + ".202610.xml", uuid, null, "2026-10",
                        "A &amp; B &unknown;", "A &amp; B &unknown;"),
                new StatEntry(uuid + ".202610.xml", uuid, null, "2026-10",
                        "é é\r\nline & !", "é é\r\nline & !"))));
        require(Integer.toUnsignedLong(rows.get(2).group()) == 4294967295L);
        input = "replaced";
        require(rows.getFirst().name().equals("Minimal"));
        for (String malformed : List.of("", "<LI>", "<li>broken", "\0é")) {
            require(operation.run(malformed).isEmpty());
        }
        String raw = "raw\0é é\r\nvalue";
        require(operation.run("<li><a href=\"" + uuid + ".000000.xml\">" + raw + "</a>")
                .getFirst().name().equals(raw));
        require(operation.run("<li><a href=\"" + uuid.toUpperCase(Locale.ROOT)
                + ".202610.xml\">bad</a><li><a href=\"" + uuid + ".202610.xml\">good</a>")
                .getFirst().name().equals("good"));
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::parseStatisticsIndex);
        NativeStatisticsIndexProbe.verify("Java", LoxArchive::parseStatisticsIndex);
        System.out.println("Java statistics index typed values/unsigned/order/duplicates/ownership PASS");
    }
}
