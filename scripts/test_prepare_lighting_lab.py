"""Synthetic envelope tests only; no test here executes the official GUI."""

import copy
import hashlib
import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET


SPEC = importlib.util.spec_from_file_location(
    "prepare_lighting_lab",
    Path(__file__).resolve().parents[1] / "tests" / "ux" / "light-controller2" / "prepare-lab.py")
LAB = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(LAB)


class LightingEnvelopeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.path = Path(self.temporary.name) / "synthetic.Loxone"
        self.data = b"""<ControlList>
<C Type="Document" U="document">
 <C Type="CategoryCaption" U="categories"/>
 <C Type="PlaceCaption" U="places"/>
 <C Type="LoxLIVE" U="server" IntAddr="synthetic.invalid" ExtAddr="synthetic.invalid">
  <C Type="User" U="user" DemoMetadata="synthetic-not-a-password"/>
 </C>
 <C Type="Program" U="old-program">
  <C Type="OldBlock" U="aaaaaaaa-bbbb-cccc-dddd000000000000"/>
 </C>
 <C Type="Retained" U="11111111-2222-3333-4444000000000000"/>
 <C Type="Reference" U="ref1" Input="aaaaaaaa-bbbb-cccc-dddd000000000000"/>
 <C Type="Reference" U="ref2" Input="11111111-2222-3333-4444000000000000"/>
 <C Type="Reference" U="ref3" Input="literal"/>
</C></ControlList>"""
        self.path.write_bytes(self.data)

    def seed(self):
        with patch.object(LAB, "PUBLIC_ENVELOPE_SHA256", hashlib.sha256(self.data).hexdigest()):
            return ET.fromstring(LAB.seed(self.path).decode("utf-8-sig"))

    def test_public_envelope_metadata_survives_program_replacement(self):
        root = self.seed()
        document = root.find("C")
        self.assertEqual(root.find(".//C[@Type='User']").attrib,
                         {"Type": "User", "U": "user", "DemoMetadata": "synthetic-not-a-password"})
        self.assertEqual([node.get("Title") for node in document
                          if node.get("Type") == "Program"], ["Lighting Slice Program"])
        self.assertEqual(root.find(".//C[@Type='Program']").get("Ref"), "server")
        self.assertEqual(document.get("NumO"), str(len(list(root.iter("C")))))

    def test_connection_addresses_and_only_removed_program_links_are_cleared(self):
        root = self.seed()
        server = root.find(".//C[@Type='LoxLIVE']")
        self.assertEqual((server.get("IntAddr"), server.get("ExtAddr")), ("", ""))
        refs = {node.get("U"): node.get("Input")
                for node in root.findall(".//C[@Type='Reference']")}
        self.assertIsNone(refs["ref1"])
        self.assertEqual(refs["ref2"], "11111111-2222-3333-4444000000000000")
        self.assertEqual(refs["ref3"], "literal")

    def test_unreviewed_envelope_bytes_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "pinned public"):
            LAB.seed(self.path)


class DigitalSourcePlanTests(unittest.TestCase):
    def fixture(self):
        return ET.fromstring("""<Root><C Type="Document" U="document" NumO="6">
 <C Type="VirtualIn" U="vi-on" IName="VI4"
 Analog="false" Title="Existing on" Untouched="preserve">
 <Co K="Q" U="source-on"/><Co K="Qm" U="source-on-m"/></C>
 <C Type="VirtualIn" U="vi-off" IName="VI5" Analog="false" Title="Existing off">
 <Co K="Q" U="source-off"/><Co K="Qm" U="source-off-m"/></C>
 <C Type="Page" U="page"><C Type="LightController2" U="light">
 <Co K="On" U="on"/><Co K="Reset" U="off"/><LightscenesC Num="4"/>
 </C><C Type="StateV" U="observer"/></C></C></Root>""")

    def test_existing_digital_sources_are_distinct_and_metadata_is_read_only(self):
        root = self.fixture()
        before = ET.tostring(root)
        plan = LAB.digital_source_plan(root, "light")
        self.assertEqual([(n["source_uuid"], n["target_key"], n["target_uuid"])
                          for n in plan], [("vi-on", "On", "on"), ("vi-off", "Reset", "off")])
        self.assertEqual(ET.tostring(root), before)

    def test_analog_or_shared_source_and_already_wired_target_are_rejected(self):
        for mutation, message in [
            (lambda r: r.find(".//C[@Type='VirtualIn']").set("Analog", "true"), "digital source"),
            (lambda r: ET.SubElement(r, "C", Type="InputRef", Ref="vi-on"), "already has"),
            (lambda r: ET.SubElement(r.find(".//Co[@K='Reset']"), "In", Input="other"),
             "already wired"),
        ]:
            with self.subTest(message=message):
                root = self.fixture()
                mutation(root)
                with self.assertRaisesRegex(ValueError, message):
                    LAB.digital_source_plan(root, "light")

    def test_only_new_references_and_explicit_controller_routes_may_change(self):
        baseline = self.fixture()
        plan = LAB.digital_source_plan(baseline, "light")
        prepared = copy.deepcopy(baseline)
        page = prepared.find(".//C[@Type='Page']")
        for route in plan:
            uuid = "reference-" + route["target_key"]
            route.update(reference_uuid=uuid, reference_aq=uuid + "-AQ")
            ref = ET.SubElement(page, "C", Type="InputRef", U=uuid,
                                Ref=route["source_uuid"], RefL=route["source_uuid"])
            for key in ["AI", "I", "AQ", "Q"]:
                ET.SubElement(ref, "Co", K=key, U=uuid + "-" + key)
            target = prepared.find(f".//Co[@U='{route['target_uuid']}']")
            target.set("Nc", "1")
            ET.SubElement(target, "In", Input=route["reference_aq"])
        LAB.verify_harness_preservation(baseline, prepared, plan)
        for selector, key, value in [
            (".//C[@U='vi-on']", "Analog", "true"),
            (".//C[@U='observer']", "Title", "Unexpected observer edit"),
            (".//LightscenesC", "Num", "0"),
        ]:
            with self.subTest(selector=selector):
                mutated = copy.deepcopy(prepared)
                mutated.find(selector).set(key, value)
                with self.assertRaisesRegex(ValueError, "baseline metadata"):
                    LAB.verify_harness_preservation(baseline, mutated, plan)


if __name__ == "__main__":
    unittest.main()
