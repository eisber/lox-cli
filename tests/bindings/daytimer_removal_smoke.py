"""Installed DayTimer edit values, transaction refusals and RustBuffer attribution."""

import ctypes
import gc
import os
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


BLOCK = "ABCDEF00-0000-0004-FFFF000000000000"
BLOCK_RFC = "ABCDEF00-0000-0004-FFFF-000000000000"
MODE = "abcdef01-abcd-0004-1500-0000abcdef00"
MODE_NATIVE = "ABCDEF01-ABCD-0004-15000000ABCDEF00"
MODE_B = "00000000-0000-0005-1500000000000000"
CLOCK = lox.DaytimerRemovalClock(unix_seconds=1230768000, nanoseconds=123456789, offset_seconds=3661)
LATER = lox.DaytimerRemovalClock(unix_seconds=1230940800, nanoseconds=987654321, offset_seconds=-3661)
FIXTURE = Path("crates", "lox-bindings", "tests", "fixtures", "daytimer_removal.xml")
XML = "\ufeff" + FIXTURE.read_text(encoding="utf-8").replace("\n", "\r\n")
MARKER = "entfernt & 灯"
SELECTED = '\t\t\t\t<Entry To="180" V="+005" Marker="entfernt &amp; 灯"/>\r\n'
SURVIVOR = '\t\t\t\t<Entry To="180" Ix="+01" V="+004" NeedActivate="TrUe"/>\r\n'
MALFORMED = b"<Root><C attr='1' attr='2'/></Root>"
Failure = lox.DaytimerRemovalError


def request(data, block_uuid=BLOCK, mode_uuid=MODE, from_minutes=0, to_minutes=180, clock=CLOCK):
    return lox.DaytimerRemovalRequest(bytes=data, block_uuid=block_uuid, mode_uuid=mode_uuid,
                                      from_minutes=from_minutes, to_minutes=to_minutes, clock=clock)


def outcome(value):
    try:
        return lox.remove_daytimer_entry(value)
    except Failure as failure:
        return failure


def first_bytes(source, selected):
    return (source.replace(selected, "", 1).replace(' N="+02"', ' N="1"', 1)
            .replace('Date="2009-01-02 00:00:00" DateS="86400"',
                     'Date="2009-01-02 01:01:02" DateS="86401"', 1)).encode()


def check_first(value, expected, marker):
    assert isinstance(value, lox.DaytimerRemovalOutcome) and value.bytes == expected
    change = value.removal
    assert isinstance(change, lox.DaytimerRemovedEntry)
    assert (change.block_uuid, change.requested_mode_uuid, change.stored_mode_uuid) == (BLOCK, MODE, MODE_NATIVE)
    assert (change.from_minutes, change.to_minutes, change.remaining_entries) == (0, 180, 1)
    assert change.removed_attributes == {"To": "180", "V": "+005", "Marker": marker}


def check_cases():
    original = request(XML.encode())
    first = lox.remove_daytimer_entry(original)
    expected_first = first_bytes(XML, SELECTED)
    check_first(first, expected_first, MARKER)
    later = lox.remove_daytimer_entry(request(original.bytes, clock=LATER))
    check_first(later, expected_first.replace(b'2009-01-02 01:01:02" DateS="86401"',
                                             b'2009-01-02 22:58:59" DateS="172800"', 1), MARKER)
    last = lox.remove_daytimer_entry(request(first.bytes, mode_uuid=MODE_B))
    expected = (first.bytes.replace(SURVIVOR.encode(), b"", 1).replace(b' N="1"', b"", 1)
                .replace(b'01:01:02" DateS="86401"', b'01:01:03" DateS="86402"', 1))
    assert last.bytes == expected
    assert last.removal.remaining_entries == 0
    assert (last.removal.block_uuid, last.removal.requested_mode_uuid, last.removal.stored_mode_uuid,
            last.removal.from_minutes, last.removal.to_minutes) == (BLOCK, MODE_B, MODE_B, 0, 180)
    assert last.removal.removed_attributes == {
        "To": "180", "Ix": "+01", "V": "+004", "NeedActivate": "TrUe"}

    program = (XML.replace(BLOCK, BLOCK_RFC).replace('<C Type="Page">', '<C Type="Program"><C Type="Page">')
               .replace('\t\t</C>\r\n\t\t<C Type="Mode"', '\t\t</C></C>\r\n\t\t<C Type="Mode"', 1))
    admitted = lox.remove_daytimer_entry(request(program.encode(), block_uuid=BLOCK_RFC, mode_uuid=MODE_NATIVE))
    assert (admitted.removal.block_uuid, admitted.removal.requested_mode_uuid,
            admitted.removal.stored_mode_uuid) == (BLOCK_RFC, MODE_NATIVE, MODE_NATIVE)

    data = XML.encode()
    for rejected, error in (
        (request(last.bytes), Failure.EntryNotFound),
        (request(data, block_uuid=BLOCK.lower()), Failure.NotFound),
        (request(data, block_uuid=BLOCK_RFC), Failure.NotFound),
        (request(data, block_uuid="Schedule [Kitchen]"), Failure.InvalidBlockUuid),
        (request(data, block_uuid=f"uuid:{BLOCK}"), Failure.InvalidBlockUuid),
        (request(data, mode_uuid="00000000-0000-0000-0000-000000000000"), Failure.InvalidModeUuid),
        (request(data, from_minutes=180), Failure.InvalidInterval),
        (request(data, to_minutes=1441), Failure.InvalidInterval),
        (request(MALFORMED, clock=lox.DaytimerRemovalClock(unix_seconds=0, nanoseconds=0, offset_seconds=86400)), Failure.InvalidClock),
        (request(b"<Root/>"), Failure.NotFound),
        (request(XML.replace('Type="Page"', 'Type="Folder"').encode()), Failure.OutsidePage),
        (request(XML.replace('Analog="false"', 'Analog="true"').encode()), Failure.AnalogSchedule),
        (request(XML.replace(' N="+02"', ' N="3"').encode()), Failure.EntryCountMismatch),
        (request(XML.replace(f"{MODE_NATIVE},{MODE_B}", f"{MODE_NATIVE},{MODE}").encode()),
         Failure.DuplicateModes),
        (request(XML.replace(SELECTED, '<x:Entry xmlns:x="urn:x" To="180"/>').encode()),
         Failure.NamespacedEntry),
        (request(XML.replace(SURVIVOR, '<Entry xmlns="urn:x" To="180" Ix="1"/>').encode()),
         Failure.NamespacedEntry),
    ):
        before = memoryview(rejected.bytes).tobytes()
        failure = outcome(rejected)
        assert isinstance(failure, error), (type(failure), error)
        assert not hasattr(failure, "bytes") and rejected.bytes == before
        if before == b"<Root/>":
            assert failure.titles == []
    assert original.bytes == data
    print("Python DayTimer last-entry/empty suggestions/payload-free branches: value checks only")


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


def calibrate(value, success, large):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_remove_daytimer_entry(
        native._UniffiFfiConverterTypeDaytimerRemovalRequest.lower(value), ctypes.byref(status))
    buffer = status.error_buf if status.code else result
    try:
        assert status.code == (0 if success else 1)
        assert success or result.len == 0, "Typed refusal returned a result buffer"
        length = buffer.len
        prefix_len, minimum = (128, 64000) if large else (12, length)
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


def check_rollback(field):
    xml = f"<!--daytimer-removal-{field}-rejected-input:".encode() + b"q" * 65536 + b"--><Root/>"
    minimum = 4 + len(xml)
    control = native._UniffiFfiConverterTypeDaytimerRemovalRequest.lower(request(xml))
    try:
        assert control.len >= minimum
        address = ctypes.cast(control.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, 128)
        assert prefix == len(xml).to_bytes(4, "big") + xml[:124]
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        control.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0
    bad = request(xml, **{field: "\ud800"})
    for _ in range(64):
        try:
            lox.remove_daytimer_entry(bad)
        except UnicodeEncodeError as failure:
            assert failure.encoding == "utf-8" and failure.object == getattr(bad, field)
            assert (failure.start, failure.end) == (0, 1)
        else:
            raise AssertionError(f"Expected strict pre-call {field} refusal")
    assert matching(prefix, minimum) == 0, f"Rejected {field} retained marked native XML"
    print(f"Python DayTimer {field} pre-call UTF-8: input_live=1 input_free=0 "
          "rejections=64 input_native_retained=0 GC_requested=false")


def check_error(value, variant, fields):
    assert isinstance(value, variant) and not hasattr(value, "bytes")
    for key, expected in fields.items():
        actual = getattr(value, key)
        if expected is None:
            assert actual
        else:
            assert actual == expected


assert "RUST_MIN_STACK" not in os.environ
assert sys.version_info[:2] == (3, 13) and sys._is_gil_enabled()
gc.disable()
check_cases()
large = "daytimer-removal-Καλημέρα-" + "x" * 65536
marked = XML.replace("KeEp", large).replace("entfernt", large)
wrong_type = XML.replace('Type="DayTimer"', 'Type="And"')
missing_mode = XML.replace(f'U="{MODE}"', f'U="{BLOCK_RFC}"')
selector = f"uuid:{BLOCK}"
trials = [
    ("success", request(marked.encode()), True, None, {}),
    ("load", request(MALFORMED), False, Failure.Load, {"source_message": None}),
    ("missing", request(XML.replace('Title="Schedule"', f'Title="{large}"').encode(),
                        block_uuid=BLOCK_RFC), True, Failure.NotFound,
     {"selector": f"uuid:{BLOCK_RFC}", "titles": [large]}),
    ("wrong-type", request(wrong_type.encode()), False, Failure.NotDayTimer, {"selector": selector}),
    ("unresolved-mode", request(missing_mode.encode()), False, Failure.UnresolvedMode,
     {"stored_mode_uuid": MODE_NATIVE}),
    ("invalid-integer", request(XML.replace(' N="+02"', f' N="{large}"').encode()), True,
     Failure.InvalidInteger, {"attribute": "N", "value": large}),
]
for branch, value, is_large, variant, fields in trials:
    success = variant is None
    prefix, minimum, length = calibrate(value, success, is_large)
    snapshot = memoryview(value.bytes).tobytes()
    held = [outcome(value) for _ in range(64)]
    assert len({id(item) for item in held}) == 64 and value.bytes == snapshot
    assert matching(prefix, minimum) == 0, f"{branch} retained a marked native result"
    if success:
        assert len({id(item.bytes) for item in held}) == 64
        assert len({id(item.removal.removed_attributes) for item in held}) == 64
        expected = first_bytes(marked, SELECTED.replace("entfernt", large))
        for item in held:
            check_first(item, expected, MARKER.replace("entfernt", large))
    else:
        for item in held:
            check_error(item, variant, fields)
        if branch == "load":
            assert all(item.source_message == held[0].source_message for item in held)
    print(f"Python DayTimer branch={branch} actual_bytes={length} live=1 free=0 "
          "calls=64 host_retained=64 native_retained=0 GC_requested=false")
for field in ("block_uuid", "mode_uuid"):
    check_rollback(field)
print("Python DayTimer removal exact bytes/typed atomic errors PASS")
