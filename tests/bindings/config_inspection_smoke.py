"""Installed native wheel; no XML parser or CLI wrapper in this caller."""

import os
from pathlib import Path

import lox_archive as lox


assert "RUST_MIN_STACK" not in os.environ
xml = (
    Path(__file__).resolve().parents[2]
    / "crates" / "lox-bindings" / "tests" / "fixtures" / "config_inspection.xml"
).read_bytes()
before = bytes(xml)
rooms = lox.parse_rooms(xml)
summary = lox.parse_config_summary(xml)
assert xml == before
assert all(isinstance(room, lox.RoomInfo) for room in rooms)
assert [(r.name, r.item_count) for r in rooms] == [
    ("", 0), ("A&amp; Küche &#x41;", 4), ("Ignored", 0),
    ("a", 2), ("same", 0), ("same", 0),
]
assert [r.uuid for r in rooms[:4]] == ["", "r", "invalid-attr", "z"]
assert {r.uuid for r in rooms[4:]} == {"tie1", "tie2"}
assert all(type(r.item_count) is int for r in rooms)
assert isinstance(summary, lox.ConfigSummary)
assert (summary.version, summary.date) == ("16&amp;raw", "2026&amp;raw")
assert summary.rooms == {
    "": "", "r": "A&amp; Küche &#x41;", "z": "a",
    "tie1": "same", "tie2": "same", "invalid-attr": "Ignored",
}
assert summary.categories == {"cat": "Raw &amp; &#65;", "": ""}
assert summary.users == ["Zoë", "", "Zoë"]
assert all(isinstance(c, lox.ConfigControlEntry) for c in summary.controls.values())
assert {key: (c.name, c.control_type, c.room_uuid)
        for key, c in summary.controls.items()} == {
    "case": ("Case matters", "place", ""),
    "dup": ("Last &amp; raw", "FutureBlock", "r"),
    "nested": ("Nested", "And", "r"),
    "": ("Empty type and ID", "", ""),
    "orphan": ("No direct metadata", "And", ""),
}
# Host-owned records/maps survive further native calls and input replacement.
xml = b"replaced"
for empty in (b"", b"plain text"):
    assert lox.parse_rooms(empty) == []
    value = lox.parse_config_summary(empty)
    assert (value.version, value.date, value.controls, value.rooms,
            value.categories, value.users) == ("", "", {}, {}, {}, [])
assert summary.controls["dup"].name == "Last &amp; raw"
assert rooms[1].item_count == 4
assert lox.parse_rooms(b'<C Type="Place" U="r" Title="Open"><IoData Pr="r">')[0].item_count == 1
assert lox.parse_config_summary(
    b'<C Type="And" U="c" Title="Open"><IoData Pr="r">'
).controls["c"].room_uuid == "r"
reset = lox.parse_config_summary(
    b'<ControlList Version="old"/><ControlList/><C Type="Document" Date="old"/><C Type="Document"/>'
)
assert (reset.version, reset.date) == ("", "")
invalid = b'<C Type="Place" U="r" Title="\xff"/>'
assert lox.parse_rooms(invalid) == []
assert lox.parse_config_summary(invalid).rooms == {}
for parse in (lox.parse_rooms, lox.parse_config_summary):
    for data in (b"<root><secret>not-an-error-payload</root>", b"</unexpected>"):
        try:
            parse(data)
        except lox.ConfigurationError.Xml as error:
            assert error.source_message and "not-an-error-payload" not in error.source_message
        else:
            raise AssertionError("Expected typed XML reader failure")
    for _ in range(64):
        try:
            parse(("<" + "configuration-error-" + "x" * 65536 + "></other>").encode())
        except lox.ConfigurationError.Xml as error:
            assert len(error.source_message) > 65536
        else:
            raise AssertionError("Expected large typed error")
print("Python configuration: typed full records/maps/raw fields/order/distinct contracts/permissive/errors PASS")
