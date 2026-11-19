import net.eisber.lox.archive.*

fun main() {
    val invalid: ByteArray = byteArrayOf(0xff.toByte(), '<'.code.toByte(), 0)
    val output: ByteArray = LoxArchive.mergeOverlays(invalid, emptyList<LoxccFragment>())
    check(output.contentEquals(invalid))
    val operation = FragmentApplicationSmoke.Operation { base: ByteArray, fragments: List<LoxccFragment> ->
        LoxArchive.mergeOverlays(base, fragments)
    }
    FragmentApplicationSmoke.values(operation)
    FragmentApplicationSmoke.verify("Kotlin", operation)
}
