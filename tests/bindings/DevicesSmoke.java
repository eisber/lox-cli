import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import net.eisber.lox.archive.*;

public final class DevicesSmoke {
    private static void require(boolean value) {
        if (!value) throw new AssertionError("Device inventory contract");
    }

    public static void main(String[] args) throws Throwable {
        byte[] xml = Files.readAllBytes(Path.of(args[0]));
        byte[] before = xml.clone();
        List<XmlDeviceInfo> rows = LoxArchive.parseDevices(xml);
        require(Arrays.equals(xml, before));
        require(rows.equals(List.of(
                new XmlDeviceInfo(" raw&amp;&#65;&custom; ", XmlDeviceBus.TREE, "", 4294967295L,
                        "Unknown (4294967295)", null, null),
                new XmlDeviceInfo("same", XmlDeviceBus.AIR, null, 7L, "Nano IO Air", null, null),
                new XmlDeviceInfo("same", XmlDeviceBus.NETWORK, null, 4294967295L,
                        "Network device", "", "raw&amp;"),
                new XmlDeviceInfo("", XmlDeviceBus.TREE, null, null, "Unknown", null, null),
                new XmlDeviceInfo("repeat", XmlDeviceBus.TREE, null, null, "Unknown", null, null),
                new XmlDeviceInfo("repeat", XmlDeviceBus.TREE, null, null, "Unknown", null, null),
                new XmlDeviceInfo("", XmlDeviceBus.TREE, null, null, "Unknown", null, null),
                new XmlDeviceInfo("", XmlDeviceBus.AIR, null, 32794L, "Presence sensor", null, null),
                new XmlDeviceInfo("", XmlDeviceBus.NETWORK, null, 7L, "Network device", null, null),
                new XmlDeviceInfo("", XmlDeviceBus.TREE, null, 7L, "Nano IO Air", null, null),
                new XmlDeviceInfo("before-bad", XmlDeviceBus.TREE, null, null, "Unknown", null, null))));
        Arrays.fill(xml, (byte) 0);
        require(rows.get(0).subType() == 4294967295L && rows.get(0).serial().isEmpty());
        for (String empty : List.of("", "plain text")) {
            require(LoxArchive.parseDevices(empty.getBytes(StandardCharsets.UTF_8)).isEmpty());
        }
        require(LoxArchive.parseDevices("<C Type=\"TreeDevice\" Title=\"open\">"
                .getBytes(StandardCharsets.UTF_8)).get(0).name().equals("open"));
        byte[] invalid = "<C Type=\"TreeDevice\" Title=\"X\" Serial=\"X\"/>".getBytes(StandardCharsets.UTF_8);
        for (int i = 0; i < invalid.length; i++) if (invalid[i] == 'X') invalid[i] = (byte) 0xff;
        var absent = LoxArchive.parseDevices(invalid).get(0);
        require(absent.name().isEmpty() && absent.serial() == null);
        for (String malformed : List.of("<Root><C Type=\"TreeDevice\" Title=\"not partial\"/>"
                + "<secret>not-an-error-payload</Root>", "</unexpected>")) {
            try {
                LoxArchive.parseDevices(malformed.getBytes(StandardCharsets.UTF_8));
                throw new AssertionError("Expected typed reader error without partial results");
            } catch (ConfigurationException.Xml error) {
                require(!error.sourceMessage().isEmpty()
                        && !error.sourceMessage().contains("not-an-error-payload"));
            }
        }
        NativeDeviceProbe.verify("Java", LoxArchive::parseDevices);
        System.out.println("Java devices: typed raw values/defaults/unsigned/order/errors/ownership PASS");
    }
}
