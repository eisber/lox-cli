import net.eisber.lox.archive.*;
import java.nio.file.*;
import java.nio.charset.StandardCharsets;
import java.util.*;

public final class MqttCatalogSmoke {
    static void require(boolean value) {
        if (!value) throw new AssertionError("MQTT values");
    }

    public static void values(NativeMqttCatalogProbe.Operation operation) throws Throwable {
        byte[] input = Files.readAllBytes(Path.of(
                "crates", "lox-bindings", "tests", "fixtures", "mqtt_catalog.xml"));
        byte[] before = input.clone();
        List<MqttTopic> rows = operation.run(input);
        require(Arrays.equals(input, before));
        require(rows.equals(List.of(
                new MqttTopic("Küche & 灯", MqttDirection.PUBLISH, "", " raw "),
                new MqttTopic("", MqttDirection.SUBSCRIBE, " /raw/+ ", ""),
                new MqttTopic("", MqttDirection.SUBSCRIBE, " /raw/+ ", ""))));
        Arrays.fill(input, (byte) 0);
        require(operation.run(before).equals(rows));
        require(operation.run("<Root/>".getBytes(StandardCharsets.UTF_8)).isEmpty());
        require(operation.run(("<C Type=\"GenTSensor\" Title=\"literal 12hTF=x\" "
                + "12hTF=\"true\"/>").getBytes(StandardCharsets.UTF_8))
                .get(0).title().equals("literal _12hTF=x"));
        for (byte[] malformed : new byte[][] {
                "<C>".getBytes(StandardCharsets.UTF_8),
                "<C Type='a' Type='b'/>".getBytes(StandardCharsets.UTF_8),
                {60, 67, 32, 84, 105, 116, 108, 101, 61, 39, (byte)255, 39, 47, 62}}) {
            try {
                operation.run(malformed);
                throw new AssertionError("Expected Core loader refusal");
            } catch (MqttCatalogException.Load failure) {
                require(!failure.sourceMessage().isEmpty());
            }
        }
    }

    public static void main(String[] args) throws Throwable {
        values(LoxArchive::listMqttTopics);
        NativeMqttCatalogProbe.verify("Java", LoxArchive::listMqttTopics);
        System.out.println("Java MQTT typed values/order/duplicates/raw QoS/defaults/load errors PASS");
    }
}
