import copy
from contextlib import closing
import hashlib
import importlib.util
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import tempfile
import unittest


SPEC = importlib.util.spec_from_file_location("sim_scorecard", Path(__file__).with_name("official-sim-scorecard.py"))
SCORE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SCORE)
APP = "a" * 64


class ScorecardTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.definition = self.artifact("case.json", b'{"id":"positive"}')
        self.fixture = self.artifact("fixture.xml", b"<independently-authored/>")
        self.source = self.artifact("native-output.txt", b"actual native sample")
        self.log = self.artifact("native.log", b"synthetic coverage fixture")

    def artifact(self, name, content):
        (self.root / name).write_bytes(content)
        return {"path": name, "sha256": hashlib.sha256(content).hexdigest()}

    def save(self, name, value):
        path = self.root / name
        path.write_text(json.dumps(value), encoding="utf-8")
        return path

    def run_record(self, name, mode="plain", output=1, status="pass"):
        record = {
            "schema_version": 1, "kind": "official-native-replay", "id": name,
            "application": {"sha256": APP, "version": "synthetic"},
            "case": {
                "id": "positive", "sha256": self.definition["sha256"],
                "fixture_sha256": self.fixture["sha256"], "definition": self.definition,
            },
            "execution": {"run_id": name, "mode": mode, "status": status, "native_running_verified": True},
            "artifacts": [self.definition, self.fixture, self.source, self.log],
            "coverage_logs": [self.log] if mode == "drcov" else [],
            "observations": [{
                "phase": "after_first_calculation", "elapsed_ms": 70000,
                "outputs": {"OL1": output},
                "source": {"kind": "native-readable", "artifacts": [self.source]},
            }],
        }
        return self.save(name + ".json", record)

    def coverage(self, **changes):
        record = {
            "schema_version": 1, "kind": "native-basic-block-coverage",
            "application": {"sha256": APP},
            "measurement": {"raw_log_sha256": self.log["sha256"]},
            "artifacts": [self.log], "scope": {"status": "unknown"},
            "observed": {"target_bb_count": 12, "executed_function_count": 3},
            "coverage": {"basic_block_percentage": None},
        }
        record.update(changes)
        return self.save("coverage.json", record)

    def test_actual_loop_association_never_claims_complete_engine(self):
        runs = [self.run_record("plain1"), self.run_record("plain2"), self.run_record("instrumented", "drcov")]
        result = SCORE.compare(runs, [self.coverage()], self.root)
        self.assertTrue(result["gate"]["loop_established"])
        self.assertFalse(result["gate"]["engine_complete"])
        self.assertIsNone(result["scope"]["native_basic_block_percentage"])
        self.assertEqual(result["cases"][0]["native_coverage_runs"][0]["target_bb_count"], 12)

    def test_incomplete_loop_writes_evidence_but_returns_nonzero(self):
        run = self.run_record("plain")
        output = self.root / "scorecard.json"
        result = subprocess.run([
            sys.executable, "-B", str(Path(SCORE.__file__)), "compare",
            "--receipt", str(run), "--artifact-root", str(self.root), "--output", str(output),
        ], capture_output=True, text=True, check=False)
        self.assertEqual(result.returncode, 1, result.stderr)
        self.assertFalse(json.loads(output.read_text(encoding="utf-8"))["gate"]["loop_established"])

    def test_missing_repeat_or_coverage_cannot_complete_loop(self):
        plain = self.run_record("plain")
        instrumented = self.run_record("instrumented", "drcov")
        result = SCORE.compare([plain, instrumented], [], self.root)
        self.assertFalse(result["gate"]["loop_established"])
        self.assertFalse(result["cases"][0]["repeat_plain"])

    def test_native_replay_agreement_does_not_imply_instrumented_coverage(self):
        runs = [self.run_record("plain1"), self.run_record("plain2")]
        result = SCORE.compare(runs, [], self.root)
        self.assertTrue(result["gate"]["native_replay_established"])
        self.assertFalse(result["gate"]["loop_established"])
        self.run_record("plain2", output=0)
        result = SCORE.compare(runs, [], self.root)
        self.assertFalse(result["gate"]["native_replay_established"])

    def test_output_mismatch_or_blocked_run_stays_incomplete(self):
        for output, status in [(0, "pass"), (1, "blocked")]:
            runs = [self.run_record("plain1"), self.run_record("plain2"),
                    self.run_record("instrumented", "drcov", output, status)]
            result = SCORE.compare(runs, [self.coverage()], self.root)
            self.assertFalse(result["gate"]["loop_established"])

    def test_duplicate_receipt_cannot_count_as_independent_run(self):
        path = self.run_record("one")
        with self.assertRaisesRegex(SCORE.EvidenceError, "duplicate receipt"):
            SCORE.compare([path, path], [], self.root)

    def test_two_admissions_of_one_native_execution_are_not_independent(self):
        first = self.run_record("original")
        second = self.run_record("admission")
        record = json.loads(second.read_text(encoding="utf-8"))
        record["execution"]["run_id"] = "original"
        self.save("admission.json", record)
        with self.assertRaisesRegex(SCORE.EvidenceError, "Duplicate execution run ID"):
            SCORE.compare([first, second], [], self.root)

    def test_stale_application_and_unbound_coverage_rejected_or_incomplete(self):
        runs = [self.run_record("plain1"), self.run_record("plain2"), self.run_record("instrumented", "drcov")]
        with self.assertRaisesRegex(SCORE.EvidenceError, "different application"):
            SCORE.compare(runs, [self.coverage(application={"sha256": "b" * 64})], self.root)
        other_log = self.artifact("other.log", b"separate actual coverage fixture")
        unbound = self.coverage(measurement={"raw_log_sha256": other_log["sha256"]}, artifacts=[other_log])
        self.assertFalse(SCORE.compare(runs, [unbound], self.root)["gate"]["loop_established"])

    def test_analysis_cannot_name_an_unverified_raw_log(self):
        runs = [self.run_record("plain")]
        with self.assertRaisesRegex(SCORE.EvidenceError, "raw-log identity"):
            SCORE.compare(runs, [self.coverage(measurement={"raw_log_sha256": "c" * 64})], self.root)

    def test_tampered_or_escaped_native_evidence_is_not_valid(self):
        path = self.run_record("run")
        (self.root / self.source["path"]).write_text("changed", encoding="utf-8")
        with self.assertRaisesRegex(SCORE.EvidenceError, "hash mismatch"):
            SCORE.receipt(path, self.root)
        escaped = copy.deepcopy(self.source)
        escaped["path"] = "..\\outside.txt"
        with self.assertRaisesRegex(SCORE.EvidenceError, "escape"):
            SCORE.check_artifact(escaped, self.root)

    def test_source_can_name_an_already_hash_bound_manifest_artifact(self):
        path = self.run_record("run")
        record = json.loads(path.read_text(encoding="utf-8"))
        record["observations"][0]["source"]["artifacts"] = [self.source["path"]]
        self.save("run.json", record)
        SCORE.receipt(path, self.root)
        record["observations"][0]["source"]["artifacts"] = ["unbound-source.png"]
        self.save("run.json", record)
        with self.assertRaisesRegex(SCORE.EvidenceError, "not bound"):
            SCORE.receipt(path, self.root)

    def test_coverage_log_can_name_a_hash_bound_manifest_artifact(self):
        paths = [self.run_record("plain1"), self.run_record("plain2"), self.run_record("instrumented", "drcov")]
        record = json.loads(paths[-1].read_text(encoding="utf-8"))
        record["coverage_logs"] = [self.log["path"]]
        self.save("instrumented.json", record)
        result = SCORE.compare(paths, [self.coverage()], self.root)
        self.assertTrue(result["gate"]["loop_established"])
        record["coverage_logs"] = ["unbound.log"]
        self.save("instrumented.json", record)
        with self.assertRaisesRegex(SCORE.EvidenceError, "not bound"):
            SCORE.receipt(paths[-1], self.root)

    def test_passing_without_observation_or_case_bytes_is_invalid(self):
        path = self.run_record("run")
        record = json.loads(path.read_text(encoding="utf-8"))
        record["observations"] = []
        self.save("run.json", record)
        with self.assertRaisesRegex(SCORE.EvidenceError, "actual native execution"):
            SCORE.receipt(path, self.root)
        record["observations"] = [{"phase": "sample", "elapsed_ms": 1, "outputs": {"OL1": 10**400},
                                   "source": {"kind": "native-readable", "artifacts": [self.source]}}]
        self.save("run.json", record)
        with self.assertRaisesRegex(SCORE.EvidenceError, "finite scalar"):
            SCORE.receipt(path, self.root)

    def test_unknown_scope_cannot_claim_percentage(self):
        runs = [self.run_record("plain")]
        with self.assertRaisesRegex(SCORE.EvidenceError, "Unknown static scope"):
            SCORE.compare(runs, [self.coverage(coverage={"basic_block_percentage": 100})], self.root)

    def test_baseline_separates_export_from_simulator_scope(self):
        index = {
            "target_sha256": APP, "provider": {"id": "synthetic"},
            "functions": [
                {"status": "written", "body": {"available": True}},
                {"status": "written", "body": {"available": True}},
                {"status": "failed", "body": {"available": False}},
                {"status": "external", "body": {"available": False}},
            ],
        }
        index_path = self.save("index.json", index)
        database = self.root / "research.sqlite3"
        with closing(sqlite3.connect(database)) as db:
            db.execute("CREATE TABLE functions(body TEXT)")
            db.executemany("INSERT INTO functions VALUES(?)", [
                ("synthetic body",), ("/* WARNING: Could not recover jumptable */",),
            ])
            db.commit()
        latest = {
            "binaries": {"LoxoneConfig.exe": {
                "export": {"index_path": str(index_path), "target_sha256": APP},
                "database": str(database),
            }},
        }
        result = SCORE.baseline(self.save("latest.json", latest))
        self.assertEqual(result["export"]["body_export_fraction"], 2 / 3)
        self.assertEqual(result["quality_flags"]["bodies_with_warning_comments"], 1)
        self.assertFalse(result["gate"]["engine_complete"])
        self.assertIsNone(result["simulator_scope"]["basic_block_denominator"])


if __name__ == "__main__":
    unittest.main()
