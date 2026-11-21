"""Installed native package: lossless weather observations and deterministic disposal."""
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
INTS = [-2147483648, -83, 2147483647, -7, -1024]
FIELDS = ("temperature_bits", "perceived_temperature_bits", "dew_point_bits",
          "precipitation_bits", "wind_speed_bits", "barometric_pressure_bits")


def header(update, count):
    return WIRE + struct.pack("<Ii", update, count)


def payload(repeats=1):
    return b"".join(header(0xffffffff - i, 3) + b"".join(
        struct.pack("<iiiii6Q", *INTS, *(BITS[(i + j + k) % len(BITS)] for k in range(6)))
        for j in range(3)) for _ in range(repeats) for i in range(len(BITS))) + header(0x80000000, 0)


def verify(states):
    assert len(states) == len(BITS) + 1
    for i, state in enumerate(states[:-1]):
        assert isinstance(state, lox.WeatherState)
        assert state.uuid_wire_bytes == WIRE
        assert state.uuid_native35 == "03020100-0504-0706-08090a0b0c0d0e0f"
        assert state.last_update == 0xffffffff - i and state.entry_count == 3
        for j, entry in enumerate(state.entries):
            assert isinstance(entry, lox.WeatherEntry)
            assert [entry.timestamp, entry.weather_type, entry.wind_direction,
                    entry.solar_radiation, entry.relative_humidity] == INTS
            assert [getattr(entry, field) for field in FIELDS] == [
                BITS[(i + j + k) % len(BITS)] for k in range(6)]
    assert states[-1].entries == [] and states[-1].last_update == 0x80000000
    assert states[-1].entry_count == 0


assert lox.decode_weather_state_payload(b"") == []
data = payload()
before = bytes(data)
owned = lox.decode_weather_state_payload(data)
verify(owned)
assert data == before
for length in range(1, 228):
    try:
        lox.decode_weather_state_payload(header(0, 0) + before[:length])
    except lox.WeatherStatePayloadError.TruncatedHeader as failure:
        assert length < 24 and (failure.record_offset, failure.available) == (24, length)
    except lox.WeatherStatePayloadError.TruncatedEntries as failure:
        assert length >= 24
        assert (failure.record_offset, failure.declared, failure.expected, failure.available) == (
            24, 3, 204, length - 24)
    else:
        raise AssertionError("Atomic rejection required")
for count in (-1, -2147483648, 2147483647):
    try:
        lox.decode_weather_state_payload(header(0, 0) + header(0, count))
    except lox.WeatherStatePayloadError.NegativeEntryCount as failure:
        assert count < 0 and (failure.record_offset, failure.declared) == (24, count)
    except lox.WeatherStatePayloadError.TruncatedEntries as failure:
        assert count == 2147483647
        assert (failure.record_offset, failure.declared, failure.expected, failure.available) == (
            24, count, count * 68, 0)
    else:
        raise AssertionError("Count rejection required")
data = b"replaced"
verify(owned)

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_catalog_error_count.argtypes = (
    ctypes.c_uint64, ctypes.c_uint64, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, error):
    count = ctypes.c_size_t()
    code = (probe.lox_test_heap_catalog_error_count(24, 17, ctypes.byref(count))
            if error else probe.lox_test_heap_count(
                prefix, len(prefix), 64000, ctypes.byref(count)))
    assert code == 0
    return count.value


gc.disable()
for error in (False, True):
    data = header(0, 0) + b"\xa5" * 17 if error else payload(512)
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_decode_weather_state_payload(
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
            values = lox.decode_weather_state_payload(data)
            assert not error and len(values) == len(BITS) * 512 + 1
            assert values[-2].entries[-1].barometric_pressure_bits == BITS[6]
        except lox.WeatherStatePayloadError.TruncatedHeader as failure:
            assert error and (failure.record_offset, failure.available) == (24, 17)
    retained = matching(prefix, error)
    assert retained == 0
    print(f"Python Weather error={error} calls=64 calibrated_live=1 calibrated_released=0 "
          f"retained_buffers={retained} GC_requested=false")
print("Python Weather unsigned/signed/sixIEEEbits/order/atomic/errors/ownership PASS")
