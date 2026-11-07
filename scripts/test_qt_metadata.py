import copy
import hashlib
from pathlib import Path
import struct
import tempfile
import unittest

from qt_metadata import PeImage, recover_qt_metadata


BASE = 0x140000000
DEFAULT_PROFILE = object()


def fixture(header_words=1):
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
    header = [0] if header_words == 1 else [0, BASE + 0x1080]
    struct.pack_into(f"<{header_words + 6}Q", data, 0x400, *header,
                     BASE + 0x2100, BASE + 0x2300, BASE + 0x1000, 0, 0, 0)
    write_return(data, 0x1040, 0x2000)
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


def write_return(data, return_rva, target_rva):
    offset = 0x200 + return_rva - 0x1000
    data[offset:offset + 8] = (
        b"\x48\x8d\x05" + struct.pack("<i", target_rva - return_rva - 7) + b"\xc3"
    )


def fixture_profiles(data, header_words=1):
    return {
        "source_sha256": hashlib.sha256(data).hexdigest(),
        "format": "PE32+/AMD64", "metadata_revision": 13,
        "objects": [{
            "profile": f"qt6-superdata-{'one' if header_words == 1 else 'two'}-word",
            "pointer_triple_address": hex(BASE + 0x2000 + header_words * 8),
            "base_return_address": hex(BASE + 0x1040),
            "base_return_sha256": hashlib.sha256(data[0x240:0x248]).hexdigest(),
        }],
    }


class QtMetadataTests(unittest.TestCase):
    def recover(self, data, layout_profiles=DEFAULT_PROFILE):
        private = Path(__file__).resolve().parents[1] / ".reverse-engineering"
        private.mkdir(exist_ok=True)
        if layout_profiles is DEFAULT_PROFILE:
            layout_profiles = fixture_profiles(data)
        with tempfile.TemporaryDirectory(dir=private) as temporary:
            path = Path(temporary) / "fixture.exe"
            path.write_bytes(data)
            return recover_qt_metadata(path, layout_profiles=layout_profiles)

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
        self.assertEqual(cls["header_layout"]["profile"], "qt6-superdata-one-word")
        self.assertEqual(cls["header_layout"]["callback_offset"], 0x18)
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

    def test_two_word_profile_preserves_absolute_callback_and_corrects_base(self):
        data = fixture(header_words=2)
        original = copy.copy(data)
        cls = self.recover(data, fixture_profiles(data, 2))["classes"][0]
        self.assertEqual(data, original)
        self.assertEqual(cls["metaobject_address"], hex(BASE + 0x2000))
        self.assertEqual(cls["strings_address"], hex(BASE + 0x2100))
        self.assertEqual(cls["metadata_address"], hex(BASE + 0x2300))
        self.assertEqual(cls["dispatcher_address"], hex(BASE + 0x1000))
        self.assertEqual(cls["header_layout"]["callback_offset"], 0x20)
        self.assertEqual(cls["header_layout"]["callback_pointer_address"], hex(BASE + 0x2020))
        self.assertEqual(cls["header_layout"]["profile"], "qt6-superdata-two-word")

    def test_missing_profile_does_not_publish_a_guessed_base(self):
        result = self.recover(fixture(), None)
        self.assertEqual(result["classes"], [])
        self.assertEqual(len(result["issues"]), 1)
        self.assertIn("Unvalidated Qt header layout", result["issues"][0]["reason"])
        self.assertNotIn("metaobject_address", result["issues"][0])

    def test_profiles_are_bound_to_source_format_and_revision(self):
        data = fixture()
        for field, invalid in [("source_sha256", "0" * 64), ("format", "ELF"),
                               ("metadata_revision", 12)]:
            with self.subTest(field=field):
                profiles = fixture_profiles(data)
                profiles[field] = invalid
                with self.assertRaisesRegex(ValueError, "must match"):
                    self.recover(data, profiles)
        profiles = fixture_profiles(data)
        profiles["objects"][0]["profile"] = "guess-from-revision"
        with self.assertRaisesRegex(ValueError, "Unknown Qt header-layout profile"):
            self.recover(data, profiles)
        profiles["objects"] = None
        with self.assertRaisesRegex(ValueError, "objects list"):
            self.recover(data, profiles)

    def test_wrong_layout_or_return_witness_is_explicitly_unvalidated(self):
        data = fixture(header_words=2)
        for fault in ("layout", "hash", "return", "base"):
            with self.subTest(fault=fault):
                candidate = copy.copy(data)
                if fault == "base":
                    write_return(candidate, 0x1040, 0x2008)
                profiles = fixture_profiles(candidate, 2)
                row = profiles["objects"][0]
                if fault == "layout":
                    row["profile"] = "qt6-superdata-one-word"
                elif fault == "hash":
                    row["base_return_sha256"] = "0" * 64
                elif fault == "return":
                    row["base_return_address"] = hex(BASE + 0x2100)
                result = self.recover(candidate, profiles)
                self.assertEqual(result["classes"], [])
                self.assertIn("Unvalidated Qt header layout", result["issues"][0]["reason"])

    def test_conflicting_valid_return_witnesses_are_ambiguous(self):
        data = fixture(header_words=2)
        struct.pack_into("<Q", data, 0x408, 0)
        write_return(data, 0x1050, 0x2008)
        profiles = fixture_profiles(data, 2)
        alternative = copy.copy(profiles["objects"][0])
        alternative.update({
            "profile": "qt6-superdata-one-word",
            "base_return_address": hex(BASE + 0x1050),
            "base_return_sha256": hashlib.sha256(data[0x250:0x258]).hexdigest(),
        })
        profiles["objects"].append(alternative)
        result = self.recover(data, profiles)
        self.assertEqual(result["classes"], [])
        self.assertIn("Ambiguous Qt header layout", result["issues"][0]["reason"])

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
