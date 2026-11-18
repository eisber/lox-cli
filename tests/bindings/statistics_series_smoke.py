"""Installed monthly XML series values, typed failures and actual RustBuffer disposal."""

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
    return struct.unpack("<Q", struct.pack("<d", value))[0]


xml = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
       / "tests" / "fixtures" / "statistics_series.xml").read_text(encoding="utf-8")
series = lox.parse_statistics_series(xml)
assert isinstance(series, lox.StatSeries)
assert (series.name, series.group) == ("Café & 気", 4294967295)
assert series.outputs == ["Energy & power", "V2", "A", "A"]
assert all(isinstance(row, lox.StatRow) for row in series.rows)
assert [row.timestamp for row in series.rows] == [
    " opaque & timestamp ", "duplicate", "duplicate"]
assert all(isinstance(value, float) for row in series.rows for value in row.values)
assert [bits(v) for v in series.rows[0].values] == [
    0x8000000000000000, 1, 0x8000000000000001, 0x7fefffffffffffff]
assert series.rows[1].values[:2] == [1.0, 20.0]
assert [bits(v) for v in series.rows[1].values[2:]] == [0, 0x8000000000000000]
assert series.rows[2].values == [4.0, 5.0, 6.0, 7.0]
xml = "replaced"
series.rows[0].values[0] = 9.0
assert bits(lox.parse_statistics_series(
    '<Statistics><S T="raw" V="-0"/></Statistics>').rows[0].values[0]) == 1 << 63
assert lox.parse_statistics_series("<Statistics/>").outputs == ["V"]
assert lox.parse_statistics_series('<Statistics Outputs=""/>').outputs == ["V"]
zero = lox.parse_statistics_series(
    '<Statistics NumOutputs="0" Outputs="ignored"><S T=""/></Statistics>')
assert zero.group is None and zero.outputs == []
assert zero.rows[0].timestamp == "" and zero.rows[0].values == []
lazy = lox.parse_statistics_series(
    '<Statistics Name="n" StatsGroup="0" Outputs="A" NumOutputs="1" broken/>'
    '<S T="outside" V="2" broken/>')
assert lazy.rows[0].timestamp == "outside" and lazy.rows[0].values == [2.0]
assert lox.parse_statistics_series(
    '<Statistics Other="&unknown;"/><Ignored broken/>').rows == []
assert lox.parse_statistics_series('<Statistics StatsGroup="0"/>').group == 0

for payload, variant in (
        ("", lox.StatisticsError.MissingStatistics),
        ("<S/>", lox.StatisticsError.RowBeforeStatistics),
        ("<Statistics/><Statistics StatsGroup=\"bad\"/>", lox.StatisticsError.MultipleStatistics),
        ("<Statistics><S T=\"valid\" V=\"1\"/>", lox.StatisticsError.TruncatedXml),
        ("<Statistics><S/>", lox.StatisticsError.MissingTimestamp),
        ("<Statistics></Other>", lox.StatisticsError.Xml),
        ("<Statistics broken/>", lox.StatisticsError.MalformedAttribute),
        ("<Statistics Name=\"&unknown;\"/>", lox.StatisticsError.AttributeValue),
        ("<Statistics StatsGroup=\"4294967296\"/>", lox.StatisticsError.InvalidStatsGroup),
        ("<Statistics NumOutputs=\"-1\"/>", lox.StatisticsError.InvalidNumOutputs),
        ('<Statistics NumOutputs="2"><S T="first" V="1" V2="2"/>'
         '<S T="later &amp; raw" V="3"/></Statistics>', lox.StatisticsError.MissingValue),
        ('<Statistics><S T="first" V="1"/><S T="later" V=" warm "/></Statistics>',
         lox.StatisticsError.InvalidValue)):
    try:
        lox.parse_statistics_series(payload)
    except variant as failure:
        if hasattr(failure, "source_message"):
            assert failure.source_message
        if isinstance(failure, (lox.StatisticsError.MalformedAttribute,
                                lox.StatisticsError.AttributeValue)):
            assert failure.attribute == "Name"
        if isinstance(failure, lox.StatisticsError.MissingValue):
            assert (failure.timestamp, failure.attribute) == ("later & raw", "V2")
        if isinstance(failure, lox.StatisticsError.InvalidValue):
            assert (failure.timestamp, failure.attribute, failure.value, failure.source_message) == (
                "later", "V", " warm ", "invalid float literal")
        if isinstance(failure, lox.StatisticsError.InvalidStatsGroup):
            assert failure.value == "4294967296"
        if isinstance(failure, lox.StatisticsError.InvalidNumOutputs):
            assert failure.value == "-1"
    else:
        raise AssertionError(f"Expected atomic {variant}")
for raw in ("NaN", "inf", "-inf", "1e9999"):
    try:
        lox.parse_statistics_series(
            f'<Statistics><S T="first" V="1"/><S T="later" V="{raw}"/></Statistics>')
    except lox.StatisticsError.NonFiniteValue as failure:
        assert (failure.timestamp, failure.attribute, failure.value) == ("later", "V", raw)
    else:
        raise AssertionError("Core nonfinite refusal required")

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count)) == 0
    return count.value


marker = "series-owned-allocation-" + "x" * 65536
gc.disable()
for error in (False, True):
    payload = (f'<Statistics Name="{marker}"><S T="raw" V="-0"/></Statistics>' if not error else
               '<Statistics><S T="first" V="1"/>'
               f'<S T="later" V="{marker}"/></Statistics>')
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_statistics_series(
        native._UniffiFfiConverterString.lower(payload), ctypes.byref(status))
    assert status.code == int(error)
    buffer = status.error_buf if error else result
    assert buffer.len > 64000
    length = buffer.len
    prefix = ctypes.string_at(buffer.data, 128)
    assert matching(prefix) == 1
    if error:
        buffer.free()
    result.free()
    assert matching(prefix) == 0
    retained_value = None
    retained_failure = None
    for i in range(64):
        try:
            value = lox.parse_statistics_series(payload)
            assert not error and value.name == marker
            assert value.rows[0].timestamp == "raw" and bits(value.rows[0].values[0]) == 1 << 63
            if i == 0:
                retained_value = value
        except lox.StatisticsError.InvalidValue as failure:
            assert error
            assert (failure.timestamp, failure.attribute, failure.value, failure.source_message) == (
                "later", "V", marker, "invalid float literal")
            if i == 0:
                retained_failure = failure
    if error:
        assert retained_failure.value == marker
    else:
        assert retained_value.name == marker
    assert matching(prefix) == 0
    print(f"Python statistics-series error={error} actual_result_bytes={length} live=1 free=0 "
          "calls=64 retained=0 GC_requested=false String_lowering=generated")
print("Python monthly XML typed rows/finite bits/opaque time/atomic errors/causes/ownership PASS")
