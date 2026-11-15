import net.eisber.lox.archive.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;

public final class AutopilotCatalogSmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("Autopilot values");
    }

    public static void values(NativeAutopilotProbe.Operation operation) throws Throwable {
        byte[] input = Files.readAllBytes(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "autopilot_catalog.json"));
        byte[] before = input.clone();
        List<AutopilotRule> value = operation.run(input);
        require(Arrays.equals(input, before));
        require(value.equals(List.of(new AutopilotRule("", "empty", "", ""),
                new AutopilotRule("?", "", "", ""), new AutopilotRule("?", "bad", "", ""),
                new AutopilotRule("A", "01234567-89ab-cdef-0123456789abcdef", "changed", ""),
                new AutopilotRule("Same", "a", "changed", ""),
                new AutopilotRule("Same", "z", "", "history"),
                new AutopilotRule("a", "12345678-1234-1234-1234-123456789abc", "", "history"),
                new AutopilotRule("é é\r\n<xml>", "raw\0ID", "raw\0state", "<xml>\r\n"))));
        Arrays.fill(input, (byte) 0);
        require(value.get(7).uuid().equals("raw\0ID"));
        require(operation.run(before).equals(value));
        for (String shape : List.of("null", "true", "4", "\"text\"", "[]", "{}",
                "{\"autopilot\":[]}", "{\"autopilot\":false}", "{\"autopilot\":null}")) {
            require(operation.run(shape.getBytes(StandardCharsets.UTF_8)).isEmpty());
        }
        byte[][] malformed = { "{\n\"autopilot\":".getBytes(StandardCharsets.UTF_8),
                "{} secret".getBytes(StandardCharsets.UTF_8), {34, (byte)255, 34} };
        long[][] positions = {{2,12}, {1,4}, {1,3}};
        for (int i = 0; i < malformed.length; i++) {
            try {
                operation.run(malformed[i]);
                throw new AssertionError("Expected existing CatalogDecodeException");
            } catch (CatalogDecodeException.InvalidJson failure) {
                require(failure.line() == positions[i][0] && failure.column() == positions[i][1]);
                require(!failure.toString().contains("secret"));
            }
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::listLoxapp3AutopilotRules);
        NativeAutopilotProbe.verify("Java", LoxArchive::listLoxapp3AutopilotRules);
        System.out.println("Java autopilot typed values/raw ordering/defaults/errors/input independence PASS");
    }
}
