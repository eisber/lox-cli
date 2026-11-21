import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import net.eisber.lox.archive.*;

public final class XmlControlsSmoke {
    private static void require(boolean value) {
        if (!value) throw new AssertionError("XML control inventory contract");
    }

    public static void main(String[] args) throws Throwable {
        byte[] xml = Files.readAllBytes(Path.of(args[0]));
        byte[] before = xml.clone();
        List<XmlControlInfo> rows = LoxArchive.parseControls(xml, null, null);
        require(Arrays.equals(xml, before));
        require(rows.equals(List.of(
                new XmlControlInfo("", "", "", "Default", "Empty ID"),
                new XmlControlInfo("Other", "Inner", "inner", "KÜCHE", "Raw &amp; &#65;"),
                new XmlControlInfo("Page", "Page", "page", "page-room", "Empty ID"),
                new XmlControlInfo("Switch", "Pending", "pending", "KÜCHE", "Raw &amp; &#65;"),
                new XmlControlInfo("Switch", "Same", "second", "KÜCHE", "Raw &amp; &#65;"),
                new XmlControlInfo("Switch", "Same", "first", "KÜCHE", "Empty ID"),
                new XmlControlInfo("Switch", "Same", "first", "raw&amp;id", "unknown"),
                new XmlControlInfo("TreeDevice", "Device", "device", "Default", "Empty ID"),
                new XmlControlInfo("switch", "Z", "lower", "Default", "Empty ID"),
                new XmlControlInfo("Ä", " A&amp;&#65;&custom; ", "室", "Default", "Empty ID"))));
        require(LoxArchive.parseControls(xml, "sWiTcH", "küc").stream().map(XmlControlInfo::uuid)
                .toList().equals(List.of("outer", "pending", "second", "first")));
        require(LoxArchive.parseControls(xml, null, "raw&amp;").get(0).category().equals("unknown"));
        require(LoxArchive.parseControls(xml, "", null).size() == 1);
        require(LoxArchive.parseControls(xml, null, "").equals(rows));
        for (String typ : List.of("ä", "Sw", "absent")) {
            require(LoxArchive.parseControls(xml, typ, null).isEmpty());
        }
        Arrays.fill(xml, (byte) 0);
        for (String empty : List.of("", "plain text")) {
            require(LoxArchive.parseControls(empty.getBytes(StandardCharsets.UTF_8), null, null).isEmpty());
        }
        require(rows.get(1).title().equals("Inner"));
        require(LoxArchive.parseControls(("<C Type=\"Switch\" U=\"c\" Title=\"Open\">"
                + "<IoData Pr=\"r\">").getBytes(StandardCharsets.UTF_8), null, null)
                .equals(List.of(new XmlControlInfo("Switch", "Open", "c", "r", ""))));
        byte[] invalid = "<C Type=\"Switch\" U=\"c\" Title=\"X\"/><IoData/>".getBytes(StandardCharsets.UTF_8);
        for (int i = 0; i < invalid.length; i++) if (invalid[i] == 'X') invalid[i] = (byte) 0xff;
        require(LoxArchive.parseControls(invalid, null, null).isEmpty());
        require(LoxArchive.parseControls("<C Type=\"Switch\" U=\"c\"/><IoData/>"
                .getBytes(StandardCharsets.UTF_8), null, null).isEmpty());
        for (String malformed : List.of("<Root><C Type=\"Switch\" U=\"c\" Title=\"ok\"/>"
                + "<IoData/><secret>not-an-error-payload</Root>", "</unexpected>")) {
            try {
                LoxArchive.parseControls(malformed.getBytes(StandardCharsets.UTF_8), null, null);
                throw new AssertionError("Expected typed reader error, not partial success");
            } catch (ConfigurationException.Xml error) {
                require(!error.sourceMessage().isEmpty()
                        && !error.sourceMessage().contains("not-an-error-payload"));
            }
        }
        NativeConfigInspectionProbe.verifyControls("Java", LoxArchive::parseControls);
        System.out.println("Java XML controls: five raw fields/filter admission/pending association/order/errors/ownership PASS");
    }
}
