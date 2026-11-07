import hashlib
from pathlib import Path
import struct
import tempfile
import unittest

from pe_dialogs import decode_dialog, decode_string_table, recover_dialog_resources


def text(value):
    return value.encode("utf-16-le") + b"\0\0"


def template(extended=False):
    if extended:
        data = bytearray(struct.pack("<HHIIIHhhhh", 1, 0xFFFF, 9, 0, 0x40, 1, 0, 0, 100, 50))
    else:
        data = bytearray(struct.pack("<IIHhhhh", 0x40, 0, 1, 0, 0, 100, 50))
    data += b"\0\0\0\0" + text("Synthetic")
    data += struct.pack("<H", 9)
    if extended:
        data += struct.pack("<HBB", 400, 0, 1)
    data += text("Test Font")
    data += b"\0" * ((-len(data)) % 4)
    if extended:
        data += struct.pack("<IIIhhhhI", 0, 0, 0, 2, 3, 10, 5, 0x10001)
    else:
        data += struct.pack("<IIhhhhH", 0, 0, 2, 3, 10, 5, 101)
    data += struct.pack("<HH", 0xFFFF, 0x81) + text("Example") + b"\0\0"
    return data


class DialogTests(unittest.TestCase):
    def test_decodes_both_dialog_formats_and_preserves_ordinals(self):
        for extended in (False, True):
            with self.subTest(extended=extended):
                result = decode_dialog(template(extended))
                self.assertEqual(result["caption"]["value"], "Synthetic")
                self.assertEqual(result["font"]["typeface"], "Test Font")
                self.assertEqual(result["controls"][0]["id"], 0x10001 if extended else 101)
                self.assertEqual(result["controls"][0]["window_class"], {
                    "kind": "ordinal", "id": 0x81,
                })
                self.assertEqual(result["controls"][0]["bounds_dialog_units"], [2, 3, 10, 5])
                self.assertEqual(result["trailing_bytes"], 0)

    def test_truncation_and_unsupported_creation_data_are_explicit(self):
        with self.assertRaisesRegex(ValueError, "exceeds"):
            decode_dialog(template()[:-1])
        data = template()
        data[-2:] = b"\x02\0"
        with self.assertRaisesRegex(ValueError, "creation data"):
            decode_dialog(data)
        data = template(extended=True)
        struct.pack_into("<H", data, 0, 2)
        with self.assertRaisesRegex(ValueError, "dialog version"):
            decode_dialog(data)

    def test_payload_and_artifact_are_bound_to_the_rea_evidence(self):
        data = template()
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "fixture.dll"
            path.write_bytes(data)
            sha = hashlib.sha256(data).hexdigest()
            evidence = {
                "evidence_id": "synthetic", "operation": "inspect_pe_resources",
                "normalized_result": {
                    "artifact": {"sha256": sha},
                    "resources": [{
                        "index": 0, "type": {"kind": "id", "id": 5},
                        "name": {"kind": "id", "id": 10},
                        "language": {"kind": "id", "id": 1033},
                        "payload": {"location": {"offset": 0, "bytes": len(data)}, "sha256": sha},
                    }],
                },
            }
            result = recover_dialog_resources(path, evidence)
            self.assertEqual(result["coverage"]["decoded"], 1)
            self.assertEqual(result["resource_evidence_id"], "synthetic")
            evidence["normalized_result"]["resources"][0]["payload"]["sha256"] = "0" * 64
            with self.assertRaisesRegex(ValueError, "payload digest"):
                recover_dialog_resources(path, evidence)

    def test_string_ids_and_empty_slots_are_preserved(self):
        values = ["Synthetic label", "Synthetic error", "\U0001f642"] + [""] * 13
        data = b"".join(struct.pack("<H", len(value.encode("utf-16-le")) // 2) + value.encode("utf-16-le")
                        for value in values)
        result = decode_string_table(data, 2)
        self.assertEqual(result["entries"][0], {"id": 16, "text": "Synthetic label"})
        self.assertEqual(result["entries"][2]["text"], "\U0001f642")
        self.assertEqual(result["entries"][15], {"id": 31, "text": ""})
        self.assertEqual(result["trailing_bytes"], 0)
        with self.assertRaisesRegex(ValueError, "exceeds"):
            decode_string_table(data[:-1], 2)
        with self.assertRaisesRegex(ValueError, "numeric resource ID"):
            decode_string_table(data, None)


if __name__ == "__main__":
    unittest.main()
