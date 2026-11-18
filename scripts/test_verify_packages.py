import importlib.util
import io
from pathlib import Path
import tarfile
import tempfile
import unittest


spec = importlib.util.spec_from_file_location(
    "verify_packages", Path(__file__).with_name("verify-packages.py")
)
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


def inspect(entries):
    data = io.BytesIO()
    with tarfile.open(fileobj=data, mode="w") as archive:
        for name, content in entries.items():
            member = tarfile.TarInfo(name)
            encoded = content.encode()
            member.size = len(encoded)
            archive.addfile(member, io.BytesIO(encoded))
    data.seek(0)
    with tarfile.open(fileobj=data, mode="r") as archive:
        return verifier.inspect_archive(archive, "in-memory.crate")


class PackageInventoryTests(unittest.TestCase):
    def test_packaged_embedded_schema(self):
        report = inspect({
            "lox-cli-0.14.2/src/config_edit/blocks.rs":
                'include_str!("../../docs/schemas/connector-map.json")',
            "lox-cli-0.14.2/docs/schemas/connector-map.json": "{}",
        })
        self.assertEqual(report["errors"], [])
        self.assertEqual(report["file_count"], 2)

    def test_missing_embedded_schema(self):
        report = inspect({
            "lox-cli-0.14.2/src/main.rs": 'include_bytes!("../schema.json")',
        })
        self.assertIn("missing embedded file", report["errors"][0])

    def test_private_tracking_and_vendor_worktree_rejected(self):
        report = inspect({
            "lox-cli-0.14.2/.beads/backup/issues.jsonl": "{}",
            "lox-cli-0.14.2/.beads/README.md": "local task documentation",
            "lox-cli-0.14.2/.reverse-engineering/vendor.dll": "vendor",
        })
        self.assertEqual(len(report["errors"]), 3)
        self.assertTrue(all("private asset" in error for error in report["errors"]))

    def test_traversal_rejected(self):
        report = inspect({"lox-cli-0.14.2/../../outside": "x"})
        self.assertIn("unsafe archive entry", report["errors"][0])

    def test_windows_drive_and_backslash_paths_rejected(self):
        report = inspect({
            "lox-sim-0.1.0/C:\\outside.txt": "x",
            "lox-sim-0.1.0/C:/outside.txt": "x",
        })
        self.assertEqual(len(report["errors"]), 2)
        self.assertTrue(all("unsafe archive entry" in error for error in report["errors"]))

    def test_same_name_and_version_changed_source_gets_distinct_standalone_identity(self):
        package_spec = importlib.util.spec_from_file_location(
            "simulator_package", Path(__file__).with_name("test-simulator-package.py")
        )
        package = importlib.util.module_from_spec(package_spec)
        package_spec.loader.exec_module(package)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            archive_path = root / "lox-sim-0.1.0.crate"
            identities = []
            for content in (b"original source", b"different source, identical package version"):
                with tarfile.open(archive_path, "w:gz") as archive:
                    for name, data in (("src/lib.rs", content), ("Cargo.lock", b"version = 4\n")):
                        member = tarfile.TarInfo("lox-sim-0.1.0/" + name)
                        member.size = len(data)
                        archive.addfile(member, io.BytesIO(data))
                source, identity, _ = package.unpack_archive(archive_path, root / "unpacked")
                identities.append(identity)
                self.assertEqual((source / "src/lib.rs").read_bytes(), content)
            self.assertNotEqual(*identities)


if __name__ == "__main__":
    unittest.main()
