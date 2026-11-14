"""Exercise the guarded generator correction without editing the input sources."""

import argparse
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
MODULE_SPEC = importlib.util.spec_from_file_location(
    "archive_java_correction", ROOT / "scripts" / "patch-archive-java-generator.py"
)
PATCHER = importlib.util.module_from_spec(MODULE_SPEC)
MODULE_SPEC.loader.exec_module(PATCHER)
SPEC = json.loads(PATCHER.SPEC_PATH.read_text(encoding="utf-8"))


class GeneratorCorrection(unittest.TestCase):
    def setUp(self):
        self.directory = SCRATCH / self.id().split(".")[-1]
        self.directory.mkdir(parents=True)
        for key in ("target", "companion"):
            path = self.directory.joinpath(*SPEC[key].split("/"))
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(INPUT.joinpath(*SPEC[key].split("/")).read_bytes())

    def test_exact_input_corrects_once_and_rejects_reapplication(self):
        target = self.directory.joinpath(*SPEC["target"].split("/"))
        receipt = self.directory / "receipt.json"
        PATCHER.correct(self.directory, "0.5.2", receipt)
        corrected = target.read_bytes()
        self.assertEqual(PATCHER.digest(corrected), SPEC["corrected_sha256"])
        with self.assertRaises(ValueError):
            PATCHER.correct(self.directory, "0.5.2", receipt)
        self.assertEqual(target.read_bytes(), corrected)

    def test_unreviewed_version_or_either_changed_source_refuses_without_editing(self):
        for change in ("version", "target", "companion"):
            with self.subTest(change=change):
                directory = self.directory / change
                directory.mkdir()
                for key in ("target", "companion"):
                    destination = directory.joinpath(*SPEC[key].split("/"))
                    destination.parent.mkdir(parents=True, exist_ok=True)
                    original = INPUT.joinpath(*SPEC[key].split("/")).read_bytes()
                    destination.write_bytes(original + (b"\n" if change == key else b""))
                target = directory.joinpath(*SPEC["target"].split("/"))
                before = target.read_bytes()
                with self.assertRaises(ValueError):
                    PATCHER.correct(directory, "0.5.3" if change == "version" else "0.5.2",
                                    directory / "receipt.json")
                self.assertEqual(target.read_bytes(), before)
                self.assertFalse((directory / "receipt.json").exists())


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--generated-dir", type=Path, required=True)
    parser.add_argument("--scratch", type=Path, required=True)
    args = parser.parse_args()
    INPUT, SCRATCH = args.generated_dir, args.scratch
    unittest.main(argv=[__file__], verbosity=2)
