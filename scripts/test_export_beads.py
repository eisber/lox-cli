import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location(
    "export_beads", Path(__file__).with_name("export-beads.py")
)
EXPORT = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(EXPORT)


class IssueExportTests(unittest.TestCase):
    def test_preserves_issue_semantics_without_local_owner_routing(self):
        issue = {
            "id": "lox-example", "owner": "local-owner-routing",
            "created_at": "2026-11-05T07:00:00Z", "assignee": "energy-oracle",
            "metadata": {"large": 9007199254740993, "optional": None},
            "dependencies": [{"depends_on_id": "lox-contract", "type": "blocks"}],
        }
        result = EXPORT.portable_issue(issue)
        self.assertNotIn("owner", result)
        self.assertEqual(result["created_at"], issue["created_at"])
        self.assertEqual(result["metadata"], issue["metadata"])
        self.assertEqual(result["dependencies"], issue["dependencies"])
        self.assertIn("owner", issue)

    def test_failed_export_does_not_replace_existing_portable_state(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "issues.jsonl"
            output.write_text('{"id":"lox-existing"}\n')
            with patch.object(
                EXPORT.subprocess, "run", side_effect=EXPORT.subprocess.CalledProcessError(1, "bd")
            ):
                with self.assertRaises(EXPORT.subprocess.CalledProcessError):
                    EXPORT.export_issues(Path("bd"), output)
            self.assertEqual(output.read_text(), '{"id":"lox-existing"}\n')

    def test_export_is_atomic_and_omits_only_the_local_owner_field(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "issues.jsonl"

            def export(command, **kwargs):
                self.assertIn("--sandbox", command)
                path = Path(command[command.index("--output") + 1])
                path.write_text(
                    '{"id":"lox-case","owner":"local-routing","status":"in_progress"}\n'
                )

            with patch.object(EXPORT.subprocess, "run", side_effect=export):
                self.assertEqual(EXPORT.export_issues(Path("bd"), output), 1)
            self.assertEqual(EXPORT.json.loads(output.read_text()), {
                "id": "lox-case", "status": "in_progress",
            })
            self.assertEqual(list(Path(temporary).iterdir()), [output])


if __name__ == "__main__":
    unittest.main()
