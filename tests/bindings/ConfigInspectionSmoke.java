import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Set;
import net.eisber.lox.archive.*;

public final class ConfigInspectionSmoke {
    public static void require(boolean value, String context) {
        if (!value) throw new AssertionError(context);
    }

    public static byte[] bytes(String value) { return value.getBytes(StandardCharsets.UTF_8); }

    public static void main(String[] args) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null, "native caller must use stock stack");
        byte[] xml = Files.readAllBytes(Path.of(args[0]));
        byte[] before = xml.clone();
        List<RoomInfo> rooms = LoxArchive.parseRooms(xml);
        ConfigSummary summary = LoxArchive.parseConfigSummary(xml);
        require(Arrays.equals(xml, before), "input bytes unchanged");
        require(rooms.stream().map(RoomInfo::name).toList().equals(
                List.of("", "A&amp; Küche &#x41;", "Ignored", "a", "same", "same")), "name ordering/raw");
        require(rooms.stream().map(RoomInfo::itemCount).toList().equals(
                List.of(0L, 4L, 0L, 2L, 0L, 0L)), "u64 counts anywhere/before/duplicates");
        require(rooms.subList(0, 4).stream().map(RoomInfo::uuid).toList().equals(
                List.of("", "r", "invalid-attr", "z")), "IDs");
        require(Set.copyOf(rooms.subList(4, 6).stream().map(RoomInfo::uuid).toList()).equals(
                Set.of("tie1", "tie2")), "equal-name ordering unspecified");
        require(summary.version().equals("16&amp;raw") && summary.date().equals("2026&amp;raw"),
                "last raw version/date");
        require(summary.rooms().equals(Map.of("", "", "r", "A&amp; Küche &#x41;", "z", "a",
                "tie1", "same", "tie2", "same", "invalid-attr", "Ignored")), "room map");
        require(summary.categories().equals(Map.of("cat", "Raw &amp; &#65;", "", "")), "categories");
        require(summary.users().equals(List.of("Zoë", "", "Zoë")), "users order/duplicates/empty");
        require(summary.controls().equals(Map.of(
                "case", new ConfigControlEntry("Case matters", "place", ""),
                "dup", new ConfigControlEntry("Last &amp; raw", "FutureBlock", "r"),
                "nested", new ConfigControlEntry("Nested", "And", "r"),
                "", new ConfigControlEntry("Empty type and ID", "", ""),
                "orphan", new ConfigControlEntry("No direct metadata", "And", ""))),
                "all fields/blacklist/unknown/duplicate/direct-parent metadata");
        Arrays.fill(xml, (byte) 0);
        for (String empty : List.of("", "plain text")) {
            require(LoxArchive.parseRooms(bytes(empty)).isEmpty(), "permissive empty rooms");
            require(LoxArchive.parseConfigSummary(bytes(empty)).equals(
                    new ConfigSummary("", "", Map.of(), Map.of(), Map.of(), List.of())),
                    "permissive empty summary");
        }
        require(summary.controls().get("dup").name().equals("Last &amp; raw")
                && rooms.get(1).itemCount() == 4L, "host-owned results");
        require(LoxArchive.parseRooms(bytes(
                "<C Type=\"Place\" U=\"r\" Title=\"Open\"><IoData Pr=\"r\">"))
                .get(0).itemCount() == 1L, "permissive EOF inventory");
        require(LoxArchive.parseConfigSummary(bytes(
                "<C Type=\"And\" U=\"c\" Title=\"Open\"><IoData Pr=\"r\">"))
                .controls().get("c").roomUuid().equals("r"), "permissive EOF summary");
        ConfigSummary reset = LoxArchive.parseConfigSummary(bytes(
                "<ControlList Version=\"old\"/><ControlList/><C Type=\"Document\" Date=\"old\"/>"
                + "<C Type=\"Document\"/>"));
        require(reset.version().isEmpty() && reset.date().isEmpty(), "missing metadata resets");
        byte[] invalid = bytes("<C Type=\"Place\" U=\"r\" Title=\"X\"/>");
        invalid[invalid.length - 4] = (byte) 0xff;
        require(LoxArchive.parseRooms(invalid).isEmpty()
                && LoxArchive.parseConfigSummary(invalid).rooms().isEmpty(), "invalid UTF8 attribute");
        for (boolean inventory : List.of(false, true)) {
            for (String malformed : List.of("<root><secret>not-an-error-payload</root>", "</unexpected>")) {
                try {
                    if (inventory) LoxArchive.parseRooms(bytes(malformed));
                    else LoxArchive.parseConfigSummary(bytes(malformed));
                    throw new AssertionError("Expected XML reader failure");
                } catch (ConfigurationException.Xml error) {
                    require(!error.sourceMessage().isEmpty()
                            && !error.sourceMessage().contains("not-an-error-payload"), "typed source");
                }
            }
        }
        NativeConfigInspectionProbe.verify("Java",
                (inventory, data) -> {
                    if (inventory) return LoxArchive.parseRooms(data);
                    return LoxArchive.parseConfigSummary(data);
                });
        System.out.println("Java configuration: full typed fields/maps/order/raw/distinct/permissive/errors PASS");
    }
}
