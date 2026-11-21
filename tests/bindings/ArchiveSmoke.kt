import java.nio.file.Files
import java.nio.file.Path
import java.security.MessageDigest
import java.util.HexFormat
import net.eisber.lox.archive.ArchiveException
import net.eisber.lox.archive.LoxArchive

private inline fun <reified T : ArchiveException> expect(
    operation: () -> ByteArray,
    context: (T) -> Unit = {},
) {
    try {
        operation()
    } catch (error: ArchiveException) {
        if (error !is T) throw error
        context(error)
        println("Kotlin typed error: ${error.javaClass.simpleName}(${error.message})")
        return
    }
    error("Expected ${T::class.java.name}")
}

fun main(args: Array<String>) {
    val fixtures = Path.of(args[0])
    fun read(name: String) = Files.readAllBytes(fixtures.resolve(name))
    val raw = read("raw.bin")
    val encoded = read("encoded.bin")
    check(LoxArchive.compressLoxcc(raw).contentEquals(encoded))
    check(LoxArchive.decompressLoxcc(encoded).contentEquals(raw))
    check(LoxArchive.extractAndDecompress(encoded).contentEquals(raw))
    check(LoxArchive.extractAndDecompress(read("backup.zip")).contentEquals(raw))
    check(LoxArchive.decompressLoxcc(read("permissive.bin")).contentEquals(raw))
    for (input in listOf(byteArrayOf(), ByteArray(256 * 1024) { it.toByte() })) {
        check(LoxArchive.decompressLoxcc(LoxArchive.compressLoxcc(input)).contentEquals(input))
    }
    val hash = HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(raw))
    println("Kotlin ByteArray roundtrip: ${raw.size} bytes sha256=$hash")
    expect<ArchiveException.HeaderTooShort>({ LoxArchive.decompressLoxcc(ByteArray(3)) }) {
        check(it.actual() == 3L)
    }
    expect<ArchiveException.InvalidMagic>({ LoxArchive.decompressLoxcc(ByteArray(16)) }) {
        check(it.actual() == 0)
    }
    expect<ArchiveException.ZeroBackReference>({ LoxArchive.decompressLoxcc(read("zero-offset.bin")) }) {
        check(it.position() == 1L)
    }
    expect<ArchiveException.BackReferenceOutOfBounds>({
        LoxArchive.decompressLoxcc(read("bad-offset.bin"))
    }) {
        check(it.offset() == 1L && it.outputLen() == 0L)
    }
    expect<ArchiveException.InvalidZip>({ LoxArchive.extractAndDecompress(ByteArray(3)) }) {
        check(it.sourceMessage().isNotEmpty())
    }
    expect<ArchiveException.MissingConfigEntry>({ LoxArchive.extractAndDecompress(read("empty.zip")) })
    expect<ArchiveException.ReadZipEntry>({ LoxArchive.extractAndDecompress(read("badcrc.zip")) }) {
        check(it.index() == 0L && it.name() == "sps0.LoxCC" && it.sourceMessage().isNotEmpty())
    }
}
