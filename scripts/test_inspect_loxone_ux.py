import importlib.util
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "inspect_loxone_ux", Path(__file__).with_name("inspect-loxone-ux.py")
)
UX = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(UX)


def block(xml_type="Example", control_type="10", **io):
    return {
        "attributes": {"LxType": xml_type, "ControlType": control_type, "Name": "Example"},
        "groups": [{"attributes": {"Type": "Parameter"}, "io": [
            {"Id": "0", "Name": "Duration", "Default": "1", **io},
        ]}],
    }


class CoverageTests(unittest.TestCase):
    def test_existing_qt_snapshot_is_not_overwritten_during_layout_migration(self):
        private = Path(__file__).resolve().parents[1] / ".reverse-engineering"
        private.mkdir(exist_ok=True)
        with tempfile.TemporaryDirectory(dir=private) as temporary:
            output = Path(temporary)
            snapshot = output / "qt-metadata.json"
            snapshot.write_text("historical address-derived identities", encoding="utf-8")
            with patch.object(UX, "load_json") as load:
                with self.assertRaisesRegex(ValueError, "existing Qt snapshot"):
                    UX.inspect(Path("unused.json"), output, Path("."), Path("lox.exe"))
            load.assert_not_called()
            self.assertEqual(snapshot.read_text(encoding="utf-8"), "historical address-derived identities")

    def test_only_exact_name_and_control_identity_map_parameter_keys(self):
        schema = {
            "control_type": "10", "connectors": [
                {"io_type": "Parameter", "name_pattern": "Duration", "k_values": ["Time"]},
            ],
        }
        rows = UX.parameter_rows(block(), schema)
        self.assertEqual(rows[0]["known_xml_keys"], ["Time"])
        self.assertEqual(rows[0]["mapping_status"], "name_and_control_id_match")
        wrong_identity = UX.parameter_rows(block(control_type="11"), schema)
        self.assertEqual(wrong_identity[0]["known_xml_keys"], [])
        self.assertEqual(wrong_identity[0]["mapping_status"], "unmapped")

    def test_pattern_expansion_is_explicitly_sampled_not_exhaustive(self):
        rows = UX.parameter_rows(block(Name="Load%d"), {
            "control_type": "10", "connectors": [
                {"io_type": "Parameter", "name_pattern": "Load%d", "k_values": ["Load1", "Load2"]},
            ],
        })
        self.assertEqual(rows[0]["mapping_status"], "sampled_pattern_keys")
        self.assertEqual(rows[0]["known_xml_keys"], ["Load1", "Load2"])

    def test_documentation_template_does_not_invent_repeating_ui_rows(self):
        catalog = {"blocks": [block(TemplateId="12")]}
        row = UX.audit_catalog(catalog, {}, {}, None)[0]
        self.assertEqual(row["documentation_template_references"], 1)
        self.assertEqual(row["repeated_connector_patterns"], [])
        self.assertFalse(row["advanced_candidate"])
        self.assertEqual(row["ux_parity"]["overall"], "unverified")
        self.assertIsNone(row["ux_parity"]["total_gui_field_count"])

    def test_specialized_family_is_included_even_with_few_scalar_parameters(self):
        catalog = {"blocks": [block(xml_type="EFM", control_type="524")]}
        row = UX.audit_catalog(catalog, {}, {}, None)[0]
        self.assertTrue(row["advanced_candidate"])
        self.assertIn("energy", row["families"])
        self.assertIn("specialized_configuration_family", row["advanced_reasons"])

    def test_numeric_identity_and_unmapped_types_are_not_lost(self):
        unknown = block()
        del unknown["attributes"]["LxType"]
        rows = UX.audit_catalog({"blocks": [block(), unknown]}, {}, {}, None)
        self.assertEqual(len(rows), 2)
        self.assertEqual(rows[1]["identity_status"], "xml_type_unknown")
        self.assertIsNone(rows[1]["xml_type"])

    def test_fixture_shapes_are_preserved_without_values_or_gui_claims(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "fixture.Loxone"
            path.write_text(
                '<ControlList><C Type="LightController2" Title="Synthetic" U="example">'
                '<Co K="Q"/><LightscenesC><LightsceneC Name="Synthetic"/></LightscenesC>'
                '<SET><option v="sensitive-value-must-not-be-copied"/></SET>'
                '</C></ControlList>', encoding="utf-8"
            )
            invalid = Path(temporary) / "pointer.Loxone"
            invalid.write_text("version https://git-lfs.github.com/spec/v1")
            def surface_reader(command, **kwargs):
                self.assertEqual(kwargs["env"]["LOX_TELEMETRY"], "0")
                self.assertEqual(command[1:6], [
                    "--quiet", "--output", "json", "config", "surfaces",
                ])
                if Path(command[-1]) == invalid:
                    return SimpleNamespace(returncode=1, stderr="Invalid XML", stdout="")
                return SimpleNamespace(returncode=0, stderr="", stdout=UX.json.dumps({
                    "ok": True, "surfaces": [{
                        "element_type": "LightController2", "attributes": ["Type", "Title", "U"],
                        "storage_paths": ["LightscenesC", "LightscenesC/LightsceneC",
                                          "SET", "SET/option"],
                    }],
                }))
            with patch.object(UX.subprocess, "run", side_effect=surface_reader):
                storage, omissions = UX.fixture_storage([path, invalid], Path("lox.exe"))
            self.assertEqual(storage["LightController2"]["paths"], [
                "LightscenesC", "LightscenesC/LightsceneC", "SET", "SET/option",
            ])
            self.assertNotIn("sensitive-value-must-not-be-copied", str(storage))
            self.assertEqual(len(omissions), 1)
            self.assertEqual(omissions[0]["path"], str(invalid))


if __name__ == "__main__":
    unittest.main()
