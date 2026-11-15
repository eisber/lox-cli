"""Installed native payload-only decoder, exact bits, errors and heap lifetime."""

import ctypes
import gc
import math
import os
from pathlib import Path
import struct
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


assert "RUST_MIN_STACK" not in os.environ
assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())
wire = bytes(range(16))
bits = [0, 1, 0x3ff0000000000000, 0xbff0000000000000, 0x8000000000000001,
        0x8000000000000000, 0x7ff0000000000000, 0xfff0000000000000,
        0x7ff0000000000001, 0xfff8123456789abc, 0xffffffffffffffff]
payload = b"".join((wire if i % 2 == 0 else wire[::-1]) + value.to_bytes(8, "little")
                   for i, value in enumerate(bits))
before = bytes(payload)
assert lox.decode_value_state_payload(b"") == []
states = lox.decode_value_state_payload(payload)
assert payload == before and len(states) == len(bits)
for i, (state, expected) in enumerate(zip(states, bits)):
    assert isinstance(state, lox.NumericState)
    assert isinstance(state.uuid_wire_bytes, bytes)
    assert state.uuid_wire_bytes == (wire if i % 2 == 0 else wire[::-1])
    assert state.uuid_native35 == ("03020100-0504-0706-08090a0b0c0d0e0f" if i % 2 == 0
                                   else "0c0d0e0f-0a0b-0809-0706050403020100")
    assert isinstance(state.value, float) and isinstance(state.value_bits, int)
    assert state.value_bits == expected  # Python u64 is nonnegative, including bit 63.
    if expected & 0x7fffffffffffffff > 0x7ff0000000000000:
        assert math.isnan(state.value)
    else:
        assert struct.unpack("<Q", struct.pack("<d", state.value))[0] == expected
for length in range(1, 73):
    if length % 24:
        try:
            lox.decode_value_state_payload(b"\xa5" * length)
        except lox.NumericStatePayloadError.InvalidLength as failure:
            assert (failure.actual, failure.trailing) == (length, length % 24)
            assert "a5" not in str(failure)
        else:
            raise AssertionError("Expected atomic typed length failure")
        assert len(lox.decode_value_state_payload(before)) == len(bits)
payload = b"replaced"
assert states[0].uuid_wire_bytes == wire and states[-1].value_bits == bits[-1]

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
# Both errors serialize as one variant tag followed by two u64 fields. Reuse
# the existing by-value probe so measurement cannot allocate a matching error.
probe.lox_test_heap_catalog_error_count.argtypes = (
    ctypes.c_uint64, ctypes.c_uint64, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, error):
    count = ctypes.c_size_t()
    code = (probe.lox_test_heap_catalog_error_count(65537, 17, ctypes.byref(count))
            if error else probe.lox_test_heap_count(
                prefix, len(prefix), 64000, ctypes.byref(count)))
    assert code == 0
    return count.value


gc.disable()
for error in (False, True):
    payload = b"\xa5" * 65537 if error else before * 512
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_decode_value_state_payload(
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
            values = lox.decode_value_state_payload(payload)
            assert not error and len(values) == len(bits) * 512
            assert values[-1].value_bits == bits[-1]
        except lox.NumericStatePayloadError.InvalidLength as failure:
            assert error and (failure.actual, failure.trailing) == (65537, 17)
    retained = matching(prefix, error)
    assert retained == 0
    print(f"Python numeric error={error} calls=64 calibrated_live=1 calibrated_released=0 "
          f"retained_buffers={retained} GC_requested=false")
print("Python numeric payload UUID/order/duplicate/IEEEbits/errors/host ownership PASS")
