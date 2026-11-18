"""Installed native three-way bytes, typed cause frames and actual-buffer disposal."""
import ctypes
import gc
import os
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native

assert "RUST_MIN_STACK" not in os.environ
assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())


def witness(identifier):
    a = '<C U="a" Type="And">{}</C>'
    b = '<C U="b" Type="And">{}</C>'
    child = f'<C U="{identifier}" Type="And"/>'
    return tuple(("<Root>" + a.format(x) + b.format(y) + "</Root>").encode()
                 for x, y in (("", ""), (child, ""), ("", child)))


def chain(failure, identifier):
    assert isinstance(failure, lox.OverlayError.Refused)
    assert len(failure.frames) == 2
    assert isinstance(failure.frames[0], lox.OverlayErrorFrame.MERGED_CONSISTENCY)
    assert isinstance(failure.frames[1], lox.OverlayErrorFrame.DUPLICATE_UUID)
    assert failure.frames[1].id == identifier


base = b'<Root><C U="a" Type="And" V="0"/><C U="b" Type="Or" V="0"/></Root>'
local = (b'\xef\xbb\xbf<Root>\r\n<C V="1" U="a" Type="And"></C>'
         b'<C U="b" Type="Or" V="0"/>\r\n</Root>')
live = base.replace(b'V="0"/></Root>', b'V="2"/></Root>')
expected = (b'<Root><C V="1" U="a" Type="And"></C>'
            b'<C U="b" Type="Or" V="2"/></Root>')
before = (base, local, live)
assert lox.three_way(base, local, live) == expected
assert (base, local, live) == before
assert lox.three_way(base, base, local) == local
assert lox.three_way(base, local, local) == local
try:
    lox.three_way(base, local, base.replace(b'V="0"', b'V="3"', 1))
except lox.OverlayError.Refused as failure:
    assert len(failure.frames) == 1
    assert isinstance(failure.frames[0], lox.OverlayErrorFrame.SHELL_CONFLICT)
    assert failure.frames[0].id == "a"
else:
    raise AssertionError("Typed shell conflict required")
for data, kind in ((b'<Root a="1" a="2"/>', lox.OverlayErrorFrame.ATTRIBUTE),
                   (b"<Root></wrong>", lox.OverlayErrorFrame.XML)):
    try:
        lox.three_way(data, b"<Root/>", b"<Root/>")
    except lox.OverlayError.Refused as failure:
        assert len(failure.frames) == 1 and isinstance(failure.frames[0], kind)
        assert failure.frames[0].context == lox.OverlayXmlContext.DOCUMENT
        assert failure.frames[0].source_message
    else:
        raise AssertionError("Typed native parser boundary required")
try:
    lox.three_way(b"<\xff", b"<Root/>", b"<Root/>")
except lox.OverlayError.Refused as failure:
    frame, = failure.frames
    assert isinstance(frame, lox.OverlayErrorFrame.DOCUMENT_UTF8)
    assert (frame.valid_up_to, frame.error_len) == (1, 1)
else:
    raise AssertionError("Typed UTF-8 byte positions required")
try:
    lox.three_way(*witness("c"))
except lox.OverlayError.Refused as failure:
    chain(failure, "c")
else:
    raise AssertionError("Merged consistency cause chain required")

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), 64000, ctypes.byref(count)) == 0
    return count.value


gc.disable()
retained = []
marker = "overlay-owned-" + "x" * 65536
for error in (False, True):
    inputs = (witness(marker) if error else
              (f'<Root Marker="{marker}"/>'.encode(),) * 3)
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_three_way(
        *(native._UniffiFfiConverterBytes.lower(data) for data in inputs),
        ctypes.byref(status))
    assert status.code == int(error)
    buffer = status.error_buf if error else result
    assert buffer.len > 64000
    prefix = ctypes.string_at(buffer.data, 128)
    assert matching(prefix) == 1
    if error:
        buffer.free()
    result.free()
    assert matching(prefix) == 0
    for _ in range(64):
        try:
            value = lox.three_way(*inputs)
            assert not error and value == inputs[0]
            retained.append(value)
        except lox.OverlayError.Refused as failure:
            assert error
            chain(failure, marker)
            retained.append(failure)
    assert matching(prefix) == 0
    print(f"Python runtime-overlay error={error} calibrated_live=1 calibrated_released=0 "
          "calls=64 retained_buffers=0 GC_requested=false")
assert all(value == f'<Root Marker="{marker}"/>'.encode() for value in retained[:64])
for failure in retained[64:]:
    chain(failure, marker)
print("Python runtime-overlay selected-bytes/typed-two-frame-causes/native-boundaries/owned PASS")
