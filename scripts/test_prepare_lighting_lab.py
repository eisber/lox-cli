"""Synthetic envelope tests only; no test here executes the official GUI."""

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


if __name__ == "__main__":
    unittest.main()
