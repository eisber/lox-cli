"""Installed infallible HTML index values and actual returned RustBuffer ownership."""

import ctypes
import gc
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native

uuid = "01234567-89ab-cdef-0123456789abcdef"
html = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
        / "tests" / "fixtures" / "statistics_index.html").read_text(encoding="utf-8")
rows = lox.parse_statistics_index(html)
assert all(isinstance(row, lox.StatEntry) for row in rows)
assert [(v.file, v.uuid, v.group, v.month, v.name, v.label) for v in rows] == [
    (uuid + ".202613.xml", uuid, None, "2026-13", "Minimal", "Minimal"),
    (uuid + "_0.000000.xml", uuid, 0, "0000-00", "Zero", "Zero (Statistic Group: 0) (Room)"),
    (uuid + "_4294967295.999999.xml", uuid, 4294967295, "9999-99", "Max",
     "Max (Statistic Group: 4294967295) (Room)"),
    *[(uuid + ".202610.xml", uuid, None, "2026-10", "Dup", "Dup (Room)")] * 2,
    (uuid + ".202610.xml", uuid, None, "2026-10", "A &amp; B &unknown;", "A &amp; B &unknown;"),
    (uuid + ".202610.xml", uuid, None, "2026-10", "é é\r\nline & !", "é é\r\nline & !")]
rows[0].name = "changed"
assert lox.parse_statistics_index(html)[0].name == "Minimal"
for malformed in ("", "<LI>", "<li>broken", "\0é"):
    assert lox.parse_statistics_index(malformed) == []
raw = "raw\0é é\r\nvalue"
assert lox.parse_statistics_index(f'<li><a href="{uuid}.000000.xml">{raw}</a>')[0].name == raw
assert lox.parse_statistics_index(
    f'<li><a href="{uuid.upper()}.202610.xml">bad</a>'
    f'<li><a href="{uuid}.202610.xml">good</a>')[0].name == "good"

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count)) == 0
    return count.value


marker = "index-owned-allocation-" + "x" * 65536
payload = f'<li><a href="{uuid}.202610.xml">{marker}</a>'
gc.disable()
status = native._UniffiRustCallStatus.default()
result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_statistics_index(
    native._UniffiFfiConverterString.lower(payload), ctypes.byref(status))
assert status.code == 0 and result.len >= 64000
prefix = ctypes.string_at(result.data, 128)
assert matching(prefix) == 1
length = result.len
result.free()
assert matching(prefix) == 0
for _ in range(64):
    value = lox.parse_statistics_index(payload)
    assert len(value) == 1 and value[0].name == marker and value[0].label == marker
    assert value[0].file == uuid + ".202610.xml" and value[0].group is None
assert matching(prefix) == 0
print(f"Python statistics-index actual_result_bytes={length} live=1 free=0 "
      "calls=64 retained=0 GC_requested=false String_lowering=generated PASS")
