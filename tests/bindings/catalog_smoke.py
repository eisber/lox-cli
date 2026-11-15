"""Installed native catalog API; no language JSON parser or retained handles."""

import ctypes
import gc
import os
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


assert "RUST_MIN_STACK" not in os.environ
assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())
data = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
        / "tests" / "fixtures" / "control_catalog.json").read_bytes()
all_filter = lox.ControlFilter(typ=None, room=None, category=None, favorites_only=False)
before = bytes(data)
controls = lox.select_loxapp3_controls(data, all_filter)
assert data == before
assert len(controls) == 10 and all(isinstance(c, lox.Control) for c in controls)
assert [c.name for c in controls] == ["?"] * 6 + ["Alpha", "Alpha", "Zulu", "a"]
by_id = {c.uuid: c for c in controls}
for key in ("", "scalar", "array", "number", "string", "wrong"):
    assert by_id[key] == lox.Control(name="?", uuid=key, typ="?", room="Default",
                                     cat="Category", is_favorite=False, is_secured=False)
for key, typ in (("a", "Switch"), ("b", "SWITCH")):
    assert by_id[key] == lox.Control(name="Alpha", uuid=key, typ=typ, room=None,
                                     cat=None, is_favorite=False, is_secured=False)
assert by_id["empty"] == lox.Control(name="a", uuid="empty", typ="", room="", cat="",
                                    is_favorite=False, is_secured=False)
expected = lox.Control(name="Zulu", uuid="opaque / key", typ="Straße", room="KÜCHE",
                       cat="LÜFTUNG", is_favorite=True, is_secured=True)
assert by_id["opaque / key"] == expected
assert lox.select_loxapp3_controls(data, lox.ControlFilter(
    typ="STRAß", room="kü", category="lüft", favorites_only=True)) == [expected]
for typ in ("STRASSE", " Straße "):
    assert lox.select_loxapp3_controls(data, lox.ControlFilter(
        typ=typ, room=None, category=None, favorites_only=False)) == []
assert len(lox.select_loxapp3_controls(data, lox.ControlFilter(
    typ=None, room="", category="", favorites_only=False))) == 10
assert len(lox.select_loxapp3_controls(data, lox.ControlFilter(
    typ=None, room="", category="", favorites_only=True))) == 1
for shape in (b"null", b"true", b"4", b'"text"', b"[]", b"{}", b'{"controls":null}',
              b'{"controls":[]}', b'{"controls":false,"rooms":7,"cats":[]}'):
    assert lox.select_loxapp3_controls(shape, all_filter) == []
for payload, position in ((b'{\n"controls":', (2, 11)), (b'{} secret', (1, 4)),
                          (b'"\xff"', (1, 3))):
    try:
        lox.select_loxapp3_controls(payload, all_filter)
    except lox.CatalogDecodeError.InvalidJson as failure:
        assert (failure.line, failure.column) == position
        assert "secret" not in str(failure)
    else:
        raise AssertionError("Expected typed JSON/UTF8 error")
data = b"replaced"
assert by_id["opaque / key"] == expected

# Reuse the native Windows HeapWalk probe. Small payload-free errors need a
# by-value prefix so the measurement itself creates no matching heap allocation.
probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_catalog_error_count.argtypes = (
    ctypes.c_uint64, ctypes.c_uint64, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, error):
    count = ctypes.c_size_t()
    if error:
        code = probe.lox_test_heap_catalog_error_count(258, 65540, ctypes.byref(count))
    else:
        code = probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count))
    assert code == 0
    return count.value


marker = "catalog-owned-allocation-" + "x" * 65536
gc.disable()
for error in (False, True):
    payload = (b"\n" * 257 + b" " * 65539 + b"!" if error else
               ('{"controls":{"opaque":{"name":"' + marker + '"}}}').encode())
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_select_loxapp3_controls(
        native._UniffiFfiConverterBytes.lower(payload),
        native._UniffiFfiConverterTypeControlFilter.lower(all_filter), ctypes.byref(status))
    assert status.code == int(error)
    buffer = status.error_buf if error else result
    prefix = b"" if error else ctypes.string_at(buffer.data, 128)
    assert buffer.len == 20 if error else buffer.len > 64000
    assert matching(prefix, error) == 1
    if error:
        buffer.free()
    result.free()
    assert matching(prefix, error) == 0
    for _ in range(64):
        try:
            values = lox.select_loxapp3_controls(payload, all_filter)
            assert not error and values[0].name == marker
        except lox.CatalogDecodeError.InvalidJson as failure:
            assert error and (failure.line, failure.column) == (258, 65540)
    retained = matching(prefix, error)
    assert retained == 0
    print(f"Python catalog error={error} calls=64 calibrated_live=1 calibrated_released=0 "
          f"retained_buffers={retained} GC_requested=false")
print("Python catalog all fields/default semantics/filters/errors/host ownership PASS")
