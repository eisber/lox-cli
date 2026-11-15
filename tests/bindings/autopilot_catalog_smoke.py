"""Installed native typed Autopilot rows and actual result/error ownership."""

import ctypes
import gc
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


payload = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
           / "tests" / "fixtures" / "autopilot_catalog.json").read_bytes()
before = bytes(payload)
rows = lox.list_loxapp3_autopilot_rules(payload)
assert payload == before and isinstance(rows, list)
assert all(isinstance(v, lox.AutopilotRule) for v in rows)
assert [(v.name, v.uuid, v.state_changed, v.state_history) for v in rows] == [
    ("", "empty", "", ""), ("?", "", "", ""), ("?", "bad", "", ""),
    ("A", "01234567-89ab-cdef-0123456789abcdef", "changed", ""),
    ("Same", "a", "changed", ""), ("Same", "z", "", "history"),
    ("a", "12345678-1234-1234-1234-123456789abc", "", "history"),
    ("é é\r\n<xml>", "raw\0ID", "raw\0state", "<xml>\r\n")]
payload = b"replaced"
rows[0].name = "mutated"
assert lox.list_loxapp3_autopilot_rules(before)[0].name == ""
assert rows[-1].uuid == "raw\0ID"
for shape in (b"null", b"true", b"4", b'"text"', b"[]", b"{}",
              b'{"autopilot":[]}', b'{"autopilot":false}', b'{"autopilot":null}'):
    assert lox.list_loxapp3_autopilot_rules(shape) == []
for malformed, position in ((b'{\n"autopilot":', (2, 12)), (b'{} secret', (1, 4)),
                            (b'"\xff"', (1, 3))):
    try:
        lox.list_loxapp3_autopilot_rules(malformed)
    except lox.CatalogDecodeError.InvalidJson as failure:
        assert (failure.line, failure.column) == position
        assert "secret" not in str(failure)
    else:
        raise AssertionError("Expected existing typed JSON error")

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


marker = "autopilot-owned-allocation-" + "x" * 65536
gc.disable()
for error in (False, True):
    payload = (b"\n" * 257 + b" " * 65539 + b"!" if error else
               ('{"autopilot":{"opaque":{"name":"' + marker + '"}}}').encode())
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_list_loxapp3_autopilot_rules(
        native._UniffiFfiConverterBytes.lower(payload), ctypes.byref(status))
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
            value = lox.list_loxapp3_autopilot_rules(payload)
            assert not error and value[0].name == marker and value[0].uuid == "opaque"
        except lox.CatalogDecodeError.InvalidJson as failure:
            assert error and (failure.line, failure.column) == (258, 65540)
    retained = matching(prefix, error)
    assert retained == 0
    print(f"Python autopilot error={error} calls=64 calibrated_live=1 calibrated_released=0 "
          f"retained_buffers={retained} GC_requested=false")
print("Python autopilot typed values/raw ordering/defaults/errors/input independence PASS")
