import net.eisber.lox.archive.*;
import java.util.*;

public final class BackupCatalogSmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("Backup catalog values");
    }

    public static void values(NativeBackupCatalogProbe.Operation operation) {
        String name = "sps_4294967295_20261399259999999.zip";
        BackupInfo value = operation.run(name, -1L);
        require(value.equals(new BackupInfo(name, -1, "20261399259999999", -1L)));
        require(Integer.toUnsignedLong(value.version()) == 4294967295L);
        require(Long.toUnsignedString(value.size()).equals("18446744073709551615"));
        require(operation.run("sps_+0000_00000000000000.zip", 0).version() == 0);
        for (String bad : List.of("SPS_0_20260101120000.zip", "sps_0_20260101120000.ZIP",
                "sps_0_2026010112000.zip", "sps_4294967296_20260101120000.zip",
                "sps_0_２０２６0101120000.zip", "")) {
            require(operation.run(bad, 0) == null);
        }
        List<BackupInfo> rows = LoxArchive.parseBackupListing(List.of(
                "d a b c 18446744073709551615 sps_3_99999999999999.zip",
                "l a b c 18446744073709551616 sps_1_20261399259999.zip",
                "? a b c invalid sps_4_20261399259999.zip",
                "a b c sps_1_20261399259999.zip",
                "a b c sps_2_100000000000000.zip",
                "a b sps_0_20260101120000.zip"));
        require(rows.stream().map(BackupInfo::version).toList().equals(List.of(3, 1, 4, 1, 2)));
        require(rows.stream().map(BackupInfo::size).toList().equals(List.of(-1L, 0L, 0L, 0L, 0L)));
        require(LoxArchive.parseBackupListing(List.of()).isEmpty());
        require(operation.run("sps_0_20260101120000.zip", 1536).size() == 1536);
        require(value.filename().equals(name));
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::parseBackupName);
        NativeBackupCatalogProbe.verify("Java", LoxArchive::parseBackupName);
        System.out.println("Java backup catalog nullable/owned/unsigned/list/raw-order PASS");
    }
}
