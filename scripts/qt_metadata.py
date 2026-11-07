"""Read Qt 6.9/6.10 revision-13 meta-objects from an x64 PE without execution."""

import hashlib
import re
import struct
from pathlib import Path


QT_SOURCE = "https://github.com/qt/qtbase/tree/v6.10.2/src/corelib/kernel"
CLASS_NAME = re.compile(r"[A-Za-z_][A-Za-z_0-9:<> ,]*")


class PeImage:
    def __init__(self, data):
        self.data = data
        if data[:2] != b"MZ" or len(data) < 64:
            raise ValueError("Expected a complete DOS/PE header")
        pe = self.unpack("<I", 60)[0]
        if self.read(pe, 4) != b"PE\0\0":
            raise ValueError("Invalid PE signature")
        machine, count = self.unpack("<HH", pe + 4)
        optional_size = self.unpack("<H", pe + 20)[0]
        optional = pe + 24
        if machine != 0x8664 or optional_size < 112:
            raise ValueError("Only native AMD64 PE32+ images are supported")
        if self.unpack("<H", optional)[0] != 0x20B:
            raise ValueError("Expected a PE32+ optional header")
        self.base = self.unpack("<Q", optional + 24)[0]
        self.sections = []
        table = optional + optional_size
        for i in range(count):
            position = table + 40 * i
            header = self.read(position, 40)
            virtual_size, rva, size, offset = struct.unpack_from("<IIII", header, 8)
            flags = struct.unpack_from("<I", header, 36)[0]
            self.read(offset, size)
            self.sections.append({
                "name": header[:8].rstrip(b"\0").decode("ascii", errors="replace"),
                "rva": rva, "offset": offset, "size": size,
                "virtual_size": virtual_size, "flags": flags,
            })

    def read(self, offset, size):
        if offset < 0 or size < 0 or offset + size > len(self.data):
            raise ValueError("Read exceeds original file bytes")
        return self.data[offset:offset + size]

    def unpack(self, layout, offset):
        return struct.unpack(layout, self.read(offset, struct.calcsize(layout)))

    def mapping(self, address, size=1):
        rva = address - self.base
        matches = [
            section for section in self.sections
            if section["rva"] <= rva
            and rva + size <= section["rva"] + section["size"]
        ]
        if len(matches) != 1:
            raise ValueError("Address has no unique original-file mapping")
        section = matches[0]
        return section, section["offset"] + rva - section["rva"]

    def words(self, address, count):
        _, offset = self.mapping(address, count * 4)
        return self.unpack(f"<{count}I", offset)

    def pointer_offset(self, address, size=1):
        try:
            return self.mapping(address, size)[1]
        except ValueError:
            return None


class QtMetaObject:
    def __init__(self, image, address, strings, metadata, callback):
        self.image = image
        self.address = address
        self.strings = strings
        self.metadata = metadata
        self.callback = callback
        self.header = image.words(metadata, 14)

    def string(self, index):
        relative, length = self.image.words(self.strings + index * 8, 2)
        if relative < (index + 1) * 8:
            raise ValueError("Qt string overlaps its offsets-and-sizes table")
        _, offset = self.image.mapping(self.strings + relative, length + 1)
        raw = self.image.read(offset, length + 1)
        if raw[-1] != 0 or b"\0" in raw[:-1]:
            raise ValueError("Qt string is not a single terminated string")
        return raw[:-1].decode("utf-8")

    def type_reference(self, encoded):
        if encoded & 0x80000000:
            return {"encoded": encoded, "name": self.string(encoded & 0x7FFFFFFF)}
        return {"encoded": encoded, "qt_metatype_id": encoded}

    def method(self, index, table):
        row = self.image.words(self.metadata + (table + 6 * index) * 4, 6)
        name, argc, parameters, tag, flags, metatype_offset = row
        values = self.image.words(self.metadata + parameters * 4, 1 + 2 * argc)
        return {
            "index": index,
            "name": self.string(name),
            "kind": ("method", "signal", "slot", "constructor")[(flags & 0xC) >> 2],
            "access": {0: "private", 1: "protected", 2: "public"}.get(flags & 3),
            "flags": flags,
            "tag": self.string(tag),
            "return_type": self.type_reference(values[0]),
            "arguments": [
                {
                    "name": self.string(values[1 + argc + i]),
                    "type": self.type_reference(values[1 + i]),
                }
                for i in range(argc)
            ],
            "metatype_offset": metatype_offset,
            "dispatcher_address": hex(self.callback),
        }

    def decode(self):
        h = self.header
        if h[0] != 13:
            raise ValueError(f"Unsupported Qt metadata revision {h[0]}")
        if h[13] > h[4]:
            raise ValueError("Signal count exceeds method count")
        for count, table, width in ((h[2], h[3], 2), (h[4], h[5], 6),
                                    (h[6], h[7], 5), (h[8], h[9], 5)):
            if count:
                self.image.mapping(self.metadata + table * 4, count * width * 4)
        name = self.string(h[1])
        if not CLASS_NAME.fullmatch(name):
            raise ValueError("Not a C++ class name")
        methods = [self.method(i, h[5]) for i in range(h[4])]
        if sum(method["kind"] == "signal" for method in methods) != h[13]:
            raise ValueError("Method flags disagree with the declared signal count")
        properties = []
        for i in range(h[6]):
            row = self.image.words(self.metadata + (h[7] + 5 * i) * 4, 5)
            prop_name, prop_type, flags, notify, revision = row
            properties.append({
                "index": i, "name": self.string(prop_name),
                "type": self.type_reference(prop_type), "flags": flags,
                "readable": bool(flags & 1), "writable": bool(flags & 2),
                "notify_index": None if notify == 0xFFFFFFFF else notify,
                "revision": revision,
            })
        class_info = []
        for i in range(h[2]):
            key, value = self.image.words(self.metadata + (h[3] + 2 * i) * 4, 2)
            class_info.append({"name": self.string(key), "value": self.string(value)})
        enums = []
        for i in range(h[8]):
            enum_name, alias, flags, count, table = self.image.words(
                self.metadata + (h[9] + 5 * i) * 4, 5
            )
            enums.append({
                "name": self.string(enum_name), "alias": self.string(alias),
                "flags": flags, "key_count": count, "data_word_offset": table,
                "values_status": "not_decoded",
            })
        return {
            "class_name": name, "metaobject_address": hex(self.address),
            "metadata_address": hex(self.metadata),
            "strings_address": hex(self.strings), "revision": h[0],
            "dispatcher_address": hex(self.callback),
            "methods": methods, "properties": properties, "class_info": class_info,
            "enums": enums, "constructor_count": h[10], "flags": h[12],
        }


def recover_qt_metadata(path):
    path = Path(path)
    data = path.read_bytes()
    image = PeImage(data)
    classes = []
    issues = []
    unsupported = []
    examined = 0
    for section in image.sections:
        if section["flags"] & 0x20000000:
            continue
        start = section["offset"]
        for position in range(start, start + section["size"] - 23, 8):
            strings, metadata, callback = image.unpack("<QQQ", position)
            if image.pointer_offset(strings, 8) is None:
                continue
            if image.pointer_offset(metadata, 56) is None:
                continue
            try:
                callback_section, _ = image.mapping(callback)
            except ValueError:
                continue
            if not callback_section["flags"] & 0x20000000:
                continue
            header = image.words(metadata, 14)
            # Qt 6 uses a different string carrier from Qt 5. Recognize only Qt 6.
            if header[0] < 9 or header[1] != 0:
                continue
            address = image.base + section["rva"] + position - start - 8
            if image.pointer_offset(address, 56) is None:
                continue
            obj = QtMetaObject(image, address, strings, metadata, callback)
            try:
                name = obj.string(header[1])
                if not CLASS_NAME.fullmatch(name):
                    continue
            except (ValueError, UnicodeDecodeError):
                continue
            examined += 1
            if header[0] != 13:
                unsupported.append({
                    "class_name": name, "metaobject_address": hex(address),
                    "revision": header[0],
                })
                continue
            try:
                classes.append(obj.decode())
            except (ValueError, UnicodeDecodeError) as error:
                issues.append({
                    "class_name": name, "metaobject_address": hex(address),
                    "reason": str(error),
                })
    return {
        "schema_version": 1,
        "source": {"path": str(path.resolve()), "sha256": hashlib.sha256(data).hexdigest()},
        "format": "PE32+/AMD64",
        "layout_reference": QT_SOURCE,
        "supported_revisions": [13],
        "classes": classes,
        "unsupported_objects": unsupported,
        "issues": issues,
        "coverage": {
            "status": "recognized_metadata_only", "candidate_objects": examined,
            "decoded_classes": len(classes),
            "methods": sum(len(item["methods"]) for item in classes),
            "properties": sum(len(item["properties"]) for item in classes),
        },
        "limitations": [
            "Static metadata identifies declared names and a shared dispatcher, not each method's implementation address.",
            "Only revision-13 Qt 6.9/6.10 metadata in native AMD64 PE images is decoded.",
            "Dynamically built or unidentified meta-objects, inherited members, ordinary non-moc C++ methods and MFC dialogs are outside this inventory.",
            "Enum values and constructor signatures are not decoded; original source, validation semantics and runtime signal connections are not recovered.",
        ],
    }
