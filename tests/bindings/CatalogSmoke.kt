import java.nio.file.Files
import java.nio.file.Path
import net.eisber.lox.archive.*

fun main(args: Array<String>) {
    check(System.getenv("RUST_MIN_STACK") == null)
    val data = Files.readAllBytes(Path.of(args[0]))
    val before = data.clone()
    val all = ControlFilter(null, null, null, false)
    val controls = LoxArchive.selectLoxapp3Controls(data, all)
    check(data.contentEquals(before))
    check(controls.map { it.name() } == listOf("?", "?", "?", "?", "?", "?", "Alpha", "Alpha", "Zulu", "a"))
    val byId = controls.associateBy { it.uuid() }
    for (id in listOf("", "scalar", "array", "number", "string", "wrong")) {
        check(byId[id] == Control("?", id, "?", "Default", "Category", false, false))
    }
    check(byId["a"] == Control("Alpha", "a", "Switch", null, null, false, false))
    check(byId["b"] == Control("Alpha", "b", "SWITCH", null, null, false, false))
    check(byId["empty"] == Control("a", "empty", "", "", "", false, false))
    val expected = Control("Zulu", "opaque / key", "Straße", "KÜCHE", "LÜFTUNG", true, true)
    check(byId["opaque / key"] == expected)
    check(LoxArchive.selectLoxapp3Controls(data, ControlFilter("STRAß", "kü", "lüft", true)) == listOf(expected))
    for (typ in listOf("STRASSE", " Straße ")) {
        check(LoxArchive.selectLoxapp3Controls(data, ControlFilter(typ, null, null, false)).isEmpty())
    }
    check(LoxArchive.selectLoxapp3Controls(data, ControlFilter(null, "", "", false)).size == 10)
    check(LoxArchive.selectLoxapp3Controls(data, ControlFilter(null, "", "", true)).size == 1)
    for (shape in listOf("null", "true", "4", "\"text\"", "[]", "{}",
        "{\"controls\":null}", "{\"controls\":[]}", "{\"controls\":false,\"rooms\":7,\"cats\":[]}")) {
        check(LoxArchive.selectLoxapp3Controls(shape.toByteArray(), all).isEmpty())
    }
    val malformed = listOf("{\n\"controls\":".toByteArray(), "{} secret".toByteArray(),
        byteArrayOf(34, 0xff.toByte(), 34))
    val positions = listOf(2L to 11L, 1L to 4L, 1L to 3L)
    for ((i, payload) in malformed.withIndex()) {
        try {
            LoxArchive.selectLoxapp3Controls(payload, all)
            error("Expected JSON/UTF8 error")
        } catch (failure: CatalogDecodeException.InvalidJson) {
            check(failure.line() to failure.column() == positions[i])
            check(!failure.toString().contains("secret"))
        }
    }
    data.fill(0)
    check(byId["opaque / key"] == expected)
    NativeCatalogProbe.verify("Kotlin", LoxArchive::selectLoxapp3Controls)
    println("Kotlin same Java API catalog all fields/default semantics/filters/errors/host ownership PASS")
}
