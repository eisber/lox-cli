import net.eisber.lox.archive.*

fun main() {
    val rows: List<MqttTopic> = LoxArchive.listMqttTopics(
        """<C Type="GenTActor"><SET><mqtt_topic v=" raw/+ "/><mqtt_qos v="nonnumeric"/></SET></C>"""
            .toByteArray(Charsets.UTF_8))
    check(rows.single() == MqttTopic("", MqttDirection.PUBLISH, " raw/+ ", "nonnumeric"))
    val operation = NativeMqttCatalogProbe.Operation { input -> LoxArchive.listMqttTopics(input) }
    MqttCatalogSmoke.values(operation)
    NativeMqttCatalogProbe.verify("Kotlin", operation)
    println("Kotlin SAME Java MQTT API typed values/order/duplicates/raw QoS/defaults/load errors PASS")
}
