import net.eisber.lox.archive.*

fun main() {
    val operation = NativeMessageHeaderProbe.Operation { input ->
        LoxArchive.decodeMessageHeader(input)
    }
    MessageHeaderSmoke.values(operation)
    val bytes: ByteArray = LoxArchive.encodeMessageHeader(
        0xff.toByte(), 0xff.toByte(), 0x91.toByte(), -1)
    val header: MessageHeader = LoxArchive.decodeMessageHeader(bytes)
    check(header.payloadSize().toUInt() == UInt.MAX_VALUE)
    check(header.messageType().toUByte() == 255.toUByte())
    println("Kotlin SAME Java API actual native header values/unsigned bits/roundtrip/errors PASS")
    NativeMessageHeaderProbe.verify("Kotlin", operation)
}
