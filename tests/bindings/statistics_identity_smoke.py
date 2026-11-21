"""Installed typed identity selection and actual success/error RustBuffer ownership."""

import ctypes
import gc
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


def row(name, uuid, group=None):
    return lox.StatEntry(file="not-an-identity.xml", uuid=uuid, group=group,
                         month="opaque-month", name=name, label="not-an-identity-label")


rows = [row("Heat", "winner", 4294967295), row("winner", "other"),
        row("Heat Pump", "third"), row("Heat", "winner", 0)]
assert lox.resolve_statistics_identity(rows, "WINNER") == "winner"
assert lox.resolve_statistics_identity(rows, "hEaT") == "winner"
assert lox.resolve_statistics_identity(rows, "pump") == "third"
assert lox.resolve_statistics_identity([row("unrelated", ""), row("", "other")], "") == ""
assert lox.resolve_statistics_identity([row("a", "same"), row("b", "same", 1)], "") == "same"
raw = "RAW\0é é\r\nuuid"
assert lox.resolve_statistics_identity([row("İSTANBUL", raw)], "İstanbul") == raw
for query in ("ABC", "Istanbul", "café", "Café ", "not-an-identity.xml",
              "not-an-identity-label", "opaque-month"):
    try:
        lox.resolve_statistics_identity([row("İSTANBUL", "ABC"), row("Café", "x")], query)
        raise AssertionError("No trimming, normalization, stored-UUID folding or metadata lookup")
    except lox.StatisticResolutionError.NotFound as error:
        assert error.names == ["Café", "İSTANBUL"]

diagnostic_rows = [row("Same", "z", 4294967295), row("Same", "z"),
                   row("Same", "a", 0), row("Same", "z", 4294967295),
                   row("same", "a")]
retained = []
try:
    lox.resolve_statistics_identity(diagnostic_rows, "SAME")
    raise AssertionError("Raw UUID ambiguity required")
except lox.StatisticResolutionError.Ambiguous as error:
    retained.append(error)
    assert error.distinct_uuids == 2
    assert all(isinstance(value, lox.StatisticIdentity) for value in error.candidates)
    assert [(v.name, v.group, v.uuid) for v in error.candidates] == [
        ("Same", None, "z"), ("Same", 0, "a"), ("Same", 4294967295, "z"), ("same", None, "a")]
try:
    lox.resolve_statistics_identity(diagnostic_rows, "missing")
    raise AssertionError("Typed not-found required")
except lox.StatisticResolutionError.NotFound as error:
    retained.append(error)
    assert error.names == ["Same", "same"]
diagnostic_rows.clear()
assert retained[0].candidates[2].group == 4294967295
assert retained[1].names == ["Same", "same"]
try:
    lox.resolve_statistics_identity([], "")
    raise AssertionError("Empty inventory is not-found")
except lox.StatisticResolutionError.NotFound as error:
    assert error.names == []
try:
    lox.resolve_statistics_identity([row("\U00010000", "a"), row("\ue000", "b")], "")
    raise AssertionError("Rust scalar ordering, not Java UTF-16 ordering")
except lox.StatisticResolutionError.Ambiguous as error:
    assert [v.name for v in error.candidates] == ["\ue000", "\U00010000"]

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count)) == 0
    return count.value


gc.disable()
for variant in range(3):
    marker = f"identity-owned-{variant}-" + "x" * 65536
    if variant == 0:
        entries, query = [row("winner", marker)], "winner"
    elif variant == 1:
        entries, query = [row(marker, "a")], "missing"
    else:
        entries, query = [row(marker, "a"), row(marker, "b", 4294967295)], marker
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_resolve_statistics_identity(
        native._UniffiFfiConverterSequenceTypeStatEntry.lower(entries),
        native._UniffiFfiConverterString.lower(query), ctypes.byref(status))
    assert status.code == (0 if variant == 0 else 1)
    buffer = result if variant == 0 else status.error_buf
    assert buffer.len >= 64000
    length = buffer.len
    prefix = ctypes.string_at(buffer.data, 128)
    assert matching(prefix) == 1
    if variant != 0:
        buffer.free()
    result.free()
    assert matching(prefix) == 0
    for _ in range(64):
        try:
            value = lox.resolve_statistics_identity(entries, query)
            assert variant == 0 and value == marker
        except lox.StatisticResolutionError.NotFound as error:
            assert variant == 1 and error.names == [marker]
        except lox.StatisticResolutionError.Ambiguous as error:
            assert variant == 2 and error.distinct_uuids == 2
            assert [v.uuid for v in error.candidates] == ["a", "b"]
            assert error.candidates[1].group == 4294967295
    assert matching(prefix) == 0
    print(f"Python statistics-identity variant={variant} actual_bytes={length} "
          "live=1 free=0 calls=64 retained=0 GC_requested=false")
print("Python statistics-identity raw/precedence/typed-owned-diagnostics PASS")
