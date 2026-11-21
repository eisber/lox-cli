"""Installed native strict listing values, data-free errors and no-GC disposal."""
import ctypes
import gc
import os
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native

assert "RUST_MIN_STACK" not in os.environ
assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())
rows = [
    "d ignored/path.txt", "l ignored\\path.txt", "- false.ℓoxcc", "- false.loxccé",
    "- z.loXcC", "- A.LoxCC", "- z.loXcC", "D upper.loxcc", "L upper-link.loxcc",
    " d leading.loxcc", " l leading-link.loxcc", "nonnumeric\u2003unicode.loxcc",
    "- nul\0.loxcc", "- not-a-uuid.loxcc", "- first.loxcc last-token",
    "- \U00010000.loxcc", "- \ue000.loxcc",
]
before = rows.copy()
expected = [
    "A.LoxCC", "leading-link.loxcc", "leading.loxcc", "not-a-uuid.loxcc",
    "nul\0.loxcc", "unicode.loxcc", "upper-link.loxcc", "upper.loxcc",
    "z.loXcC", "z.loXcC", "\ue000.loxcc", "\U00010000.loxcc",
]
owned = lox.parse_custom_listing(rows)
assert isinstance(owned, list) and all(isinstance(v, str) for v in owned)
assert owned == expected and rows == before
owned[0] = "changed"
assert lox.parse_custom_listing(rows) == expected
assert lox.parse_custom_listing([]) == []
retained = []
for bad in ("", "\u2003\t ", "d private.loxcc", "l private.loxcc",
            "- private/path.loxcc", "- private\\path.loxcc"):
    try:
        lox.parse_custom_listing(["- valid.loxcc", "- unrelated", bad, ""])
    except lox.CustomListingError as failure:
        kind = (lox.CustomListingError.Malformed if not bad.strip()
                else lox.CustomListingError.Unsupported)
        assert isinstance(failure, kind) and failure.row == 2
        assert "private" not in str(failure) and "private" not in repr(failure)
        retained.append(failure)
    else:
        raise AssertionError("Atomic first-error rejection required")
try:
    lox.parse_custom_listing(["", "- x/y.loxcc"])
except lox.CustomListingError.Malformed as failure:
    assert failure.row == 0
else:
    raise AssertionError("First malformed row required")

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_custom_listing_error_count.argtypes = (
    ctypes.c_uint32, ctypes.c_uint64, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, variant):
    count = ctypes.c_size_t()
    code = (probe.lox_test_heap_custom_listing_error_count(variant, 4097, ctypes.byref(count))
            if variant else probe.lox_test_heap_count(
                prefix, len(prefix), 64000, ctypes.byref(count)))
    assert code == 0
    return count.value


gc.disable()
name = "custom-listing-owned-" + "x" * 65536 + ".loXcC"
for variant in (0, 1, 2):
    rows = (["- " + name] if not variant else
            ["- ignored"] * 4097 + ["" if variant == 1 else "d private.loxcc"])
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_parse_custom_listing(
        native._UniffiFfiConverterSequenceString.lower(rows), ctypes.byref(status))
    assert status.code == int(variant != 0)
    buffer = status.error_buf if variant else result
    assert (buffer.len == 12) if variant else (buffer.len > 64000)
    prefix = b"" if variant else ctypes.string_at(buffer.data, 128)
    assert matching(prefix, variant) == 1
    if variant:
        buffer.free()
    result.free()
    assert matching(prefix, variant) == 0
    for _ in range(64):
        try:
            values = lox.parse_custom_listing(rows)
            assert not variant and values == [name]
        except lox.CustomListingError as failure:
            kind = (lox.CustomListingError.Malformed if variant == 1
                    else lox.CustomListingError.Unsupported)
            assert variant and isinstance(failure, kind) and failure.row == 4097
    assert matching(prefix, variant) == 0
    print(f"Python custom-listing variant={variant} calibrated_live=1 calibrated_released=0 "
          "calls=64 retained_buffers=0 GC_requested=false")
assert all(failure.row == 2 for failure in retained)
print("Python custom-listing raw/sorted/duplicates/quirks/typed-atomic-errors/owned PASS")
