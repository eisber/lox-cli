import net.eisber.lox.archive.*

fun main() {
    val rows = listOf("- z.loxcc", "- A.LoxCC", "- z.loxcc")
    val owned: List<String> = LoxArchive.parseCustomListing(rows)
    check(owned == listOf("A.LoxCC", "z.loxcc", "z.loxcc"))
    try {
        LoxArchive.parseCustomListing(listOf("- ignored", "d private.loxcc"))
        error("Typed atomic rejection required")
    } catch (failure: CustomListingException.Unsupported) {
        check(failure.row() == 1L && !failure.toString().contains("private"))
    }
    val operation = NativeCustomListingProbe.Operation { input ->
        LoxArchive.parseCustomListing(input)
    }
    CustomListingSmoke.values(operation)
    NativeCustomListingProbe.verify("Kotlin", operation)
    check(owned == listOf("A.LoxCC", "z.loxcc", "z.loxcc"))
    println("Kotlin SAME generated Java custom-listing API typed/raw/sorted/owned PASS")
}
