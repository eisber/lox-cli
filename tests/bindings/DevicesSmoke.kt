import java.nio.file.Files
import java.nio.file.Path
import net.eisber.lox.archive.*

fun main(args: Array<String>) {
    val input = Files.readAllBytes(Path.of(args[0]))
    val before = input.clone()
    val rows = LoxArchive.parseDevices(input)
    check(input.contentEquals(before))
    check(rows.size == 11)
    check(rows[0] == XmlDeviceInfo(" raw&amp;&#65;&custom; ", XmlDeviceBus.TREE, "", 4294967295L,
        "Unknown (4294967295)", null, null))
    check(rows[1].bus() == XmlDeviceBus.AIR && rows[1].serial() == null)
    check(rows[2] == XmlDeviceInfo("same", XmlDeviceBus.NETWORK, null, 4294967295L,
        "Network device", "", "raw&amp;"))
    check(rows[3].subType() == null)
    check(rows[4] == rows[5])
    check(rows[6].subType() == null)
    check(rows[7].typeLabel() == "Presence sensor")
    check(rows[8].typeLabel() == "Network device")
    check(rows[9].subType() == 7L)
    check(rows[10].name() == "before-bad")
    input.fill(0)
    check(rows[0].subType() == 4294967295L)
    check(LoxArchive.parseDevices("plain text".toByteArray()).isEmpty())
    check(LoxArchive.parseDevices("<C Type=\"TreeDevice\" Title=\"open\">".toByteArray())[0].name() == "open")
    val invalid = "<C Type=\"TreeDevice\" Title=\"X\" Serial=\"X\"/>".toByteArray()
    for (i in invalid.indices) if (invalid[i] == 'X'.code.toByte()) invalid[i] = 0xff.toByte()
    check(LoxArchive.parseDevices(invalid)[0].let { it.name().isEmpty() && it.serial() == null })
    try {
        LoxArchive.parseDevices("<Root><C Type=\"TreeDevice\"/><secret>not-an-error-payload</Root>".toByteArray())
        error("Expected typed reader error without partial results")
    } catch (failure: ConfigurationException.Xml) {
        check(failure.sourceMessage().isNotEmpty() && !failure.sourceMessage().contains("not-an-error-payload"))
    }
    NativeDeviceProbe.verify("Kotlin") { LoxArchive.parseDevices(it) }
    println("Kotlin devices: SAME Java API/raw values/unsigned/defaults/errors/ownership PASS")
}
