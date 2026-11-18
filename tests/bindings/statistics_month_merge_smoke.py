"""Installed month merge contract and actual success/error RustBuffer ownership."""

import ctypes
import gc
import os
from pathlib import Path
import struct
import sys

import lox_archive as lox
from lox_archive import lox_archive as native

assert "RUST_MIN_STACK" not in os.environ
assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())


def bits(value):
    return struct.unpack(">Q", struct.pack(">d", value))[0]


def part(month, name, group, outputs, *rows):
    return lox.StatMonthPart(
        month=month,
        series=lox.StatSeries(
            name=name, group=group, outputs=list(outputs),
            rows=[lox.StatRow(timestamp=stamp, values=values) for stamp, values in rows]))


def check_failure(failure, name, previous_month, month, previous_outputs, outputs):
    assert (failure.name, failure.previous_month, failure.month) == (name, previous_month, month)
    assert failure.previous_outputs == previous_outputs
    assert failure.outputs == outputs


empty = lox.merge_statistics_months([])
assert (empty.name, empty.group, empty.outputs, empty.rows) == ("", None, [], [])
labels = [" watts ", "", " watts "]
stamp = " \0e\u0301\r\n "
witness = [0x8000000000000000, 1, 0x7ff0000000000000, 0xfff0000000000000,
           0x7ff8000000001234, 0xfff8000000005678]
values = [struct.unpack(">d", struct.pack(">Q", word))[0] for word in witness]
parts = [
    part("z\0", "first \0", 4294967295, labels, (stamp, values), (stamp, [])),
    part("", "ignored", 0, labels, (stamp, [99.0]), ("", [2.0]), ("", [3.0])),
    part("z\0", "", None, labels, ("", [99.0]), ("a", [4.0, 5.0])),
]
merged = lox.merge_statistics_months(parts)
values[:] = [99.0]
parts[0].series.outputs.clear()
parts.clear()
assert (merged.name, merged.group, merged.outputs) == ("first \0", 4294967295, labels)
assert [row.timestamp for row in merged.rows] == [stamp, stamp, "", "", "a"]
assert [bits(v) for v in merged.rows[0].values] == witness
assert [row.values for row in merged.rows[1:]] == [[], [2.0], [3.0], [4.0, 5.0]]

different = ["", " watts ", " watts "]
try:
    lox.merge_statistics_months([
        part("first", "raw \0name", 0, labels, ("x", [])),
        part(" empty \0", "ignored", None, labels),
        part("bad\0 ", "also ignored", None, different, ("x", [])),
    ])
except lox.StatisticsMergeError.OutputMismatch as failure:
    check_failure(failure, "raw \0name", " empty \0", "bad\0 ", labels, different)
else:
    raise AssertionError("Exact output-vector mismatch required")

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count)) == 0
    return count.value


def check_value(value, marker):
    assert (value.name, value.group, value.outputs) == (marker, None, [" A ", ""])
    assert len(value.rows) == 1 and value.rows[0].timestamp == "raw"
    assert [bits(v) for v in value.rows[0].values] == [1 << 63]


gc.disable()
for error in (False, True):
    marker = f"month-merge-owned-{int(error)}-" + "x" * 65536
    parts = [part("first", marker, None, [" A ", ""], ("raw", [-0.0]))]
    if error:
        parts += [
            part("empty", "ignored", 0, [" A ", ""]),
            part("bad", "ignored", None, ["", " A "], ("raw", [])),
        ]
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_merge_statistics_months(
        native._UniffiFfiConverterSequenceTypeStatMonthPart.lower(parts), ctypes.byref(status))
    assert status.code == int(error)
    buffer = status.error_buf if error else result
    length = buffer.len
    assert length >= 64000
    prefix = ctypes.string_at(buffer.data, 128)
    assert matching(prefix) == 1
    if error:
        buffer.free()
    result.free()
    assert matching(prefix) == 0
    retained = []
    for _ in range(64):
        try:
            value = lox.merge_statistics_months(parts)
        except lox.StatisticsMergeError.OutputMismatch as failure:
            assert error
            retained.append(failure)
        else:
            assert not error
            retained.append(value)
    parts.clear()
    assert len(retained) == 64 and len({id(value) for value in retained}) == 64
    assert matching(prefix) == 0
    for value in retained:
        if error:
            check_failure(value, marker, "empty", "bad", [" A ", ""], ["", " A "])
        else:
            check_value(value, marker)
    print(f"Python statistics-month-merge error={error} actual_result_bytes={length} "
          "live=1 free=0 calls=64 host_retained=64 native_retained=0 GC_requested=false")
print("Python statistics-month-merge opaque order/rows/float bits/exact typed errors/ownership PASS")
