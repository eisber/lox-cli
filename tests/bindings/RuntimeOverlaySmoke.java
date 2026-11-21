import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.List;
import net.eisber.lox.archive.*;

public final class RuntimeOverlaySmoke {
    public static byte[] bytes(String value) {
        return value.getBytes(StandardCharsets.UTF_8);
    }

    static void require(boolean value) {
        if (!value) throw new AssertionError("Offline three-way contract");
    }

    public static byte[][] witness(String id) {
        String a = "<C U=\"a\" Type=\"And\">%s</C>";
        String b = "<C U=\"b\" Type=\"And\">%s</C>";
        String child = "<C U=\"" + id + "\" Type=\"And\"/>";
        return new byte[][] {
            bytes("<Root>" + a.formatted("") + b.formatted("") + "</Root>"),
            bytes("<Root>" + a.formatted(child) + b.formatted("") + "</Root>"),
            bytes("<Root>" + a.formatted("") + b.formatted(child) + "</Root>")
        };
    }

    public static void chain(OverlayException.Refused failure, String id) {
        List<OverlayErrorFrame> frames = failure.frames();
        require(frames.size() == 2 && frames.get(0) instanceof OverlayErrorFrame.MergedConsistency);
        require(frames.get(1) instanceof OverlayErrorFrame.DuplicateUuid duplicate
                && duplicate.id().equals(id));
    }

    public static void values(NativeRuntimeOverlayProbe.Operation operation) throws Throwable {
        byte[] base = bytes("<Root><C U=\"a\" Type=\"And\" V=\"0\"/><C U=\"b\" Type=\"Or\" V=\"0\"/></Root>");
        byte[] local = bytes("\ufeff<Root>\r\n<C V=\"1\" U=\"a\" Type=\"And\"></C><C U=\"b\" Type=\"Or\" V=\"0\"/>\r\n</Root>");
        byte[] live = bytes("<Root><C U=\"a\" Type=\"And\" V=\"0\"/><C U=\"b\" Type=\"Or\" V=\"2\"/></Root>");
        byte[] expected = bytes("<Root><C V=\"1\" U=\"a\" Type=\"And\"></C><C U=\"b\" Type=\"Or\" V=\"2\"/></Root>");
        byte[] before = local.clone();
        byte[] owned = operation.run(base, local, live);
        require(Arrays.equals(owned, expected) && Arrays.equals(local, before));
        require(Arrays.equals(operation.run(base, base, local), local));
        require(Arrays.equals(operation.run(base, local, local), local));
        try {
            operation.run(base, local, bytes(new String(base, StandardCharsets.UTF_8).replaceFirst("V=\"0\"", "V=\"3\"")));
            throw new AssertionError("Typed shell conflict required");
        } catch (OverlayException.Refused failure) {
            require(failure.frames().size() == 1
                    && failure.frames().get(0) instanceof OverlayErrorFrame.ShellConflict conflict
                    && conflict.id().equals("a"));
        }
        for (String invalid : List.of("<Root a=\"1\" a=\"2\"/>", "<Root></wrong>")) {
            try {
                operation.run(bytes(invalid), bytes("<Root/>"), bytes("<Root/>"));
                throw new AssertionError("Typed native parser boundary required");
            } catch (OverlayException.Refused failure) {
                require(failure.frames().size() == 1);
                OverlayErrorFrame frame = failure.frames().get(0);
                if (invalid.contains("a=")) {
                    require(frame instanceof OverlayErrorFrame.Attribute value
                            && value.context() == OverlayXmlContext.DOCUMENT && !value.sourceMessage().isEmpty());
                } else {
                    require(frame instanceof OverlayErrorFrame.Xml value
                            && value.context() == OverlayXmlContext.DOCUMENT && !value.sourceMessage().isEmpty());
                }
            }
        }
        try {
            operation.run(new byte[] {'<', (byte) 0xff}, bytes("<Root/>"), bytes("<Root/>"));
            throw new AssertionError("Typed UTF-8 boundary required");
        } catch (OverlayException.Refused failure) {
            require(failure.frames().size() == 1
                    && failure.frames().get(0) instanceof OverlayErrorFrame.DocumentUtf8 value
                    && value.validUpTo() == 1L && value.errorLen() == 1L);
        }
        byte[][] duplicate = witness("c");
        try {
            operation.run(duplicate[0], duplicate[1], duplicate[2]);
            throw new AssertionError("Merged consistency cause required");
        } catch (OverlayException.Refused failure) {
            chain(failure, "c");
        }
        Arrays.fill(local, (byte) 0);
        require(Arrays.equals(owned, expected));
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::threeWay);
        NativeRuntimeOverlayProbe.verify("Java", LoxArchive::threeWay);
        System.out.println("Java runtime-overlay selected-bytes/typed-two-frame-causes/native-boundaries/owned PASS");
    }
}
