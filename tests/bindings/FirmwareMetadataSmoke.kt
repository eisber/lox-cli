import net.eisber.lox.archive.*

fun main() {
    val version: FirmwareVersion = LoxArchive.parseFirmwareVersion("016.2.4294967295")
    check(version.components().last().toUInt() == UInt.MAX_VALUE)
    check(version.normalized() == "16.2.4294967295")
    val operation = NativeFirmwareMetadataProbe.Operation { xml -> LoxArchive.parseFirmwareMetadata(xml) }
    FirmwareMetadataSmoke.values(operation)
    NativeFirmwareMetadataProbe.verify("Kotlin", operation)
    println("Kotlin SAME generated Java firmware metadata API typed values/unsigned/ownership PASS")
}
