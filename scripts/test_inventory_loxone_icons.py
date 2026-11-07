import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import zipfile


SPEC = importlib.util.spec_from_file_location(
    "inventory_loxone_icons", Path(__file__).with_name("inventory-loxone-icons.py")
)
ICONS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ICONS)
ROOT = Path(__file__).resolve().parents[1]

# Independently authored catalog records and non-artwork payloads.
A = '<Icon uuid="abcdef01-00FF-0000-0123456789ABCDEF" Idx="987" Id="demo-a.svg" filled="true" Tags="private localized words"/>'
B = '<Icon uuid="00000002-00FF-0000-0000000000000000" Idx="1" Id="demo-b.svg" filled="false"/>'
C = '<Icon uuid="00000003-00FF-0000-0000000000000000" Idx="2" Id="" filled="true"/>'


class IconInventoryTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory(prefix=".icon-inventory-", dir=ROOT)
        self.addCleanup(self.directory.cleanup)
        self.archive = Path(self.directory.name) / "independent.zip"

    def write(self, records=A + B, *, version=b"20400103", assets=True, locale=None):
        with zipfile.ZipFile(self.archive, "w") as archive:
            archive.writestr("version", version)
            archive.writestr("IconLibrary.xml", "<Icons>" + records + "</Icons>")
            if assets:
                archive.writestr("IconsFilled/demo-a.svg", b"independently authored opaque asset")
            if locale is not None:
                archive.writestr("IconLibrary_DEU.xml", "<Icons>" + locale + "</Icons>")

    def test_loaded_filter_exact_identity_and_private_data_exclusion(self):
        self.write(A + B + C, locale=A.replace("private localized words", "other words") + B + C)
        result = ICONS.inventory(self.archive)
        self.assertEqual(result["icons"], [{
            "native_id": "abcdef01-00ff-0000-0123456789abcdef", "filename": "demo-a.svg",
        }])
        self.assertEqual(result["catalog"], {
            "version": "20400103",
            "archive_sha256": hashlib.sha256(self.archive.read_bytes()).hexdigest(),
        })
        encoded = json.dumps(result)
        for private in ["private", "other words", "Tags", "Idx", "opaque asset", str(self.archive)]:
            self.assertNotIn(private, encoded)

    def test_case_alias_duplicate_and_locale_mismatch_are_rejected(self):
        self.write(A + A.replace("abcdef01", "ABCDEF01").replace("demo-a.svg", "demo-c.svg"))
        with self.assertRaisesRegex(ValueError, "duplicate icon identity"):
            ICONS.inventory(self.archive)
        self.write(locale=A.replace('Idx="987"', 'Idx="986"') + B)
        with self.assertRaisesRegex(ValueError, "identity differs"):
            ICONS.inventory(self.archive)
        self.write(A + B.replace("demo-b.svg", "demo-a.svg"))
        with self.assertRaisesRegex(ValueError, "duplicate canonical filename"):
            ICONS.inventory(self.archive)

    def test_missing_loaded_asset_or_version_fails_without_partial_inventory(self):
        self.write(assets=False)
        with self.assertRaisesRegex(ValueError, "missing or empty loaded asset"):
            ICONS.inventory(self.archive)
        self.write(version=b"not-an-archive-version")
        with self.assertRaisesRegex(ValueError, "archive version"):
            ICONS.inventory(self.archive)
        self.write(A.replace("abcdef01-00FF", "abcdef0-100FF"))
        with self.assertRaisesRegex(ValueError, "identity"):
            ICONS.inventory(self.archive)

    def test_determinism_is_independent_of_archive_path_and_catalog_order(self):
        self.write()
        expected = ICONS.inventory(self.archive)
        renamed = self.archive.with_name("renamed.zip")
        renamed.write_bytes(self.archive.read_bytes())
        self.assertEqual(expected, ICONS.inventory(renamed))
        self.assertEqual(expected, ICONS.inventory(self.archive))
        self.write(B + A, version=b"20400104")
        refreshed = ICONS.inventory(self.archive)
        self.assertEqual(refreshed["icons"], expected["icons"])
        self.assertEqual(refreshed["catalog"]["version"], "20400104")
        self.assertNotEqual(refreshed["catalog"]["archive_sha256"], expected["catalog"]["archive_sha256"])


if __name__ == "__main__":
    unittest.main()
