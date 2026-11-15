import java.nio.file.Files
import java.nio.file.Path
import net.eisber.lox.archive.*

fun main(args: Array<String>) {
    val input = Files.readAllBytes(Path.of(args[0]))
    val before = input.clone()
    val rows = LoxArchive.parseUsers(input)
    check(input.contentEquals(before))
    check(rows == listOf(
        XmlUserInfo(" raw&amp;&#65;&custom; ", "not_a_uuid", true, "one  two &#95; &amp;"),
        XmlUserInfo("", "", false, ""),
        XmlUserInfo("repeat", "same", false, " "),
        XmlUserInfo("repeat", "same", false, " "),
        XmlUserInfo("first", "", true, "a b"),
        XmlUserInfo("Grün", "raw_under", true, "Grüße 世界")))
    input.fill(0)
    check(rows[0].uuid() == "not_a_uuid")
    for (empty in listOf("", "plain text")) check(LoxArchive.parseUsers(empty.toByteArray()).isEmpty())
    check(LoxArchive.parseUsers("<C Type=\"User\" Title=\"open\">".toByteArray())[0].name() == "open")
    check(LoxArchive.parseUsers("<C Type=\"User\" Title=\"before-bad\" bad />".toByteArray())[0].name() == "before-bad")
    val invalid = ("<C Type=\"User\" Title=\"X\" U=\"valid_raw\" Desc=\"ok_value\" NFCArr=\"x\"/>" +
        "<C Type=\"User\" Title=\"valid\" U=\"X\" Desc=\"X\" NFCArr=\"X\"/>" +
        "<C Type=\"X\" Title=\"excluded\"/>" +
        "<C Type=\"User\" Title=\"X\" Title=\"not-a-fallback\"/>").toByteArray()
    for (i in invalid.indices) if (invalid[i] == 'X'.code.toByte()) invalid[i] = 0xff.toByte()
    check(LoxArchive.parseUsers(invalid) == listOf(
        XmlUserInfo("", "valid_raw", true, "ok value"),
        XmlUserInfo("valid", "", false, ""),
        XmlUserInfo("", "", false, "")))
    for (malformed in listOf(
        "<Root><C Type=\"User\" Title=\"not partial\"/><secret>not-an-error-payload</Root>",
        "</unexpected>")) {
        try {
            LoxArchive.parseUsers(malformed.toByteArray())
            error("Expected typed reader error without partial results")
        } catch (failure: ConfigurationException.Xml) {
            check(failure.sourceMessage().isNotEmpty() && !failure.sourceMessage().contains("not-an-error-payload"))
            if (malformed.startsWith("<Root>")) check(failure.sourceMessage() ==
                "ill-formed document: expected `</secret>`, but `</Root>` was found")
        }
    }
    NativeUserProbe.verify("Kotlin") { LoxArchive.parseUsers(it) }
    println("Kotlin users: SAME Java API/raw values/defaults/UTF8/order/errors/ownership PASS")
}
