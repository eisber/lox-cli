"""Actual generated device exports, owned values and calibrated RustBuffer release."""

import ctypes
import os
from pathlib import Path
import sys

import lox_archive as lox

try:
    from lox_archive import lox_archive as native
except ImportError:
    # Direct generated-module admission precedes the independently installed wheel.
    native = lox


assert "RUST_MIN_STACK" not in os.environ
xml = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
       / "tests" / "fixtures" / "xml_devices.xml").read_bytes()
before = bytes(xml)
rows = lox.parse_devices(xml)
assert xml == before
assert all(isinstance(row, lox.XmlDeviceInfo) for row in rows)
assert [(r.name, r.bus, r.serial, r.sub_type, r.type_label, r.mac, r.address)
        for r in rows] == [
    (" raw&amp;&#65;&custom; ", lox.XmlDeviceBus.TREE, "", 4294967295,
     "Unknown (4294967295)", None, None),
    ("same", lox.XmlDeviceBus.AIR, None, 7, "Nano IO Air", None, None),
    ("same", lox.XmlDeviceBus.NETWORK, None, 4294967295, "Network device", "", "raw&amp;"),
    ("", lox.XmlDeviceBus.TREE, None, None, "Unknown", None, None),
    ("repeat", lox.XmlDeviceBus.TREE, None, None, "Unknown", None, None),
    ("repeat", lox.XmlDeviceBus.TREE, None, None, "Unknown", None, None),
    ("", lox.XmlDeviceBus.TREE, None, None, "Unknown", None, None),
    ("", lox.XmlDeviceBus.AIR, None, 32794, "Presence sensor", None, None),
    ("", lox.XmlDeviceBus.NETWORK, None, 7, "Network device", None, None),
    ("", lox.XmlDeviceBus.TREE, None, 7, "Nano IO Air", None, None),
    ("before-bad", lox.XmlDeviceBus.TREE, None, None, "Unknown", None, None),
]
xml = b"replaced"
for empty in (b"", b"plain text"):
    assert lox.parse_devices(empty) == []
assert rows[0].sub_type == 4294967295 and rows[0].serial == ""
assert lox.parse_devices(b'<C Type="TreeDevice" Title="open">')[0].name == "open"
invalid = lox.parse_devices(b'<C Type="TreeDevice" Title="\xff" Serial="\xff"/>')[0]
assert invalid.name == "" and invalid.serial is None
for malformed in (
    b'<Root><C Type="TreeDevice" Title="not partial"/><secret>not-an-error-payload</Root>',
    b"</unexpected>",
):
    try:
        lox.parse_devices(malformed)
    except lox.ConfigurationError.Xml as error:
        assert error.source_message and "not-an-error-payload" not in error.source_message
    else:
        raise AssertionError("Expected reader failure without partial results")

probe = ctypes.CDLL(sys.argv[1])
count = probe.lox_test_heap_count
count.argtypes = [ctypes.c_void_p, ctypes.c_uint64, ctypes.c_uint64,
                  ctypes.POINTER(ctypes.c_uint64)]
count.restype = ctypes.c_int


def matching(prefix):
    owned = ctypes.create_string_buffer(prefix)
    result = ctypes.c_uint64()
    assert count(owned, len(prefix), 64000, ctypes.byref(result)) == 0
    return result.value


marker = "xml-devices-python-allocation-" + "x" * 65536
for fail in (False, True):
    data = (f"<{marker}></other>" if fail else
            f'<C Type="TreeDevice" Title="{marker}" Serial="" SubType="4294967295"/>').encode()
    status = native._UniffiRustCallStatus()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_devices(
        native._UniffiFfiConverterBytes.lower(data), ctypes.byref(status))
    assert bool(status.code) == fail
    buffer = status.error_buf if fail else result
    assert buffer.len >= 64000
    prefix = ctypes.string_at(buffer.data, 128)
    live = matching(prefix)
    print(f"Actual device export error={fail} len={buffer.len} cap={buffer.capacity} live={live}",
          flush=True)
    assert live == 1
    if fail:
        buffer.free()
    result.free()
    assert matching(prefix) == 0
    for _ in range(64):
        try:
            actual = lox.parse_devices(data)
            assert not fail and len(actual) == 1
            value = actual[0]
            assert (value.name, value.bus, value.serial, value.sub_type,
                    value.type_label, value.mac, value.address) == (
                        marker, lox.XmlDeviceBus.TREE, "", 4294967295,
                        "Unknown (4294967295)", None, None)
        except lox.ConfigurationError.Xml as error:
            assert fail and len(error.source_message) > 65536
    retained = matching(prefix)
    print(f"Python devices error={fail} calls=64 calibrated_live=1 "
          f"calibrated_released=0 retained_buffers={retained} GC_requested=false", flush=True)
    assert retained == 0
print("Python device inventory: raw typed values/defaults/unsigned/order/errors/ownership PASS")
