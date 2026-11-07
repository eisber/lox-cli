"""Synthetic atlas fixtures only; these tests never execute Loxone Config."""

import copy
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location(
    "element_atlas", Path(__file__).with_name("element-atlas.py"))
ATLAS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ATLAS)


class ElementAtlasTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.application = {"version": "synthetic", "sha256": "a" * 64}
        self.inventory = {
            "source": {"executable_sha256": self.application["sha256"]},
            "blocks": [
                {"catalog_index": 0, "control_type": "12", "xml_type": None,
                 "name": "Project", "parameters": []},
                {"catalog_index": 1, "control_type": "12", "xml_type": "Synthetic",
                 "name": "<script>alert(1)</script>", "families": ["test"],
                 "parameters": [{"known_xml_keys": ["Value"]}, {"known_xml_keys": []}]},
            ],
        }
        self.path = self.root / "inventory.json"
        self.write(self.path, self.inventory)
        self.ref = ATLAS.source_ref(self.path, self.root, "inventory")

    def write(self, path, value):
        path.write_text(json.dumps(value), encoding="utf-8")

    def build(self, **kwargs):
        return ATLAS.build_atlas(self.inventory, self.ref, self.application, **kwargs)

    def native(self):
        return {
            "schema_version": 1, "application": self.application,
            "sources": [self.ref],
            "classes": [
                {"class_id": "first", "source_index": 0, "native_class": "Candidate"},
                {"class_id": "unjoined", "source_index": 1, "native_class": "Unjoined"}],
            "joins": [{"catalog_index": 1, "class_id": "first", "status": "candidate"}],
        }

    def test_duplicates_null_names_and_unjoined_classes_remain_visible(self):
        atlas = self.build(native=self.native())
        self.assertEqual(len(atlas["records"]), 2)
        self.assertEqual(len({record["record_id"] for record in atlas["records"]}), 2)
        self.assertEqual(atlas["exceptions"]["duplicate_control_type_ids"],
                         [{"control_type": "12", "catalog_indices": [0, 1]}])
        self.assertEqual(atlas["exceptions"]["without_xml_name"], [0])
        self.assertEqual(atlas["exceptions"]["candidate_only_native_joins"], [1])
        self.assertEqual(atlas["exceptions"]["native_classes_without_catalog_lead"],
                         ["unjoined"])
        self.assertEqual(atlas["summary"]["native_classes_with_proven_catalog_identity"]["count"],
                         0)
        self.assertEqual(atlas["exceptions"]["native_classes_without_proven_catalog_identity"],
                         ["first", "unjoined"])
        self.assertIsNone(atlas["summary"]["overall_percentage"])
        self.assertIsNone(atlas["summary"]["contract_stages"]["gui_oracle"]["percentage"])
        self.assertEqual(atlas["summary"]["parameter_xml_key_mapping"],
                         {"count": 1, "total": 2, "remaining": 1, "percentage": 50.0})

    def test_record_order_does_not_change_frozen_record_identity(self):
        before = self.build()
        self.inventory["blocks"].reverse()
        self.assertEqual(before, self.build())

    def test_native_join_cannot_relabel_the_pinned_catalog_record(self):
        native = self.native()
        native["joins"][0]["xml_type"] = "Different"
        with self.assertRaisesRegex(ValueError, "disagrees"):
            self.build(native=native)

    def test_duplicate_catalog_indices_and_invalid_parameters_are_rejected(self):
        self.inventory["blocks"][1]["catalog_index"] = 0
        with self.assertRaisesRegex(ValueError, "catalog_index"):
            self.build()
        self.inventory["blocks"][1]["catalog_index"] = 1
        self.inventory["blocks"][1]["parameters"] = [{"known_xml_keys": "Value"}]
        with self.assertRaisesRegex(ValueError, "known_xml_keys"):
            self.build()

    def test_stale_sidecars_bad_membership_and_missing_class_are_rejected(self):
        path = self.root / "native.json"
        sidecar = self.native()
        self.write(path, sidecar)
        self.assertEqual(ATLAS.read_sidecar(
            path, self.root, self.application, self.ref, "native", {0, 1}), sidecar)
        for mutation, message in [
                (lambda value: value["application"].update(sha256="b" * 64), "mismatch"),
                (lambda value: value["joins"][0].update(catalog_index=99), "join"),
                (lambda value: value["joins"][0].update(class_id="absent"), "join"),
                (lambda value: value["sources"][0].update(sha256="b" * 64), "SHA256")]:
            value = copy.deepcopy(sidecar)
            mutation(value)
            self.write(path, value)
            with self.assertRaisesRegex(ValueError, message):
                ATLAS.read_sidecar(path, self.root, self.application, self.ref,
                                   "native", {0, 1})

    def test_reference_metadata_overlay_does_not_create_execution_evidence(self):
        reference = {
            "source": self.ref, "block_type": "Synthetic", "control_type": 12,
            "status": "reference_snapshot_not_verified_execution",
            "document": {"expected_native_ordered_connector_count": 2},
        }
        atlas = self.build(references=[reference],
                           connector_map={"Synthetic": {"c": ["I", "O"]}},
                           connector_ref=self.ref)
        row = atlas["records"][1]
        self.assertTrue(row["metadata_overlays"][0]["count_matches"])
        self.assertEqual(row["contract_evidence"]["status"], "unverified")
        self.assertEqual(row["kind"], "not_classified")
        self.assertIsNone(row["overall_percentage"])

    def test_overlapping_snapshots_are_not_merged_into_a_complete_capability(self):
        reports = []
        for index, observed_stage in enumerate(("unit_test", "gui_oracle")):
            reports.append({
                "application": self.application, "block_type": "Synthetic", "control_type": 12,
                "contract": {"path": f"synthetic-{index}.json", "sha256": "b" * 64},
                "capabilities": [{"id": "same-capability"}],
                "gate": {"input_valid": True, "complete": False},
                "stages": {stage: {"verified": int(stage == observed_stage), "declared": 1}
                           for stage in ATLAS.GATES.STAGES},
            })
        atlas = self.build(reports=reports)
        evidence = atlas["records"][1]["contract_evidence"]
        self.assertEqual(evidence["status"], "independent_snapshots")
        self.assertEqual(len(evidence["snapshots"]), 2)
        self.assertIsNone(evidence["report"])
        self.assertEqual(atlas["summary"]["contract_stages"]["complete"]["count"], 0)
        self.assertEqual(atlas["summary"]["input_valid_contract_capabilities"], 2)

    def test_invalid_sidecar_fails_before_writing_a_report(self):
        sidecar = self.native()
        sidecar["application"] = {"version": "old", "sha256": "b" * 64}
        path = self.root / "stale.json"
        self.write(path, sidecar)
        output = self.root / "not-written"
        with contextlib.redirect_stderr(io.StringIO()):
            code = ATLAS.main([
                "--root", str(self.root), "--inventory", str(self.path),
                "--application-version", self.application["version"],
                "--native-map", str(path), "--output", str(output)])
        self.assertEqual(code, 2)
        self.assertFalse(output.exists())

    def test_empty_denominators_are_unknown_and_html_is_escaped(self):
        atlas = self.build()
        self.assertIsNone(atlas["records"][0]["parameter_mapping"]["percentage"])
        rendered = ATLAS.render_html(atlas)
        self.assertNotIn("<script>alert(1)</script>", rendered)
        self.assertIn("&lt;script&gt;alert(1)&lt;/script&gt;", rendered)
        self.assertEqual(rendered.count("<tbody>"), 3)

    def test_utf8_bom_duplicate_keys_and_invalid_json_numbers(self):
        self.path.write_text("\ufeff" + json.dumps(self.inventory), encoding="utf-8")
        self.assertEqual(ATLAS.load_json(self.path), self.inventory)
        for value in ('{"same":1,"same":2}', '{"value":NaN}'):
            self.path.write_text(value, encoding="utf-8")
            with self.assertRaises(ValueError):
                ATLAS.load_json(self.path)

    def test_writes_complete_deterministic_outputs(self):
        atlas = self.build(native=self.native())
        output = self.root / "report"
        ATLAS.write_atlas(atlas, output)
        before = {path.name: path.read_bytes() for path in output.iterdir()}
        ATLAS.write_atlas(atlas, output)
        self.assertEqual(before, {path.name: path.read_bytes() for path in output.iterdir()})
        self.assertEqual(len(ATLAS.load_json(output / "atlas.json")["records"]), 2)
        self.assertEqual(len((output / "catalog.csv").read_text().splitlines()), 3)


if __name__ == "__main__":
    unittest.main()
