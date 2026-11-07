#!/usr/bin/env python3
"""Synthetic tests; no vendor corpus or executable is needed."""

import copy
import importlib.util
import json
from pathlib import Path
import unittest


SPEC = importlib.util.spec_from_file_location(
    "element_native_map", Path(__file__).with_name("element-native-map.py"))
NATIVE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(NATIVE)


def fixture():
    inventory = {
        "schema_version": 1, "source": {"executable_sha256": NATIVE.TARGET},
        "summary": {"catalog_records": 3, "explicit_xml_types": 1},
        "blocks": [
            {"catalog_index": 0, "xml_type": None, "control_type": "1", "name": "Project"},
            {"catalog_index": 1, "xml_type": "Demo", "control_type": "9", "name": "First"},
            {"catalog_index": 2, "xml_type": "Demo", "control_type": "9", "name": "Second"},
        ],
    }
    native = {
        "schema_version": 1, "target_sha256": NATIVE.TARGET,
        "classes": [
            {"type": {"address": "0x100", "display_name": "CControlObjectDemo"},
             "primary_tables": [{"validated_candidate": True, "target": "0x200",
                                 "vtable": "0x300", "exact_owners": ["0x200"],
                                 "target_is_canonical_entry": True}],
             "secondary_loci": [{"address": "0x444", "handler_slot_read": False}],
             "xml_type_name": None, "simulator_supported": None},
            {"type": {"address": "0x101", "display_name": "UnmatchedClass"},
             "primary_tables": [], "secondary_loci": []},
        ],
        "handlers": [{"address": "0x200", "role_status": "candidate"}],
        "summary": {"class_count": 2, "classes_with_validated_primary_candidate": 1,
                    "unique_handler_count": 1, "secondary_loci_preserved": 1},
    }
    sources = [
        {"path": ".reverse-engineering\\inventory.json", "sha256": "a" * 64, "role": "inventory"},
        {"path": ".reverse-engineering\\native.json", "sha256": "b" * 64, "role": "native_catalog"},
    ]
    return inventory, native, sources


def proof(xml="Demo", ct="9", descriptor="0x100"):
    return {"xml_type": xml, "control_type": ct, "type_descriptor": descriptor,
            "basis": "factory_constructor_and_xml_registration",
            "evidence": [{"role": "synthetic", "fact": "synthetic_registration"}]}


class NativeMapTests(unittest.TestCase):
    def test_duplicate_numeric_ids_preserve_all_source_records(self):
        inventory, native, sources = fixture()
        result = NATIVE.build_map(inventory, native, sources)
        self.assertEqual([0, 1, 2], [r["catalog_index"] for r in result["catalog_records"]])
        self.assertEqual([1, 2], [r["catalog_index"] for r in result["joins"]])
        self.assertEqual([0], result["unjoined_catalog_indices"])
        self.assertEqual("Project", result["catalog_records"][0]["name"])
        self.assertEqual(2, result["summary"]["distinct_control_type_ids"])

    def test_validated_handler_and_name_are_only_candidates(self):
        inventory, native, sources = fixture()
        result = NATIVE.build_map(inventory, native, sources)
        self.assertEqual(["candidate", "candidate"], [r["status"] for r in result["joins"]])
        self.assertIn("name_only", result["joins"][0]["basis"])
        self.assertIsNone(result["classes"][0]["simulator_supported"])
        self.assertEqual(native["classes"][0]["secondary_loci"],
                         result["classes"][0]["secondary_loci"])

    def test_factory_and_xml_proof_promotes_only_matching_record_pair(self):
        inventory, native, sources = fixture()
        inventory["blocks"][2]["control_type"] = "10"
        result = NATIVE.build_map(inventory, native, sources, [proof()])
        self.assertEqual(["exact", "candidate"], [r["status"] for r in result["joins"]])

    def test_alias_proof_overrides_coincidental_name_collision(self):
        inventory, native, sources = fixture()
        result = NATIVE.build_map(inventory, native, sources, [proof(descriptor="0x101")])
        self.assertEqual(["alias", "alias"], [r["status"] for r in result["joins"]])
        self.assertEqual(result["classes"][1]["class_id"], result["joins"][0]["class_id"])
        self.assertIn(result["classes"][0]["class_id"], result["unjoined_class_ids"])

    def test_same_class_name_different_rtti_is_ambiguous_not_deduplicated(self):
        inventory, native, sources = fixture()
        native["classes"][1]["type"]["display_name"] = "CControlObjectDemo"
        result = NATIVE.build_map(inventory, native, sources)
        self.assertEqual(4, len(result["joins"]))
        self.assertTrue(all(j["status"] == "ambiguous" for j in result["joins"]))
        self.assertNotEqual(result["classes"][0]["class_id"], result["classes"][1]["class_id"])

    def test_conflicting_or_unbacked_proofs_are_rejected(self):
        inventory, native, sources = fixture()
        with self.assertRaisesRegex(ValueError, "Conflicting"):
            NATIVE.build_map(inventory, native, sources, [proof(), proof(descriptor="0x101")])
        p = proof()
        p["basis"] = "name"
        with self.assertRaisesRegex(ValueError, "require factory"):
            NATIVE.build_map(inventory, native, sources, [p])

    def test_sha_mismatch_and_declared_count_mismatch_are_explicit(self):
        inventory, native, sources = fixture()
        for change, message in (
            (lambda i, n: i["source"].update(executable_sha256="0" * 64), "SHA-256"),
            (lambda i, n: i["summary"].update(catalog_records=4), "disagrees"),
            (lambda i, n: n["summary"].update(class_count=3), "disagrees"),
            (lambda i, n: n.update(handlers=[]), "Handler records"),
        ):
            i, n = copy.deepcopy(inventory), copy.deepcopy(native)
            change(i, n)
            with self.subTest(message=message), self.assertRaisesRegex(ValueError, message):
                NATIVE.build_map(i, n, sources)

    def test_invalid_or_duplicate_record_indices_and_rtti_are_rejected(self):
        inventory, native, sources = fixture()
        for value in (True, -1, 1):
            modified = copy.deepcopy(inventory)
            modified["blocks"][2]["catalog_index"] = value
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "catalog_index"):
                NATIVE.build_map(modified, native, sources)
        native["classes"][1]["type"]["address"] = "0x100"
        with self.assertRaisesRegex(ValueError, "Duplicate native RTTI"):
            NATIVE.build_map(inventory, native, sources)

    def test_invalid_control_types_schema_and_source_paths_are_rejected(self):
        inventory, native, sources = fixture()
        for value in (True, None, "-1", "nine", 9.0):
            modified = copy.deepcopy(inventory)
            modified["blocks"][1]["control_type"] = value
            with self.subTest(value=value), self.assertRaisesRegex(ValueError, "ControlType"):
                NATIVE.build_map(modified, native, sources)
        inventory["schema_version"] = True
        with self.assertRaisesRegex(ValueError, "schema_version"):
            NATIVE.build_map(inventory, native, sources)
        inventory["schema_version"] = 1
        for path in ("C:\\evidence.json", "\\evidence.json", "..\\evidence.json",
                     "evidence/source.json"):
            modified = copy.deepcopy(sources)
            modified[0]["path"] = path
            with self.subTest(path=path), self.assertRaisesRegex(ValueError, "root-relative"):
                NATIVE.build_map(inventory, native, modified)

    def test_original_catalog_prevents_dropped_or_misidentified_rows(self):
        inventory, _, _ = fixture()
        original = {"blocks": [
            {"attributes": {"Name": b["name"], "ControlType": b["control_type"],
                            **({"LxType": b["xml_type"]} if b["xml_type"] else {})}}
            for b in inventory["blocks"]]}
        NATIVE.reconcile_source_catalog(inventory, original)
        for mutate in (
            lambda x: x["blocks"].pop(),
            lambda x: x["blocks"][1].update(control_type="10"),
            lambda x: x["blocks"][0].update(xml_type="Project"),
        ):
            modified = copy.deepcopy(inventory)
            mutate(modified)
            with self.assertRaises(ValueError):
                NATIVE.reconcile_source_catalog(modified, original)

    def test_utf8_bom_and_duplicate_keys(self):
        self.assertEqual({"x": "ä"}, NATIVE.decode_json(b'\xef\xbb\xbf{"x":"\\u00e4"}'))
        with self.assertRaisesRegex(ValueError, "Duplicate JSON"):
            NATIVE.decode_json(b'{"x":1,"x":2}')
        with self.assertRaisesRegex(ValueError, "Invalid JSON"):
            NATIVE.decode_json(b'{"x":NaN}')

    def test_factory_groups_and_primary_constructor_proof(self):
        text = """
  switch(param_1) {
  case 8:
  case 0x9:
    pvVar4 = operator_new(0x20);
    if (pvVar4 != (void *)0x0) {
      plVar5 = (longlong *)FUN_123(pvVar4);
    }
    break;
  }
  return plVar5;
"""
        cases = NATIVE.factory_cases(text)
        self.assertEqual(cases["8"], cases["9"])
        NATIVE.prove_factory_class(cases["9"], "Demo",
                                   "*param_1 = Demo::vftable;", "0x123")
        with self.assertRaisesRegex(ValueError, "class mismatch"):
            NATIVE.prove_factory_class(cases["9"], "Demo",
                                       "*param_1 = Other::vftable;", "0x123")
        with self.assertRaisesRegex(ValueError, "ambiguous"):
            NATIVE.prove_factory_class(cases["9"] + "\nplVar5 = other;", "Demo",
                                       "*param_1 = Demo::vftable;", "0x123")

    def test_inline_factory_proof_and_unsupported_alternatives(self):
        case = """
    plVar3 = operator_new(0x20);
    if (plVar3 != (longlong *)0x0) {
      *plVar3 = (longlong)Demo::vftable;
      plVar5 = plVar3;
    }
    break;
"""
        NATIVE.prove_factory_class(case, "Demo")
        with self.assertRaisesRegex(ValueError, "alternate control flow"):
            NATIVE.prove_factory_class(case + " else {other();}", "Demo")

    def test_deterministic_pure_output_and_no_mutation(self):
        inventory, native, sources = fixture()
        before = copy.deepcopy((inventory, native, sources))
        first = NATIVE.build_map(inventory, native, sources, [proof()])
        second = NATIVE.build_map(inventory, native, list(reversed(sources)), [proof()])
        self.assertEqual(json.dumps(first), json.dumps(second))
        self.assertEqual(before, (inventory, native, sources))


if __name__ == "__main__":
    unittest.main()
