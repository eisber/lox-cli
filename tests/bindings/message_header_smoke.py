"""Actual native header fields, Core errors, owned records and RustBuffers."""

import ctypes
import gc
import os
import sys

import lox_archive as lox
from lox_archive import lox_archive as native

assert "RUST_MIN_STACK" not in os.environ
known = [lox.MessageType.TEXT, lox.MessageType.BINARY_FILE, lox.MessageType.VALUE_STATES,
         lox.MessageType.TEXT_STATES, lox.MessageType.DAYTIMER_STATES,
         lox.MessageType.OUT_OF_SERVICE, lox.MessageType.KEEP_ALIVE,
         lox.MessageType.WEATHER_STATES]
for kind in [*range(8), 0xe7, 0xff]:
    for flags in (0x80, 0xff):
        wire = bytes([3, kind, flags, 0x91, 0x76, 0x54, 0x32, 0xf0])
        assert lox.encode_message_header(kind, flags, 0x91, 0xf0325476) == wire
        h = lox.decode_message_header(wire)
        assert isinstance(h, lox.MessageHeader) and isinstance(h.wire_bytes, bytes)
        assert (h.message_type, h.info_flags, h.reserved, h.payload_size) == (
            kind, flags, 0x91, 0xf0325476)
        assert h.known_type == (known[kind] if kind < 8 else None)
        assert h.size_is_estimated == bool(flags & 1) and h.wire_bytes == wire
h.wire_bytes = b"changed"
assert lox.decode_message_header(wire).wire_bytes == wire
for length in (0, 1, 7, 9, 32):
    try:
        lox.decode_message_header(b"\x83" * length)
    except lox.MessageHeaderError.InvalidLength as failure:
        assert failure.actual == length
    else:
        raise AssertionError("Expected strict Core length error")
try:
    lox.decode_message_header(b"\x83" * 8)
except lox.MessageHeaderError.InvalidMagic as failure:
    assert failure.actual == 0x83
else:
    raise AssertionError("Expected Core magic error")
print("Python actual native header values/unsigned fields/classification/roundtrip/errors PASS",
      flush=True)

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, minimum):
    count = ctypes.c_size_t()
    code = probe.lox_test_heap_count(prefix, len(prefix), minimum, ctypes.byref(count))
    assert code == 0
    return count.value


gc.disable()
for branch, payload in (
        ("success", bytes([3, 0xe7, 0xff, 0x91, 0x76, 0x54, 0x32, 0xf0])),
        ("length", b"\x83" * 65537), ("magic", b"\x83" * 8)):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_decode_message_header(
        native._UniffiFfiConverterBytes.lower(payload), ctypes.byref(status))
    assert status.code == int(branch != "success")
    buffer = status.error_buf if branch != "success" else result
    length = buffer.len
    prefix = ctypes.string_at(buffer.data, length)
    live = matching(prefix, length)
    print(f"Python header branch={branch} actual_result_bytes={length} live={live}", flush=True)
    assert live == 1, "Unresolved actual header live calibration"
    if branch != "success":
        buffer.free()
    result.free()
    assert matching(prefix, length) == 0
    retained_value = None
    retained_failure = None
    for _ in range(64):
        try:
            retained_value = lox.decode_message_header(payload)
            assert branch == "success" and retained_value.wire_bytes == payload
        except lox.MessageHeaderError.InvalidLength as failure:
            assert branch == "length" and failure.actual == 65537
            retained_failure = failure
        except lox.MessageHeaderError.InvalidMagic as failure:
            assert branch == "magic" and failure.actual == 0x83
            retained_failure = failure
    assert retained_value is not None if branch == "success" else retained_failure is not None
    assert matching(prefix, length) == 0
    print(f"Python header branch={branch} live=1 free=0 calls=64 retained=0 GC_requested=false",
          flush=True)
