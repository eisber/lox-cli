"""Actual generated firmware metadata values, refusal, and RustBuffer ownership."""

import ctypes
import gc
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


xml = ('<Miniserversoftware><Release Version="016.2.4294967295" Path="a&amp;b"/>'
       '<RemoteConnect MinVersion="16.2.0" Deadline="2028-02-29"/></Miniserversoftware>')
metadata = lox.parse_firmware_metadata(xml)
assert metadata.release.version.components == [16, 2, 4294967295]
assert metadata.release.version.normalized == "16.2.4294967295"
assert metadata.release.config_download == "a&b"
assert metadata.remote_connect.deadline == "2028-02-29"
assert lox.compare_firmware_versions("1.2", "1.2.0") == lox.FirmwareOrdering.LESS
assert lox.compare_firmware_versions("01.2", "1.2") == lox.FirmwareOrdering.EQUAL
for date, relation, warning in (
        ("2028-02-28", lox.DeadlineRelation.BEFORE, True),
        ("2028-02-29", lox.DeadlineRelation.ON, True),
        ("2028-03-01", lox.DeadlineRelation.PAST, False)):
    value = lox.assess_firmware_metadata(xml, "16.1", "16.2", date)
    assert value.release_status == lox.ReleaseStatus.NEWER
    assert value.program_compatibility == lox.ProgramCompatibility.SAME_MAJOR_MINOR
    assert value.remote_connect.below_minimum
    assert (value.remote_connect.deadline, value.remote_connect.warning_due) == (relation, warning)
empty = lox.assess_firmware_metadata("<Miniserversoftware/>", "1", None, "2000-01-01")
assert empty.release_status == lox.ReleaseStatus.UNAVAILABLE
assert empty.program_compatibility is None and empty.remote_connect is None
try:
    lox.assess_firmware_metadata(xml, "1", None, "2027-02-29")
except lox.FirmwareMetadataError.InvalidDate as failure:
    assert failure.value == "2027-02-29"
else:
    raise AssertionError("Expected typed invalid calendar date")
try:
    lox.parse_firmware_metadata("<Miniserversoftware><Release/></Miniserversoftware>")
except lox.FirmwareMetadataError.MissingAttribute as failure:
    assert (failure.element, failure.attribute) == ("Release", "Version")
else:
    raise AssertionError("Expected typed missing attribute")

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix):
    count = ctypes.c_size_t()
    code = probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count))
    assert code == 0
    return count.value


marker = "firmware-owned-allocation-" + "x" * 65536
gc.disable()
for error in (False, True):
    payload = ('<Miniserversoftware><Release Version="' + (marker if error else "1.2")
               + '" Path="' + ("unused" if error else marker) + '"/></Miniserversoftware>')
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_firmware_metadata(
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
    for _ in range(64):
        try:
            value = lox.parse_firmware_metadata(payload)
            assert not error and value.release.config_download == marker
        except lox.FirmwareMetadataError.InvalidVersion as failure:
            assert error and failure.value == marker
            assert failure.message == f'invalid numeric firmware version "{marker}"'
    retained = matching(prefix)
    assert retained == 0
    print(f"Python firmware error={error} actual_result_bytes={length} live=1 free=0 "
          f"calls=64 retained={retained} GC_requested=false")
print("Python firmware owned typed metadata/comparison/explicit-date/refusal PASS")
