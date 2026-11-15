"""Actual generated user inventory exports and owned success/error buffers."""

import ctypes
import os
from pathlib import Path
import sys

import lox_archive as lox

try:
    from lox_archive import lox_archive as native
except ImportError:
    native = lox


assert "RUST_MIN_STACK" not in os.environ
xml = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
       / "tests" / "fixtures" / "xml_users.xml").read_bytes()
before = bytes(xml)
rows = lox.parse_users(xml)
assert xml == before
assert all(isinstance(row, lox.XmlUserInfo) for row in rows)
assert [(r.name, r.uuid, r.nfc, r.description) for r in rows] == [
    (" raw&amp;&#65;&custom; ", "not_a_uuid", True, "one  two &#95; &amp;"),
    ("", "", False, ""),
    ("repeat", "same", False, " "),
    ("repeat", "same", False, " "),
    ("first", "", True, "a b"),
    ("Grün", "raw_under", True, "Grüße 世界"),
]
xml = b"replaced"
assert rows[0].uuid == "not_a_uuid"
for empty in (b"", b"plain text"):
    assert lox.parse_users(empty) == []
assert lox.parse_users(b'<C Type="User" Title="open">')[0].name == "open"
assert lox.parse_users(b'<C Type="User" Title="before-bad" bad />')[0].name == "before-bad"
invalid = lox.parse_users(
    b'<C Type="User" Title="\xff" U="valid_raw" Desc="ok_value" NFCArr="x"/>'
    b'<C Type="User" Title="valid" U="\xff" Desc="\xff" NFCArr="\xff"/>'
    b'<C Type="\xff" Title="excluded"/>'
    b'<C Type="User" Title="\xff" Title="not-a-fallback"/>')
assert [(r.name, r.uuid, r.nfc, r.description) for r in invalid] == [
    ("", "valid_raw", True, "ok value"), ("valid", "", False, ""), ("", "", False, "")]
for malformed in (
    b'<Root><C Type="User" Title="not partial"/><secret>not-an-error-payload</Root>',
    b"</unexpected>",
):
    try:
        lox.parse_users(malformed)
    except lox.ConfigurationError.Xml as error:
        assert error.source_message and "not-an-error-payload" not in error.source_message
        if malformed.startswith(b"<Root>"):
            assert error.source_message == (
                "ill-formed document: expected `</secret>`, but `</Root>` was found")
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


marker = "xml-users-python-allocation-" + "x" * 65536
for fail in (False, True):
    data = (f"<{marker}></other>" if fail else
            f'<C Type="User" Title="{marker}" U="raw_uuid" Desc="raw_desc" NFCArr="0"/>').encode()
    status = native._UniffiRustCallStatus()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_users(
        native._UniffiFfiConverterBytes.lower(data), ctypes.byref(status))
    assert bool(status.code) == fail
    buffer = status.error_buf if fail else result
    assert buffer.len >= 64000
    prefix = ctypes.string_at(buffer.data, 128)
    live = matching(prefix)
    print(f"Actual user export error={fail} len={buffer.len} cap={buffer.capacity} live={live}",
          flush=True)
    assert live == 1
    if fail:
        buffer.free()
    result.free()
    assert matching(prefix) == 0
    for _ in range(64):
        try:
            actual = lox.parse_users(data)
            assert not fail and len(actual) == 1
            value = actual[0]
            assert (value.name, value.uuid, value.nfc, value.description) == (
                marker, "raw_uuid", True, "raw desc")
        except lox.ConfigurationError.Xml as error:
            assert fail and len(error.source_message) > 65536
    retained = matching(prefix)
    print(f"Python users error={fail} calls=64 calibrated_live=1 "
          f"calibrated_released=0 retained_buffers={retained} GC_requested=false", flush=True)
    assert retained == 0
print("Python user inventory: raw typed values/defaults/UTF8/order/errors/ownership PASS")
