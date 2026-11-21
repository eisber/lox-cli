"""Installed Load Manager values, named RustBuffer witnesses, and selector rollback."""

import ctypes
import gc
import inspect
import os
from pathlib import Path
import platform
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


FIXTURE = Path(sys.argv[2]).read_text(encoding="utf-8")
FAILURE = lox.LoadManagerInspectionError
FIRST = '<Load nm="" maxOff="" pwr="unknown" Note="at 7am=ok"/>'
ONE = '<Load ioIdx="1"/>'
TWIN = "load-manager inventory twin"

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_count.restype = ctypes.c_uint32
probe.lox_test_heap_live.argtypes = (ctypes.c_void_p,)
probe.lox_test_heap_live.restype = ctypes.c_int


def request(xml, selector):
    return lox.LoadManagerInspectionRequest(bytes=xml.encode(), selector=selector)


def outcome(supplied):
    try:
        return lox.inspect_load_manager(supplied)
    except FAILURE as failure:
        return failure


def check_keyword_only():
    for record, names in (
        (lox.LoadManagerInspectionRequest, ("bytes", "selector")),
        (lox.LoadManagerInspection, ("uuid", "title", "loads")),
        (lox.LoadManagerLoad, ("record_order", "io_index", "attributes")),
        (lox.LoadManagerAttribute, ("key", "value")),
        (lox.LoadManagerSelectionIdentity, ("title", "block_type", "uuid")),
    ):
        parameters = inspect.signature(record).parameters
        assert tuple(parameters) == names
        assert all(parameter.kind is inspect.Parameter.KEYWORD_ONLY
                   and parameter.default is inspect.Parameter.empty
                   for parameter in parameters.values()), record.__name__
    print("Python Load Manager: all five generated record constructors are keyword-only")


def check_inventory(value, uuid):
    assert isinstance(value, lox.LoadManagerInspection)
    assert (value.uuid, value.title) == (uuid, "Loads & 灯")
    assert all(isinstance(row, lox.LoadManagerLoad) for row in value.loads)
    order = [5, 0, 3, 1, 2, 4, 6, 7, 8, 9, 10, 11]
    assert [(row.record_order, row.io_index) for row in value.loads] == list(enumerate(order, 1))
    expected = [
        [("Future", ""), ("ioIdx", "05"), ("minOn", "+0012"), ("nm", " A & B "), ("pwr", "02.5000e0")],
        [("Note", "at _7am=ok"), ("maxOff", ""), ("nm", ""), ("pwr", "unknown")],
        *[[("ioIdx", str(index))] for index in order[2:]],
    ]
    assert [row.attributes for row in value.loads] == [
        [lox.LoadManagerAttribute(key=key, value=text) for key, text in entries]
        for entries in expected]


def check_values():
    for selector in ("uuid:load-manager", "Loads [work]", "gid:LOAD-BANK", "Type:loadshed"):
        check_inventory(lox.inspect_load_manager(request(FIXTURE, selector)), "load-manager")
    unnamed = lox.inspect_load_manager(request(
        FIXTURE.replace(' U="load-manager" Title="Loads &amp; 灯"', "").replace(FIRST, "<Load/>"),
        "Type:LoadShed"))
    assert (unnamed.uuid, unnamed.title, len(unnamed.loads)) == ("", "", 12)
    assert unnamed.loads[1].io_index == 0 and unnamed.loads[1].attributes == []

    malformed = outcome(request("<Root><C attr='1' attr='2'/></Root>", "uuid:missing"))
    assert isinstance(malformed, FAILURE.Load) and malformed.source_message
    wrong = outcome(request('<C Type="loadshed"/>', "Type:LoadShed"))
    assert isinstance(wrong, FAILURE.NotLoadManager) and wrong.selector == "Type:LoadShed"
    for xml, found in (
        (FIXTURE.replace(ONE, '<Load ioIdx="+1"/>').replace('<Load ioIdx="11"/>', ""), 11),
        (FIXTURE.replace(ONE, ONE + "<Load/>"), 13),
    ):
        count = outcome(request(xml, "uuid:load-manager"))
        assert isinstance(count, FAILURE.RecordCount) and count.found == found
    for raw in ("", "+1", " 1", "12", "opaque"):
        invalid = outcome(request(FIXTURE.replace(ONE, f'<Load ioIdx="{raw}"/>'), "uuid:load-manager"))
        assert isinstance(invalid, FAILURE.InvalidIoIndex) and invalid.raw == raw
    for replacement in ('<Load ioIdx="000"/>', "<Load/>"):
        duplicate = outcome(request(FIXTURE.replace(ONE, replacement), "uuid:load-manager"))
        assert isinstance(duplicate, FAILURE.DuplicateIoIndex) and duplicate.io_index == 0


def matching(prefix, minimum):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), minimum, ctypes.byref(count)) == 0
    return count.value


def verify_live(buffer, prefix):
    length = buffer.len
    assert len(prefix) == 128 and length > 65536
    pointer = ctypes.cast(buffer.data, ctypes.c_void_p).value
    assert ctypes.string_at(pointer, len(prefix)) == prefix, "Not the named wire payload"
    assert probe.lox_test_heap_live(pointer), "No live allocation at the returned address"
    assert matching(prefix, length) == 1, "Expected one live allocation with the named payload"
    return pointer, length


def verify_freed(label, pointer, prefix, length):
    address_live = bool(probe.lox_test_heap_live(pointer))
    count = matching(prefix, length)
    print(f"Python Load Manager {label}: bytes={length} live_address=true live_payload_count=1 "
          f"post_free_address_live={address_live} post_free_payload_count={count}", flush=True)
    assert count == 0, f"{label}: named payload remains live"
    assert not address_live, f"{label}: released address still live; allocation identity unresolved"


def calibrate(label, supplied, prefix, code):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_inspect_load_manager(
        native._UniffiFfiConverterTypeLoadManagerInspectionRequest.lower(supplied), ctypes.byref(status))
    buffer = status.error_buf if status.code else result
    try:
        assert status.code == code
        assert code == 0 or result.len == 0, "Typed refusal returned a result payload"
        pointer, length = verify_live(buffer, prefix)
    finally:
        if status.code:
            buffer.free()
        result.free()
    verify_freed(label, pointer, prefix, length)
    return length


def uint(value):
    return value.to_bytes(4, "big")


def wire_string(value):
    data = value.encode()
    return uint(len(data)) + data


def check_rollback():
    xml = b"<!--load-manager-invalid-selector-v1:" + b"q" * 65536 + b"--><Root/>"
    prefix = (uint(len(xml)) + xml)[:128]
    control = native._UniffiFfiConverterTypeLoadManagerInspectionRequest.lower(
        lox.LoadManagerInspectionRequest(bytes=xml, selector="uuid:valid"))
    try:
        pointer, length = verify_live(control, prefix)
    finally:
        control.free()
    verify_freed("marked input control", pointer, prefix, length)

    bad = lox.LoadManagerInspectionRequest(bytes=xml, selector="\ud800")
    for _ in range(64):
        try:
            lox.inspect_load_manager(bad)
        except UnicodeEncodeError as error:
            assert (error.encoding, error.object, error.start, error.end) == ("utf-8", "\ud800", 0, 1)
        else:
            raise AssertionError("Expected strict pre-call UTF-8 refusal")
    assert matching(prefix, 4 + len(xml)) == 0, "UTF-8 refusal stranded the marked input"
    print("Python Load Manager selector rollback: rejections=64 input_native_retained=0 GC_requested=false")


def check_owned(label, value, expected):
    if label == "inventory":
        check_inventory(value, expected)
    elif label == "not-found":
        assert isinstance(value, FAILURE.NotFound)
        assert (value.selector, value.titles) == (expected, ["Alpha", "Zulu"])
    elif label == "ambiguous":
        assert isinstance(value, FAILURE.Ambiguous) and value.selector == TWIN
        assert value.matches == [
            lox.LoadManagerSelectionIdentity(title=expected, block_type="Switch", uuid="first"),
            lox.LoadManagerSelectionIdentity(title=TWIN + " other", block_type="LoadShed", uuid="second"),
        ]
    elif label == "wrong-type":
        assert isinstance(value, FAILURE.NotLoadManager) and value.selector == expected
    elif label == "invalid-index":
        assert isinstance(value, FAILURE.InvalidIoIndex) and value.raw == expected
    else:
        raise AssertionError(f"Unknown branch {label}")


assert "RUST_MIN_STACK" not in os.environ
assert sys.platform == "win32" and platform.machine() == "AMD64"
assert sys.implementation.name == "cpython" and sys.version_info[:2] == (3, 13) and sys._is_gil_enabled()
gc.disable()
check_keyword_only()
check_values()
uuid = "load-manager-inventory-v1-灯-" + "u" * 65536
missing = "uuid:load-manager-missing-v1-灯-" + "m" * 65536
twin = TWIN + " load-manager-ambiguity-v1-灯-" + "a" * 65536
wrong = "load-manager-wrong-type-v1-灯-" + "w" * 65536
invalid = "load-manager-invalid-index-v1-灯-" + "i" * 65536
# Exact wire prefixes: first field of the success record, or the one-based error tag and fields.
trials = [
    ("inventory", request(FIXTURE.replace('U="load-manager"', f'U="{uuid}"'), "gid:LOAD-BANK"),
     wire_string(uuid), 0, uuid),
    ("not-found", request('<Root><C Title="Zulu"/><C Title="Alpha"/><C Title="Zulu"/></Root>', missing),
     uint(2) + wire_string(missing), 1, missing),
    ("ambiguous", request(f'<Root><C Type="Switch" Title="{twin}" U="first"/>'
                          f'<C Type="LoadShed" Title="{TWIN} other" U="second"/></Root>', TWIN),
     uint(3) + wire_string(TWIN) + uint(2) + wire_string(twin), 1, twin),
    ("wrong-type", request(f'<C Type="loadshed" U="{wrong}"/>', "uuid:" + wrong),
     uint(4) + wire_string("uuid:" + wrong), 1, "uuid:" + wrong),
    ("invalid-index", request(FIXTURE.replace(ONE, f'<Load ioIdx="{invalid}"/>'), "uuid:load-manager"),
     uint(6) + wire_string(invalid), 1, invalid),
]
for label, supplied, encoded, code, expected in trials:
    prefix = encoded[:128]
    length = calibrate(label, supplied, prefix, code)
    held = [outcome(supplied) for _ in range(64)]
    assert len({id(value) for value in held}) == 64
    assert matching(prefix, length) == 0, f"{label}: public calls retained a named native payload"
    for value in held:
        check_owned(label, value, expected)
    print(f"Python Load Manager {label}: host_retained=64 native_payload_retained=0 GC_requested=false")
check_rollback()
print("Python Load Manager: Load, RecordCount, DuplicateIoIndex, and empty fields are values only; Other unobserved")
