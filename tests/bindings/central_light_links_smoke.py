"""Installed CentralLight projection and same-export RustBuffer ownership."""

import ctypes
import gc
import os
from pathlib import Path
import platform
import sys
import sysconfig

import lox_archive as lox
from lox_archive import lox_archive as native


OWNER = "ABCDEF01-0002-0003-0000000000000001"
FIRST = "F00BABE1-0002-0003-000000000000000A"
SECOND = "f00babe2-0002-0003-000000000000000b"
MISSING = "f00babe3-0002-0003-000000000000000c"
TITLE = "Zentrale & 灯"
ROW_VALUES = [(1, FIRST, "", ""), (2, SECOND, "Küche & status", "Not")]
TITLES = ["Den", "Empty", "Küche & status", TITLE]
ERROR = lox.CentralLightLinksError
FLAGS = lox.CentralLightFlagsCause
SELECT = "gid:primary"


def outcome(data, selector):
    try:
        return lox.inspect_central_light_links(lox.CentralLightLinksRequest(bytes=data, selector=selector))
    except ERROR as failure:
        return failure


def result(value):
    assert isinstance(value, lox.CentralLightLinks)
    assert all(isinstance(link, lox.CentralLightLink) for link in value.links)
    return value.uuid, value.title, [
        (link.position, link.uuid, link.title, link.control_type) for link in value.links
    ]


def identity(value):
    assert isinstance(value, lox.CentralLightElementIdentity)
    return value.title, value.block_type, value.uuid


def check(value, kind, expected):
    assert isinstance(value, kind), (type(value), kind)
    if kind == lox.CentralLightLinks:
        assert result(value) == expected
    elif kind == ERROR.Load:
        assert value.source_message
    elif kind == ERROR.NotFound:
        assert (value.selector, value.titles) == expected
    elif kind == ERROR.Ambiguous:
        assert (value.selector, [identity(element) for element in value.matches]) == expected
    elif kind == ERROR.NotPlainCentralLight:
        assert value.selector == expected
    elif kind in (ERROR.InvalidIdentity, ERROR.DuplicateReference, ERROR.UnresolvedControl):
        assert value.uuid == expected
    elif kind == ERROR.InvalidFlags:
        assert (value.value, value.kind) == expected and value.source_message
    elif kind == ERROR.InconsistentMarker:
        assert value.identity == expected


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


def calibration(buffer, large):
    length = buffer.len
    prefix_len, minimum = (128, 64000) if large else (16, length)
    assert length >= max(2 * prefix_len, minimum)
    address = ctypes.cast(buffer.data, ctypes.c_void_p).value
    prefix = ctypes.string_at(address, prefix_len)
    assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    return address, prefix, minimum, length


def calibrate(data, selector, kind, large):
    status = native._UniffiRustCallStatus.default()
    returned = native._UniffiLib.uniffi_lox_archive_fn_func_inspect_central_light_links(
        native._UniffiFfiConverterTypeCentralLightLinksRequest.lower(
            lox.CentralLightLinksRequest(bytes=data, selector=selector)), ctypes.byref(status))
    error = kind != lox.CentralLightLinks
    try:
        assert status.code == (1 if error else 0)
        address, prefix, minimum, length = calibration(status.error_buf if error else returned, large)
    finally:
        if status.code != 0:
            status.error_buf.free()
        returned.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0
    return prefix, minimum, length


def rejected_selector_releases_input(xml):
    data = b"<!--central-light-links-python-rejected-input:" + b"p" * 65536 + b"-->" + xml
    minimum = 4 + len(data)
    control = native._UniffiFfiConverterTypeCentralLightLinksRequest.lower(
        lox.CentralLightLinksRequest(bytes=data, selector=SELECT))
    try:
        assert control.len >= minimum
        address = ctypes.cast(control.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, 128)
        assert prefix == len(data).to_bytes(4, "big") + data[:124]
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        control.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0
    request = lox.CentralLightLinksRequest(bytes=data, selector="\ud800")
    for _ in range(64):
        try:
            lox.inspect_central_light_links(request)
        except UnicodeEncodeError as failure:
            assert failure.encoding == "utf-8" and failure.object == request.selector
            assert (failure.start, failure.end) == (0, 1)
        else:
            raise AssertionError("Expected strict host selector-encoding refusal")
    assert matching(prefix, minimum) == 0, "Rejected selector retained a marked native input"
    print("Python central-light-links invalid selector: input_live=1 input_free=0 "
          "rejections=64 input_native_retained=0 GC_requested=false")


assert sys.platform == "win32" and platform.machine() == "AMD64"
assert sys.implementation.name == "cpython" and sys.version_info[:2] == (3, 13)
assert not sysconfig.get_config_var("Py_GIL_DISABLED") and sys._is_gil_enabled()
assert "RUST_MIN_STACK" not in os.environ
gc.disable()
xml = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
       / "tests" / "fixtures" / "central_light_links.xml").read_bytes()
for selector in (f"uuid:{OWNER}", SELECT, "Zentrale & 灯 [dEn]"):
    assert result(outcome(xml, selector)) == (OWNER, TITLE, ROW_VALUES)
assert result(outcome(xml, "Empty")) == (
    "abcdef01-0002-0003-0000000000000002", "Empty", [])
selector = f"uuid:{OWNER.lower()}"
check(outcome(xml, selector), ERROR.NotFound, (selector, TITLES))
for data, kind in [
    (xml.replace(b"</ControlList>", b'<C Type="Document"/></ControlList>'), ERROR.NoSingleOwningDocument),
    (xml.replace(b'<C Type="Document">', b'<x:C Type="Document" xmlns:x="urn:test">')
        .replace(b"</C></ControlList>", b"</x:C></ControlList>"), ERROR.NotPlainDocument),
    (xml.replace(b'Type="Document">', b'Type="Document"><Template/>'), ERROR.DocumentTemplate),
    (xml.replace(b'Visu="true" Pr="den"', b'Visu="True" Pr="den"'), ERROR.NotDirectVisibleOwner),
    (xml.replace(f' U="{OWNER}"'.encode(), b""), ERROR.MissingOwnerIdentity),
]:
    check(outcome(data, SELECT), kind, None)
check(outcome(xml, "uuid:ordinary"), ERROR.NotPlainCentralLight, "uuid:ordinary")
check(outcome(xml.replace(b"</C></ControlList>", b'<C WF="268435456"/></C></ControlList>'), SELECT),
      ERROR.InconsistentMarker, None)
for raw, cause in [("", FLAGS.EMPTY), ("4294967296", FLAGS.POSITIVE_OVERFLOW)]:
    check(outcome(xml.replace(b"268451840", raw.encode()), SELECT), ERROR.InvalidFlags, (raw, cause))

marker = "links-marker-Καλημέρα-" + "x" * 65536
query = "links-absent-" + "y" * 65536
invalid_id = "links-invalid-id-" + "z" * 65536
invalid_flags = "links-invalid-flags-" + "f" * 65536
wrong_selector = f"uuid:{marker}"
long_xml = xml.replace("Zentrale &amp; 灯".encode(), marker.encode())
ambiguous_xml = long_xml.replace(b'U="ordinary"', f'U="ordinary" Title="{marker}"'.encode())
trials = [
    (long_xml, SELECT, lox.CentralLightLinks, (OWNER, marker, ROW_VALUES), True),
    (b"<C A='1' A='2'/>", SELECT, ERROR.Load, None, False),
    (xml, query, ERROR.NotFound, (query, TITLES), True),
    (ambiguous_xml, "links-marker-", ERROR.Ambiguous,
     ("links-marker-", [(marker, "CentralLight", OWNER), (marker, "Other", "ordinary")]), True),
    (f'<C Type="Other" U="{marker}"/>'.encode(), wrong_selector, ERROR.NotPlainCentralLight,
     wrong_selector, True),
    (xml.replace(SECOND.encode(), invalid_id.encode(), 1), SELECT, ERROR.InvalidIdentity, invalid_id, True),
    (xml.replace(SECOND.encode(), FIRST.lower().encode(), 1), SELECT, ERROR.DuplicateReference,
     FIRST.lower(), False),
    (xml.replace(SECOND.encode(), MISSING.upper().encode(), 1), SELECT, ERROR.UnresolvedControl, MISSING, False),
    (xml.replace(b"268451840", invalid_flags.encode()), SELECT, ERROR.InvalidFlags,
     (invalid_flags, FLAGS.INVALID_DIGIT), True),
    (xml.replace(b"268451840", b"16384"), SELECT, ERROR.InconsistentMarker, SECOND, False),
]
for data, selector, kind, expected, large in trials:
    prefix, minimum, length = calibrate(data, selector, kind, large)
    values = [outcome(data, selector) for _ in range(64)]
    assert len({id(value) for value in values}) == 64
    assert matching(prefix, minimum) == 0, f"{kind.__name__} retained a native result buffer"
    for value in values:
        check(value, kind, expected)
    if kind in (ERROR.Load, ERROR.InvalidFlags):
        assert all(value.source_message == values[0].source_message for value in values)
    print(f"Python central-light-links branch={kind.__name__} actual_bytes={length} live=1 free=0 "
          "calls=64 host_retained=64 native_retained=0 GC_requested=false")
rejected_selector_releases_input(xml)
print("Python central-light-links stored order/empty/typed errors PASS")
