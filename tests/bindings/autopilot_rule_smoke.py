"""Typed supplied-row selection and actual success/error RustBuffer ownership."""

import ctypes
import gc
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


def row(uuid, name):
    return lox.AutopilotRule(name, uuid, "raw/\0", "<history>\r\n")


uuid = "abcdef01-2345-6789-abcdef0123456789"
catalog = lox.list_loxapp3_autopilot_rules(
    (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
     / "tests" / "fixtures" / "autopilot_catalog.json").read_bytes())
catalog_uuid = "01234567-89ab-cdef-0123456789abcdef"
assert lox.resolve_autopilot_rule(catalog, catalog_uuid) == next(
    value for value in catalog if value.uuid == catalog_uuid)
rows = [row(uuid, "first"), row(uuid, "second")]
selected = lox.resolve_autopilot_rule(rows, uuid)
assert selected == rows[0]
rows[0].state_changed = "mutated"
rows.clear()
assert selected.state_changed == "raw/\0"
for query, supplied, error in (
        (uuid.upper(), [row(uuid, uuid.upper())], lox.RuleResolutionError.UuidNotFound),
        ("-" * 21, [row("opaque", "-" * 21)], lox.RuleResolutionError.UuidNotFound),
        ("missing", [], lox.RuleResolutionError.NotFound),
        ("missing\0query\r\n", [], lox.RuleResolutionError.NotFound),
        ("", [], lox.RuleResolutionError.NotFound),
        (" Kitchen", [row("Kitchen", "Kitchen")], lox.RuleResolutionError.NotFound),
        ("Kitchen [Room]", [row("opaque", "Kitchen")], lox.RuleResolutionError.NotFound)):
    try:
        lox.resolve_autopilot_rule(supplied, query)
    except error as failure:
        assert failure.query == query
    else:
        raise AssertionError("Expected distinct typed selection failure")

rows = [row("same", "Kitchen Extra"), row("same", "Kitchen"), row("same", "Kitchen")]
try:
    lox.resolve_autopilot_rule(rows, "Kitchen")
except lox.RuleResolutionError.Ambiguous as failure:
    assert failure.query == "Kitchen" and failure.candidates == rows
    rows[0].name = "changed"
    rows.clear()
    assert [v.name for v in failure.candidates] == ["Kitchen Extra", "Kitchen", "Kitchen"]
    assert all(v.state_history == "<history>\r\n" for v in failure.candidates)
else:
    raise AssertionError("Exact names and identical IDs must remain ambiguous")

unicode_row = row("raw/\0ID", "Ü İ Σ Straße é")
for query in ("", "ü", "i\u0307", "σ", "straße", "é"):
    assert lox.resolve_autopilot_rule([unicode_row], query) == unicode_row
for query in ("-" * 20, "abcdef0123456789abcdef0123456789",
              "gggggggg-gggg-gggg-gggggggggggggggg"):
    value = row("opaque", query)
    assert lox.resolve_autopilot_rule([value], query) == value
for query in ("STRASSE", "e\u0301", " ü", "ς"):
    try:
        lox.resolve_autopilot_rule([unicode_row], query)
    except lox.RuleResolutionError.NotFound as failure:
        assert failure.query == query
    else:
        raise AssertionError("Only Core lowercase substring semantics apply")
try:
    lox.resolve_autopilot_rule([row("a", ""), row("b", "?")], "")
except lox.RuleResolutionError.Ambiguous as failure:
    assert [v.uuid for v in failure.candidates] == ["a", "b"]
else:
    raise AssertionError("Empty substring matches every supplied row")

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count)) == 0
    return count.value


gc.disable()
for branch in range(4):
    marker = "autopilot-selector-owned-" + "x" * 65536
    query = "a" * 65536 + "-" if branch == 1 else marker
    if branch == 0:
        supplied = [row("raw/\0ID", marker)]
    elif branch == 3:
        supplied = [row("same", marker + " extra"), row("same", marker)]
    else:
        supplied = []
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_resolve_autopilot_rule(
        native._UniffiFfiConverterSequenceTypeAutopilotRule.lower(supplied),
        native._UniffiFfiConverterString.lower(query), ctypes.byref(status))
    assert status.code == int(branch != 0)
    buffer = status.error_buf if branch else result
    assert buffer.len > 64000
    prefix = ctypes.string_at(buffer.data, 128)
    assert matching(prefix) == 1
    if branch:
        buffer.free()
    result.free()
    assert matching(prefix) == 0
    for _ in range(64):
        try:
            value = lox.resolve_autopilot_rule(supplied, query)
            assert branch == 0 and value == supplied[0]
        except lox.RuleResolutionError.UuidNotFound as failure:
            assert branch == 1 and failure.query == query
        except lox.RuleResolutionError.NotFound as failure:
            assert branch == 2 and failure.query == query
        except lox.RuleResolutionError.Ambiguous as failure:
            assert branch == 3 and failure.query == query and failure.candidates == supplied
    retained = matching(prefix)
    assert retained == 0
    print(f"Python autopilot-selector branch={branch} calls=64 calibrated_live=1 "
          f"calibrated_released=0 retained_buffers={retained} GC_requested=false")
print("Python supplied typed Autopilot selection/owned values/errors PASS")
