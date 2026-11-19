"""Installed fragment application, typed causes, and actual RustBuffers."""

import ctypes
import gc
import os
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native

assert "RUST_MIN_STACK" not in os.environ
assert Path(lox.__file__).resolve().is_relative_to(Path(sys.prefix).resolve())

A = "abcdef00-0000-0000-ffff000000000001"
B = "fedcba00-0000-0000-ffff000000000002"
FRAME = lox.FragmentApplicationErrorFrame


def fragment(identifier, xml):
    return lox.LoxccFragment(name=identifier + ".LoxCC", data=lox.compress_loxcc(xml))


def control(identifier, kind="And", extra=""):
    return f'<C U="{identifier}" Type="{kind}"{extra}/>'.encode()


def refused(base, fragments):
    try:
        lox.merge_overlays(base=base, fragments=fragments)
    except lox.FragmentApplicationError.Refused as failure:
        return failure
    raise AssertionError("Expected atomic typed refusal")


invalid = b"\xff<\0"
assert lox.merge_overlays(base=invalid, fragments=[]) == invalid
page = control("page", "Page", ' SpNo="1"')
base = b"\xef\xbb\xbf<Root>\r\n<!--keep-->" + page + b"\r\n</Root>"
a_rule = control(A, "AutopilotRule", ' V="line\r\nbreak"')
b_rule = control(B, "AutopilotRule")
assert lox.merge_overlays(base, [fragment(B, b_rule), fragment(A, a_rule)]) == base.replace(
    page, page[:-2] + b">" + a_rule + b_rule + b"</C>"
)
a = fragment(A, control(A, extra=' V="after"'))
b = fragment(B, b_rule)
try:
    lox.merge_overlays(base, [b, b, a, lox.LoxccFragment(name=a.name, data=b"")])
except lox.FragmentApplicationError.DuplicateFilename as failure:
    assert failure.name == a.name
else:
    raise AssertionError("Duplicate exact names were collapsed")
alias = fragment("ABCDEF00-0000-0000-FFFF-000000000001", control(A))
assert refused(base, [a, alias]).frames == [FRAME.DUPLICATE_OVERLAY(id=A.replace("-", ""))]
assert refused(b"<Root/>", [fragment(B, b_rule)]).frames == [FRAME.RULE_PLACEMENT(name=B + ".LoxCC")]
assert refused(b"<Root/>", [a]).frames == [FRAME.UNKNOWN_CONTROL(name=a.name, kind="And")]
assert refused(b"<Root/>", [lox.LoxccFragment(name="bad", data=a.data)]).frames == [
    FRAME.FILENAME_SUFFIX()
]
before = control(A, extra=' V="before"')
small_base = b"<Root>" + before + b"</Root>"
corrupt = bytearray(a.data)
corrupt[12] ^= 1

header = lox.compress_loxcc(b"")[:16]
archive_cases = (
    (bytes(15), FRAME.ARCHIVE_HEADER_TOO_SHORT(actual=15)),
    (bytes(16), FRAME.ARCHIVE_INVALID_MAGIC(actual=0)),
    (header + b"\x10x\x00\x00", FRAME.ARCHIVE_ZERO_BACK_REFERENCE(position=2)),
    (header + b"\x10x\x02\x00", FRAME.ARCHIVE_BACK_REFERENCE_OUT_OF_BOUNDS(offset=2, output_len=1)),
)
cases = []
for index, (data, cause) in enumerate(archive_cases):
    name = f"{0xfedcba00 + index:08x}" + B[8:] + ".LoxCC"
    parts = [lox.LoxccFragment(name=name, data=data), a]
    # Filename order applies the valid replacement before refusing this fragment.
    cases.append((
        f"archive-{index}", small_base, parts,
        [FRAME.FRAGMENT_READ(name=name), FRAME.ARCHIVE(), cause],
    ))
cases.append((
    "integrity", small_base, [lox.LoxccFragment(name=a.name, data=bytes(corrupt))],
    [FRAME.FRAGMENT_READ(name=a.name), FRAME.FRAGMENT_INTEGRITY()],
))

collision_base = b"<Root>" + before + control("Keep-Me", "Or") + b"</Root>"
collision = fragment(A, before[:-2] + b">" + control("keep-me", "Or") + b"</C>")
cases.append((
    "consistency", collision_base, [collision], [
        FRAME.UUID_CONSISTENCY(name=a.name),
        FRAME.OVERLAY(frame=lox.OverlayErrorFrame.DUPLICATE_UUID(id="Keep-Me")),
    ],
))

# A multi-control document; the selected replacement and all untouched bytes
# participate in the exact result. Nothing is added to the error payloads.
controls = b"".join(control(f"{i:08x}-0000-0000-0000000000000000", extra=f' V="{i}"')
                    for i in range(1100))
large_base = b"<Root>\r\n" + before + b"\r\n" + controls + b"\r\n</Root>"
expected = large_base.replace(before, control(A, extra=' V="after"'))
cases.insert(0, ("success", large_base, [a], expected))

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))


def matching(prefix, minimum):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), minimum, ctypes.byref(count)) == 0
    return count.value


gc.disable()
for label, base, parts, expected in cases:
    error = isinstance(expected, list)
    inputs_before = [(part.name, part.data) for part in parts]
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_merge_overlays(
        native._UniffiFfiConverterBytes.lower(base),
        native._UniffiFfiConverterSequenceTypeLoxccFragment.lower(parts),
        ctypes.byref(status),
    )
    assert status.code == int(error)
    buffer = status.error_buf if error else result
    # As in MQTT, the prefix copy is shorter than the actual short buffer.
    # Here 32 bytes include 16 bytes of the admitted filename, not just tags.
    assert buffer.len >= (60 if error else 64000)
    prefix = ctypes.string_at(buffer.data, 32 if error else 128)
    if error:
        assert prefix[16:] == expected[0].name.encode("ascii")[:16]
    minimum = buffer.len if error else 64000
    assert matching(prefix, minimum) == 1
    if error:
        buffer.free()
    result.free()
    assert matching(prefix, minimum) == 0
    retained = [
        refused(base, parts) if error else lox.merge_overlays(base=base, fragments=parts)
        for _ in range(64)
    ]
    assert matching(prefix, minimum) == 0
    assert len({id(value) for value in retained}) == 64
    assert all((value.frames if error else value) == expected for value in retained)
    assert [(part.name, part.data) for part in parts] == inputs_before
    print(f"Python fragment-application {label} calibrated_live=1 calibrated_released=0 "
          "calls=64 retained_buffers=0 GC_requested=false")
