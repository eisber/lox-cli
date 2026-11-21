import java.nio.file.Files
import java.nio.file.Path
import net.eisber.lox.archive.*

private fun bytes(value: String) = value.toByteArray(Charsets.UTF_8)

fun main(args: Array<String>) {
    check(System.getenv("RUST_MIN_STACK") == null)
    val xml = Files.readAllBytes(Path.of(args[0]))
    val before = xml.clone()
    val rooms = LoxArchive.parseRooms(xml)
    val summary = LoxArchive.parseConfigSummary(xml)
    check(xml.contentEquals(before))
    check(rooms.map { it.name() } == listOf("", "A&amp; Küche &#x41;", "Ignored", "a", "same", "same"))
    check(rooms.map { it.itemCount() } == listOf(0L, 4L, 0L, 2L, 0L, 0L))
    check(rooms.take(4).map { it.uuid() } == listOf("", "r", "invalid-attr", "z"))
    check(rooms.drop(4).map { it.uuid() }.toSet() == setOf("tie1", "tie2"))
    check(summary.version() == "16&amp;raw" && summary.date() == "2026&amp;raw")
    check(summary.rooms() == mapOf("" to "", "r" to "A&amp; Küche &#x41;", "z" to "a",
        "tie1" to "same", "tie2" to "same", "invalid-attr" to "Ignored"))
    check(summary.categories() == mapOf("cat" to "Raw &amp; &#65;", "" to ""))
    check(summary.users() == listOf("Zoë", "", "Zoë"))
    check(summary.controls() == mapOf(
        "case" to ConfigControlEntry("Case matters", "place", ""),
        "dup" to ConfigControlEntry("Last &amp; raw", "FutureBlock", "r"),
        "nested" to ConfigControlEntry("Nested", "And", "r"),
        "" to ConfigControlEntry("Empty type and ID", "", ""),
        "orphan" to ConfigControlEntry("No direct metadata", "And", "")))
    xml.fill(0)
    for (empty in listOf("", "plain text")) {
        check(LoxArchive.parseRooms(bytes(empty)).isEmpty())
        check(LoxArchive.parseConfigSummary(bytes(empty)) ==
            ConfigSummary("", "", emptyMap(), emptyMap(), emptyMap(), emptyList()))
    }
    check(summary.controls()["dup"]!!.name() == "Last &amp; raw" && rooms[1].itemCount() == 4L)
    check(LoxArchive.parseRooms(bytes("<C Type=\"Place\" U=\"r\" Title=\"Open\"><IoData Pr=\"r\">"))
        .single().itemCount() == 1L)
    check(LoxArchive.parseConfigSummary(bytes("<C Type=\"And\" U=\"c\" Title=\"Open\"><IoData Pr=\"r\">"))
        .controls()["c"]!!.roomUuid() == "r")
    val reset = LoxArchive.parseConfigSummary(bytes(
        "<ControlList Version=\"old\"/><ControlList/><C Type=\"Document\" Date=\"old\"/><C Type=\"Document\"/>"))
    check(reset.version().isEmpty() && reset.date().isEmpty())
    val invalid = bytes("<C Type=\"Place\" U=\"r\" Title=\"X\"/>")
    invalid[invalid.size - 4] = 0xff.toByte()
    check(LoxArchive.parseRooms(invalid).isEmpty() && LoxArchive.parseConfigSummary(invalid).rooms().isEmpty())
    for (inventory in listOf(false, true)) {
        for (malformed in listOf("<root><secret>not-an-error-payload</root>", "</unexpected>")) {
            try {
                if (inventory) LoxArchive.parseRooms(bytes(malformed))
                else LoxArchive.parseConfigSummary(bytes(malformed))
                error("Expected XML reader failure")
            } catch (failure: ConfigurationException.Xml) {
                check(failure.sourceMessage().isNotEmpty() &&
                    !failure.sourceMessage().contains("not-an-error-payload"))
            }
        }
    }
    NativeConfigInspectionProbe.verify("Kotlin") { inventory, data ->
        if (inventory) LoxArchive.parseRooms(data) else LoxArchive.parseConfigSummary(data)
    }
    println("Kotlin same Java API: full typed fields/maps/order/raw/distinct/permissive/errors PASS")
}
