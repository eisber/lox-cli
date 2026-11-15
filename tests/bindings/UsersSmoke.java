import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import net.eisber.lox.archive.*;

public final class UsersSmoke {
    private static void require(boolean value) {
        if (!value) throw new AssertionError("User inventory contract");
    }

    public static void main(String[] args) throws Throwable {
        byte[] xml = Files.readAllBytes(Path.of(args[0]));
        byte[] before = xml.clone();
        List<XmlUserInfo> rows = LoxArchive.parseUsers(xml);
        require(Arrays.equals(xml, before));
        require(rows.equals(List.of(
                new XmlUserInfo(" raw&amp;&#65;&custom; ", "not_a_uuid", true, "one  two &#95; &amp;"),
                new XmlUserInfo("", "", false, ""),
                new XmlUserInfo("repeat", "same", false, " "),
                new XmlUserInfo("repeat", "same", false, " "),
                new XmlUserInfo("first", "", true, "a b"),
                new XmlUserInfo("Grün", "raw_under", true, "Grüße 世界"))));
        Arrays.fill(xml, (byte) 0);
        require(rows.get(0).uuid().equals("not_a_uuid"));
        for (String empty : List.of("", "plain text")) {
            require(LoxArchive.parseUsers(empty.getBytes(StandardCharsets.UTF_8)).isEmpty());
        }
        require(LoxArchive.parseUsers("<C Type=\"User\" Title=\"open\">"
                .getBytes(StandardCharsets.UTF_8)).get(0).name().equals("open"));
        require(LoxArchive.parseUsers("<C Type=\"User\" Title=\"before-bad\" bad />"
                .getBytes(StandardCharsets.UTF_8)).get(0).name().equals("before-bad"));
        byte[] invalid = ("<C Type=\"User\" Title=\"X\" U=\"valid_raw\" Desc=\"ok_value\" NFCArr=\"x\"/>"
                + "<C Type=\"User\" Title=\"valid\" U=\"X\" Desc=\"X\" NFCArr=\"X\"/>"
                + "<C Type=\"X\" Title=\"excluded\"/>"
                + "<C Type=\"User\" Title=\"X\" Title=\"not-a-fallback\"/>").getBytes(StandardCharsets.UTF_8);
        for (int i = 0; i < invalid.length; i++) if (invalid[i] == 'X') invalid[i] = (byte) 0xff;
        require(LoxArchive.parseUsers(invalid).equals(List.of(
                new XmlUserInfo("", "valid_raw", true, "ok value"),
                new XmlUserInfo("valid", "", false, ""),
                new XmlUserInfo("", "", false, ""))));
        for (String malformed : List.of("<Root><C Type=\"User\" Title=\"not partial\"/>"
                + "<secret>not-an-error-payload</Root>", "</unexpected>")) {
            try {
                LoxArchive.parseUsers(malformed.getBytes(StandardCharsets.UTF_8));
                throw new AssertionError("Expected typed reader error without partial results");
            } catch (ConfigurationException.Xml error) {
                require(!error.sourceMessage().isEmpty()
                        && !error.sourceMessage().contains("not-an-error-payload"));
                if (malformed.startsWith("<Root>")) {
                    require(error.sourceMessage().equals(
                            "ill-formed document: expected `</secret>`, but `</Root>` was found"));
                }
            }
        }
        NativeUserProbe.verify("Java", LoxArchive::parseUsers);
        System.out.println("Java users: typed raw values/defaults/UTF8/order/errors/ownership PASS");
    }
}
