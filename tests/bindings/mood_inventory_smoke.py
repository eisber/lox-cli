"""Installed mood inventory values, typed failures, and this export's RustBuffers."""

import ctypes
import gc
import os
from pathlib import Path
import platform
import sys
import sysconfig

import lox_archive as lox
from lox_archive import lox_archive as native

assert sys.platform == "win32" and platform.machine() == "AMD64"
assert sys.implementation.name == "cpython" and sys.version_info[:2] == (3, 13)
assert not sysconfig.get_config_var("Py_GIL_DISABLED") and sys._is_gil_enabled()


def outcome(data, selector):
    try:
        return lox.list_moods(lox.MoodInventoryRequest(bytes=data, selector=selector))
    except lox.MoodInventoryError as failure:
        return failure


def check(value, branch, expected):
    if branch == "moods":
        assert isinstance(value, list) and all(isinstance(m, lox.MoodInfo) for m in value)
        assert [(m.name, m.uuid, m.sid, m.cid, m.q1) for m in value] == expected
    elif branch == "load":
        assert isinstance(value, lox.MoodInventoryError.Load) and value.source_message
    elif branch == "not-found":
        assert isinstance(value, lox.MoodInventoryError.NotFound)
        assert (value.selector, value.titles) == expected
    elif branch == "ambiguous":
        assert isinstance(value, lox.MoodInventoryError.Ambiguous)
        assert all(isinstance(m, lox.MoodElementIdentity) for m in value.matches)
        assert (value.selector, [(m.title, m.block_type, m.uuid) for m in value.matches]) == expected
    elif branch == "not-light-controller":
        assert isinstance(value, lox.MoodInventoryError.NotLightController)
        assert (value.selector, value.block_type) == expected
    else:
        raise AssertionError(f"Unknown branch {branch}")


xml = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
       / "tests" / "fixtures" / "mood_inventory.xml").read_bytes()
rows = [
    ("Outer & 灯", "same", 2, -3, "00037"),
    ("Inner", "same", 2, None, ""),
    ("", "", None, None, None),
    ("", "", None, (1 << 63) - 1, "opaque&unparsed"),
]
titles = ["Case only", "Kitchen", "Kitchen lamps", "Mixed lights"]
for selector in ("uuid:lc", "gid:LIGHT", "Lamps [kit]"):
    check(outcome(xml, selector), "moods", rows)
check(outcome(xml, "uuid:empty"), "moods", [])
check(outcome(xml, "uuid:LC"), "not-found", ("uuid:LC", titles))
check(outcome(xml, "Mixed lights"), "ambiguous",
      ("Mixed lights", [("Mixed lights", "And", "a"), ("Mixed lights", "LightController2", "b")]))
check(outcome(xml, "uuid:case"), "not-light-controller", ("uuid:case", "lightcontroller2"))
check(outcome(xml, "uuid:sparse"), "not-light-controller", ("uuid:sparse", ""))
malformed = b"<Root>\n  <C Duplicate='1' Duplicate='2'/>\n</Root>"
check(outcome(malformed, "missing"), "load", None)

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_count.restype = ctypes.c_uint32
probe.lox_test_heap_live.argtypes = (ctypes.c_void_p,)
probe.lox_test_heap_live.restype = ctypes.c_int


def matching(prefix, minimum):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), minimum, ctypes.byref(count)) == 0
    return count.value


def lower(data, selector):
    return native._UniffiFfiConverterTypeMoodInventoryRequest.lower(
        lox.MoodInventoryRequest(bytes=data, selector=selector))


def calibrate(branch, data, selector, tag, marker):
    success = branch == "moods"
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_list_moods(
        lower(data, selector), ctypes.byref(status))
    assert status.code == (0 if success else 1)
    buffer = result if success else status.error_buf
    try:
        length = buffer.len
        prefix_length, minimum = (16, length) if branch == "load" else (128, 64000)
        assert length >= max(2 * prefix_length, minimum)
        address = ctypes.cast(buffer.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, prefix_length)
        assert int.from_bytes(prefix[:4], "big") == tag
        if marker is not None:
            assert marker.encode() in prefix
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        if not success:
            buffer.free()
        result.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0
    return prefix, minimum, length


def rejected_selector_releases_input():
    data = b"<!--lox-mood-inventory-rejected-selector-input:" + b"m" * 65536 + b"--><Root/>"
    minimum = 4 + len(data)
    control = lower(data, "uuid:lc")
    try:
        assert control.len >= minimum
        address = ctypes.cast(control.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, 128)
        assert prefix == len(data).to_bytes(4, "big") + data[:124]
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        control.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0

    request = lox.MoodInventoryRequest(bytes=data, selector="\ud800")
    for _ in range(64):
        try:
            lox.list_moods(request)
        except UnicodeEncodeError as failure:
            assert failure.encoding == "utf-8" and failure.object == request.selector
            assert (failure.start, failure.end) == (0, 1)
        else:
            raise AssertionError("Expected strict host selector-encoding refusal")
    assert matching(prefix, minimum) == 0, "Rejected mood selector retained marked XML"
    print("Python mood-inventory invalid selector: input_live=1 input_free=0 "
          "rejections=64 input_native_retained=0 GC_requested=false")


q1 = "mood-inventory-owned-Q1-Καλημέρα-" + "x" * 65536
missing = "mood-inventory-missing-" + "y" * 65536
long_title = "mood-inventory-matched-title-" + "z" * 65536
wrong_type = "mood-inventory-wrong-type-" + "w" * 65536
trials = [
    ("moods", xml.replace(b'Q1="00037"', f'Q1="{q1}"'.encode(), 1), "uuid:lc",
     [(*rows[0][:4], q1), *rows[1:]], 4, "mood-inventory-owned-Q1-"),
    ("load", malformed, "missing", None, 1, None),
    ("not-found", xml, missing, (missing, titles), 2, "mood-inventory-missing-"),
    ("ambiguous", (f'<Root><C Type="And" U="a" Title="{long_title}"/>'
                   f'<C Type="LightController2" U="b" Title="{long_title}"/></Root>').encode(),
     "matched-title-", ("matched-title-", [(long_title, "And", "a"),
                                           (long_title, "LightController2", "b")]),
     3, "mood-inventory-matched-title-"),
    ("not-light-controller", f'<C Type="{wrong_type}" U="w"/>'.encode(),
     "uuid:w", ("uuid:w", wrong_type), 4, "mood-inventory-wrong-type-"),
]
assert "RUST_MIN_STACK" not in os.environ
gc.disable()
for branch, data, selector, expected, tag, marker in trials:
    prefix, minimum, length = calibrate(branch, data, selector, tag, marker)
    retained = [outcome(data, selector) for _ in range(64)]
    assert len({id(value) for value in retained}) == 64
    assert matching(prefix, minimum) == 0, f"{branch} retained a native result buffer"
    for value in retained:
        check(value, branch, expected)
    if branch == "load":
        assert len({value.source_message for value in retained}) == 1
    if branch == "moods":
        assert len({id(value[0]) for value in retained}) == 64
    print(f"Python mood-inventory branch={branch} actual_bytes={length} live=1 free=0 "
          "calls=64 host_retained=64 native_retained=0 GC_requested=false")
rejected_selector_releases_input()
print("Python mood-inventory order/defaults/duplicates/empty/typed-errors PASS")
