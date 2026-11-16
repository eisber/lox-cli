import net.eisber.lox.archive.*

fun main() {
    val value: BackupInfo = LoxArchive.parseBackupName(
        "sps_4294967295_20261399259999999.zip", -1L)!!
    check(value.version().toUInt() == UInt.MAX_VALUE)
    check(value.size().toULong() == ULong.MAX_VALUE)
    check(LoxArchive.parseBackupName("invalid", 0L) == null)
    check(LoxArchive.parseBackupListing(emptyList()).isEmpty())
    val operation = NativeBackupCatalogProbe.Operation { name, size ->
        LoxArchive.parseBackupName(name, size)
    }
    BackupCatalogSmoke.values(operation)
    NativeBackupCatalogProbe.verify("Kotlin", operation)
    println("Kotlin SAME generated Java backup API nullable/unsigned/owned/list PASS")
}
