"""Installed generated metadata snapshot values and actual RustBuffer ownership."""

import ctypes
import gc
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


payload = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
           / "tests" / "fixtures" / "structure_metadata.json").read_bytes()
before = bytes(payload)
snapshot = lox.parse_loxapp3_structure_metadata(payload)
assert payload == before and isinstance(snapshot, lox.StructureMetadata)
assert [(v.uuid, v.name) for v in snapshot.categories] == [
    ("", ""), ("a", "Same"), ("z", "Same"), ("lower", "a"),
    ("raw\0ID", "é é\r\n<xml>")]
assert [(v.name, v.uuid) for v in snapshot.global_states] == [
    ("A", ""), ("z", "raw\0UUID"), ("é é", "<xml>\r\n")]
assert [(v.raw_id, v.name) for v in snapshot.operating_modes] == [
    ("-1", "minus"), ("", "empty key"), (" 1", "whitespace"), ("0", "zero"),
    ("2147483648", "overflow"), ("bad", "fallback"), ("+1", "plus"),
    ("01", "zero padded"), ("2", "two")]
payload = b"replaced"
assert snapshot.categories[-1].uuid == "raw\0ID"
for shape in (b"null", b"true", b"4", b'"text"', b"[]", b"{}",
              b'{"cats":[],"globalStates":false,"operatingModes":null}'):
    value = lox.parse_loxapp3_structure_metadata(shape)
    assert not value.categories and not value.global_states and not value.operating_modes
for malformed, position in ((b'{\n"cats":', (2, 7)), (b'{} secret', (1, 4)),
                            (b'"\xff"', (1, 3))):
    try:
        lox.parse_loxapp3_structure_metadata(malformed)
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


marker = "metadata-owned-allocation-" + "x" * 65536
gc.disable()
for error in (False, True):
    payload = (b"\n" * 257 + b" " * 65539 + b"!" if error else
               ('{"cats":{"' + marker + '":{"name":"' + marker + '"}}}').encode())
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_loxapp3_structure_metadata(
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
            value = lox.parse_loxapp3_structure_metadata(payload)
            assert not error and value.categories[0].uuid == marker
            assert value.categories[0].name == marker
        except lox.CatalogDecodeError.InvalidJson as failure:
            assert error and (failure.line, failure.column) == (258, 65540)
    retained = matching(prefix, error)
    assert retained == 0
    print(f"Python metadata error={error} calls=64 calibrated_live=1 calibrated_released=0 "
          f"retained_buffers={retained} GC_requested=false")
print("Python metadata typed values/raw ordering/omissions/errors/input independence PASS")
