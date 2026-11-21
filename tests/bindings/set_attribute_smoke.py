"""Installed set_attribute bytes, typed refusals, and native ownership witnesses."""

import ctypes
import gc
import os
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


CLOCK = lox.SetAttributeClock(unix_seconds=1230768000, nanoseconds=123456789, offset_seconds=3661)
LATER = lox.SetAttributeClock(unix_seconds=1230854400, nanoseconds=0, offset_seconds=-3661)
BAD_CLOCK = lox.SetAttributeClock(unix_seconds=0, nanoseconds=0, offset_seconds=86400)
HEADER = b'<?xml version="1.0" encoding="utf-8"?>'
ORIGINAL = (
    '\ufeff<C Type="Document" DateS="30" Note="one\ntwo" 12hTF="true">\r\n'
    '<C U="a" Type="And" Title="Gâte &amp; 灯" gid="synthetic" Empty=""/>'
    '<C U="a" Title="Mirror"/>\r\n</C>\r\n'
).encode()
CANONICAL = (
    b"\xef\xbb\xbf" + HEADER + b"\r\n"
    + ('<C Type="Document" DateS="30" Note="one\ntwo" 12hTF="true">\r\n'
       '\t<C U="a" Type="And" Title="Gâte &amp; 灯" gid="synthetic" Empty=""/>\r\n'
       '\t<C U="a" Title="Mirror"/>\r\n</C>\r\n').encode()
)
MALFORMED = b"<Root><C attr='1' attr='2'/></Root>"
ELEMENT = ("Gâte & 灯", "And", "a")


def request(data, selector="uuid:a", attribute="Future", value="new", clock=CLOCK):
    return lox.SetAttributeRequest(
        bytes=data, selector=selector, attribute=attribute, value=value, clock=clock)


def outcome(req):
    try:
        return lox.set_attribute(req)
    except lox.SetAttributeError as failure:
        return failure


def identity(value):
    assert isinstance(value, lox.SetAttributeElementIdentity)
    return value.title, value.block_type, value.uuid


def check_change(value, expected_bytes, element, attr, old, new, changed, stamped):
    assert isinstance(value, lox.SetAttributeOutcome)
    assert isinstance(value.bytes, bytes) and value.bytes == expected_bytes
    assert identity(value.change.element) == element
    assert (value.change.attribute, value.change.old_value, value.change.new_value) == (attr, old, new)
    assert (value.content_changed, value.document_stamped) == (changed, stamped)


def check_cases():
    supplied = request(ORIGINAL, attribute="Empty", value="", clock=LATER)
    noop = lox.set_attribute(supplied)
    check_change(noop, CANONICAL, ELEMENT, "Empty", "", "", False, False)
    assert supplied.bytes == ORIGINAL and noop.bytes != ORIGINAL

    renamed = CANONICAL.replace(b'12hTF="true">', b'12hTF="true" Date="2009-01-01 01:01:32">')
    renamed = renamed.replace(b'DateS="30"', b'DateS="31"').replace(
        'Title="Gâte &amp; 灯"'.encode(), b'Title="New"')
    rename = lox.set_attribute(request(ORIGINAL, selector="gid:SYNTHETIC", attribute="Title", value="New"))
    check_change(rename, renamed, ELEMENT, "Title", "Gâte & 灯", "New", True, True)

    raw = '<&"\n室'
    updated = renamed.replace(b'DateS="31"', b'DateS="32"').replace(b"01:01:32", b"01:01:33")
    updated = updated.replace(b'Empty=""/>', 'Empty="" title="&lt;&amp;&quot;\n室"/>'.encode())
    added = lox.set_attribute(request(rename.bytes, "Type:AND", "title", raw))
    check_change(added, updated, ("New", "And", "a"), "title", None, raw, True, True)
    repeat = lox.set_attribute(request(added.bytes, attribute="Empty", value="", clock=LATER))
    check_change(repeat, updated, ("New", "And", "a"), "Empty", "", "", False, False)

    sparse = lox.set_attribute(request(b'<C U="a"/>', value=""))
    check_change(sparse, HEADER + b'\n<C U="a" Future=""/>\n', ("", "", "a"),
                 "Future", None, "", True, False)
    legacy = lox.set_attribute(request(b'<C U="a" Title="old 12hTF=value"/>'))
    check_change(legacy, HEADER + b'\n<C U="a" Title="old 12hTF=value" Future="new"/>\n',
                 ("old _12hTF=value", "", "a"), "Future", None, "new", True, False)
    empty_titles = outcome(request(b"<Root/>", selector="missing"))
    assert isinstance(empty_titles, lox.SetAttributeError.NotFound) and empty_titles.titles == []
    assert isinstance(outcome(request(MALFORMED, clock=BAD_CLOCK)), lox.SetAttributeError.InvalidClock)
    print("Python set-attribute normalized/no-op/short/empty/InvalidClock: value checks only")


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


def calibrate(req, branch, expected):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_set_attribute(
        native._UniffiFfiConverterTypeSetAttributeRequest.lower(req), ctypes.byref(status))
    buffer = status.error_buf if status.code else result
    try:
        assert status.code == (0 if branch == "changed" else 1)
        if status.code:
            assert result.len == 0, "Refusal returned a result buffer"
        length = buffer.len
        prefix_len, minimum = (12, length) if branch == "load" else (128, 64000)
        assert length >= max(2 * prefix_len, minimum)
        address = ctypes.cast(buffer.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, prefix_len)
        if branch == "changed":
            assert prefix == len(expected).to_bytes(4, "big") + expected[:124]
        assert probe.lox_test_heap_live(address)
        assert matching(prefix, minimum) == 1
    finally:
        if status.code:
            buffer.free()
        result.free()
    assert not probe.lox_test_heap_live(address), "Specific result address still live after free"
    assert matching(prefix, minimum) == 0, "Result prefix still present after free"
    return prefix, minimum, length


def rejected_string_releases_input(field):
    marker = f"set-attribute-rejected-{field}-"
    data = f"<!--{marker}".encode() + b"q" * 65536 + b"--><Root/>"
    minimum = 4 + len(data)
    control = native._UniffiFfiConverterTypeSetAttributeRequest.lower(request(data))
    try:
        assert control.len >= minimum
        address = ctypes.cast(control.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, 128)
        assert prefix == len(data).to_bytes(4, "big") + data[:124]
        assert probe.lox_test_heap_live(address)
        assert matching(prefix, minimum) == 1
    finally:
        control.free()
    assert not probe.lox_test_heap_live(address), "Specific input address still live after free"
    assert matching(prefix, minimum) == 0, "Input prefix still present after free"

    bad = marker + "\ud800"
    supplied = request(data, **{field: bad})
    for _ in range(64):
        try:
            lox.set_attribute(supplied)
        except UnicodeEncodeError as failure:
            assert failure.encoding == "utf-8" and failure.object == bad
            assert (failure.start, failure.end) == (len(marker), len(marker) + 1)
        else:
            raise AssertionError(f"Expected strict host {field} encoding refusal")
    assert matching(prefix, minimum) == 0, f"Rejected {field} retained a marked native input"
    print(f"Python set-attribute invalid {field}: input_address_live=1 input_address_post_free=0 "
          "input_prefix_live_count=1 input_prefix_post_free_count=0 rejections=64 "
          "input_matching_native_retained=0 GC_requested=false")


def check_owned(value, branch, expected):
    if branch == "changed":
        check_change(value, expected, ("", "", "a"), "Future", None, "new", True, True)
    elif branch == "load":
        assert isinstance(value, lox.SetAttributeError.Load) and value.source_message
    elif branch == "not-found":
        assert isinstance(value, lox.SetAttributeError.NotFound)
        assert (value.selector, value.titles) == (expected, ["Gâte & 灯", "Mirror"])
    elif branch == "ambiguous":
        assert isinstance(value, lox.SetAttributeError.Ambiguous) and value.selector == "set-attr-match-"
        assert [identity(element) for element in value.matches] == [
            (expected, "And", "a"), ("set-attr-match-a", "Or", "b")]
    else:
        raise AssertionError(f"Unknown branch {branch}")


assert "RUST_MIN_STACK" not in os.environ
assert sys.version_info[:2] == (3, 13) and sys._is_gil_enabled()
gc.disable()
check_cases()
mark = ("set-attribute-result-Καλημέρα-" + "x" * 65536).encode()
expected_output = (HEADER + b'\n<C Type="Document" DateS="31" Marker="' + mark
                   + b'" Date="2009-01-01 01:01:32">\n\t<C U="a" Future="new"/>\n</C>\n')
missing = "set-attr-missing-" + "y" * 65536
title = "set-attr-match-z-" + "z" * 65536
trials = [
    ("changed", request(b'<C Type="Document" DateS="30" Marker="' + mark
                        + b'"><C U="a"/></C>'), expected_output),
    ("load", request(MALFORMED, selector="missing", attribute="", value=""), None),
    ("not-found", request(ORIGINAL, selector=missing, attribute="", value=""), missing),
    ("ambiguous", request((f'<Root><C U="a" Type="And" Title="{title}"/>'
                           '<C U="b" Type="Or" Title="set-attr-match-a"/>'
                           '<C U="a" Title="set-attr-match-mirror"/></Root>').encode(),
                          selector="set-attr-match-", attribute="", value=""), title),
]
for branch, supplied, expected in trials:
    prefix, minimum, length = calibrate(supplied, branch, expected)
    original = memoryview(supplied.bytes).tobytes()
    held = [outcome(supplied) for _ in range(64)]
    assert len({id(item) for item in held}) == 64
    assert supplied.bytes == original, "Caller bytes changed"
    assert matching(prefix, minimum) == 0, f"{branch} retained a matching native result"
    for item in held:
        check_owned(item, branch, expected)
    if branch == "changed":
        assert len({id(item.bytes) for item in held}) == 64
        assert all(item.bytes is not supplied.bytes for item in held)
    elif branch == "load":
        assert all(item.source_message == held[0].source_message for item in held)
    print(f"Python set-attribute branch={branch} actual_bytes={length} "
          "address_live=1 address_post_free=0 prefix_live_count=1 prefix_post_free_count=0 "
          "calls=64 host_retained=64 matching_native_retained=0 GC_requested=false")
for field in ("selector", "attribute", "value"):
    rejected_string_releases_input(field)
print("Python set-attribute exact Core bytes, explicit clock and typed atomic refusals PASS")
