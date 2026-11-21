"""Installed generated wheel, including calibrated native RustBuffer release."""

import ctypes
import os
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


assert "RUST_MIN_STACK" not in os.environ
xml = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
       / "tests" / "fixtures" / "xml_controls.xml").read_bytes()
before = bytes(xml)
rows = lox.parse_controls(xml, None, None)
assert xml == before
assert all(isinstance(row, lox.XmlControlInfo) for row in rows)
assert [(r.control_type, r.title, r.uuid, r.room, r.category) for r in rows] == [
    ("", "", "", "Default", "Empty ID"),
    ("Other", "Inner", "inner", "KÜCHE", "Raw &amp; &#65;"),
    ("Page", "Page", "page", "page-room", "Empty ID"),
    ("Switch", "Pending", "pending", "KÜCHE", "Raw &amp; &#65;"),
    ("Switch", "Same", "second", "KÜCHE", "Raw &amp; &#65;"),
    ("Switch", "Same", "first", "KÜCHE", "Empty ID"),
    ("Switch", "Same", "first", "raw&amp;id", "unknown"),
    ("TreeDevice", "Device", "device", "Default", "Empty ID"),
    ("switch", "Z", "lower", "Default", "Empty ID"),
    ("Ä", " A&amp;&#65;&custom; ", "室", "Default", "Empty ID"),
]
assert [r.uuid for r in lox.parse_controls(xml, "sWiTcH", "küc")] == [
    "outer", "pending", "second", "first"]
assert lox.parse_controls(xml, None, "raw&amp;")[0].category == "unknown"
assert len(lox.parse_controls(xml, "", None)) == 1
assert [r.uuid for r in lox.parse_controls(xml, None, "")] == [r.uuid for r in rows]
for typ in ("ä", "Sw", "absent"):
    assert lox.parse_controls(xml, typ, None) == []
xml = b"replaced"
for empty in (b"", b"plain text"):
    assert lox.parse_controls(empty, None, None) == []
assert rows[1].title == "Inner"
assert lox.parse_controls(
    b'<C Type="Switch" U="c" Title="Open"><IoData Pr="r">', None, None
)[0].room == "r"
assert lox.parse_controls(b'<C Type="Switch" U="c" Title="\xff"/><IoData/>', None, None) == []
for malformed in (
    b'<Root><C Type="Switch" U="c" Title="ok"/><IoData/><secret>not-an-error-payload</Root>',
    b"</unexpected>",
):
    try:
        lox.parse_controls(malformed, None, None)
    except lox.ConfigurationError.Xml as error:
        assert error.source_message and "not-an-error-payload" not in error.source_message
    else:
        raise AssertionError("Expected reader failure, not partial success")

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


marker = "xml-controls-python-allocation-" + "x" * 65536
for fail in (False, True):
    data = (f"<{marker}></other>" if fail else
            f'<C Type="Switch" U="c" Title="{marker}"/><IoData Pr="r" Cr="cat"/>').encode()
    status = native._UniffiRustCallStatus()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_controls(
        native._UniffiFfiConverterBytes.lower(data),
        native._UniffiFfiConverterOptionalString.lower("SWITCH"),
        native._UniffiFfiConverterOptionalString.lower("r"), ctypes.byref(status))
    assert bool(status.code) == fail
    buffer = status.error_buf if fail else result
    assert buffer.len >= 64000
    prefix = ctypes.string_at(buffer.data, 128)
    assert matching(prefix) == 1
    if fail:
        buffer.free()
    result.free()
    assert matching(prefix) == 0
    for _ in range(64):
        try:
            actual = lox.parse_controls(data, "SWITCH", "r")
            assert not fail and len(actual) == 1
            value = actual[0]
            assert (value.control_type, value.title, value.uuid, value.room, value.category) == (
                "Switch", marker, "c", "r", "cat")
        except lox.ConfigurationError.Xml as error:
            assert fail and len(error.source_message) > 65536
    retained = matching(prefix)
    print(f"Python XML controls error={fail} calls=64 calibrated_live=1 "
          f"calibrated_released=0 retained_buffers={retained} GC_requested=false")
    assert retained == 0
print("Python XML controls: full raw records/filters/association/errors/ownership PASS")
