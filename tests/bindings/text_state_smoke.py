"""Installed native strict text payload decoder and calibrated buffer ownership."""

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
ICON = bytes(range(240, 256))
TEXTS = ["", "A", "é", "😀\0", "é e\u0301 😀\0tail\0", "é"]


def header(declared):
    return WIRE + ICON + struct.pack("<I", declared)


def record(text):
    return header(len(text)) + text + b"\xa5" * (-len(text) % 4)


DATA = b"".join(record(text.encode("utf-8")) for text in TEXTS)


def verify(states):
    assert len(states) == len(TEXTS)
    for state, text in zip(states, TEXTS):
        assert isinstance(state, lox.TextState)
        assert isinstance(state.uuid_wire_bytes, bytes)
        assert isinstance(state.icon_uuid_wire_bytes, bytes)
        assert state.uuid_wire_bytes == WIRE and state.icon_uuid_wire_bytes == ICON
        assert state.uuid_native35 == "03020100-0504-0706-08090a0b0c0d0e0f"
        assert state.icon_uuid_native35 == "f3f2f1f0-f5f4-f7f6-f8f9fafbfcfdfeff"
        assert isinstance(state.text, str) and state.text == text
        assert state.text.encode("utf-8") == text.encode("utf-8")
        assert state.text_byte_len == len(text.encode("utf-8"))


prefix_record = record(b"p" * 65500)
assert len(prefix_record) == 65536
# tag, first/second by-value probe fields, serialized length, suffix, exception, fields
CASES = [
    (1, 17, 0, 20, b"\xa5" * 17, lox.TextStatePayloadError.TruncatedHeader,
     {"available": 17}),
    (3, 0xffffffff, 7, 24, header(0xffffffff) + b"private",
     lox.TextStatePayloadError.TruncatedText, {"declared": 0xffffffff, "available": 7}),
    (4, 3, 1, 28, header(1) + b"x\xa5",
     lox.TextStatePayloadError.TruncatedPadding, {"expected": 3, "available": 1}),
    (5, 12, 1, 29, record(b"private-text\xff"),
     lox.TextStatePayloadError.InvalidUtf8, {"valid_up_to": 12, "error_len": 1}),
    (5, 12, 0xffffffffffffffff, 21, record(b"private-text\xf0\x9f"),
     lox.TextStatePayloadError.InvalidUtf8, {"valid_up_to": 12, "error_len": None}),
]


def check_error(failure, case):
    assert isinstance(failure, case[5])
    assert failure.record_offset == 65536
    for field, expected in case[6].items():
        assert getattr(failure, field) == expected
    assert "private" not in str(failure) and "pppp" not in str(failure)


assert lox.decode_text_state_payload(b"") == []
data = bytearray(DATA)
states = lox.decode_text_state_payload(bytes(data))
assert data == DATA
verify(states)
data[:] = b"\0" * len(data)
verify(states)
for case in CASES:
    try:
        lox.decode_text_state_payload(prefix_record + case[4])
    except lox.TextStatePayloadError as failure:
        check_error(failure, case)
    else:
        raise AssertionError("Expected complete-input rejection, not prefix results")
    verify(lox.decode_text_state_payload(DATA))
for length in range(1, 36):
    try:
        lox.decode_text_state_payload(b"\xa5" * length)
    except lox.TextStatePayloadError.TruncatedHeader as failure:
        assert (failure.record_offset, failure.available) == (0, length)
    else:
        raise AssertionError("Expected short-header failure")
# LengthOverflow is impossible for a u32 declaration on qualified 64-bit hosts.
# Its complete conversion is tested in Rust, not claimed as a live decoder path.
assert issubclass(lox.TextStatePayloadError.LengthOverflow, lox.TextStatePayloadError)

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_text_error_count.argtypes = (
    ctypes.c_uint32, ctypes.c_uint64, ctypes.c_uint64, ctypes.c_uint64,
    ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, case):
    count = ctypes.c_size_t()
    code = (probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count))
            if case is None else probe.lox_test_heap_text_error_count(
                case[0], 65536, case[1], case[2], ctypes.byref(count)))
    assert code == 0
    return count.value


gc.disable()
for case in [None, *CASES]:
    payload = DATA * 512 if case is None else prefix_record + case[4]
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_decode_text_state_payload(
        native._UniffiFfiConverterBytes.lower(payload), ctypes.byref(status))
    assert status.code == int(case is not None)
    buffer = result if case is None else status.error_buf
    prefix = ctypes.string_at(buffer.data, 128) if case is None else b""
    assert buffer.len > 64000 if case is None else buffer.len == case[3]
    assert matching(prefix, case) == 1
    if case is not None:
        buffer.free()
    result.free()
    assert matching(prefix, case) == 0
    for _ in range(64):
        try:
            values = lox.decode_text_state_payload(payload)
            assert case is None and len(values) == len(TEXTS) * 512
            verify(values[-len(TEXTS):])
        except lox.TextStatePayloadError as failure:
            assert case is not None
            check_error(failure, case)
    retained = matching(prefix, case)
    assert retained == 0
    label = "success" if case is None else f"tag{case[0]}-option{case[2]}"
    print(f"Python text path={label} calls=64 calibrated_live=1 calibrated_released=0 "
          f"retained_buffers={retained} GC_requested=false")
print("Python text UUID/icon/order/duplicate/UTF8/bytecount/NUL/atomic errors/ownership PASS")
