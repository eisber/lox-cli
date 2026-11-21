"""Installed control move values, atomic errors, and scoped RustBuffer witnesses."""

import ctypes
import gc
import os
from pathlib import Path
import platform
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


ERROR = lox.ControlMoveError
SELECTOR = lox.ControlMoveSelector
CLOCK = lox.ControlMoveClock(unix_seconds=1230768000, nanoseconds=123456789, offset_seconds=3661)
LATER = lox.ControlMoveClock(unix_seconds=1230854400, nanoseconds=0, offset_seconds=-3661)
ROOM = "KÜCHE 室"
MALFORMED = b"<C A='1' A='2'/>"
TITLES = ["Dry", "Küche 室", "Küche 室 Annex", "Lamp Ω", "Mirror"]
TWIN = "move-element-"
ROOM_QUERY = "move-room-"


def input_bytes(text):
    return ("\ufeff" + text.replace("\n", "\r\n")).encode()


def request(data, type_filter=None, title="uuid:a", target_room=ROOM, exclude=(), clock=CLOCK):
    return lox.ControlMoveRequest(bytes=data, type_filter=type_filter, title=title,
                                  target_room=target_room, exclude=list(exclude), clock=clock)


def outcome(supplied):
    try:
        return lox.move_control(supplied)
    except ERROR as failure:
        return failure


def changed_bytes(data, mirror=False, later=False):
    output = (data.replace(b'Pr="old"', b'Pr="Raw-r"')
              .replace(b'Pr=""', b'Pr="Raw-r"')
              .replace(b'<IoData/>', b'<IoData Pr="Raw-r"/>'))
    if mirror:
        output = output.replace(b'Pr="mirror"', b'Pr="Raw-r"')
    return (output.replace(b'Date="old"', b'Date="2009-01-01 22:58:59"' if later else
                           b'Date="2009-01-01 01:01:32"')
            .replace(b'DateS="30"', b'DateS="86400"' if later else b'DateS="31"'))


def check_move(value, expected, selector, count, changed, stamped):
    assert isinstance(value, lox.ControlMoveOutcome)
    assert isinstance(value.bytes, bytes) and value.bytes == expected
    assert value.change == lox.ControlMoveChange(
        selector=selector, target_room=ROOM, room_uuid="Raw-r",
        selected_count=count, association_changed=changed)
    assert (value.content_changed, value.document_stamped) == (changed, stamped)


def raw_call(supplied):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_move_control(
        native._UniffiFfiConverterTypeControlMoveRequest.lower(supplied), ctypes.byref(status))
    return status, result


def refuse(supplied, kind):
    before = memoryview(supplied.bytes).tobytes()
    failure = outcome(supplied)
    assert isinstance(failure, kind) and supplied.bytes == before
    status, result = raw_call(supplied)
    try:
        assert status.code == 1 and result.len == 0, "Refusal returned revised bytes"
    finally:
        if status.code:
            status.error_buf.free()
        result.free()
    return failure


def check_values(data):
    for constructor, args in (
        (lox.ControlMoveClock, (1230768000, 123456789, 3661)),
        (lox.ControlMoveRequest, (data, None, "uuid:a", ROOM, [], CLOCK)),
    ):
        try:
            constructor(*args)
        except TypeError:
            pass
        else:
            raise AssertionError(f"{constructor.__name__} should be keyword-only")
    title = request(data, exclude=("future",))
    first = lox.move_control(title)
    check_move(first, changed_bytes(data), SELECTOR.TITLE("uuid:a"), 1, True, True)
    check_move(lox.move_control(request(first.bytes, clock=LATER)), first.bytes,
               SELECTOR.TITLE("uuid:a"), 1, False, False)
    check_move(lox.move_control(request(data, type_filter="fUtUrE", title="absent", clock=LATER)),
               changed_bytes(data, mirror=True, later=True), SELECTOR.TYPE("fUtUrE"), 2, True, True)
    check_move(lox.move_control(request(data, type_filter="Future", exclude=("FUTURE",))),
               data, SELECTOR.TYPE("Future"), 0, False, False)
    check_move(lox.move_control(request(data, title="Dry")), data, SELECTOR.TITLE("Dry"), 1, False, False)
    compact = '<Root><C Type="Place" U="Raw-r" Title="Küche 室"/></Root>'.encode()
    canonical = ('<?xml version="1.0" encoding="utf-8"?>\n<Root>\n'
                 '\t<C Type="Place" U="Raw-r" Title="Küche 室"/>\n</Root>\n').encode()
    check_move(lox.move_control(request(compact, type_filter="")), canonical,
               SELECTOR.TYPE(""), 0, False, False)
    unstamped = '<Root><C Type="Place" U="Raw-r" Title="Küche 室"/><C U="a"><IoData/></C></Root>'.encode()
    unstamped_result = (canonical.decode().replace("</Root>", '\t<C U="a">\n'
                        '\t\t<IoData Pr="Raw-r"/>\n\t</C>\n</Root>')).encode()
    check_move(lox.move_control(request(unstamped)), unstamped_result,
               SELECTOR.TITLE("uuid:a"), 1, True, False)
    assert title.bytes == data
    assert refuse(request(MALFORMED, title=None), ERROR.Load).source_message
    bad_clock = lox.ControlMoveClock(unix_seconds=0, nanoseconds=0, offset_seconds=86400)
    refuse(request(MALFORMED, title=None, clock=bad_clock), ERROR.InvalidClock)
    refuse(request(data, title=None, target_room="absent"), ERROR.SelectorMissing)
    assert refuse(request(data, title="absent", target_room="absent"), ERROR.NotFound).titles == TITLES
    assert refuse(request(data, type_filter="Future", title="absent", target_room="absent"),
                  ERROR.RoomNotFound).room == "absent"
    print("Python control-move keyword records/positional selector variants, exact bytes and small refusals: values only")


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


def live_extent(buffer, prefix):
    length = buffer.len
    address = ctypes.cast(buffer.data, ctypes.c_void_p).value
    assert address and len(prefix) == 128 and length > 2 * len(prefix)
    assert ctypes.string_at(address, len(prefix)) == prefix, "Not the named wire payload"
    assert probe.lox_test_heap_live(address), "Named buffer address not live"
    assert matching(prefix, length) == 1, "Named payload did not identify one live allocation"
    return address, length


def freed(label, extent, prefix):
    address, length = extent
    address_live = bool(probe.lox_test_heap_live(address))
    count = matching(prefix, length)
    print(f"Python control-move {label}: bytes={length} live_address=true live_payload_count=1 "
          f"post_free_address_live={address_live} post_free_payload_count={count}")
    assert count == 0, f"{label}: named payload still present in a live extent"
    assert not address_live, f"{label}: post-free address remains live; allocation identity unresolved"
    return length


def calibrate(label, supplied, prefix, code):
    status, result = raw_call(supplied)
    try:
        assert status.code == code
        assert code == 0 or result.len == 0, "Refusal returned revised bytes"
        extent = live_extent(status.error_buf if code else result, prefix)
    finally:
        if status.code:
            status.error_buf.free()
        result.free()
    return freed(label, extent, prefix)


def uint(value):
    return value.to_bytes(4, "big")


def append_value(prefix, data):
    head = prefix + uint(len(data))
    return head + data[:128 - len(head)]


def check_rollback():
    bad_string = "\ud800"
    for label, kwargs in (
        ("type-filter", {"type_filter": bad_string}),
        ("title", {"title": bad_string}),
        ("target-room", {"target_room": bad_string}),
        ("exclude-item", {"exclude": ("valid", bad_string)}),
    ):
        data = (f"<!--control-move-{label}-rejected-input:" + "q" * 65536 + "--><Root/>").encode()
        prefix = append_value(b"", data)
        control = native._UniffiFfiConverterTypeControlMoveRequest.lower(request(data))
        try:
            extent = live_extent(control, prefix)
        finally:
            control.free()
        freed(f"{label} marked input", extent, prefix)
        for _ in range(64):
            try:
                lox.move_control(request(data, **kwargs))
            except UnicodeEncodeError as error:
                assert (error.encoding, error.object, error.start, error.end) == ("utf-8", bad_string, 0, 1)
            else:
                raise AssertionError(f"{label}: expected strict pre-call UTF-8 refusal")
        assert matching(prefix, 4 + len(data)) == 0, f"{label}: marked native input retained"
        print(f"Python control-move rollback {label}: rejections=64 input_native_retained=0 GC_requested=false")


def check_owned(value, label, expected):
    if label == "changed":
        check_move(value, expected, SELECTOR.TITLE("uuid:a"), 1, True, True)
    elif label == "not-found":
        assert isinstance(value, ERROR.NotFound) and (value.selector, value.titles) == (expected, TITLES)
    elif label == "ambiguous":
        assert isinstance(value, ERROR.Ambiguous) and value.selector == TWIN
        assert value.matches == [
            lox.ControlMoveElementIdentity(title=expected, block_type="Future", uuid="first"),
            lox.ControlMoveElementIdentity(title=TWIN + "other", block_type="And", uuid="second"),
        ]
    elif label == "room-not-found":
        assert isinstance(value, ERROR.RoomNotFound) and value.room == expected
    elif label == "room-ambiguous":
        assert isinstance(value, ERROR.RoomAmbiguous) and value.room == ROOM_QUERY
        assert value.matches == [expected, ROOM_QUERY + "other", expected]
    else:
        raise AssertionError(f"Unknown branch {label}")


assert "RUST_MIN_STACK" not in os.environ
assert sys.platform == "win32" and platform.machine() == "AMD64"
assert sys.implementation.name == "cpython" and sys.version_info[:2] == (3, 13) and sys._is_gil_enabled()
gc.disable()
fixture = Path(sys.argv[2]).read_text(encoding="utf-8")
data = input_bytes(fixture)
check_values(data)
mark = "control-move-wire-result-灯-" + "x" * 65536
marked = input_bytes(fixture.replace("<ControlList>", f'<ControlList Marker="{mark}">'))
expected_bytes = changed_bytes(marked)
missing = "control-move-wire-missing-" + "y" * 65536
twin = TWIN + "control-move-wire-ambiguous-" + "z" * 65536
room_missing = "control-move-wire-room-missing-" + "m" * 65536
room_twin = ROOM_QUERY + "control-move-wire-room-ambiguous-" + "a" * 65536
rooms = (f'<Root><C Type="Place" U="r" Title="{room_twin}"/>'
         f'<C Type="Place" U="other" Title="{ROOM_QUERY}other"/>'
         f'<C Type="Place" U="r" Title="{room_twin}"/></Root>').encode()
# Exact UniFFI prefixes: byte length and raw output, or error tag and leading fields.
trials = [
    ("changed", request(marked), append_value(b"", expected_bytes), 0, expected_bytes),
    ("not-found", request(data, title=missing, target_room="absent"),
     append_value(uint(4), missing.encode()), 1, missing),
    ("ambiguous", request((f'<Root><C U="first" Type="Future" Title="{twin}"/>'
                           f'<C U="second" Type="And" Title="{TWIN}other"/></Root>').encode(),
                          title=TWIN, target_room="absent"),
     append_value(append_value(uint(5), TWIN.encode()) + uint(2), twin.encode()), 1, twin),
    ("room-not-found", request(data, type_filter="Future", title=missing, target_room=room_missing),
     append_value(uint(6), room_missing.encode()), 1, room_missing),
    ("room-ambiguous", request(rooms, type_filter="", title=None, target_room=ROOM_QUERY),
     append_value(append_value(uint(7), ROOM_QUERY.encode()) + uint(3), room_twin.encode()), 1, room_twin),
]
for label, supplied, prefix, code, expected in trials:
    length = calibrate(label, supplied, prefix, code)
    original = memoryview(supplied.bytes).tobytes()
    held = [outcome(supplied) for _ in range(64)]
    assert len({id(value) for value in held}) == 64 and supplied.bytes == original
    assert matching(prefix, length) == 0, f"{label}: public calls retained a named native payload"
    for value in held:
        check_owned(value, label, expected)
    if code == 0:
        assert len({id(value.bytes) for value in held}) == 64
    print(f"Python control-move {label}: host_retained=64 native_payload_retained=0 GC_requested=false")
check_rollback()
print("Python control-move Load, short/no-op results and payload-free errors: values only")
