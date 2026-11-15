import net.eisber.lox.archive.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;

public final class StructureMetadataSmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("Metadata values");
    }

    public static void values(NativeMetadataProbe.Operation operation) throws Throwable {
        byte[] input = Files.readAllBytes(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "structure_metadata.json"));
        byte[] before = input.clone();
        StructureMetadata value = operation.run(input);
        require(Arrays.equals(input, before));
        require(value.categories().equals(List.of(new Category("", ""),
                new Category("a", "Same"), new Category("z", "Same"), new Category("lower", "a"),
                new Category("raw\0ID", "é é\r\n<xml>"))));
        require(value.globalStates().equals(List.of(new GlobalState("A", ""),
                new GlobalState("z", "raw\0UUID"), new GlobalState("é é", "<xml>\r\n"))));
        require(value.operatingModes().equals(List.of(
                new OperatingMode("-1", "minus"), new OperatingMode("", "empty key"),
                new OperatingMode(" 1", "whitespace"), new OperatingMode("0", "zero"),
                new OperatingMode("2147483648", "overflow"), new OperatingMode("bad", "fallback"),
                new OperatingMode("+1", "plus"), new OperatingMode("01", "zero padded"),
                new OperatingMode("2", "two"))));
        Arrays.fill(input, (byte) 0);
        require(value.categories().get(4).uuid().equals("raw\0ID"));
        for (String shape : List.of("null", "true", "4", "\"text\"", "[]", "{}",
                "{\"cats\":[],\"globalStates\":false,\"operatingModes\":null}")) {
            StructureMetadata empty = operation.run(shape.getBytes(StandardCharsets.UTF_8));
            require(empty.categories().isEmpty() && empty.globalStates().isEmpty()
                    && empty.operatingModes().isEmpty());
        }
        byte[][] malformed = { "{\n\"cats\":".getBytes(StandardCharsets.UTF_8),
                "{} secret".getBytes(StandardCharsets.UTF_8), {34, (byte)255, 34} };
        long[][] positions = {{2,7}, {1,4}, {1,3}};
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
        values(LoxArchive::parseLoxapp3StructureMetadata);
        NativeMetadataProbe.verify("Java", LoxArchive::parseLoxapp3StructureMetadata);
        System.out.println("Java metadata typed values/raw ordering/omissions/errors/input independence PASS");
    }
}
