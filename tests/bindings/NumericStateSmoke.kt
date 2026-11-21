import net.eisber.lox.archive.*

fun main() {
    check(System.getenv("RUST_MIN_STACK") == null)
    check(LoxArchive.decodeValueStatePayload(byteArrayOf()).isEmpty())
    val data = NumericStateSmoke.payload(1)
    val before = data.clone()
    val states = LoxArchive.decodeValueStatePayload(data)
    check(data.contentEquals(before))
    for ((i, state) in states.withIndex()) {
        val wire = ByteArray(16) { (if (i % 2 == 0) it else 15 - it).toByte() }
        check(state.uuidWireBytes().contentEquals(wire))
        check(state.uuidNative35() == if (i % 2 == 0)
            "03020100-0504-0706-08090a0b0c0d0e0f" else "0c0d0e0f-0a0b-0809-0706050403020100")
        val expected = NumericStateSmoke.BITS[i]
        check(state.valueBits() == expected)
        check(state.valueBits().toULong() == expected.toULong())
        if (Double.fromBits(expected).isNaN()) check(state.value().isNaN())
        else check(state.value().toRawBits() == expected)
    }
    for (length in 1..72) {
        if (length % 24 == 0) continue
        try {
            LoxArchive.decodeValueStatePayload(ByteArray(length) { 0xa5.toByte() })
            error("Expected atomic typed length failure")
        } catch (failure: NumericStatePayloadException.InvalidLength) {
            check(failure.actual() == length.toLong() && failure.trailing() == (length % 24).toLong())
            check(!failure.toString().contains("a5"))
        }
        NumericStateSmoke.verify(LoxArchive.decodeValueStatePayload(before))
    }
    data.fill(0)
    NumericStateSmoke.verify(states)
    NativeNumericStateProbe.verify("Kotlin", LoxArchive::decodeValueStatePayload)
    println("Kotlin same Java API numeric payload UUID/order/duplicate/IEEEbits/errors/host ownership PASS")
}
