import net.eisber.lox.archive.Control
import net.eisber.lox.archive.ControlResolutionException
import net.eisber.lox.archive.IdentifierException
import net.eisber.lox.archive.LoxArchive
import net.eisber.lox.archive.PreparedControlQuery
import net.eisber.lox.archive.ResolvedControl

private inline fun <reified T : Exception> expect(
    operation: () -> Unit,
    context: (T) -> Unit = {},
) {
    try {
        operation()
    } catch (error: Exception) {
        if (error !is T) throw error
        context(error)
        return
    }
    error("Expected ${T::class.java.name}")
}

private fun catalog(): List<Control> = listOf(
    Control("Licht Extra", "opaque-first", "Switch", "KÜCHE", "Indoor", true, false),
    Control("Licht", "opaque-exact", "LightController2", null, null, false, true),
    Control("Panel [West]", "nested", "Display", "KÜCHE", "", false, false),
    Control("Licht", "spaced", "Dimmer", " Küche ", null, true, true),
    Control("Other", "roomless", "State", null, null, false, false),
    Control("Open [", "unclosed", "Switch", null, null, false, false),
    Control("Straße", "unicode", "Sensor", "ROOM", "x", false, true),
    Control("İSTANBUL", "dotted", "Sensor", "ROOM", "x", false, true),
    Control("Literal [tag] ", "trailing", "Switch", null, null, false, false),
)

private fun resolve(query: String, room: String? = null, controls: List<Control> = catalog())
    : ResolvedControl = LoxArchive.resolveControlQuery(
        LoxArchive.prepareControlQuery(query, room, emptyMap()), controls,
    )

fun main() {
    check(System.getenv("RUST_MIN_STACK") == null)
    for (id in listOf("-".repeat(21), "-".repeat(51), "1FBC668C-005C-7471-FFFFED57184A04D2")) {
        val prepared = LoxArchive.prepareControlQuery(id, "ignored", mapOf(id to "must-not-win"))
        check(prepared is PreparedControlQuery.Resolved)
        check(LoxArchive.resolveControlQuery(prepared, emptyList()).uuid() == id)
    }
    val aliases = mutableMapOf(
        "Alias [Room]" to "../raw-alias", "../raw-alias" to "must-not-recurse", "" to "",
    )
    val alias = LoxArchive.prepareControlQuery("Alias [Room]", "ignored", aliases)
    aliases["Alias [Room]"] = "changed-after-prepare"
    repeat(2) { check(LoxArchive.resolveControlQuery(alias, emptyList()).uuid() == "../raw-alias") }
    check(LoxArchive.resolveControlQuery(
        LoxArchive.prepareControlQuery("", null, aliases), emptyList(),
    ).uuid() == "")
    check(LoxArchive.prepareControlQuery("alias [Room]", null, aliases)
        is PreparedControlQuery.Catalog)
    check(LoxArchive.resolveControlQuery(
        LoxArchive.prepareControlQuery("other", null, mapOf("Other" to "alias")), catalog(),
    ).uuid() == "roomless")
    println("Kotlin same Java API identifiers/owned aliases: PASS")

    val query = " Panel [West] [küche]"
    val prepared = LoxArchive.prepareControlQuery(query, "ignored", emptyMap())
    check(prepared is PreparedControlQuery.Catalog)
    check(prepared.query() == query && prepared.roomFilter() == "ignored")
    repeat(2) { check(LoxArchive.resolveControlQuery(prepared, catalog()).uuid() == "nested") }
    val controls = catalog().toMutableList()
    val expected = listOf(controls[0], controls[1], controls[3])
    expect<ControlResolutionException.Ambiguous>({ resolve("Licht", controls = controls) }) {
        check(it.query() == "Licht")
        val candidates: List<Control> = it.candidates()
        check(candidates == expected)
        controls.clear()
        check(candidates == expected)
    }
    val other = catalog()[4]
    expect<ControlResolutionException.Ambiguous>({
        resolve("Other", controls = listOf(other, other))
    }) { check(it.candidates() == listOf(other, other)) }
    for (missing in listOf(" Missing [ Küche ]", " Licht ", "Panel [West] [küche", " Licht [küche] ")) {
        expect<ControlResolutionException.NotFound>({ resolve(missing) }) {
            check(it.query() == missing)
        }
    }
    println("Kotlin typed success/ordered seven-field ambiguity/not-found: PASS")

    for ((name, room, expectedId) in listOf(
        Triple(" Licht [ küche ]", "missing", "spaced"),
        Triple(" Panel [West] [küche]", "missing", "nested"),
        Triple("Other []", "missing", "roomless"),
        Triple("", " küche ", "spaced"),
        Triple("Open [", null, "unclosed"),
        Triple("Literal [tag] ", null, "trailing"),
    )) check(resolve(name, room).uuid() == expectedId)
    expect<ControlResolutionException.NotFound>({ resolve("Licht [ küche  ]", "küche") })
    for (room in listOf(null, "")) {
        expect<ControlResolutionException.Ambiguous>({ resolve("", room) }) {
            check(it.candidates() == catalog())
        }
    }
    check(resolve("straße").uuid() == "unicode")
    check(resolve("i\u0307stan").uuid() == "dotted")
    for (missing in listOf("STRASSE", "ISTANBUL")) {
        expect<ControlResolutionException.NotFound>({ resolve(missing) })
    }
    println("Kotlin literal room/rightmost bracket/Unicode/empty query: PASS")

    for (id in listOf("a", "-", "-".repeat(20), "a".repeat(30), "é".repeat(30) + "-")) {
        check(!LoxArchive.isUuid(id))
    }
    for (id in listOf("-".repeat(21), "-".repeat(51))) check(LoxArchive.isUuid(id))
    for (id in listOf("a", "-", "-".repeat(50))) LoxArchive.validateUuid(id)
    for (id in listOf("", "-".repeat(51), "é".repeat(26), "/".repeat(51))) {
        expect<IdentifierException.InvalidLength>({ LoxArchive.validateUuid(id) })
    }
    for (id in listOf("..", "a/b", "a\\b", "a%b")) {
        expect<IdentifierException.InvalidCharacters>({ LoxArchive.validateUuid(id) })
    }
    for (id in listOf("a.b", "é".repeat(25), "opaque-id")) {
        expect<IdentifierException.NonHex>({ LoxArchive.validateUuid(id) })
    }
    println("Kotlin distinct byte-length/path identifier errors: PASS")
}
