"""Installed monthly XML records, payload-free errors and native buffer ownership."""

import ctypes
import gc
import math
import os
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


assert "RUST_MIN_STACK" not in os.environ
assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())
xml = Path(sys.argv[2]).read_text(encoding="utf-8")


def verify(series):
    assert isinstance(series, lox.StatSeries)
    assert series.name == "Café & 気" and series.group == 4294967295
    assert series.outputs == ["Energy & power", "V2", "Energy & power"]
    assert all(isinstance(row, lox.StatRow) for row in series.rows)
    assert [(row.timestamp, row.values) for row in series.rows] == [
        (" arbitrary & timestamp ", [-0.0, 20.0, 3.0]),
        ("earlier", [1.0, 2.0, 3.0]), ("earlier", [4.0, 5.0, 6.0])]
    assert math.copysign(1, series.rows[0].values[0]) == -1


verify(lox.parse_statistics_series(xml))
empty = lox.parse_statistics_series('<Statistics NumOutputs="0"/>')
assert (empty.name, empty.group, empty.outputs, empty.rows) == ("", None, [], [])
zero = lox.parse_statistics_series('<Statistics StatsGroup="0" NumOutputs="0"><S T=""/></Statistics>')
assert zero.group == 0 and zero.rows[0].timestamp == "" and zero.rows[0].values == []
assert lox.parse_statistics_series("<Statistics/>").outputs == ["V"]
# Retain Core's lazy attributes and rows outside a closed header.
assert lox.parse_statistics_series(
    '<Statistics Name="n" StatsGroup="0" Outputs="A" NumOutputs="1" broken/>'
    '<S T="outside" V="2" broken/>').rows[0].values == [2.0]
for bad, variant in [
    ("<Statistics></secret>", lox.StatisticsError.Xml),
    ("<Statistics broken/>", lox.StatisticsError.MalformedAttribute),
    ('<Statistics Name="&secret;"/>', lox.StatisticsError.AttributeValue),
    ("<Statistics>", lox.StatisticsError.TruncatedXml),
    ("<Statistics/><Statistics/>", lox.StatisticsError.MultipleStatistics),
    ("", lox.StatisticsError.MissingStatistics),
    ("<S/>", lox.StatisticsError.RowBeforeStatistics),
    ('<Statistics StatsGroup="secret"/>', lox.StatisticsError.InvalidStatsGroup),
    ('<Statistics StatsGroup="4294967296"/>', lox.StatisticsError.InvalidStatsGroup),
    ('<Statistics NumOutputs="-1"/>', lox.StatisticsError.InvalidNumOutputs),
    ("<Statistics><S/>", lox.StatisticsError.MissingTimestamp),
    ('<Statistics><S T="secret"/>', lox.StatisticsError.MissingValue),
    ('<Statistics><S T="secret" V="secret"/>', lox.StatisticsError.InvalidValue),
    ('<Statistics><S T="secret" V="NaN"/>', lox.StatisticsError.NonFiniteValue),
]:
    try:
        lox.parse_statistics_series(bad)
    except variant as failure:
        assert "secret" not in str(failure) and "secret" not in repr(failure)
    else:
        raise AssertionError("Expected atomic statistics error")
    verify(lox.parse_statistics_series(xml))

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_statistics_error_count.argtypes = (ctypes.POINTER(ctypes.c_size_t),)


def matching(prefix, error):
    count = ctypes.c_size_t()
    code = (probe.lox_test_heap_statistics_error_count(ctypes.byref(count))
            if error else probe.lox_test_heap_count(
                prefix, len(prefix), 64000, ctypes.byref(count)))
    assert code == 0
    return count.value


gc.disable()
marker = "monthly-statistics-owned-" + "x" * 65536
for error in (False, True):
    data = ('<Statistics><S T="secret" V="NaN"/></Statistics>' if error else
            f'<Statistics Name="{marker}" StatsGroup="4294967295" Outputs="a,b">'
            '<S T="opaque &amp; duplicate" V="-0" V2="2"/>'
            '<S T="opaque &amp; duplicate" V="3" V2="4"/></Statistics>')
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_statistics_series(
        native._UniffiFfiConverterString.lower(data), ctypes.byref(status))
    assert status.code == int(error)
    buffer = status.error_buf if error else result
    assert buffer.len == 4 if error else buffer.len > 64000
    prefix = b"" if error else ctypes.string_at(buffer.data, 128)
    assert matching(prefix, error) == 1
    if error:
        buffer.free()
    result.free()
    assert matching(prefix, error) == 0
    for _ in range(64):
        try:
            series = lox.parse_statistics_series(data)
            assert not error and series.name == marker and series.group == 4294967295
            assert series.outputs == ["a", "b"]
            assert [(r.timestamp, r.values) for r in series.rows] == [
                ("opaque & duplicate", [-0.0, 2.0]), ("opaque & duplicate", [3.0, 4.0])]
            assert math.copysign(1, series.rows[0].values[0]) == -1
        except lox.StatisticsError.NonFiniteValue as failure:
            assert error and "secret" not in str(failure)
    retained = matching(prefix, error)
    assert retained == 0
    print(f"Python statistics error={error} calls=64 calibrated_live=1 calibrated_released=0 "
          f"retained_buffers={retained} GC_requested=false")
print("Python monthly statistics nested/opaque/group/empty/errors/recovery PASS")
