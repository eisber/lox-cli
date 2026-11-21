"""Actual native selected connector values, errors and owned RustBuffers."""
import ctypes
import gc
from pathlib import Path
import sys
import lox_archive as lox
from lox_archive import lox_archive as native

payload = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
           / "tests" / "fixtures" / "wiring_catalog.xml").read_bytes()
rows = lox.list_connector_wiring(payload, "uuid:raw-id")
assert [(v.connector, v.direction, v.target_uuid, v.connected) for v in rows] == [
    ("I1", lox.WiringDirection.INPUT, "00000000-0000-0000-0000000000000000", False),
    ("AQ1", lox.WiringDirection.OUTPUT, "00000000-0000-0000-0000-000000000000", True),
    ("input", lox.WiringDirection.PARAMETER, "", False),
    ("OutputSuffix", lox.WiringDirection.OUTPUT, "", True),
    ("Qx", lox.WiringDirection.OUTPUT, " raw & 灯 ", True),
    ("Qx", lox.WiringDirection.OUTPUT, " raw & 灯 ", True),
    ("", lox.WiringDirection.PARAMETER, "", False)]
rows[0].connector = "changed"
assert lox.list_connector_wiring(payload, "uuid:raw-id")[0].connector == "I1"
probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, minimum):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), minimum, ctypes.byref(count)) == 0
    return count.value


marker = "wiring-owned-allocation-" + "x" * 65536
cases = [
    (('<C Type="And" Title="Selected" U="raw-id"><Co K="' + marker + '"/></C>').encode(),
     "uuid:raw-id", None),
    (b"<C>", "uuid:raw-id", lox.WiringCatalogError.Load),
    (payload, marker, lox.WiringCatalogError.NotFound),
    (payload, "Twin", lox.WiringCatalogError.Ambiguous)]
gc.disable()
for index, (data, selector, expected_error) in enumerate(cases):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_list_connector_wiring(
        native._UniffiFfiConverterBytes.lower(data),
        native._UniffiFfiConverterString.lower(selector), ctypes.byref(status))
    assert status.code == int(expected_error is not None)
    buffer = status.error_buf if expected_error else result
    length = buffer.len
    assert length >= 16
    prefix = ctypes.string_at(buffer.data, 16 if expected_error else 128)
    minimum = length if expected_error else 64000
    assert matching(prefix, minimum) == 1
    if expected_error:
        buffer.free()
    result.free()
    assert matching(prefix, minimum) == 0
    saved = None
    for _ in range(64):
        try:
            value = lox.list_connector_wiring(data, selector)
            assert expected_error is None and value[0].connector == marker
            saved = value
        except lox.WiringCatalogError as failure:
            assert expected_error is not None and isinstance(failure, expected_error)
            if index == 1:
                assert failure.source_message
            elif index == 2:
                assert failure.selector == marker and "Selected" in failure.titles
            else:
                assert failure.selector == "Twin"
                assert [(v.title, v.block_type, v.uuid) for v in failure.matches] == [
                    ("Twin one", "And", "one"), ("Twin two", "Or", "two")]
            if saved is None:
                saved = failure
    assert saved is not None and matching(prefix, minimum) == 0
    print(f"Python wiring branch={index} actual_result_bytes={length} "
          "live=1 free=0 calls=64 retained=0 GC_requested=false")
print("Python wiring typed direct catalog and all applicable errors PASS")
