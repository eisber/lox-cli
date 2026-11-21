"""Installed stored Status Monitor inspection and RustBuffer ownership."""

import ctypes
import gc
import os
from pathlib import Path
import sys

import lox_archive as lox
from lox_archive import lox_archive as native


def inspect(data, selector):
    return lox.inspect_status_monitor(lox.StatusMonitorRequest(bytes=data, selector=selector))


def outcome(data, selector):
    try:
        return inspect(data, selector)
    except lox.StatusMonitorError as failure:
        return failure


def check_inventory(value, title="Monitor & 灯", uuid="mon"):
    assert isinstance(value, lox.StatusMonitorInspection)
    assert (value.uuid, value.title) == (uuid, title)
    assert [state.priority for state in value.states] == list(range(1, 12))
    assert [state.output_index for state in value.states] == [2, 0, 1, *range(3, 11)]
    ids = ["A0000000-0000-0000-0000000000000001"]
    ids.extend(f"a0000000-0000-0000-{i:016x}" for i in range(2, 12))
    assert [state.id for state in value.states] == ids
    assert [state.configured for state in value.states] == [True] + [False] * 10
    expected = [
        [("Future", ""), ("Id", ids[0]), ("Na", "  Küche e\u0301 & 灯  "),
         ("OIdx", "02"), ("Val", "+01.2500")],
        [("Id", ids[1]), ("Na", ""), ("OIdx", "0"), ("Val", "-002.0")],
        [("Id", ids[2]), ("Na", "No value"), ("OIdx", "1")],
        [("Id", ids[3]), ("OIdx", "3"), ("Val", "3e0")],
        [("Id", ids[4]), ("Legacy", "at _7am=ok"), ("OIdx", "4")],
        *([("Id", ids[i]), ("OIdx", str(i))] for i in range(5, 10)),
        [("Id", ids[10]), ("Na", " Explicit Rest "), ("OIdx", "10")],
    ]
    assert [[(attribute.name, attribute.value) for attribute in state.attributes]
            for state in value.states] == expected


def check_ambiguity(value, title="Monitor & 灯", selector="Monitor & 灯"):
    assert isinstance(value, lox.StatusMonitorError.Ambiguous)
    assert value.selector == selector
    assert [(item.title, item.block_type, item.uuid) for item in value.matches] == [
        (title, "StatusMonitor", "mon"), (title, "statusmonitor", "wrong")]


def check_invalid_value(value, raw):
    assert isinstance(value, lox.StatusMonitorError.InvalidValue) and value.raw == raw


def check_semantics(xml):
    for selector in ("uuid:mon", "gid:PRIMARY", "Monitor & 灯 [study]"):
        check_inventory(inspect(xml, selector))
    anonymous = xml.replace(' U="mon" Title="Monitor &amp; 灯"'.encode(), b"", 1)
    check_inventory(inspect(anonymous, "gid:primary"), "", "")
    malformed = b"<C A='1' A='2'/>"
    load = outcome(malformed, "missing")
    assert isinstance(load, lox.StatusMonitorError.Load) and load.source_message
    missing = outcome(xml, "missing")
    assert isinstance(missing, lox.StatusMonitorError.NotFound)
    assert (missing.selector, missing.titles) == ("missing", ["Monitor & 灯", "Study"])
    check_ambiguity(outcome(xml, "Monitor & 灯"))
    inherited = xml.replace(b"<Root>", b'<Root><Other iObjS=""/>', 1)
    wrong = outcome(inherited, "uuid:wrong")
    assert isinstance(wrong, lox.StatusMonitorError.NotStatusMonitor) and wrong.selector == "uuid:wrong"
    assert isinstance(outcome(inherited, "uuid:mon"), lox.StatusMonitorError.InheritedConfiguration)
    count = outcome(b'<C Type="StatusMonitor" U="short"><Other><State/></Other></C>', "uuid:short")
    assert isinstance(count, lox.StatusMonitorError.StateCount) and count.found == 0
    duplicate_id = "a0000000-0000-0000-0000000000000001"
    duplicate = outcome(xml.replace(b"a0000000-0000-0000-0000000000000002",
                                    duplicate_id.encode(), 1), "uuid:mon")
    assert isinstance(duplicate, lox.StatusMonitorError.DuplicateStateId) and duplicate.id == duplicate_id
    output = outcome(xml.replace(b'OIdx="0"', b'OIdx="002"', 1), "uuid:mon")
    assert isinstance(output, lox.StatusMonitorError.DuplicateOutputIndex) and output.output_index == 2
    value = outcome(xml.replace(b'-002.0', b'1.25e0', 1), "uuid:mon")
    assert isinstance(value, lox.StatusMonitorError.DuplicateValue) and value.value == 1.25


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


def lower(data, selector):
    return native._UniffiFfiConverterTypeStatusMonitorRequest.lower(
        lox.StatusMonitorRequest(bytes=data, selector=selector))


def calibrate(data, selector, error, marker):
    status = native._UniffiRustCallStatus.default()
    result = native._UniffiLib.uniffi_lox_archive_fn_func_inspect_status_monitor(
        lower(data, selector), ctypes.byref(status))
    minimum = 64000
    buffer = status.error_buf if status.code != 0 else result
    try:
        assert status.code == (1 if error else 0)
        length = buffer.len
        assert length >= minimum
        address = ctypes.cast(buffer.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, 128)
        assert marker.encode() in prefix, "Expected marked payload in the actual native output"
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        if status.code != 0:
            buffer.free()
        result.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0
    return prefix, minimum, length


def rejected_selector_releases_input(xml):
    data = b"<!--status-monitor-rejected-selector-input:" + b"q" * 65536 + b"-->\n" + xml
    check_inventory(inspect(data, "uuid:mon"))
    minimum = 4 + len(data)
    control = lower(data, "uuid:mon")
    try:
        assert control.len >= minimum
        address = ctypes.cast(control.data, ctypes.c_void_p).value
        prefix = ctypes.string_at(address, 128)
        assert prefix == len(data).to_bytes(4, "big") + data[:124]
        assert probe.lox_test_heap_live(address) and matching(prefix, minimum) == 1
    finally:
        control.free()
    assert not probe.lox_test_heap_live(address) and matching(prefix, minimum) == 0

    invalid = "\ud800"
    for _ in range(64):
        try:
            inspect(data, invalid)
        except UnicodeEncodeError as failure:
            assert failure.encoding == "utf-8" and failure.object == invalid
            assert (failure.start, failure.end) == (0, 1)
        else:
            raise AssertionError("Expected strict host selector-encoding refusal")
    assert matching(prefix, minimum) == 0, "Rejected selector retained a native input buffer"
    print("Python status-monitor invalid selector: input_live=1 input_free=0 "
          "rejections=64 input_native_retained=0 GC_requested=false")


def main():
    assert sys.version_info[:2] == (3, 13) and sys._is_gil_enabled()
    assert "RUST_MIN_STACK" not in os.environ
    gc.disable()
    xml = (Path(__file__).resolve().parents[2] / "crates" / "lox-bindings"
           / "tests" / "fixtures" / "status_monitor.xml").read_bytes()
    check_semantics(xml)
    title_marker = "status-monitor-owned-title-"
    value_marker = "status-monitor-invalid-value-"
    title = title_marker + "x" * 65536
    raw = value_marker + "z" * 65536
    ambiguous_selector = "owned-ambiguous"
    ambiguous_marker = f"status-monitor-{ambiguous_selector}-"
    ambiguous_title = ambiguous_marker + "a" * 65536
    for branch, data, selector, error, marker, check, expected in (
        ("inventory", xml.replace("Monitor &amp; 灯".encode(), title.encode(), 1),
         "uuid:mon", False, title_marker, check_inventory, (title,)),
        ("invalid-value", xml.replace(b"+01.2500", raw.encode(), 1),
         "uuid:mon", True, value_marker, check_invalid_value, (raw,)),
        ("ambiguous", xml.replace("Monitor &amp; 灯".encode(), ambiguous_title.encode()),
         ambiguous_selector, True, ambiguous_marker, check_ambiguity, (ambiguous_title, ambiguous_selector)),
    ):
        prefix, minimum, length = calibrate(data, selector, error, marker)
        retained = [outcome(data, selector) for _ in range(64)]
        assert len({id(value) for value in retained}) == 64
        assert matching(prefix, minimum) == 0, f"{branch} retained a native result buffer"
        for value in retained:
            check(value, *expected)
        print(f"Python status-monitor branch={branch} actual_bytes={length} live=1 free=0 "
              "calls=64 host_retained=64 native_retained=0 GC_requested=false")
    rejected_selector_releases_input(xml)
    print("Python status-monitor stored order/text/defaults/admission/typed-errors PASS")


if __name__ == "__main__":
    main()
