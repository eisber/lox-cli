"""Installed Unwire bytes, structured refusals, and native ownership witnesses."""

import ctypes
import gc
import os
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


CLOCK = lox.UnwireClock(unix_seconds=1230768000, nanoseconds=123456789, offset_seconds=3661)
LATER = lox.UnwireClock(unix_seconds=1230854400, nanoseconds=0, offset_seconds=-3661)
MALFORMED = b"<Root><C attr='1' attr='2'/></Root>"
ORIGINAL = (
    '\ufeff<?xml version="1.0" encoding="utf-8"?>\r\n'
    '<C Type="Document" DateS="30" Note="first\nsecond" 12hTF="true">'
    '<C Type="And" U="a" Title="Gâte &amp; 灯" WF="2">'
    '<Co K="I1" U="raw35-not-a-validated-uuid" Nc="2" Other="keep">'
    '<In Input="same" FLG="2"/><In Input="same"/><Future><In Input="nested"/></Future>'
    '</Co><Co K="I1" U="duplicate"/><Co K="Q" U="other"/></C>'
    '<C U="a" Title="mirror"><Co K="I1" U="mirror"/></C></C>\r\n'
).encode()


def request(data, selector="uuid:a", connector="I1", clock=CLOCK):
    return lox.UnwireRequest(bytes=data, selector=selector, connector=connector, clock=clock)


def outcome(value):
    try:
        return lox.unwire(value)
    except lox.UnwireError as failure:
        return failure


def identity(value):
    assert isinstance(value, lox.UnwireElementIdentity)
    return value.title, value.block_type, value.uuid


def check_change(value, element, connector, previous, count, changed, stamped):
    assert isinstance(value, lox.UnwireOutcome)
    assert isinstance(value.bytes, bytes)
    assert identity(value.change.element) == element
    assert (value.change.connector, value.change.previous_uuid, value.change.removed_references) == (
        connector, previous, count)
    assert (value.content_changed, value.document_stamped) == (changed, stamped)


def check_cases():
    element = ("Gâte & 灯", "And", "a")
    first = lox.unwire(request(ORIGINAL))
    check_change(first, element, "I1", "raw35-not-a-validated-uuid", 2, True, True)
    assert first.bytes.startswith(b'\xef\xbb\xbf<?xml version="1.0" encoding="utf-8"?>\r\n')
    assert first.bytes.endswith(b"\r\n")
    for token in (
        b'Date="2009-01-01 01:01:32"', b'DateS="31"', b'Note="first\nsecond" 12hTF="true"',
        b'WF="2"', b'U="raw35-not-a-validated-uuid" Other="keep"', b'<In Input="nested"/>',
        b'<Co K="I1" U="duplicate"/>', b'<Co K="Q" U="other"/>', b'<Co K="I1" U="mirror"/>',
    ):
        assert token in first.bytes, token
    assert b"Nc=" not in first.bytes and b'Input="same"' not in first.bytes

    second = lox.unwire(request(first.bytes))
    check_change(second, element, "I1", "raw35-not-a-validated-uuid", 0, True, True)
    assert b'Date="2009-01-01 01:01:33"' in second.bytes and b'DateS="32"' in second.bytes
    assert b'raw35-not-a-validated-uuid' not in second.bytes
    third = lox.unwire(request(second.bytes, clock=LATER))
    check_change(third, element, "I1", None, 0, False, False)
    assert third.bytes == second.bytes

    legacy = b'<C U="a" Title="old 12hTF=value"><Co K="I1" U=""/></C>'
    result = lox.unwire(request(legacy))
    check_change(result, ("old _12hTF=value", "", "a"), "I1", "", 0, True, False)
    short_noop = b'<C U="a"><Co K=""/></C>'
    result = lox.unwire(request(short_noop, connector=""))
    check_change(result, ("", "", "a"), "", None, 0, False, False)
    assert result.bytes != short_noop
    assert b'Date=' not in result.bytes and b'DateS=' not in result.bytes

    mismatch = outcome(request(ORIGINAL, connector="i1"))
    assert isinstance(mismatch, lox.UnwireError.ConnectorMissing)
    assert identity(mismatch.element) == element and mismatch.connector == "i1"
    empty_titles = outcome(request(b"<Root/>", selector="missing"))
    assert isinstance(empty_titles, lox.UnwireError.NotFound) and empty_titles.titles == []
    bad_clock = lox.UnwireClock(unix_seconds=0, nanoseconds=0, offset_seconds=86400)
    assert isinstance(outcome(request(MALFORMED, clock=bad_clock)), lox.UnwireError.InvalidClock)
    print("Python unwire shared-loader/short/no-op/empty-fields/payload-free cases: value checks only")


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


def calibrate(value, branch):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_unwire(
        native._UniffiFfiConverterTypeUnwireRequest.lower(value), ctypes.byref(status))
    buffer = status.error_buf if status.code else result
    try:
        assert status.code == (0 if branch == "changed" else 1)
        if status.code:
            assert result.len == 0, "Refusal returned a byte result"
        length = buffer.len
        prefix_len, minimum = (12, length) if branch == "load" else (128, 64000)
        assert length >= max(2 * prefix_len, minimum)
        address = ctypes.cast(buffer.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, prefix_len)
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        if status.code:
            buffer.free()
        result.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0
    return prefix, minimum, length


def rejected_string_releases_input(field):
    data = f'<!--unwire-{field}-rejected-input:'.encode() + b"q" * 65536 + b"--><Root/>"
    minimum = 4 + len(data)
    control = native._UniffiFfiConverterTypeUnwireRequest.lower(request(data))
    try:
        assert control.len >= minimum
        address = ctypes.cast(control.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, 128)
        assert prefix == len(data).to_bytes(4, "big") + data[:124]
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        control.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0

    bad = request(data, **{field: "\ud800"})
    for _ in range(64):
        try:
            lox.unwire(bad)
        except UnicodeEncodeError as failure:
            assert failure.encoding == "utf-8" and failure.object == getattr(bad, field)
            assert (failure.start, failure.end) == (0, 1)
        else:
            raise AssertionError(f"Expected strict host {field} encoding refusal")
    assert matching(prefix, minimum) == 0, f"Rejected {field} retained a marked native input"
    print(f"Python unwire invalid {field}: input_live=1 input_free=0 "
          "rejections=64 input_native_retained=0 GC_requested=false")


def check_owned(value, branch, expected):
    if branch == "changed":
        check_change(value, ("", "", "a"), "I1", "raw", 1, True, True)
        assert expected in value.bytes and b'DateS="31"' in value.bytes and b"<In " not in value.bytes
    elif branch == "load":
        assert isinstance(value, lox.UnwireError.Load) and value.source_message
    elif branch == "not-found":
        assert isinstance(value, lox.UnwireError.NotFound)
        assert (value.selector, value.titles) == (expected, ["Gâte & 灯", "mirror"])
    elif branch == "ambiguous":
        assert isinstance(value, lox.UnwireError.Ambiguous) and value.selector == "unwire-match-"
        assert [identity(element) for element in value.matches] == [
            (expected, "And", "a"), ("unwire-match-other", "Or", "b")]
    elif branch == "connector-missing":
        assert isinstance(value, lox.UnwireError.ConnectorMissing)
        assert (identity(value.element), value.connector) == ((expected, "And", "a"), "i1")
    else:
        raise AssertionError(f"Unknown branch {branch}")


assert "RUST_MIN_STACK" not in os.environ
assert sys.version_info[:2] == (3, 13) and sys._is_gil_enabled()
gc.disable()
check_cases()
mark = ("unwire-result-Καλημέρα-" + "x" * 65536).encode()
missing = "unwire-missing-" + "y" * 65536
title = "unwire-match-" + "z" * 65536
trials = [
    ("changed", request(b'<C Type="Document" DateS="30" Marker="' + mark
                        + b'"><C U="a"><Co K="I1" U="raw" Nc="1"><In Input="x"/></Co></C></C>'), mark),
    ("load", request(MALFORMED), None),
    ("not-found", request(ORIGINAL, selector=missing), missing),
    ("ambiguous", request((f'<Root><C U="a" Type="And" Title="{title}"/>'
                           '<C U="b" Type="Or" Title="unwire-match-other"/></Root>').encode(),
                          selector="unwire-match-"), title),
    ("connector-missing", request(f'<C U="a" Type="And" Title="{title}"/>'.encode(),
                                  connector="i1"), title),
]
for branch, value, expected in trials:
    prefix, minimum, length = calibrate(value, branch)
    original = memoryview(value.bytes).tobytes()
    held = [outcome(value) for _ in range(64)]
    assert len({id(item) for item in held}) == 64
    assert value.bytes == original
    assert matching(prefix, minimum) == 0, f"{branch} retained a native result buffer"
    for item in held:
        check_owned(item, branch, expected)
    if branch == "changed":
        assert len({id(item.bytes) for item in held}) == 64
        assert all(item.bytes is not value.bytes and item.bytes == held[0].bytes for item in held)
    if branch == "load":
        assert all(item.source_message == held[0].source_message for item in held)
    print(f"Python unwire branch={branch} actual_bytes={length} live=1 free=0 "
          "calls=64 host_retained=64 native_retained=0 GC_requested=false")
for field in ("selector", "connector"):
    rejected_string_releases_input(field)
print("Python unwire raw bytes/clock/typed errors PASS")
