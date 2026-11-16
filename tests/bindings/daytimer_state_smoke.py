"""Real installed package: ordered signed observations, strict errors, no-GC disposal."""
import ctypes
import gc
import os
from pathlib import Path
import struct
import sys

import lox_archive as lox
from lox_archive import lox_archive as native

assert "RUST_MIN_STACK" not in os.environ
assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())
WIRE = bytes(range(16))
BITS = [0, 0x8000000000000000, 0x7ff0000000000000, 0xfff0000000000000,
        0x7ff0000000001234, 0xfff8123456789abc, 0xffffffffffffffff, 1]


def header(bits, count):
    return WIRE + struct.pack("<Qi", bits, count)


def payload(repeats=1):
    return b"".join(header(bits, 3) + b"".join(
        struct.pack("<iiiiQ", -2147483648, -83, 2147483647, word, bits)
        for word in (0, -7, -7)) for _ in range(repeats) for bits in BITS) + header(1, 0)


def verify(states):
    assert len(states) == len(BITS) + 1
    for state, bits in zip(states, BITS):
        assert isinstance(state, lox.DaytimerState)
        assert state.uuid_wire_bytes == WIRE
        assert state.uuid_native35 == "03020100-0504-0706-08090a0b0c0d0e0f"
        assert state.default_value_bits == bits
        assert len(state.entries) == 3
        for entry, word in zip(state.entries, (0, -7, -7)):
            assert isinstance(entry, lox.DaytimerEntry)
            assert (entry.mode, entry.from_minutes, entry.to_minutes,
                    entry.activation_word, entry.needs_activation, entry.value_bits) == (
                        -2147483648, -83, 2147483647, word, word != 0, bits)
    assert states[-1].entries == [] and states[-1].default_value_bits == 1


assert lox.decode_daytimer_state_payload(b"") == []
data = payload()
before = bytes(data)
owned = lox.decode_daytimer_state_payload(data)
verify(owned)
assert data == before
for length in range(1, 100):
    try:
        lox.decode_daytimer_state_payload(header(0, 0) + before[:length])
    except lox.DaytimerStatePayloadError.TruncatedHeader as failure:
        assert length < 28 and (failure.record_offset, failure.available) == (28, length)
    except lox.DaytimerStatePayloadError.TruncatedEntries as failure:
        assert length >= 28
        assert (failure.record_offset, failure.declared, failure.available) == (28, 3, length - 28)
    else:
        raise AssertionError("Atomic rejection required")
for count in (-1, -2147483648):
    try:
        lox.decode_daytimer_state_payload(header(0, 0) + header(0, count))
    except lox.DaytimerStatePayloadError.NegativeEntryCount as failure:
        assert (failure.record_offset, failure.declared) == (28, count)
    else:
        raise AssertionError("Signed count required")
try:
    lox.decode_daytimer_state_payload(header(0, 2147483647))
except lox.DaytimerStatePayloadError.TruncatedEntries as failure:
    assert (failure.record_offset, failure.declared, failure.available) == (0, 2147483647, 0)
else:
    raise AssertionError("Huge count required")
data = b"replaced"
verify(owned)

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_catalog_error_count.argtypes = (
    ctypes.c_uint64, ctypes.c_uint64, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, error):
    count = ctypes.c_size_t()
    code = (probe.lox_test_heap_catalog_error_count(28, 17, ctypes.byref(count))
            if error else probe.lox_test_heap_count(
                prefix, len(prefix), 64000, ctypes.byref(count)))
    assert code == 0
    return count.value


gc.disable()
for error in (False, True):
    data = header(0, 0) + b"\xa5" * 17 if error else payload(512)
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_decode_daytimer_state_payload(
        native._UniffiFfiConverterBytes.lower(data), ctypes.byref(status))
    assert status.code == int(error)
    buffer = status.error_buf if error else result
    prefix = b"" if error else ctypes.string_at(buffer.data, 128)
    assert (buffer.len == 20) if error else (buffer.len > 64000)
    assert matching(prefix, error) == 1
    if error:
        buffer.free()
    result.free()
    assert matching(prefix, error) == 0
    for _ in range(64):
        try:
            values = lox.decode_daytimer_state_payload(data)
            assert not error and len(values) == len(BITS) * 512 + 1
            assert values[-2].entries[-1].value_bits == BITS[-1]
        except lox.DaytimerStatePayloadError.TruncatedHeader as failure:
            assert error and (failure.record_offset, failure.available) == (28, 17)
    retained = matching(prefix, error)
    assert retained == 0
    print(f"Python Daytimer error={error} calls=64 calibrated_live=1 calibrated_released=0 "
          f"retained_buffers={retained} GC_requested=false")
print("Python Daytimer ordered/duplicate/signed/IEEEbits/atomic/errors/ownership PASS")
