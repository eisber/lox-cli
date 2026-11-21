"""Installed program text, structured diagnostics, and same-export RustBuffer release."""

import ctypes
import gc
import os
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


def outcome(data, selector):
    try:
        return lox.get_program_text(data, selector)
    except lox.ProgramTextError as failure:
        return failure


def identity(value):
    assert isinstance(value, lox.ProgramElementIdentity)
    return value.title, value.block_type, value.uuid


def check(value, branch, expected):
    if branch == "text":
        assert isinstance(value, str) and value == expected
    elif branch == "load":
        assert isinstance(value, lox.ProgramTextError.Load) and value.source_message
    elif branch == "not-found":
        assert isinstance(value, lox.ProgramTextError.NotFound)
        assert (value.selector, value.titles) == expected
    elif branch == "ambiguous":
        assert isinstance(value, lox.ProgramTextError.Ambiguous)
        assert (value.selector, [identity(element) for element in value.matches]) == expected
    elif branch == "unsupported":
        assert isinstance(value, lox.ProgramTextError.Unsupported)
        assert identity(value.element) == expected
    else:
        raise AssertionError(f"Unknown branch {branch}")


xml = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
       / "tests" / "fixtures" / "program_text.xml").read_bytes()
expected_text = "first & Küche\ne\u0301secondthird 灯  "
wrong = ("Cook & 灯", "sequencecontroller", "wrong")
for selector in ("uuid:sequence", "gid:cook", "Cook & 灯 [Kitchen]"):
    check(outcome(xml, selector), "text", expected_text)
for selector in ("Empty", "Blank"):
    check(outcome(xml, selector), "text", "")
check(outcome(xml, "uuid:legacy"), "text", "text _12hTF=value")
check(outcome(xml, "missing"), "not-found",
      ("missing", ["Blank", "Cook & 灯", "Empty", "Kitchen", "Legacy _12hTF=x"]))
check(outcome(xml, "Cook & 灯"), "ambiguous",
      ("Cook & 灯", [("Cook & 灯", "SequenceController", "sequence"), wrong]))
check(outcome(xml, "uuid:wrong"), "unsupported", wrong)
check(outcome(xml, "uuid:sparse"), "unsupported", ("", "", "sparse"))
malformed = b"<C A='1' A='2'/>"
check(outcome(malformed, "missing"), "load", None)

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_count.restype = ctypes.c_uint32
probe.lox_test_heap_live.argtypes = (ctypes.c_void_p,)
probe.lox_test_heap_live.restype = ctypes.c_int


def matching(prefix, minimum):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), minimum, ctypes.byref(count)) == 0
    return count.value


def calibrate(branch, data, selector):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_get_program_text(
        native._UniffiFfiConverterBytes.lower(data),
        native._UniffiFfiConverterString.lower(selector), ctypes.byref(status))
    assert status.code == (0 if branch == "text" else 1)
    buffer = result if branch == "text" else status.error_buf
    try:
        length = buffer.len
        prefix_length, minimum = (16, length) if branch == "load" else (128, 64000)
        assert length >= max(2 * prefix_length, minimum)
        address = ctypes.cast(buffer.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, prefix_length)
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        if branch != "text":
            buffer.free()
        result.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0
    return prefix, minimum, length


marker = "program-result-Καλημέρα-" + "x" * 65536
query = "missing-program-" + "y" * 65536
long_title = "program-element-" + "z" * 65536
long_wrong = (long_title, "sequencecontroller", "w")
trials = [
    ("text", (f'<C Type="SequenceController" U="s"><Field Name="Configuration">'
              f'{marker}</Field></C>').encode(), "uuid:s", marker),
    ("load", malformed, "missing", None),
    ("not-found", xml, query,
     (query, ["Blank", "Cook & 灯", "Empty", "Kitchen", "Legacy _12hTF=x"])),
    ("ambiguous", (f'<Root><C Type="sequencecontroller" U="w" Title="{long_title}"/>'
                   f'<C Type="SequenceController" U="s" Title="{long_title}"/></Root>').encode(),
     "program-element-", ("program-element-", [long_wrong, (long_title, "SequenceController", "s")])),
    ("unsupported", f'<C Type="sequencecontroller" U="w" Title="{long_title}"/>'.encode(),
     "uuid:w", long_wrong),
]
assert "RUST_MIN_STACK" not in os.environ
gc.disable()
for branch, data, selector, expected in trials:
    prefix, minimum, length = calibrate(branch, data, selector)
    outcomes = [outcome(data, selector) for _ in range(64)]
    assert len({id(value) for value in outcomes}) == 64
    assert matching(prefix, minimum) == 0, f"{branch} retained a native result buffer"
    for value in outcomes:
        check(value, branch, expected)
    if branch == "load":
        assert all(value.source_message == outcomes[0].source_message for value in outcomes)
    print(f"Python program-text branch={branch} actual_bytes={length} live=1 free=0 "
          "calls=64 host_retained=64 native_retained=0 GC_requested=false")
print("Python program-text exact/empty/shared-loader/typed-errors PASS")
