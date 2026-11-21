import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.*;
import net.eisber.lox.archive.*;

public final class CatalogSmoke {
    private static void require(boolean value) {
        if (!value) throw new AssertionError("Catalog contract");
    }

    public static void main(String[] args) throws Throwable {
        require(System.getenv("RUST_MIN_STACK") == null);
        byte[] data = Files.readAllBytes(Path.of(args[0]));
        byte[] before = data.clone();
        ControlFilter all = new ControlFilter(null, null, null, false);
        List<Control> controls = LoxArchive.selectLoxapp3Controls(data, all);
        require(Arrays.equals(data, before));
        require(controls.stream().map(Control::name).toList().equals(
                List.of("?", "?", "?", "?", "?", "?", "Alpha", "Alpha", "Zulu", "a")));
        Map<String, Control> byId = new HashMap<>();
        for (Control control : controls) byId.put(control.uuid(), control);
        for (String id : List.of("", "scalar", "array", "number", "string", "wrong")) {
            require(byId.get(id).equals(new Control("?", id, "?", "Default", "Category", false, false)));
        }
        require(byId.get("a").equals(new Control("Alpha", "a", "Switch", null, null, false, false)));
        require(byId.get("b").equals(new Control("Alpha", "b", "SWITCH", null, null, false, false)));
        require(byId.get("empty").equals(new Control("a", "empty", "", "", "", false, false)));
        Control expected = new Control("Zulu", "opaque / key", "Straße", "KÜCHE", "LÜFTUNG", true, true);
        require(byId.get("opaque / key").equals(expected));
        require(LoxArchive.selectLoxapp3Controls(data,
                new ControlFilter("STRAß", "kü", "lüft", true)).equals(List.of(expected)));
        for (String typ : List.of("STRASSE", " Straße ")) {
            require(LoxArchive.selectLoxapp3Controls(data,
                    new ControlFilter(typ, null, null, false)).isEmpty());
        }
        require(LoxArchive.selectLoxapp3Controls(data, new ControlFilter(null, "", "", false)).size() == 10);
        require(LoxArchive.selectLoxapp3Controls(data, new ControlFilter(null, "", "", true)).size() == 1);
        for (String shape : List.of("null", "true", "4", "\"text\"", "[]", "{}",
                "{\"controls\":null}", "{\"controls\":[]}", "{\"controls\":false,\"rooms\":7,\"cats\":[]}")) {
            require(LoxArchive.selectLoxapp3Controls(shape.getBytes(StandardCharsets.UTF_8), all).isEmpty());
        }
        byte[][] malformed = { "{\n\"controls\":".getBytes(StandardCharsets.UTF_8),
                "{} secret".getBytes(StandardCharsets.UTF_8), {(byte) '"', (byte) 0xff, (byte) '"'} };
        long[][] positions = {{2, 11}, {1, 4}, {1, 3}};
        for (int i = 0; i < malformed.length; i++) {
            try {
                LoxArchive.selectLoxapp3Controls(malformed[i], all);
                throw new AssertionError("Expected JSON/UTF8 error");
            } catch (CatalogDecodeException.InvalidJson failure) {
                require(failure.line() == positions[i][0] && failure.column() == positions[i][1]);
                require(!failure.toString().contains("secret"));
            }
        }
        Arrays.fill(data, (byte) 0);
        require(byId.get("opaque / key").equals(expected));
        NativeCatalogProbe.verify("Java", LoxArchive::selectLoxapp3Controls);
        System.out.println("Java catalog all fields/default semantics/filters/errors/host ownership PASS");
    }
}
