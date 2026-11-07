import copy
from pathlib import Path
import struct
import tempfile
import unittest

from qt_metadata import PeImage, recover_qt_metadata


BASE = 0x140000000


def fixture():
    data = bytearray(0xC00)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 60, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<HH", data, 0x84, 0x8664, 2)
    struct.pack_into("<H", data, 0x94, 0xF0)
    struct.pack_into("<H", data, 0x98, 0x20B)
    struct.pack_into("<Q", data, 0xB0, BASE)
    for offset, name, rva, file_offset, size, flags in [
        (0x188, b".text", 0x1000, 0x200, 0x200, 0x60000020),
        (0x1B0, b".rdata", 0x2000, 0x400, 0x800, 0x40000040),
    ]:
        data[offset:offset + len(name)] = name
        struct.pack_into("<IIII", data, offset + 8, size, rva, size, file_offset)
        struct.pack_into("<I", data, offset + 36, flags)
    struct.pack_into("<7Q", data, 0x400, 0, BASE + 0x2100, BASE + 0x2300,
                     BASE + 0x1000, 0, 0, 0)
    strings = ["DemoDialog", "changed", "apply", "state", "value", ""]
    position = len(strings) * 8
    for i, value in enumerate(strings):
        raw = value.encode()
        struct.pack_into("<II", data, 0x500 + i * 8, position, len(raw))
        data[0x500 + position:0x500 + position + len(raw) + 1] = raw + b"\0"
        position += len(raw) + 1
    words = [
        13, 0, 0, 0, 2, 14, 1, 30, 0, 0, 0, 0, 0, 1,
        1, 0, 29, 5, 6, 0,
        2, 1, 26, 5, 10, 1,
        43, 1, 4,
        43,
        3, 1, 3, 0, 0,
    ]
    struct.pack_into(f"<{len(words)}I", data, 0x700, *words)
    return data


class QtMetadataTests(unittest.TestCase):
    def recover(self, data):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "fixture.exe"
            path.write_bytes(data)
            return recover_qt_metadata(path)

    def test_recovers_declared_names_and_dispatch_without_executing_image(self):
        data = fixture()
        original = copy.copy(data)
        result = self.recover(data)
        self.assertEqual(data, original)
        self.assertEqual(result["coverage"]["decoded_classes"], 1)
        cls = result["classes"][0]
        self.assertEqual(cls["class_name"], "DemoDialog")
        self.assertEqual(cls["metaobject_address"], hex(BASE + 0x2000))
        self.assertEqual(cls["dispatcher_address"], hex(BASE + 0x1000))
        self.assertEqual(
            [(m["name"], m["kind"]) for m in cls["methods"]],
            [("changed", "signal"), ("apply", "slot")],
        )
        self.assertEqual(cls["methods"][1]["arguments"], [
            {"name": "value", "type": {"encoded": 1, "qt_metatype_id": 1}},
        ])
        self.assertEqual(cls["properties"][0]["name"], "state")
        self.assertTrue(cls["properties"][0]["writable"])
        self.assertEqual(cls["properties"][0]["notify_index"], 0)

    def test_out_of_bounds_member_table_is_reported_not_partially_recovered(self):
        data = fixture()
        struct.pack_into("<I", data, 0x700 + 5 * 4, 0x100000)
        result = self.recover(data)
        self.assertEqual(result["classes"], [])
        self.assertEqual(len(result["issues"]), 1)
        self.assertIn("mapping", result["issues"][0]["reason"])

    def test_other_revisions_remain_explicitly_unsupported(self):
        for revision in (12, 14):
            with self.subTest(revision=revision):
                data = fixture()
                struct.pack_into("<I", data, 0x700, revision)
                result = self.recover(data)
                self.assertEqual(result["classes"], [])
                self.assertEqual(result["unsupported_objects"][0]["revision"], revision)

    def test_parameter_type_names_use_their_encoded_string_identity(self):
        data = fixture()
        struct.pack_into("<I", data, 0x700 + 27 * 4, 0x80000003)
        cls = self.recover(data)["classes"][0]
        self.assertEqual(cls["methods"][1]["arguments"][0]["type"], {
            "encoded": 0x80000003, "name": "state",
        })

    def test_malformed_pe_and_ambiguous_mappings_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "DOS/PE"):
            PeImage(b"MZ")
        data = fixture()
        struct.pack_into("<I", data, 0x1B0 + 12, 0x1000)
        image = PeImage(data)
        with self.assertRaisesRegex(ValueError, "unique"):
            image.mapping(BASE + 0x1000)


if __name__ == "__main__":
    unittest.main()
