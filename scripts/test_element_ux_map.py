import copy
from contextlib import redirect_stderr, redirect_stdout
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import struct
import tempfile
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "element_ux_map", Path(__file__).with_name("element-ux-map.py"))
UX = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(UX)


def version_pe():
    data = bytearray(0x600)
    data[:2] = b"MZ"
    struct.pack_into("<I", data, 60, 0x80)
    data[0x80:0x84] = b"PE\0\0"
    struct.pack_into("<HH", data, 0x84, 0x8664, 1)
    struct.pack_into("<H", data, 0x94, 0xF0)
    struct.pack_into("<H", data, 0x98, 0x20B)
    struct.pack_into("<II", data, 0x98 + 112 + 16, 0x1000, 0x400)
    section = 0x98 + 0xF0
    data[section:section + 8] = b".rsrc\0\0\0"
    struct.pack_into("<4I", data, section + 8, 0x400, 0x1000, 0x400, 0x200)
    for offset, identifier, child in [(0, 16, 0x20), (0x20, 1, 0x40)]:
        struct.pack_into("<HH", data, 0x200 + offset + 12, 0, 1)
        struct.pack_into("<II", data, 0x200 + offset + 16,
                         identifier, 0x80000000 | child)
    struct.pack_into("<HH", data, 0x240 + 12, 0, 1)
    struct.pack_into("<II", data, 0x240 + 16, 1033, 0x60)
    key = "VS_VERSION_INFO\0".encode("utf-16-le")
    value_offset = (6 + len(key) + 3) & ~3
    payload = bytearray(value_offset + 52)
    struct.pack_into("<HHH", payload, 0, len(payload), 52, 0)
    payload[6:6 + len(key)] = key
    struct.pack_into("<4I", payload, value_offset, 0xFEEF04BD, 0x10000,
                     (17 << 16) | 2, (10 << 16) | 21)
    struct.pack_into("<4I", data, 0x260, 0x1100, len(payload), 0, 0)
    data[0x300:0x300 + len(payload)] = payload
    return bytes(data)


class Fixture:
    def __init__(self, root):
        self.root = root
        self.executable = self.write("version\\source\\LoxoneConfig.exe", version_pe())
        self.target = UX.digest(self.executable.read_bytes())
        self.dll = self.write("version\\source\\resources.dll", b"synthetic-resource-dll")
        self.xml = self.write("catalog.xml", b"<SyntheticCatalog/>")
        self.catalog = self.save("catalog.json", {
            "source": {"xml": str(self.xml), "xml_sha256": UX.digest(self.xml.read_bytes())},
            "blocks": [
                {"attributes": {"ControlType": "10", "LxType": "ExampleA", "Name": "Example A"}},
                {"attributes": {"ControlType": "10", "LxType": "ExampleB", "Name": "Example B"}},
                {"attributes": {"ControlType": "1", "Name": "Project"}},
            ],
        })
        self.state = self.save("state.json", {
            "status": "complete", "version_directory": str(root / "version"),
            "binaries": {"LoxoneConfig.exe": {"export": {"target_sha256": self.target}}},
        })
        self.data = {
            "schema_version": 1,
            "source": {"executable_sha256": self.target, "state": str(self.state),
                       "catalog": str(self.catalog)},
            "blocks": [
                {"catalog_index": index, "control_type": "10", "xml_type": name,
                 "name": label, "families": ["synthetic"],
                 "parameters": [{"name_pattern": "Value", "min": "0", "max": "10",
                                 "known_xml_keys": ["V"], "mapping_status": "name_and_control_id_match"}],
                 "native_anchors": [{"address": address}],
                 "storage_observations": {"paths": ["Rows", "Rows/Row", "SET"]}}
                for index, name, label, address in [
                    (0, "ExampleA", "Example A", "0x1000"),
                    (1, "ExampleB", "Example B", "0x2000")]
            ] + [{"catalog_index": 2, "control_type": "1", "xml_type": None,
                  "name": "Project", "parameters": [], "families": []}],
            "families": {"synthetic": {"qt_name_leads": [{"class_name": "CExampleADlg"}]}},
            "qt_dialogs": [{"class_name": "CExampleADlg"}], "msvc_dialog_types": [],
        }
        self.inventory = self.save("block-ux-coverage.json", self.data)
        self.qt = self.save("qt-metadata.json", {
            "schema_version": 1, "source": {"path": str(self.executable), "sha256": self.target},
            "classes": [
                {"class_name": "CExampleADlg", "metaobject_address": "0x9000",
                 "dispatcher_address": "0x5000", "methods": [
                     {"index": 0, "name": "OnAdd"}, {"index": 1, "name": "CheckCode"}],
                 "properties": []},
                {"class_name": "CSettingEditorNumeric", "metaobject_address": "0x9010",
                 "dispatcher_address": "0x5010",
                 "methods": [{"index": 0, "name": "OnAdd"}], "properties": []},
                {"class_name": "CSettingEditorNumericFancy", "metaobject_address": "0x9020",
                 "dispatcher_address": "0x5020", "methods": [], "properties": []},
                {"class_name": "CDurationValidator", "metaobject_address": "0x9030",
                 "dispatcher_address": "0x5030", "methods": [], "properties": []},
            ],
        })
        self.dialogs = self.save("native-dialogs.json", {
            "schema_version": 1,
            "source": {"path": str(self.dll), "sha256": UX.digest(self.dll.read_bytes())},
            "dialogs": [{"resource_index": 3, "caption": {"value": "Example A"}, "controls": [
                {"window_class": {"kind": "ordinal", "id": 129}}]}],
            "string_tables": [{"entries": [
                {"id": 12, "text": "Value must be valid"}]}],
        })
        self.write("advanced-blocks.tsv", b"TYPE\tCONTROL_ID\nExampleA\t10\nExampleB\t10\n")

    def write(self, name, data):
        path = self.root.joinpath(*name.split("\\"))
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
        return path

    def save(self, name, value):
        return self.write(name, (json.dumps(value) + "\n").encode())

    def extract(self):
        return UX.extract(self.inventory, root=self.root, skip_native=True)

    def corpus(self):
        fixture = self
        texts = {
            "0x1000": '// header FUN_7000()\nvoid FUN_1000() {\n'
                      'log("An escaped \\"caption\\" first");\n'
                      'FUN_3000("ExampleA"); FUN_4000();\n'
                      '// FUN_7000()\nconst char *s = "FUN_7000()"; }\n',
            "0x2000": 'void FUN_2000() { FUN_3000("ExampleB"); FUN_4000(); }\n',
            "0x3000": 'void FUN_3000() { *ptr = CPropertyDescriptor::vftable; }\n',
            "0x4000": 'void FUN_4000() { log("Read XmlItem missing"); }\n',
        }

        class Corpus:
            def __init__(self):
                self.functions = {}
                for address, text in texts.items():
                    path = fixture.write("bodies\\" + address + ".c", text.encode())
                    self.functions[address] = {
                        "status": "written", "body": {"available": True},
                        "relative_path": "bodies\\" + address + ".c",
                        "sha256": UX.digest(path.read_bytes()), "path": path,
                    }

            def body(self, address):
                path = self.functions[address]["path"]
                return path, path.read_text()

        return Corpus()


class SharedUxTests(unittest.TestCase):
    def setUp(self):
        # Scratch stays under the project and is always removed, never system temp.
        self.scratch = tempfile.TemporaryDirectory(prefix=".element-ux-test-", dir=Path.cwd())
        self.addCleanup(self.scratch.cleanup)
        self.fixture = Fixture(Path(self.scratch.name))

    def test_pins_exact_inputs_and_extracts_resource_version(self):
        report = self.fixture.extract()
        self.assertEqual(report["application"],
                         {"version": "17.2.10.21", "sha256": self.fixture.target})
        pin = next(source for source in report["sources"] if source["role"] == "inventory")
        self.assertEqual(pin, {"path": "block-ux-coverage.json", "role": "inventory",
                               "sha256": hashlib.sha256(self.fixture.inventory.read_bytes()).hexdigest()})
        self.assertTrue(all(not Path(source["path"]).is_absolute() and "/" not in source["path"]
                            for source in report["sources"]))
        self.assertTrue(any("\\" in source["path"] for source in report["sources"]))

    def test_duplicate_control_ids_preserve_distinct_catalog_records(self):
        report = self.fixture.extract()
        shape = next(row for row in report["mechanisms"]
                     if row["kind"] == "parameter_declaration_shape")
        self.assertEqual(shape["catalog_indices"], [0, 1])
        self.assertEqual(shape["potential_impact"]["selected_parameter_entries"], 2)
        counts = report["summary"]["source_populations"]
        self.assertEqual(counts["catalog_records"], 3)
        self.assertEqual(counts["distinct_control_type_ids"], 2)
        self.assertEqual(counts["null_xml_records"], 1)
        self.assertEqual(report["summary"]["verified_execution_memberships"], 0)
        self.assertIsNone(report["summary"]["total_gui_fields"])

    def test_duplicate_catalog_indices_are_rejected(self):
        self.fixture.data["blocks"][1]["catalog_index"] = 0
        self.fixture.save("block-ux-coverage.json", self.fixture.data)
        with self.assertRaisesRegex(ValueError, "Duplicate catalog_index"):
            self.fixture.extract()

    def test_name_and_caption_matches_never_become_proven_membership(self):
        report = self.fixture.extract()
        editor = next(row for row in report["mechanisms"]
                      if row.get("class_names") == ["CExampleADlg"])
        self.assertEqual(editor["catalog_indices"], [0, 1])
        self.assertEqual(editor["exact_core_name_candidate_indices"], [0])
        self.assertEqual(editor["membership_basis"]["status"], "candidate")
        template = next(row for row in report["mechanisms"]
                        if row["kind"] == "static_dialog_control_class")
        self.assertEqual(template["catalog_indices"], [0])
        self.assertEqual(template["membership_basis"]["status"], "candidate")
        metadata = next(row for row in report["mechanisms"]
                        if row["kind"] == "parameter_declaration_shape")
        self.assertEqual(metadata["membership_basis"]["status"], "metadata_membership")
        self.assertFalse(metadata["membership_basis"]["execution_verified"])
        self.assertEqual(report["summary"]["verified_widget_bindings"], 0)

    def test_common_editors_and_nested_exceptions_are_retained_without_forced_joins(self):
        report = self.fixture.extract()
        editor = next(row for row in report["mechanisms"]
                      if row.get("group") == "shared-SettingEditor:Numeric")
        self.assertEqual(len(editor["class_ids"]), 2)
        self.assertEqual(editor["catalog_indices"], [])
        self.assertIsNone(editor["potential_impact"]["catalog_records"])
        self.assertEqual(editor["membership_basis"]["status"], "unjoined_lead")
        nested = next(row for row in report["mechanisms"]
                      if row["kind"] == "nested_storage_exception")
        self.assertEqual(nested["storage_root"], "Rows")
        self.assertEqual(nested["catalog_indices"], [0, 1])
        self.assertEqual([row["source_index"] for row in report["classes"]], [0, 1, 2, 3])
        operation = next(row for row in report["mechanisms"]
                         if row["mechanism_id"] == "qt-shared-operation:create-record")
        self.assertEqual(len(operation["declarations"]), 2)

    def test_identical_inputs_produce_deterministic_grouping_and_bytes(self):
        first = self.fixture.extract()
        second = self.fixture.extract()
        self.assertEqual(json.dumps(first, sort_keys=True), json.dumps(second, sort_keys=True))
        data = copy.deepcopy(self.fixture.data)
        data["blocks"].reverse()
        groups = UX.catalog_mechanisms(UX.validate_blocks(data), {"source": "synthetic"})
        expected = UX.catalog_mechanisms(UX.validate_blocks(self.fixture.data),
                                         {"source": "synthetic"})
        self.assertEqual(groups, expected)

    def test_missing_metadata_is_explicit_and_denominators_stay_unknown(self):
        self.fixture.qt.unlink()
        self.fixture.dialogs.unlink()
        report = self.fixture.extract()
        self.assertEqual(report["artifacts"]["qt-metadata"]["status"], "absent")
        self.assertEqual(report["artifacts"]["native-dialogs"]["status"], "absent")
        self.assertEqual(report["summary"]["status"], "partial_metadata_extraction")
        populations = report["summary"]["source_populations"]
        self.assertIsNone(populations["qt_classes"])
        self.assertIsNone(populations["native_static_controls"])
        self.assertEqual(populations["catalog_records"], 3)
        self.assertIn("artifact_absent", {row["reason"] for row in report["omissions"]})

    def test_declared_application_and_xml_pins_fail_closed(self):
        self.fixture.xml.write_bytes(b"changed")
        with self.assertRaisesRegex(ValueError, "Source hash mismatch"):
            self.fixture.extract()
        self.fixture.xml.write_bytes(b"<SyntheticCatalog/>")
        self.fixture.executable.write_bytes(version_pe() + b"changed")
        with self.assertRaisesRegex(ValueError, "Source hash mismatch"):
            self.fixture.extract()

    def test_cross_version_qt_metadata_and_catalog_identity_are_rejected(self):
        qt = json.loads(self.fixture.qt.read_text())
        qt["source"]["sha256"] = "0" * 64
        self.fixture.save("qt-metadata.json", qt)
        with self.assertRaisesRegex(ValueError, "Qt/inventory"):
            self.fixture.extract()
        qt["source"]["sha256"] = self.fixture.target
        self.fixture.save("qt-metadata.json", qt)
        raw = json.loads(self.fixture.catalog.read_text())
        raw["blocks"].reverse()
        self.fixture.save("catalog.json", raw)
        with self.assertRaisesRegex(ValueError, "Raw catalog identity"):
            self.fixture.extract()

    def test_bounded_native_joins_use_actual_body_calls_not_headers_or_strings(self):
        corpus = self.fixture.corpus()
        groups, summary = UX.native_mechanisms(
            corpus, self.fixture.data["blocks"], [], UX.Sources(self.fixture.root))
        self.assertEqual({row["kind"] for row in groups},
                         {"native_property_registration_lead", "native_serializer_lead"})
        self.assertEqual(summary["shared_direct_callees"], 2)
        self.assertEqual(summary["omissions"], [])
        for row in groups:
            self.assertEqual(row["catalog_indices"], [0, 1])
            self.assertEqual(row["membership_basis"]["status"], "candidate")
            self.assertEqual(len(row["callers"]), 2)
        self.assertEqual(UX.direct_calls(
            'void FUN_1() { log("http://x FUN_7000()"); FUN_2(); /* FUN_3() */ }'),
            {"0x2": 1})

    def test_native_hash_mismatch_does_not_fall_back_to_unverified_bytes(self):
        corpus = self.fixture.corpus()
        corpus.functions["0x3000"]["path"].write_text("corrupted")
        with self.assertRaisesRegex(ValueError, "Source hash mismatch"):
            UX.native_mechanisms(corpus, self.fixture.data["blocks"], [],
                                 UX.Sources(self.fixture.root))

    def test_unavailable_native_metadata_and_bounds_are_reported(self):
        corpus = self.fixture.corpus()
        corpus.functions.pop("0x2000")
        _, summary = UX.native_mechanisms(
            corpus, self.fixture.data["blocks"], [], UX.Sources(self.fixture.root))
        self.assertEqual(summary["omissions"], [])
        self.assertEqual(summary["diagnostics"][0]["reason"], "canonical_entry_absent")
        self.assertEqual(summary["diagnostics"][0]["canonical_entry_status"], "absent")
        self.assertEqual(summary["diagnostics"][0]["containing_canonical_ranges"], [])
        self.assertEqual(summary["evidence_status"], "incomplete")
        self.assertEqual(summary["diagnostic_count"], 1)
        corpus = self.fixture.corpus()
        _, summary = UX.native_mechanisms(
            corpus, self.fixture.data["blocks"], [], UX.Sources(self.fixture.root),
            {**UX.BOUNDS, "anchor_bodies": 1})
        self.assertEqual(summary["omissions"][0]["reason"], "anchor_count_bound")

    def test_external_and_unavailable_entries_are_diagnosed_without_invented_body_paths(self):
        for status in ("external", "unknown", "written"):
            with self.subTest(status=status):
                corpus = self.fixture.corpus()
                corpus.functions["0x2000"] = {"status": status, "body": {"available": False}}
                _, summary = UX.native_mechanisms(
                    corpus, self.fixture.data["blocks"], [], UX.Sources(self.fixture.root))
                diagnostic = summary["diagnostics"][0]
                self.assertEqual(diagnostic["canonical_entry_status"], status)
                self.assertEqual(diagnostic["status"], "preserved_incomplete_evidence")
                self.assertEqual(summary["omissions"], [])
                self.assertNotIn("relative_path", diagnostic)
                self.assertNotIn("replacement_address", diagnostic)

    def test_missing_declared_body_or_body_pin_is_required_evidence_failure(self):
        corpus = self.fixture.corpus()
        corpus.functions["0x2000"].pop("sha256")
        with self.assertRaisesRegex(UX.RequiredEvidenceError, "body digest absent"):
            UX.native_mechanisms(corpus, self.fixture.data["blocks"], [],
                                 UX.Sources(self.fixture.root))
        corpus = self.fixture.corpus()
        corpus.functions["0x2000"].pop("relative_path")
        with self.assertRaisesRegex(UX.RequiredEvidenceError, "body path absent"):
            UX.native_mechanisms(corpus, self.fixture.data["blocks"], [],
                                 UX.Sources(self.fixture.root))
        corpus = self.fixture.corpus()
        corpus.functions["0x2000"]["path"].unlink()
        with self.assertRaises(FileNotFoundError):
            UX.native_mechanisms(corpus, self.fixture.data["blocks"], [],
                                 UX.Sources(self.fixture.root))

    def test_required_metadata_pin_absence_is_explicit_partial_input(self):
        raw = json.loads(self.fixture.catalog.read_text())
        raw["source"].pop("xml_sha256")
        self.fixture.save("catalog.json", raw)
        report = self.fixture.extract()
        self.assertEqual(report["artifacts"]["catalog-xml"]["status"], "required_pin_absent")
        self.assertIn("required_source_pin_absent",
                      {row["reason"] for row in report["omissions"]})
        self.assertEqual(report["summary"]["status"], "partial_metadata_extraction")
        self.assertFalse(any(row["role"] == "catalog-xml" for row in report["sources"]))

    def test_exit_codes_distinguish_diagnosed_leads_missing_inputs_and_invalid_pins(self):
        output = self.fixture.root / ".reverse-engineering" / "map.json"
        args = ["--inventory", str(self.fixture.inventory), "--output", str(output)]
        report = {"application": {"sha256": self.fixture.target, "version": "17.2.10.21"},
                  "omissions": [], "summary": {"status": "complete_metadata_extraction",
                                              "evidence_status": "incomplete",
                                              "incomplete_evidence_diagnostics": 7}}
        with patch.object(UX, "ROOT", self.fixture.root), redirect_stdout(io.StringIO()), \
                redirect_stderr(io.StringIO()):
            with patch.object(UX, "extract", return_value=report):
                self.assertEqual(UX.main(args), 0)
            for error in (UX.RequiredEvidenceError("digest missing"),
                          FileNotFoundError("body missing")):
                with patch.object(UX, "extract", side_effect=error):
                    self.assertEqual(UX.main(args), 2)
            with patch.object(UX, "extract", side_effect=ValueError("hash mismatch")):
                self.assertEqual(UX.main(args), 1)

    def test_version_info_requires_resource_structure_not_loose_signatures(self):
        image = bytearray(version_pe())
        struct.pack_into("<II", image, 0x98 + 112 + 16, 0, 0)
        self.assertIsNone(UX.pe_version(image))
        with self.assertRaisesRegex(ValueError, "not a PE"):
            UX.pe_version(b"synthetic-not-an-executable")


if __name__ == "__main__":
    unittest.main()
