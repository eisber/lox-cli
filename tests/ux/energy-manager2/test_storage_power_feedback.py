"""Focused actual CLI checks; synthetic input only, no native GUI or Miniserver."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import unittest
import uuid

ROOT = Path(__file__).resolve().parents[3]
BINARY = None


class StoragePowerFeedbackTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        if BINARY is None:
            raise unittest.SkipTest("Run this module with --binary pointing to the isolated tested CLI.")

    def setUp(self):
        self.directory = ROOT / ".reverse-engineering" / "native-simulator" / "workstreams" / "energy-closeout" / "cli-tests" / uuid.uuid4().hex
        self.directory.mkdir(parents=True)
        self.fixture = self.directory / "case.Loxone"
        loads = "".join(f'<Load Id="b0000000-0000-0000-0000-{i:012x}" OIdx="{i}"/>' for i in range(12))
        self.fixture.write_text(
            '<ControlList><C Type="EnergyManager2" Title="Pilot" U="a0000000-0000-0000-0000-000000000001">'
            '<Co K="O"/><Co K="MinSoc" Def="50"/>'
            '<Co K="MaxSpwr"/><Co K="Soc" Nc="1"><In Input="c0000000-0000-0000-0000-000000000001"/></Co>'
            + loads
            + '</C><C Type="Constant" Title="Zero"><Co K="Q" U="c0000000-0000-0000-0000-000000000001" Def="0"/></C></ControlList>',
            encoding="utf-8",
        )

    def tearDown(self):
        shutil.rmtree(self.directory)

    def invoke(self, *arguments):
        result = subprocess.run(
            [str(BINARY), "config", *arguments, "--output", "json"],
            cwd=ROOT, env={**os.environ, "LOX_TELEMETRY": "0"},
            capture_output=True, encoding="utf-8", errors="strict", timeout=30, check=True,
        )
        return json.loads(result.stdout)

    def inspect(self):
        return self.invoke("energy", "inspect", str(self.fixture), "Pilot")["energy"]

    def test_warning_is_read_only_and_zero_edit_is_not_rejected(self):
        before = self.fixture.read_bytes()
        inspection = self.inspect()
        self.assertEqual(inspection["storage_power_feedback"]["status"], "missing")
        self.assertEqual([d["code"] for d in inspection["diagnostics"]], ["missing_storage_power"])
        self.assertEqual(self.fixture.read_bytes(), before)
        self.invoke("energy", "set", str(self.fixture), "Pilot", "MaxSpwr", "3")
        self.assertEqual(self.inspect()["storage_power_feedback"]["status"], "not_triggered")
        self.invoke("energy", "set", str(self.fixture), "Pilot", "MaxSpwr", "0")
        self.assertEqual(self.inspect()["storage_power_feedback"]["status"], "missing")

    def test_zero_wired_minimum_and_maximum_have_different_feedback(self):
        self.invoke("energy", "set", str(self.fixture), "Pilot", "MinSoc", "0")
        self.assertEqual(self.inspect()["storage_power_feedback"]["status"], "not_triggered")
        source = "c0000000-0000-0000-0000-000000000001"
        self.invoke("wire", str(self.fixture), "Pilot.MinSoc", source)
        self.assertEqual(self.inspect()["storage_power_feedback"]["status"], "missing")
        self.invoke("wire", str(self.fixture), "Pilot.MaxSpwr", source)
        self.assertEqual(self.inspect()["storage_power_feedback"]["status"], "not_triggered")
        self.assertEqual(self.inspect()["diagnostics"], [])


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    args, remaining = parser.parse_known_args()
    BINARY = args.binary.resolve(strict=True)
    unittest.main(argv=[__file__, *remaining])
