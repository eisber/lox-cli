"""Installed backup observations and actual success RustBuffer ownership; no I/O."""

import ctypes
import gc
import sys

import lox_archive as lox
from lox_archive import lox_archive as native

name = "sps_4294967295_20261399259999999.zip"
value = lox.parse_backup_name(name, 18446744073709551615)
assert isinstance(value, lox.BackupInfo)
assert (value.filename, value.version, value.timestamp, value.size) == (
    name, 4294967295, "20261399259999999", 18446744073709551615)
value.filename = "changed"
assert lox.parse_backup_name(name, 0).filename == name
assert lox.parse_backup_name("sps_+0000_00000000000000.zip", 0).version == 0
for bad in ("SPS_0_20260101120000.zip", "sps_0_20260101120000.ZIP",
            "sps_0_2026010112000.zip", "sps_4294967296_20260101120000.zip",
            "sps_0_２０２６0101120000.zip", ""):
    assert lox.parse_backup_name(bad, 0) is None
listing = [
    "d a b c 18446744073709551615 sps_3_99999999999999.zip",
    "l a b c 18446744073709551616 sps_1_20261399259999.zip",
    "? a b c invalid sps_4_20261399259999.zip",
    "a b c sps_1_20261399259999.zip",
    "a b c sps_2_100000000000000.zip",
    "a b sps_0_20260101120000.zip",
]
rows = lox.parse_backup_listing(listing)
assert [v.version for v in rows] == [3, 1, 4, 1, 2]
assert [v.size for v in rows] == [18446744073709551615, 0, 0, 0, 0]
assert lox.parse_backup_listing([]) == []
assert lox.parse_backup_name("sps_0_20260101120000.zip", 1536).size == 1536

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count)) == 0
    return count.value


# Core has no timestamp length cap; the unchanged heap matcher needs a large result.
timestamp = "20261399259999" + "7" * 65536
payload = "sps_4294967295_" + timestamp + ".zip"
gc.disable()
status = native._UniffiRustCallStatus.default()
result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_backup_name(
    native._UniffiFfiConverterString.lower(payload), 18446744073709551615,
    ctypes.byref(status))
assert status.code == 0 and result.len >= 64000
prefix = ctypes.string_at(result.data, 128)
assert matching(prefix) == 1
length = result.len
result.free()
assert matching(prefix) == 0
for _ in range(64):
    value = lox.parse_backup_name(payload, 18446744073709551615)
    assert (value.filename, value.timestamp, value.version, value.size) == (
        payload, timestamp, 4294967295, 18446744073709551615)
assert matching(prefix) == 0
print(f"Python backup-catalog actual_Some_result_bytes={length} live=1 free=0 "
      "calls=64 retained=0 GC_requested=false None=functional_only "
      "Vec=inherited_generated_ownership no_error_branch PASS")
