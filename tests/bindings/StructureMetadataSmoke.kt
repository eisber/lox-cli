import net.eisber.lox.archive.*
import java.nio.file.Files
import java.nio.file.Path

fun main() {
    val snapshot: StructureMetadata = LoxArchive.parseLoxapp3StructureMetadata(
        Files.readAllBytes(Path.of("crates", "lox-bindings", "tests", "fixtures", "structure_metadata.json")))
    check(snapshot.categories().last() == Category("raw\u0000ID", "é é\r\n<xml>"))
    check(snapshot.globalStates()[1] == GlobalState("z", "raw\u0000UUID"))
    check(snapshot.operatingModes().map { it.rawId() } ==
        listOf("-1", "", " 1", "0", "2147483648", "bad", "+1", "01", "2"))
    val operation = NativeMetadataProbe.Operation { input ->
        LoxArchive.parseLoxapp3StructureMetadata(input)
    }
    StructureMetadataSmoke.values(operation)
    NativeMetadataProbe.verify("Kotlin", operation)
    println("Kotlin SAME Java metadata API typed values/raw ordering/omissions/errors/input independence PASS")
}
