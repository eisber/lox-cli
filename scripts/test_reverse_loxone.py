"""Focused tests for snapshot identity, resume state, and REA error handling."""

import importlib.util
from contextlib import closing
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "reverse_loxone", Path(__file__).with_name("reverse-loxone.py")
)
RE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(RE)


class SnapshotTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.install = self.root / "install"
        self.output = self.root / "output"
        self.install.mkdir()
        (self.install / "LoxoneConfig.exe").write_bytes(b"test executable")
        (self.install / "catalog.LxRes").write_bytes(b"original metadata")

    def test_support_file_update_creates_a_new_frozen_version(self):
        first, before = RE.snapshot(self.install, self.output)
        (self.install / "catalog.LxRes").write_bytes(b"updated metadata")
        second, after = RE.snapshot(self.install, self.output)
        self.assertNotEqual(first, second)
        self.assertEqual(
            (first / "source" / "catalog.LxRes").read_bytes(),
            b"original metadata",
        )
        self.assertEqual(RE.installation_delta(before["files"], after["files"]), {
            "added": [], "removed": [], "changed": ["catalog.LxRes"],
        })

    def test_unchanged_snapshot_is_reused_and_verified(self):
        first, before = RE.snapshot(self.install, self.output)
        second, after = RE.snapshot(self.install, self.output)
        self.assertEqual(first, second)
        self.assertEqual(before, after)
        (first / "source" / "LoxoneConfig.exe").write_bytes(b"corruption")
        with self.assertRaisesRegex(RuntimeError, "Frozen source was modified"):
            RE.snapshot(self.install, self.output)

    def test_racing_installer_does_not_publish_a_manifest(self):
        original_copy = RE.shutil.copyfile

        def mutate_and_copy(source, destination):
            source.write_bytes(b"concurrent installation update")
            return original_copy(source, destination)

        with patch.object(RE.shutil, "copyfile", side_effect=mutate_and_copy):
            with self.assertRaisesRegex(RuntimeError, "changed during snapshot"):
                RE.snapshot(self.install, self.output)
        self.assertEqual(list(self.output.rglob("installation.json")), [])

    def test_delta_detects_removal_addition_and_content_not_size(self):
        (self.install / "removed").write_bytes(b"x")
        before = RE.inventory(self.install)
        (self.install / "removed").unlink()
        (self.install / "added").write_bytes(b"x")
        (self.install / "catalog.LxRes").write_bytes(b"changed! metadata")
        delta = RE.installation_delta(before, RE.inventory(self.install))
        self.assertEqual(delta, {
            "added": ["added"], "removed": ["removed"], "changed": ["catalog.LxRes"],
        })

    def test_concurrent_runs_cannot_share_output(self):
        self.output.mkdir()
        with RE.exclusive_run(self.output):
            with self.assertRaisesRegex(RuntimeError, "Another analysis"):
                with RE.exclusive_run(self.output):
                    self.fail("Second owner acquired the lock")
        with RE.exclusive_run(self.output):
            pass


class ReaResultTests(unittest.TestCase):
    def test_real_evidence_envelope_preserves_limitations(self):
        evidence = {
            "normalized_result": {"counts": {"failed": 2}},
            "limitations": ["Original symbols unavailable"],
        }
        result = {"structuredContent": evidence}
        self.assertIs(RE.tool_value(result), evidence)
        self.assertEqual(RE.normalized(evidence), {"counts": {"failed": 2}})
        self.assertEqual(evidence["limitations"], ["Original symbols unavailable"])

    def test_text_fallback_and_protocol_failure(self):
        result = {"content": [{"type": "text", "text": '{"normalized_result": 4}'}]}
        self.assertEqual(RE.normalized(RE.tool_value(result)), 4)
        with self.assertRaisesRegex(RuntimeError, "REA tool failed"):
            RE.tool_value({"isError": True, **result})
        with self.assertRaisesRegex(RuntimeError, "REA operation failed"):
            RE.tool_value({"structuredContent": {"ok": False, "error": "failed"}})
        with self.assertRaisesRegex(RuntimeError, "one structured tool result"):
            RE.tool_value({"content": []})

    def test_returning_to_a_cached_version_updates_latest_pointer(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            installation = root / "install"
            installation.mkdir()
            (installation / "LoxoneConfig.exe").write_bytes(b"synthetic executable")
            output = root / "output"
            version, _ = RE.snapshot(installation, output)
            facts = {"synthetic": "toolchain"}
            analysis = version / "analysis" / RE.json_digest(facts)
            analysis.mkdir(parents=True)
            complete = {"status": "complete", "installation_sha256": "cached"}
            RE.write_json(analysis / "status.json", complete)
            RE.write_json(output / "latest.json", {
                "installation_sha256": RE.read_json(version / "installation.json")[
                    "installation_sha256"
                ],
                "analysis_directory": "a different cached analysis",
            })
            with patch.object(RE, "toolchain", return_value=facts):
                self.assertEqual(RE.run(SimpleNamespace(
                    install_dir=installation, output=output,
                )), 0)
            self.assertEqual(RE.read_json(output / "latest.json"), complete)


class ResearchIndexTests(unittest.TestCase):
    def test_only_verified_successful_functions_enter_search_index(self):
        import sqlite3
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            code = directory / "code"
            functions = code / "functions"
            functions.mkdir(parents=True)
            body = functions / "0x1000.c"
            body.write_text("void f() { /* DayTimer */ }", encoding="utf-8")
            RE.write_json(code / "decompile-index.json", {"functions": [
                {"address": "0x1000", "name": "f", "status": "written",
                 "relative_path": str(Path("functions") / body.name),
                 "sha256": RE.digest(body)},
                {"address": "0x2000", "name": "g", "status": "failed"},
                {"address": "0x3000", "name": "external", "status": "external"},
            ]})
            self.assertEqual(
                RE.index_code(directory), {"written": 1, "failed": 1, "external": 1}
            )
            with closing(sqlite3.connect(directory / "research.sqlite3")) as database:
                self.assertEqual(database.execute(
                    "SELECT address FROM functions WHERE functions MATCH 'DayTimer'"
                ).fetchall(), [("0x1000",)])
            self.assertEqual(
                RE.read_json(directory / "topics.json")["block-semantics"][0]["terms"],
                ["DayTimer"],
            )
            body.write_text("tampered", encoding="utf-8")
            with self.assertRaisesRegex(RuntimeError, "digest mismatch"):
                RE.index_code(directory)


class CatalogTests(unittest.TestCase):
    def test_numeric_identity_and_ambiguous_names_are_preserved(self):
        root = RE.ET.fromstring(
            '<TechDoc><FunctionBlock Name="Timer" LxType="DayTimer" ControlType="10">'
            '<IOGroup Type="Input"><IO Id="0" Name="Trigger" ShortName="Tr" '
            'Default="0" TemplateId="4"/></IOGroup></FunctionBlock>'
            '<FunctionBlock Name="Timer" ControlType="20"/></TechDoc>'
        )
        blocks = RE.catalog_blocks(root)
        self.assertEqual(len(blocks), 2)
        self.assertEqual(blocks[0]["attributes"]["ControlType"], "10")
        self.assertEqual(blocks[1]["attributes"]["ControlType"], "20")
        self.assertNotIn("LxType", blocks[1]["attributes"])
        self.assertEqual(blocks[0]["groups"][0]["io"][0], {
            "Id": "0", "Name": "Trigger", "ShortName": "Tr",
            "Default": "0", "TemplateId": "4",
        })

    def test_decoded_catalog_is_published_only_after_crc_verification(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            version = root / "version"
            source = version / "source"
            source.mkdir(parents=True)
            analysis = root / "analysis"
            analysis.mkdir()
            content = b'<TechDoc><FunctionBlock ControlType="10"/></TechDoc>'
            archive = source / "tdc_DEV.LxRes"
            archive.write_bytes(RE.struct.pack(
                "<IIII", 0xAABBCCEE, 0, len(content), RE.zlib.crc32(content)
            ))

            def decode(command, **kwargs):
                Path(command[-1]).write_bytes(content)
                self.assertEqual(kwargs["env"]["LOX_TELEMETRY"], "0")

            args = SimpleNamespace(lox=root / "lox.exe")
            with patch.object(RE.subprocess, "run", side_effect=decode):
                result = RE.extract_resources(args, version, analysis)
                self.assertEqual(result["extracted"], 1)
                catalog_path = analysis / "resources" / "block-catalog.json"
                self.assertEqual(
                    RE.read_json(catalog_path)["blocks"][0]["attributes"],
                    {"ControlType": "10"},
                )
                catalog_path.unlink()
                archive.write_bytes(RE.struct.pack(
                    "<IIII", 0xAABBCCEE, 0, len(content), 1
                ))
                with self.assertRaisesRegex(RuntimeError, "CRC mismatch"):
                    RE.extract_resources(args, version, analysis)
                self.assertFalse(catalog_path.exists())


class RunStatusTests(unittest.TestCase):
    def test_failed_functions_are_partial_and_retried_not_cached(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            installation = root / "install"
            installation.mkdir()
            (installation / "LoxoneConfig.exe").write_bytes(b"synthetic executable")
            args = SimpleNamespace(
                install_dir=installation, output=root / "output",
                binary=["LoxoneConfig.exe"], lox=root / "lox.exe",
            )
            with (
                patch.object(RE, "toolchain", return_value={"synthetic": "tools"}),
                patch.object(RE, "analyze_binary", return_value={
                    "counts": {"remaining": 0},
                }) as analyze,
                patch.object(RE, "index_code", return_value={"failed": 1}),
            ):
                self.assertEqual(RE.run(args), 2)
                self.assertEqual(RE.read_json(args.output / "latest.json")["status"], "partial")
                self.assertEqual(RE.run(args), 2)
                self.assertEqual(analyze.call_count, 2)


if __name__ == "__main__":
    unittest.main()
