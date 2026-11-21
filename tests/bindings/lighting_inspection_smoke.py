"""Installed Lighting inspection values and exact marked allocation witnesses."""

import ctypes
import gc
import os
from pathlib import Path
import platform
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


FIXTURE = Path(sys.argv[2]).read_text(encoding="utf-8")
FAILURE = lox.LightingInspectionError
MALFORMED = b"<Root><C attr='1' attr='2'/></Root>"
AMBIGUOUS = "Lighting inventory twin"

probe = ctypes.CDLL(sys.argv[1])
probe.lox_test_heap_count.argtypes = (
    ctypes.c_void_p, ctypes.c_size_t, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t))
probe.lox_test_heap_count.restype = ctypes.c_uint32
probe.lox_test_heap_live.argtypes = (ctypes.c_void_p,)
probe.lox_test_heap_live.restype = ctypes.c_int


def request(xml, selector):
    return lox.LightingInspectionRequest(bytes=xml, selector=selector)


def outcome(value):
    try:
        return lox.inspect_lighting(value)
    except FAILURE as failure:
        return failure


def attrs(entries):
    return {entry.key: entry.value for entry in entries}


def check_inventory(value, uuid):
    assert isinstance(value, lox.LightingInspection)
    assert (value.uuid, value.title) == (uuid, "Lamps & 灯")
    assert [entry.key for entry in value.attributes] == ["CapAI0", "Title", "Type", "U", "gid"]
    assert attrs(value.attributes)["CapAI0"] == "+0007"
    assert [attrs(row.attributes) for row in value.document_versions] == [
        {"Version": "+0267"}, {"ConfigVersion": "17.2", "V": "opaque"}]
    assert (value.direct_scene_collection_count, value.sole_scene_collection) == (1, 1)
    assert [node.element for node in value.nested_storage] == ["IoData", "LightscenesC", "HCL"]
    scenes = value.nested_storage[1]
    assert attrs(scenes.attributes) == {"Num": "99", "Outputs": "0002"}
    sequence = scenes.children[0].children[0]
    assert (sequence.element, attrs(sequence.attributes)) == ("SeqConf", {"Flag": "0"})
    assert sequence.children[0].element == "LightsceneC"

    direct, nested = value.moods
    assert [(row.name, row.uuid, row.sid, row.cid, row.q1)
            for row in (direct.summary, nested.summary)] == [
        ("Direct", "same", None, 9, ""), ("Nested", "same", None, None, "-1")]
    assert direct.in_sole_direct_collection and not nested.in_sole_direct_collection
    assert attrs(direct.attributes)["SID"] == "bad" and attrs(direct.attributes)["Q01"] == "unprojected"
    assert [(row.key, row.index, row.stored) for row in direct.outputs] == [
        ("Q10", 10, "opaque"), ("Q2", 2, "+0006"), ("Q1", 1, "")]

    connected, missing = value.channels
    assert (connected.key, connected.index, connected.uuid, connected.uuid_occurrences) == (
        "AQ10", 10, "src", 2)
    assert attrs(connected.attributes)["Future"] == "keep & stay"
    assert (missing.key, missing.index, missing.uuid, missing.uuid_occurrences, missing.downstream) == (
        "AQ2", 2, None, None, [])
    wired, loose = connected.downstream
    assert (wired.block_uuid, wired.block_type, wired.block_title, wired.connector_uuid,
            wired.connector_key, attrs(wired.wire_attributes)) == (
        "wrong", "And", "Lamps & 灯", "in", "I1", {"FLG": "02", "Input": "src"})
    assert (loose.block_uuid, loose.connector_uuid, loose.connector_key, attrs(loose.wire_attributes)) == (
        "wrong", None, None, {"Future": "unresolved", "Input": "src"})


def check_values():
    check_inventory(lox.inspect_lighting(request(FIXTURE.encode(), "Lamps [study]")), "light")
    multiple = lox.inspect_lighting(request(
        b'<C Type="LightController2"><LightscenesC><LightsceneC/></LightscenesC><LightscenesC/></C>',
        "Type:LightController2"))
    assert multiple.uuid is None and multiple.title is None
    assert multiple.direct_scene_collection_count == 2 and multiple.sole_scene_collection is None
    assert not multiple.moods[0].in_sole_direct_collection
    empty = lox.inspect_lighting(request(b'<C Type="LightController2" U="empty"/>', "uuid:empty"))
    assert empty.moods == empty.channels == empty.nested_storage == []
    load = outcome(request(MALFORMED, "uuid:missing"))
    assert isinstance(load, FAILURE.Load) and load.source_message
    wrong = outcome(request(b'<C U="sparse"/>', "uuid:sparse"))
    assert isinstance(wrong, FAILURE.NotLightController) and wrong.block_type == ""


def matching(prefix, minimum):
    count = ctypes.c_size_t()
    assert probe.lox_test_heap_count(prefix, len(prefix), minimum, ctypes.byref(count)) == 0
    return count.value


def address(buffer):
    return ctypes.cast(buffer.data, ctypes.c_void_p).value


def verify_live(buffer, prefix):
    length = buffer.len
    assert len(prefix) == 128 and length > 2 * len(prefix)
    pointer = address(buffer)
    assert ctypes.string_at(pointer, len(prefix)) == prefix, "Not the named wire payload"
    assert probe.lox_test_heap_live(pointer), "No live allocation at the returned address"
    assert matching(prefix, length) == 1, "Named payload did not identify one live allocation"
    return pointer, length


def verify_freed(label, pointer, prefix, length):
    address_live = bool(probe.lox_test_heap_live(pointer))
    count = matching(prefix, length)
    print(f"Python Lighting {label}: bytes={length} live_address=true live_payload_count=1 "
          f"post_free_address_live={address_live} post_free_payload_count={count}", flush=True)
    assert count == 0, f"{label}: named payload still present in a live extent"
    assert not address_live, f"{label}: post-free address remains live; allocation identity unresolved"


def calibrate(label, value, prefix, code):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_inspect_lighting(
        native._UniffiFfiConverterTypeLightingInspectionRequest.lower(value), ctypes.byref(status))
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


def check_owned(value, label, expected):
    if label == "inventory":
        check_inventory(value, expected)
    elif label == "not-found":
        assert isinstance(value, FAILURE.NotFound)
        assert (value.selector, value.titles) == (expected, ["Alpha", "Zulu"])
    elif label == "ambiguous":
        assert isinstance(value, FAILURE.Ambiguous) and value.selector == AMBIGUOUS
        assert [(row.title, row.block_type, row.uuid) for row in value.matches] == [
            (expected, "And", "first"), (AMBIGUOUS + " other", "LightController2", "second")]
    elif label == "wrong-type":
        assert isinstance(value, FAILURE.NotLightController)
        assert (value.selector, value.block_type) == ("uuid:wrong", expected)
    else:
        raise AssertionError(f"Unknown branch {label}")


def uint(value):
    return value.to_bytes(4, "big")


def serialized_string(value):
    data = value.encode()
    return uint(len(data)) + data


def check_rollback():
    xml = b"<!--lighting-inspection-invalid-selector-v1:" + b"q" * 65536 + b"--><Root/>"
    prefix = (uint(len(xml)) + xml)[:128]
    control = native._UniffiFfiConverterTypeLightingInspectionRequest.lower(request(xml, "uuid:valid"))
    try:
        pointer, length = verify_live(control, prefix)
    finally:
        control.free()
    verify_freed("marked input control", pointer, prefix, length)
    for _ in range(64):
        try:
            lox.inspect_lighting(request(xml, "\ud800"))
        except UnicodeEncodeError as error:
            assert (error.encoding, error.object, error.start, error.end) == ("utf-8", "\ud800", 0, 1)
        else:
            raise AssertionError("Expected strict pre-call UTF-8 refusal")
    assert matching(prefix, 4 + len(xml)) == 0, "UTF-8 refusal retained the marked request"
    print("Python Lighting selector rollback: rejections=64 input_native_retained=0 GC_requested=false")


assert "RUST_MIN_STACK" not in os.environ
assert sys.platform == "win32" and platform.machine() == "AMD64"
assert sys.implementation.name == "cpython" and sys.version_info[:2] == (3, 13) and sys._is_gil_enabled()
gc.disable()
check_values()
uuid = "lighting-inspection-inventory-v1-灯-" + "u" * 65536
missing = "uuid:lighting-inspection-missing-v1-灯-" + "m" * 65536
twin = AMBIGUOUS + " lighting-inspection-ambiguity-v1-灯-" + "a" * 65536
wrong_type = "lighting-inspection-wrong-type-v1-灯-" + "w" * 65536
# Exact UniFFI prefixes: a present UUID, or a typed error tag followed by its fields.
trials = [
    ("inventory", request(FIXTURE.replace('U="light"', f'U="{uuid}"').encode(), "gid:PRIMARY"),
     b"\x01" + serialized_string(uuid), 0, uuid),
    ("not-found", request(b'<Root><C Title="Zulu"/><C Title="Alpha"/><C Title="Zulu"/></Root>', missing),
     uint(2) + serialized_string(missing), 1, missing),
    ("ambiguous", request((f'<Root><C Type="And" Title="{twin}" U="first"/>'
                           f'<C Type="LightController2" Title="{AMBIGUOUS} other" U="second"/></Root>').encode(),
                          AMBIGUOUS),
     uint(3) + serialized_string(AMBIGUOUS) + uint(2) + serialized_string(twin), 1, twin),
    ("wrong-type", request(f'<C Type="{wrong_type}" U="wrong"/>'.encode(), "uuid:wrong"),
     uint(4) + serialized_string("uuid:wrong") + serialized_string(wrong_type), 1, wrong_type),
]
for label, supplied, encoded, code, expected in trials:
    prefix = encoded[:128]
    length = calibrate(label, supplied, prefix, code)
    held = [outcome(supplied) for _ in range(64)]
    assert len({id(value) for value in held}) == 64
    assert matching(prefix, length) == 0, f"{label}: public calls retained a named native payload"
    for value in held:
        check_owned(value, label, expected)
    print(f"Python Lighting {label}: host_retained=64 native_payload_retained=0 GC_requested=false")
check_rollback()
print("Python Lighting Load, small/empty inventories, and absent fields: values only")
