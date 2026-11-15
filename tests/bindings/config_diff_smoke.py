"""Typed snapshot comparison and actual exported result-buffer release."""

import ctypes
import os
import sys

import lox_archive as lox

try:
    from lox_archive import lox_archive as native
except ImportError:
    native = lox


def snapshots():
    old = lox.ConfigSummary(
        version=" raw Ω ", date="", controls={
            " Same ": lox.ConfigControlEntry(name="Old", control_type="Switch", room_uuid="r"),
            "A": lox.ConfigControlEntry(name="Twin", control_type="Switch", room_uuid=""),
            "stable": lox.ConfigControlEntry(name="Stable", control_type="Switch", room_uuid="r"),
        }, rooms={"r": "", "gone": "Twin"}, categories={"c": "Old", "gone": "Twin"},
        users=["", "Ω", "Ω"])
    new = lox.ConfigSummary(
        version="new", date="today", controls={
            " Same ": lox.ConfigControlEntry(name="Neu Ω", control_type="Dimmer", room_uuid="missing"),
            "a": lox.ConfigControlEntry(name="Twin", control_type="Switch", room_uuid=""),
            "stable": lox.ConfigControlEntry(name="Stable", control_type="Switch", room_uuid="r"),
        }, rooms={"r": "renamed", "added": "Twin"}, categories={"c": "New", "added": "Twin"},
        users=["Ω", "Added", "Added"])
    return old, new


old, new = snapshots()
result = lox.diff_configs(old, new)
assert result.has_changes
assert (result.version_old, result.version_new, result.date_old, result.date_new) == (
    " raw Ω ", "new", "", "today")
assert result.controls_added == [lox.ConfigControlRecord(
    uuid="a", entry=lox.ConfigControlEntry(name="Twin", control_type="Switch", room_uuid=""))]
assert result.controls_removed == [lox.ConfigControlRecord(
    uuid="A", entry=lox.ConfigControlEntry(name="Twin", control_type="Switch", room_uuid=""))]
assert result.controls_changed == [
    lox.ConfigControlChange(uuid=" Same ", name="Neu Ω", field=lox.ConfigControlFieldChange.NAME("Old", "Neu Ω")),
    lox.ConfigControlChange(uuid=" Same ", name="Neu Ω", field=lox.ConfigControlFieldChange.TYPE("Switch", "Dimmer")),
    lox.ConfigControlChange(uuid=" Same ", name="Neu Ω", field=lox.ConfigControlFieldChange.ROOM(
        lox.ConfigRoomReference(uuid="r", name=""), lox.ConfigRoomReference(uuid="missing", name=None))),
]
for field in ("rooms", "categories"):
    assert getattr(result, field + "_added") == [lox.ConfigNamedEntry(uuid="added", name="Twin")]
    assert getattr(result, field + "_removed") == [lox.ConfigNamedEntry(uuid="gone", name="Twin")]
assert result.rooms_renamed == [lox.ConfigNameChange(uuid="r", old="", new="renamed")]
assert result.categories_renamed == [lox.ConfigNameChange(uuid="c", old="Old", new="New")]
assert result.users_added == ["Added"] and result.users_removed == [""]
reverse = lox.diff_configs(new, old)
room = reverse.controls_changed[2].field
assert room == lox.ConfigControlFieldChange.ROOM(
    lox.ConfigRoomReference(uuid="missing", name=None), lox.ConfigRoomReference(uuid="r", name=""))
old.controls.clear()
new.rooms.clear()
new.users.clear()
assert result.controls_changed[2].field.new.name is None
assert result.rooms_renamed[0].new == "renamed"
before, after = snapshots()
after.controls = dict(before.controls)
after.rooms = dict(before.rooms)
after.categories = dict(before.categories)
after.users = ["Ω", ""]
meta = lox.diff_configs(before, after)
assert not meta.has_changes and meta.version_new == "new" and meta.date_new == "today"
after.rooms["r"] = "rename only"
renamed = lox.diff_configs(before, after)
assert renamed.has_changes and not renamed.controls_changed and len(renamed.rooms_renamed) == 1

assert "RUST_MIN_STACK" not in os.environ
probe = ctypes.CDLL(sys.argv[1])
count = probe.lox_test_heap_count
count.argtypes = [ctypes.c_void_p, ctypes.c_uint64, ctypes.c_uint64,
                  ctypes.POINTER(ctypes.c_uint64)]
count.restype = ctypes.c_int


def matching(prefix):
    owned = ctypes.create_string_buffer(prefix)
    found = ctypes.c_uint64()
    assert count(owned, len(prefix), 64000, ctypes.byref(found)) == 0
    return found.value


marker = "config-diff-owned-python-" + "x" * 65536
old = lox.ConfigSummary(version=marker, date="", controls={}, rooms={}, categories={}, users=[])
new = lox.ConfigSummary(version="new", date="", controls={}, rooms={}, categories={}, users=[])
status = native._UniffiRustCallStatus()
buffer = native._UniffiLib.uniffi_lox_archive_fn_func_diff_configs(
    native._UniffiFfiConverterTypeConfigSummary.lower(old),
    native._UniffiFfiConverterTypeConfigSummary.lower(new), ctypes.byref(status))
assert status.code == 0 and buffer.len >= 64000
prefix = ctypes.string_at(buffer.data, 128)
assert matching(prefix) == 1
print(f"Python actual diff export len={buffer.len} live=1", flush=True)
buffer.free()
assert matching(prefix) == 0
for _ in range(64):
    actual = lox.diff_configs(old, new)
    assert actual.version_old == marker and not actual.has_changes
assert matching(prefix) == 0
print("Python diff calls=64 calibrated_live=1 calibrated_released=0 retained_buffers=0 GC_requested=false")
print("Python typed structural diff: all variants/metadata/sets/raw UUIDs/Unicode/ownership PASS")
