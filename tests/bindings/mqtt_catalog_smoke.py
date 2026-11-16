"""Installed native MQTT values, actual load errors and per-call RustBuffer release."""

import ctypes
import gc
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


payload = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
           / "tests" / "fixtures" / "mqtt_catalog.xml").read_bytes()
before = bytes(payload)
rows = lox.list_mqtt_topics(payload)
assert payload == before and all(isinstance(v, lox.MqttTopic) for v in rows)
assert [(v.title, v.direction, v.topic, v.qos) for v in rows] == [
    ("Küche & 灯", lox.MqttDirection.PUBLISH, "", " raw "),
    ("", lox.MqttDirection.SUBSCRIBE, " /raw/+ ", ""),
    ("", lox.MqttDirection.SUBSCRIBE, " /raw/+ ", "")]
rows[0].title = "changed"
assert lox.list_mqtt_topics(before)[0].title == "Küche & 灯"
assert lox.list_mqtt_topics(b"<Root/>") == []
assert lox.list_mqtt_topics(
    b'<C Type="GenTSensor" Title="literal 12hTF=x" 12hTF="true"/>'
)[0].title == "literal _12hTF=x"  # Existing lox-n37w, not native XML acceptance.
for malformed in (b"<C>", b"<C Type='a' Type='b'/>", b"<C Title='\xff'/>"):
    try:
        lox.list_mqtt_topics(malformed)
    except lox.MqttCatalogError.Load as failure:
        assert failure.source_message
    else:
        raise AssertionError("Expected existing Core loader refusal")

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, minimum):
    count = ctypes.c_size_t()
    code = probe.lox_test_heap_count(prefix, len(prefix), minimum, ctypes.byref(count))
    assert code == 0
    return count.value


marker = "mqtt-owned-allocation-" + "x" * 65536
gc.disable()
for error in (False, True):
    payload = (b"<C>" if error else
               ('<C Type="GenTSensor"><SET><mqtt_topic v="' + marker + '"/></SET></C>').encode())
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_list_mqtt_topics(
        native._UniffiFfiConverterBytes.lower(payload), ctypes.byref(status))
    assert status.code == int(error)
    buffer = status.error_buf if error else result
    length = buffer.len
    assert length >= 16
    prefix = ctypes.string_at(buffer.data, 16 if error else 128)
    minimum = length if error else 64000
    assert matching(prefix, minimum) == 1
    if error:
        buffer.free()
    result.free()
    assert matching(prefix, minimum) == 0
    saved_failure = None
    saved_value = None
    for _ in range(64):
        try:
            value = lox.list_mqtt_topics(payload)
            assert not error and value[0].topic == marker
            saved_value = value
        except lox.MqttCatalogError.Load as failure:
            assert error and failure.source_message
            if saved_failure is None:
                saved_failure = failure
            assert failure.source_message == saved_failure.source_message
    assert matching(prefix, minimum) == 0
    if error:
        assert saved_failure.source_message
    else:
        assert saved_value[0].topic == marker
    print(f"Python MQTT error={error} actual_result_bytes={length} live=1 free=0 "
          "calls=64 retained=0 GC_requested=false")
print("Python MQTT typed values/order/duplicates/raw QoS/defaults/load errors PASS")
