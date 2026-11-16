import net.eisber.lox.archive.*;
import java.util.Arrays;

public final class FirmwareMetadataSmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("Firmware metadata values");
    }

    public static void values(NativeFirmwareMetadataProbe.Operation operation) throws Throwable {
        String xml = "<Miniserversoftware><Release Version=\"016.2.4294967295\" Path=\"a&amp;b\"/>"
                + "<RemoteConnect MinVersion=\"16.2.0\" Deadline=\"2028-02-29\"/></Miniserversoftware>";
        UpdateMetadata metadata = operation.run(xml);
        require(Arrays.equals(metadata.release().version().components(), new int[] {16, 2, -1}));
        require(Integer.toUnsignedLong(metadata.release().version().components()[2]) == 4294967295L);
        require(metadata.release().version().normalized().equals("16.2.4294967295"));
        require(metadata.release().configDownload().equals("a&b"));
        require(metadata.remoteConnect().deadline().equals("2028-02-29"));
        require(LoxArchive.compareFirmwareVersions("1.2", "1.2.0") == FirmwareOrdering.LESS);
        require(LoxArchive.compareFirmwareVersions("01.2", "1.2") == FirmwareOrdering.EQUAL);
        String[] dates = {"2028-02-28", "2028-02-29", "2028-03-01"};
        DeadlineRelation[] relations = {DeadlineRelation.BEFORE, DeadlineRelation.ON, DeadlineRelation.PAST};
        for (int i = 0; i < dates.length; i++) {
            UpdateAssessment value = LoxArchive.assessFirmwareMetadata(xml, "16.1", "16.2", dates[i]);
            require(value.releaseStatus() == ReleaseStatus.NEWER);
            require(value.programCompatibility() == ProgramCompatibility.SAME_MAJOR_MINOR);
            require(value.remoteConnect().belowMinimum());
            require(value.remoteConnect().deadline() == relations[i]);
            require(value.remoteConnect().warningDue() == (i != 2));
        }
        UpdateAssessment empty = LoxArchive.assessFirmwareMetadata("<Miniserversoftware/>", "1", null, "2000-01-01");
        require(empty.releaseStatus() == ReleaseStatus.UNAVAILABLE);
        require(empty.programCompatibility() == null && empty.remoteConnect() == null);
        try {
            LoxArchive.assessFirmwareMetadata(xml, "1", null, "2027-02-29");
            throw new AssertionError("Expected calendar refusal");
        } catch (FirmwareMetadataException.InvalidDate failure) {
            require(failure.value().equals("2027-02-29"));
        }
        try {
            operation.run("<Miniserversoftware><Release/></Miniserversoftware>");
            throw new AssertionError("Expected missing attribute");
        } catch (FirmwareMetadataException.MissingAttribute failure) {
            require(failure.element().equals("Release") && failure.attribute().equals("Version"));
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::parseFirmwareMetadata);
        NativeFirmwareMetadataProbe.verify("Java", LoxArchive::parseFirmwareMetadata);
        System.out.println("Java firmware owned typed metadata/comparison/explicit-date/refusal PASS");
    }
}
