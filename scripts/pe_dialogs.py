"""Decode Windows dialogs and string tables using verified REA resource ranges."""

import hashlib
import struct
from pathlib import Path


class DialogReader:
    def __init__(self, data):
        self.data = data
        self.position = 0

    def take(self, count):
        if self.position + count > len(self.data):
            raise ValueError("Dialog field exceeds resource payload")
        result = self.data[self.position:self.position + count]
        self.position += count
        return result

    def unpack(self, layout):
        return struct.unpack(layout, self.take(struct.calcsize(layout)))

    def word(self):
        return self.unpack("<H")[0]

    def text(self, first=None):
        words = []
        value = self.word() if first is None else first
        while value:
            words.append(value)
            value = self.word()
        return struct.pack(f"<{len(words)}H", *words).decode("utf-16-le")

    def name(self):
        first = self.word()
        if first == 0:
            return {"kind": "empty"}
        if first == 0xFFFF:
            return {"kind": "ordinal", "id": self.word()}
        return {"kind": "string", "value": self.text(first)}

    def align(self):
        self.take((-self.position) % 4)


def decode_dialog(data):
    reader = DialogReader(data)
    if len(data) < 4:
        raise ValueError("Truncated dialog header")
    version, signature = struct.unpack_from("<HH", data)
    if signature == 0xFFFF and version != 1:
        raise ValueError(f"Unsupported extended dialog version {version}")
    extended = data[:4] == b"\x01\x00\xff\xff"
    if extended:
        _, _, help_id, ex_style, style, count, x, y, width, height = reader.unpack("<HHIIIHhhhh")
    else:
        style, ex_style, count, x, y, width, height = reader.unpack("<IIHhhhh")
        help_id = None
    result = {
        "format": "DLGTEMPLATEEX" if extended else "DLGTEMPLATE",
        "style": style, "extended_style": ex_style, "help_id": help_id,
        "bounds_dialog_units": [x, y, width, height],
        "menu": reader.name(), "window_class": reader.name(), "caption": reader.name(),
        "font": None, "controls": [],
    }
    if style & 0x40:
        point_size = reader.word()
        font = {"point_size": point_size}
        if extended:
            weight, italic, charset = reader.unpack("<HBB")
            font.update(weight=weight, italic=italic, charset=charset)
        font["typeface"] = reader.text()
        result["font"] = font
    for _ in range(count):
        reader.align()
        if extended:
            help_id, ex_style, style, x, y, width, height, identifier = reader.unpack("<IIIhhhhI")
        else:
            style, ex_style, x, y, width, height, identifier = reader.unpack("<IIhhhhH")
            help_id = None
        control = {
            "id": identifier, "help_id": help_id, "style": style,
            "extended_style": ex_style, "bounds_dialog_units": [x, y, width, height],
            "window_class": reader.name(), "title": reader.name(),
        }
        extra_count = reader.word()
        if extra_count:
            raise ValueError("Nonempty control creation data is not decoded")
        result["controls"].append(control)
    result["trailing_bytes"] = len(data) - reader.position
    return result


def decode_string_table(data, identifier):
    if not isinstance(identifier, int) or identifier < 1:
        raise ValueError("A Windows string table requires a positive numeric resource ID")
    reader = DialogReader(data)
    entries = []
    for slot in range(16):
        length = reader.word()
        value = reader.take(length * 2).decode("utf-16-le")
        entries.append({"id": (identifier - 1) * 16 + slot, "text": value})
    return {"entries": entries, "trailing_bytes": len(data) - reader.position}


def recover_dialog_resources(source, evidence):
    source = Path(source)
    data = source.read_bytes()
    if evidence.get("operation") != "inspect_pe_resources":
        raise ValueError("Expected REA inspect_pe_resources Evidence")
    inventory = evidence["normalized_result"]
    sha256 = hashlib.sha256(data).hexdigest()
    if inventory["artifact"]["sha256"] != sha256:
        raise ValueError("Resource Evidence does not match the selected file")
    dialogs = []
    string_tables = []
    issues = []
    for resource in inventory["resources"]:
        kind = resource["type"].get("id")
        if kind not in (5, 6):
            continue
        location = resource["payload"]["location"]
        start = location["offset"]
        size = location["bytes"]
        if start < 0 or size < 0 or start + size > len(data):
            raise ValueError("REA resource range exceeds the selected file")
        payload = data[start:start + size]
        if hashlib.sha256(payload).hexdigest() != resource["payload"]["sha256"]:
            raise ValueError("Dialog payload digest differs from REA Evidence")
        identity = {
            "resource_index": resource["index"], "resource_type": kind, "name": resource["name"],
            "language": resource["language"], "location": location,
            "payload_sha256": resource["payload"]["sha256"],
        }
        try:
            if kind == 5:
                dialogs.append({**identity, **decode_dialog(payload)})
            else:
                string_tables.append({
                    **identity, **decode_string_table(payload, resource["name"].get("id")),
                })
        except (ValueError, UnicodeDecodeError) as error:
            issues.append({**identity, "reason": str(error)})
    return {
        "schema_version": 1, "source": {"path": str(source), "sha256": sha256},
        "resource_evidence_id": evidence["evidence_id"],
        "dialogs": dialogs, "string_tables": string_tables, "issues": issues,
        "coverage": {
            "status": "partial" if issues else "all_declared_dialogs_and_string_tables_decoded",
            "declared_dialogs": sum(r["type"].get("id") == 5 for r in inventory["resources"]),
            "decoded": len(dialogs),
            "controls": sum(len(dialog["controls"]) for dialog in dialogs),
            "string_tables": len(string_tables),
            "nonempty_strings": sum(bool(entry["text"]) for table in string_tables
                                   for entry in table["entries"]),
        },
        "layout_reference": "https://learn.microsoft.com/windows/win32/dlgbox/about-dialog-boxes",
        "limitations": [
            "Template fields, styles and control IDs are static observations, not validation rules or runtime values.",
            "Controls created dynamically, Qt-created dialogs and programmatic validators are outside these templates.",
            "String IDs locate static labels, help and diagnostics; messages are not automatically classified or linked to validation paths.",
            "Nonempty control creation data is reported as an undecoded resource.",
        ],
    }
