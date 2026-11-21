import java.nio.file.Files
import java.nio.file.Path
import net.eisber.lox.archive.*

fun main(args: Array<String>) {
    val xml = Files.readAllBytes(Path.of(args[0]))
    val before = xml.clone()
    val rows = LoxArchive.parseControls(xml, null, null)
    check(xml.contentEquals(before))
    check(rows == listOf(
        XmlControlInfo("", "", "", "Default", "Empty ID"),
        XmlControlInfo("Other", "Inner", "inner", "KÜCHE", "Raw &amp; &#65;"),
        XmlControlInfo("Page", "Page", "page", "page-room", "Empty ID"),
        XmlControlInfo("Switch", "Pending", "pending", "KÜCHE", "Raw &amp; &#65;"),
        XmlControlInfo("Switch", "Same", "second", "KÜCHE", "Raw &amp; &#65;"),
        XmlControlInfo("Switch", "Same", "first", "KÜCHE", "Empty ID"),
        XmlControlInfo("Switch", "Same", "first", "raw&amp;id", "unknown"),
        XmlControlInfo("TreeDevice", "Device", "device", "Default", "Empty ID"),
        XmlControlInfo("switch", "Z", "lower", "Default", "Empty ID"),
        XmlControlInfo("Ä", " A&amp;&#65;&custom; ", "室", "Default", "Empty ID")))
    check(LoxArchive.parseControls(xml, "sWiTcH", "küc").map { it.uuid() } ==
        listOf("outer", "pending", "second", "first"))
    check(LoxArchive.parseControls(xml, null, "raw&amp;").single().category() == "unknown")
    check(LoxArchive.parseControls(xml, "", null).size == 1)
    check(LoxArchive.parseControls(xml, null, "") == rows)
    for (typ in listOf("ä", "Sw", "absent")) check(LoxArchive.parseControls(xml, typ, null).isEmpty())
    xml.fill(0)
    for (empty in listOf("", "plain text")) {
        check(LoxArchive.parseControls(empty.toByteArray(), null, null).isEmpty())
    }
    check(rows[1].title() == "Inner")
    check(LoxArchive.parseControls(
        "<C Type=\"Switch\" U=\"c\" Title=\"Open\"><IoData Pr=\"r\">".toByteArray(), null, null) ==
        listOf(XmlControlInfo("Switch", "Open", "c", "r", "")))
    check(LoxArchive.parseControls(
        "<C Type=\"Switch\" U=\"c\" Title=\"\u00ff\"/><IoData/>".toByteArray().let {
            val marker = it.indexOf(0xc3.toByte())
            it.copyOfRange(0, marker) + byteArrayOf(0xff.toByte()) + it.copyOfRange(marker + 2, it.size)
        }, null, null).isEmpty())
    for (malformed in listOf("<Root><C Type=\"Switch\" U=\"c\" Title=\"ok\"/>" +
            "<IoData/><secret>not-an-error-payload</Root>", "</unexpected>")) {
        try {
            LoxArchive.parseControls(malformed.toByteArray(), null, null)
            error("Expected typed reader failure, not partial success")
        } catch (failure: ConfigurationException.Xml) {
            check(failure.sourceMessage().isNotEmpty() &&
                !failure.sourceMessage().contains("not-an-error-payload"))
        }
    }
    NativeConfigInspectionProbe.verifyControls("Kotlin") { input, typ, room ->
        LoxArchive.parseControls(input, typ, room)
    }
    println("Kotlin shared Java API: XML controls raw fields/filters/association/errors/ownership PASS")
}
