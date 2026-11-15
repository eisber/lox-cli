import java.nio.charset.StandardCharsets.UTF_8
import net.eisber.lox.archive.*

fun main() {
    check(System.getenv("RUST_MIN_STACK") == null)
    check(LoxArchive.decodeTextStatePayload(byteArrayOf()).isEmpty())
    val data = TextStateSmoke.payload(1)
    val before = data.clone()
    val states = LoxArchive.decodeTextStatePayload(data)
    check(data.contentEquals(before))
    check(states.size == TextStateSmoke.TEXTS.size)
    for ((i, state) in states.withIndex()) {
        check(state.uuidWireBytes().contentEquals(ByteArray(16) { it.toByte() }))
        check(state.iconUuidWireBytes().contentEquals(ByteArray(16) { (240 + it).toByte() }))
        check(state.uuidNative35() == "03020100-0504-0706-08090a0b0c0d0e0f")
        check(state.iconUuidNative35() == "f3f2f1f0-f5f4-f7f6-f8f9fafbfcfdfeff")
        val exact = TextStateSmoke.TEXTS[i].toByteArray(UTF_8)
        check(state.text() == TextStateSmoke.TEXTS[i])
        check(state.text().toByteArray(UTF_8).contentEquals(exact))
        check(state.textByteLen().toUInt().toLong() == exact.size.toLong())
    }
    data.fill(0)
    TextStateSmoke.verify(states)
    for (case in TextStateSmoke.errors()) {
        try {
            LoxArchive.decodeTextStatePayload(case.input())
            error("Expected complete-input rejection, not prefix results")
        } catch (failure: TextStatePayloadException) {
            case.check(failure)
        }
        TextStateSmoke.verify(LoxArchive.decodeTextStatePayload(before))
    }
    for (length in 1..35) {
        try {
            LoxArchive.decodeTextStatePayload(ByteArray(length))
            error("Expected short-header failure")
        } catch (failure: TextStatePayloadException.TruncatedHeader) {
            check(failure.recordOffset() == 0L && failure.available() == length.toLong())
        }
    }
    NativeTextStateProbe.verifyPath("Kotlin", LoxArchive::decodeTextStatePayload,
        TextStateSmoke.payload(512), 0, 0, 0, 0)
    for (case in TextStateSmoke.errors()) {
        NativeTextStateProbe.verifyPath("Kotlin", LoxArchive::decodeTextStatePayload,
            case.input(), case.tag(), case.first(), case.second(), case.serializedLength())
    }
    println("Kotlin same Java API text UUID/icon/order/duplicate/UTF8/bytecount/NUL/atomic errors/ownership PASS")
}
