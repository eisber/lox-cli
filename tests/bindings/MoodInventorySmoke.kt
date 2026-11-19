import net.eisber.lox.archive.LoxArchive
import net.eisber.lox.archive.MoodInventoryRequest
import net.eisber.lox.archive.MoodInventorySmoke

fun main() {
    val operation = MoodInventorySmoke.Operation { input: ByteArray, selector: String ->
        LoxArchive.listMoods(MoodInventoryRequest(input, selector))
    }
    MoodInventorySmoke.verify("Kotlin", operation)
}
