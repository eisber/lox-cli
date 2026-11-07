"""Mocked execution records only; no test in this module runs or validates Loxone."""

import contextlib
import copy
import importlib.util
import io
import json
from pathlib import Path
import shutil
import unittest
from unittest.mock import patch
import uuid


SPEC = importlib.util.spec_from_file_location(
    "verify_ux_contracts", Path(__file__).with_name("verify-ux-contracts.py")
)
GATES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATES)
SNAPSHOT_SPEC = importlib.util.spec_from_file_location(
    "snapshot_energy_cli_contract", Path(__file__).with_name("snapshot-energy-cli-contract.py")
)
SNAPSHOT = importlib.util.module_from_spec(SNAPSHOT_SPEC)
SNAPSHOT_SPEC.loader.exec_module(SNAPSHOT)


class SyntheticGateTests(unittest.TestCase):
    """Even positive gate tests use invented observations, not real GUI evidence."""

    def setUp(self):
        self.root = GATES.ROOT / ".reverse-engineering" / ("gate-test-" + uuid.uuid4().hex)
        self.root.mkdir(parents=True)
        self.addCleanup(shutil.rmtree, self.root)
        self.artifact = self.root / "observed.txt"
        self.artifact.write_text("SYNTHETIC observation; no Config execution.", encoding="utf-8")
        self.ref = {"path": "observed.txt", "sha256": GATES.digest(self.artifact)}
        self.record_path = self.root / "runner.json"
        self.contract_path = self.root / "contract.json"
        self.results_path = self.root / "results.json"
        self.contract = {
            "schema_version": 1, "application": GATES.APPLICATION.copy(),
            "block_type": "Synthetic", "control_type": 0,
            "scope": {"status": "complete", "description": "Synthetic scope, not a real block.",
                      "source_evidence": []},
            "capabilities": [{
                "id": "scalar", "kind": "parameter",
                "binding": {"status": "known", "path": "Synthetic/@Value"},
                "source_evidence": [{"evidence_kind": "catalog", "artifacts": [self.ref]}],
                "required_case_ids": ["roundtrip"],
            }],
            "cases": [{
                "id": "roundtrip", "capability_ids": ["scalar"], "condition": "Changed scalar",
                "evidence_kind": "gui_oracle", "procedure": "Synthetic positive-gate input.",
                "expected": "Synthetic expected outcome.",
            }],
        }
        self.run = {
            "id": "synthetic-test-run", "runner": "unittest fabricated fixture",
            "started_at": "2026-11-05T07:00:00+00:00",
            "finished_at": "2026-11-05T07:01:00+00:00",
            "application": GATES.APPLICATION.copy(), "synthetic": False,
        }
        self.observations = [{
            "id": "roundtrip", "outcome": "pass", "evidence_kind": "gui_oracle",
            "actual": "SYNTHETIC pass fixture, not real Loxone validation.",
            "artifacts": [self.ref],
        }]
        self.record_kind = "gui_observation"
        self.operation = "Fabricated record for verifier tests; GUI never executed."

    def write(self, path, value):
        path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")

    def materialize(self):
        record = {
            "schema_version": 1, **self.run, "kind": self.record_kind,
            "operation": self.operation,
            "observations": self.observations,
        }
        self.write(self.record_path, record)
        record_ref = {"path": "runner.json", "sha256": GATES.digest(self.record_path)}
        if not self.contract["scope"]["source_evidence"]:
            self.contract["scope"]["source_evidence"] = [
                {"evidence_kind": "gui_oracle", "artifacts": [record_ref, self.ref]},
            ]
        self.write(self.contract_path, self.contract)
        result = {
            "schema_version": 1, "contract_sha256": GATES.digest(self.contract_path),
            **{key: self.contract[key] for key in ("application", "block_type", "control_type")},
            "run": {**self.run, "record": record_ref}, "cases": self.observations,
        }
        self.write(self.results_path, result)
        return result

    def verify(self):
        return GATES.verify_files(self.contract_path, self.results_path, self.root)

    def add_cli_proofs(self):
        """Construct distinct mocked CLI/unit records; never relabel GUI evidence."""
        for kind in ("implementation", "unit_test"):
            case_id = f"scalar.{kind}"
            path = self.root / f"{kind}.json"
            self.write(path, {"mocked": True, "kind": kind, "real_execution": False})
            artifact = {"path": path.name, "sha256": GATES.digest(path)}
            self.contract["cases"].append({
                "id": case_id, "capability_ids": ["scalar"],
                "condition": f"Distinct {kind} condition",
                "evidence_kind": kind, "procedure": f"Mocked {kind} procedure for gate tests.",
                "expected": f"Mocked {kind} expected outcome.",
            })
            self.contract["capabilities"][0]["required_case_ids"].append(case_id)
            self.observations.append({
                "id": case_id, "outcome": "pass", "evidence_kind": kind,
                "actual": f"Mocked {kind} outcome; no CLI/tests actually executed.",
                "artifacts": [artifact],
            })

    def assert_incomplete(self, code=None, invalid=None):
        report = self.verify()
        self.assertFalse(report["gate"]["complete"], report)
        if code:
            self.assertIn(code, [row["code"] for row in report["diagnostics"]])
        if invalid is not None:
            self.assertEqual(report["gate"]["input_valid"], not invalid)
        return report

    def test_positive_gate_is_only_a_synthetic_format_test(self):
        self.add_cli_proofs()
        self.materialize()
        report = self.verify()
        self.assertTrue(report["gate"]["complete"], report["diagnostics"])
        self.assertEqual(report["stages"]["gui_oracle"]["verified"], 1)
        self.assertEqual(report["overall_percentage"], 100)
        self.assertEqual(report["cases"][0]["condition"], "Changed scalar")
        self.assertEqual(report["stages"]["implementation"]["verified"], 1)
        self.assertEqual(report["stages"]["unit_test"]["verified"], 1)
        self.assertEqual(report["capabilities"][0]["missing_required_evidence_levels"], [])

    def test_valid_real_labeled_gui_only_evidence_cannot_complete_cli_parity(self):
        self.materialize()
        report = self.assert_incomplete("required_implementation_case_missing", invalid=False)
        self.assertTrue(report["scope_evidence_verified"])
        self.assertTrue(report["capabilities"][0]["stages"]["binding_known"])
        self.assertEqual(report["stages"]["gui_oracle"]["verified"], 1)
        self.assertEqual(report["stages"]["implementation"]["verified"], 0)
        self.assertEqual(report["stages"]["unit_test"]["verified"], 0)
        self.assertEqual(report["stages"]["complete"]["verified"], 0)
        self.assertEqual(report["overall_percentage"], 0)
        self.assertEqual(report["capabilities"][0]["missing_required_evidence_levels"],
                         ["implementation", "unit_test"])
        self.assertIn("required_unit_test_case_missing",
                      [row["code"] for row in report["diagnostics"]])
        self.assertEqual(self.cli()[0], 1)
        self.assertEqual(self.cli(["--report"])[0], 0)

    def test_passed_optional_cli_levels_cannot_replace_required_levels(self):
        self.add_cli_proofs()
        self.contract["capabilities"][0]["required_case_ids"] = ["roundtrip"]
        self.materialize()
        report = self.assert_incomplete("required_unit_test_case_missing", invalid=False)
        for kind in GATES.REQUIRED_LEVELS:
            self.assertEqual(report["stages"][kind]["verified"], 1)
        self.assertEqual(report["stages"]["complete"]["verified"], 0)

    def test_every_evidence_level_must_be_a_required_case(self):
        self.add_cli_proofs()
        required = self.contract["capabilities"][0]["required_case_ids"]
        original = required.copy()
        for kind in GATES.REQUIRED_LEVELS:
            with self.subTest(kind=kind):
                omitted = {case["id"] for case in self.contract["cases"]
                           if case["evidence_kind"] == kind}
                required[:] = [case_id for case_id in original if case_id not in omitted]
                self.contract["scope"]["source_evidence"] = []
                self.materialize()
                report = self.assert_incomplete(f"required_{kind}_case_missing", invalid=False)
                self.assertEqual(report["capabilities"][0]["missing_required_evidence_levels"], [kind])

    def test_required_cli_fail_and_not_run_preserve_passed_gui_stage(self):
        self.add_cli_proofs()
        for outcome in ("fail", "not_run"):
            with self.subTest(outcome=outcome):
                unit = next(row for row in self.observations if row["evidence_kind"] == "unit_test")
                unit["outcome"] = outcome
                unit["actual"] = f"Mocked unit-test {outcome}"
                self.contract["scope"]["source_evidence"] = []
                self.materialize()
                report = self.assert_incomplete("required_case_unverified", invalid=False)
                self.assertEqual(report["stages"]["gui_oracle"]["verified"], 1)
                self.assertEqual(report["stages"]["implementation"]["verified"], 1)
                self.assertEqual(report["stages"]["unit_test"]["verified"], 0)
                self.assertEqual(report["stages"]["complete"]["verified"], 0)

    def test_empty_denominator_never_claims_one_hundred_percent(self):
        self.contract["capabilities"] = []
        self.contract["cases"] = []
        self.observations = []
        self.materialize()
        report = self.assert_incomplete("empty_denominator")
        self.assertIsNone(report["overall_percentage"])
        self.assertIsNone(report["stages"]["complete"]["percentage"])

    def test_unknown_scope_keeps_percentage_unknown_with_passed_narrow_cases(self):
        self.contract["scope"]["status"] = "unknown"
        self.materialize()
        report = self.assert_incomplete("scope_unverified", invalid=False)
        self.assertIsNone(report["overall_percentage"])
        self.assertTrue(report["cases"][0]["verified"])

    def test_catalog_scope_cannot_establish_gui_completeness(self):
        self.contract["scope"]["source_evidence"] = [
            {"evidence_kind": "catalog", "artifacts": [self.ref]},
        ]
        self.materialize()
        self.assert_incomplete("scope_unverified")

    def test_unknown_and_ambiguous_bindings_preserve_unknown_semantics(self):
        for status in ("unknown", "ambiguous"):
            with self.subTest(status=status):
                self.contract["capabilities"][0]["binding"] = {"status": status}
                self.materialize()
                report = self.assert_incomplete("binding_unverified", invalid=False)
                self.assertNotIn("path", report["capabilities"][0]["binding"])

    def test_binding_status_and_path_must_agree(self):
        for binding in ({"status": "known"}, {"status": "unknown", "path": "Value"}):
            with self.subTest(binding=binding):
                self.contract["capabilities"][0]["binding"] = binding
                self.materialize()
                self.assert_incomplete("binding_invalid", invalid=True)

    def test_required_missing_fail_and_not_run_are_separate(self):
        for outcome in ("missing", "fail", "not_run"):
            with self.subTest(outcome=outcome):
                self.observations = [] if outcome == "missing" else [{
                    "id": "roundtrip", "outcome": outcome, "evidence_kind": "gui_oracle",
                    "actual": "Synthetic failure/not run", "artifacts": [],
                }]
                self.contract["scope"]["source_evidence"] = []
                self.materialize()
                report = self.assert_incomplete("required_case_unverified", invalid=False)
                self.assertEqual(report["cases"][0]["outcome"], outcome)

    def test_unit_pass_cannot_substitute_for_required_gui_case(self):
        self.observations[0]["evidence_kind"] = "unit_test"
        self.materialize()
        report = self.assert_incomplete("evidence_kind_mismatch", invalid=True)
        self.assertFalse(report["cases"][0]["verified"])

    def test_unit_only_requirements_do_not_establish_parity(self):
        self.contract["cases"][0]["evidence_kind"] = "unit_test"
        self.observations[0]["evidence_kind"] = "unit_test"
        self.record_kind = "unit_test"
        self.materialize()
        report = self.assert_incomplete("gui_requirement_missing")
        self.assertEqual(report["stages"]["unit_test"]["verified"], 1)
        self.assertEqual(report["stages"]["gui_oracle"]["verified"], 0)

    def test_catalog_and_native_source_stages_do_not_count_as_gui(self):
        self.contract["capabilities"][0]["source_evidence"].append({
            "evidence_kind": "native", "artifacts": [self.ref],
        })
        self.observations = []
        self.materialize()
        report = self.assert_incomplete()
        self.assertEqual(report["stages"]["catalog"]["verified"], 1)
        self.assertEqual(report["stages"]["native"]["verified"], 1)
        self.assertEqual(report["stages"]["gui_oracle"]["verified"], 0)

    def test_synthetic_runner_is_never_real_application_evidence(self):
        self.run["synthetic"] = True
        self.materialize()
        report = self.assert_incomplete("synthetic_run", invalid=False)
        self.assertEqual(report["stages"]["gui_oracle"]["verified"], 0)

    def test_real_gui_record_shape_allows_safe_synthetic_or_public_input(self):
        """Model the producer's real-execution shape; this test still mocks the GUI."""
        for origin in ("generated synthetic project", "disposable public-sample project"):
            with self.subTest(origin=origin):
                self.operation = f"Official Config GUI save/reopen on a safe {origin}."
                self.observations[0]["actual"] = f"Observed roundtrip using safe {origin}."
                self.run["synthetic"] = False
                self.contract["scope"]["source_evidence"] = []
                self.materialize()
                report = self.verify()
                self.assertTrue(report["gate"]["input_valid"], report["diagnostics"])
                self.assertFalse(report["gate"]["complete"])
                self.assertEqual(report["stages"]["gui_oracle"]["verified"], 1)

    def test_mock_execution_on_safe_synthetic_input_is_not_gui_evidence(self):
        self.operation = "Mocked official GUI run on a safe generated synthetic project."
        self.run["synthetic"] = True
        self.materialize()
        report = self.assert_incomplete("synthetic_run", invalid=False)
        self.assertFalse(report["cases"][0]["verified"])

    def test_hashes_without_actual_runner_record_are_insufficient(self):
        self.materialize()
        self.record_path.unlink()
        self.assert_incomplete("artifact_invalid", invalid=True)

    def test_wrong_runner_kind_cannot_claim_gui_observation(self):
        self.record_kind = "cli_execution"
        self.materialize()
        self.assert_incomplete("runner_invalid", invalid=True)

    def test_runner_observation_must_match_claimed_actual_outcome(self):
        result = self.materialize()
        result["cases"] = copy.deepcopy(result["cases"])
        result["cases"][0]["actual"] = "Invented different outcome"
        self.write(self.results_path, result)
        self.assert_incomplete("runner_invalid", invalid=True)

    def test_runner_observation_outcome_must_match(self):
        result = self.materialize()
        result["cases"] = copy.deepcopy(result["cases"])
        result["cases"][0]["outcome"] = "fail"
        self.write(self.results_path, result)
        self.assert_incomplete("runner_invalid", invalid=True)

    def test_bad_artifact_digest_is_rejected(self):
        self.materialize()
        self.artifact.write_text("Changed evidence", encoding="utf-8")
        self.assert_incomplete("artifact_invalid", invalid=True)

    def test_pass_without_artifact_is_invalid(self):
        self.observations[0]["artifacts"] = []
        self.materialize()
        self.assert_incomplete("pass_without_artifacts", invalid=True)

    def test_bad_artifact_paths_cannot_escape_root(self):
        for path in ("..\\outside", "E:\\outside", "/outside", "a/../../outside", "a:stream"):
            with self.subTest(path=path):
                self.observations[0]["artifacts"] = [{"path": path, "sha256": self.ref["sha256"]}]
                self.contract["scope"]["source_evidence"] = []
                self.materialize()
                self.assert_incomplete("artifact_invalid", invalid=True)

    def test_backslash_artifact_path_is_supported(self):
        self.add_cli_proofs()
        nested = self.root / "nested"
        nested.mkdir()
        nested_artifact = nested / "observed.txt"
        shutil.copyfile(self.artifact, nested_artifact)
        self.ref["path"] = "nested\\observed.txt"
        self.materialize()
        self.assertTrue(self.verify()["gate"]["complete"])

    def test_duplicate_and_conflicting_artifact_paths_are_invalid(self):
        self.observations[0]["artifacts"] = [self.ref, self.ref.copy()]
        self.materialize()
        self.assert_incomplete("artifact_invalid", invalid=True)

    def test_stale_application_version_and_bytes_are_rejected(self):
        for field, value in (("version", "17.0"), ("sha256", "a" * 64)):
            with self.subTest(field=field):
                self.contract["application"] = GATES.APPLICATION.copy()
                self.contract["application"][field] = value
                self.contract["scope"]["source_evidence"] = []
                self.materialize()
                self.assert_incomplete("application_stale", invalid=True)

    def test_result_block_and_runner_identities_are_checked(self):
        for field, value, code in (
                ("block_type", "Other", "result_identity_mismatch"),
                ("control_type", 1, "result_identity_mismatch"),
                ("application", {**GATES.APPLICATION, "version": "old"}, "result_identity_mismatch")):
            with self.subTest(field=field):
                result = self.materialize()
                result[field] = value
                self.write(self.results_path, result)
                self.assert_incomplete(code, invalid=True)

    def test_runner_binary_identity_is_checked(self):
        self.run["application"]["sha256"] = "b" * 64
        self.materialize()
        self.assert_incomplete("runner_application_stale", invalid=True)

    def test_stale_contract_bytes_are_rejected(self):
        self.materialize()
        self.contract_path.write_text(self.contract_path.read_text() + "\n", encoding="utf-8")
        self.assert_incomplete("contract_stale", invalid=True)

    def test_duplicate_capability_case_and_result_ids_are_invalid(self):
        for field in ("capabilities", "cases", "observations"):
            with self.subTest(field=field):
                rows = self.observations if field == "observations" else self.contract[field]
                rows.append(copy.deepcopy(rows[0]))
                self.materialize()
                report = self.assert_incomplete("duplicate_id", invalid=True)
                self.assertIsNone(report["overall_percentage"])
                self.assertEqual(report["stages"]["complete"]["verified"], 0)
                rows.pop()
                self.contract["scope"]["source_evidence"] = []

    def test_case_capability_links_must_exist_and_be_reciprocal(self):
        self.contract["cases"][0]["capability_ids"] = ["missing"]
        self.materialize()
        report = self.assert_incomplete("unknown_capability", invalid=True)
        self.assertIn("required_case_invalid", [row["code"] for row in report["diagnostics"]])

    def test_missing_required_case_definition_is_invalid(self):
        self.contract["capabilities"][0]["required_case_ids"] = ["undefined"]
        self.materialize()
        self.assert_incomplete("required_case_invalid", invalid=True)

    def test_unexpected_result_case_is_invalid(self):
        self.observations.append({**self.observations[0], "id": "unknown"})
        self.materialize()
        self.assert_incomplete("unexpected_case", invalid=True)

    def test_unexpected_fields_types_nulls_and_invalid_enums_are_rejected(self):
        original = copy.deepcopy(self.contract)
        mutations = [
            ("control_type", True), ("control_type", "0"), ("control_type", None),
            ("schema_version", 2), ("schema_version", True),
            ("unknown", 1), ("block_type", ""), ("block_type", " "),
            ("capabilities", {}), ("application", {"version": "v", "sha256": "not-a-hash"}),
        ]
        for field, value in mutations:
            with self.subTest(field=field, value=value):
                self.contract = copy.deepcopy(original)
                self.contract[field] = value
                self.materialize()
                self.assert_incomplete("input_invalid", invalid=True)
        self.contract = original
        self.contract["capabilities"][0]["kind"] = "metadata"
        self.materialize()
        self.assert_incomplete("input_invalid", invalid=True)

    def test_duplicate_json_keys_and_nonfinite_numbers_are_rejected(self):
        self.materialize()
        for payload in ('{"schema_version":1,"schema_version":1}', '{"x":NaN}', '{"x":Infinity}'):
            with self.subTest(payload=payload):
                self.contract_path.write_text(payload, encoding="utf-8")
                self.assert_incomplete("input_invalid", invalid=True)

    def test_timestamps_must_be_timezone_aware_and_ordered(self):
        for value in ("2026-11-05T06:00:00+00:00", "2026-11-05T07:01:00", "not-a-date"):
            with self.subTest(value=value):
                self.run["finished_at"] = value
                self.contract["scope"]["source_evidence"] = []
                self.materialize()
                self.assert_incomplete("runner_invalid", invalid=True)

    def test_gui_source_needs_record_and_case_artifact(self):
        self.contract["scope"]["source_evidence"] = [
            {"evidence_kind": "gui_oracle", "artifacts": [self.ref]},
        ]
        self.materialize()
        self.assert_incomplete("gui_source_unverified")

    def test_optional_unverified_gui_condition_prevents_complete_claim(self):
        self.contract["cases"].append({
            **self.contract["cases"][0], "id": "boundary", "condition": "Invalid boundary input",
        })
        self.materialize()
        report = self.assert_incomplete()
        self.assertEqual(report["stages"]["complete"]["verified"], 0)

    def test_symlink_artifact_escape_is_rejected_without_reading_target(self):
        link = self.root / "escape"
        try:
            link.symlink_to(GATES.ROOT, target_is_directory=True)
        except OSError:
            self.skipTest("Host does not grant symlink creation")
        with self.assertRaisesRegex(ValueError, "escapes root"):
            GATES._artifact_path(self.root, "escape\\AGENTS.md")

    def test_no_results_remains_honest_partial(self):
        self.materialize()
        report = GATES.verify_files(self.contract_path, root=self.root)
        self.assertFalse(report["gate"]["complete"])
        self.assertEqual(report["cases"][0]["outcome"], "missing")

    def cli(self, extra=None):
        args = ["--root", str(self.root), "--contract", str(self.contract_path),
                "--results", str(self.results_path)] + (extra or [])
        output = io.StringIO()
        with contextlib.redirect_stdout(output):
            code = GATES.main(args)
        return code, json.loads(output.getvalue())

    def test_strict_and_report_exit_statuses(self):
        self.add_cli_proofs()
        self.materialize()
        self.assertEqual(self.cli()[0], 0)
        self.contract["scope"]["status"] = "incomplete"
        self.materialize()
        self.assertEqual(self.cli()[0], 1)
        self.assertEqual(self.cli(["--report"])[0], 0)
        self.artifact.write_text("bad digest")
        self.assertEqual(self.cli(["--report"])[0], 2)

    def test_cli_duplicate_contracts_and_results_fail(self):
        self.materialize()
        self.assertEqual(self.cli(["--contract", str(self.contract_path)])[0], 2)
        self.assertEqual(self.cli(["--results", str(self.results_path)])[0], 2)

    def test_cli_unmatched_stale_result_fails_even_in_report_mode(self):
        result = self.materialize()
        result["contract_sha256"] = "c" * 64
        self.write(self.results_path, result)
        self.assertEqual(self.cli(["--report"])[0], 2)

    def test_inventory_adds_evidence_without_promoting_metadata(self):
        self.add_cli_proofs()
        self.materialize()
        report = self.verify()
        inventory = {
            "source": {"executable_sha256": GATES.APPLICATION["sha256"]},
            "blocks": [
                {"xml_type": "Synthetic", "control_type": "0", "ux_parity": {"overall": "unverified"}},
                {"xml_type": "Unknown", "control_type": "1", "ux_parity": {"overall": "unverified"}},
            ],
        }
        enriched = GATES.enrich_inventory(inventory, [report])
        self.assertEqual(enriched["summary"]["gui_parity_verified"], 1)
        self.assertEqual(enriched["blocks"][0]["verified_evidence"]["status"], "complete")
        self.assertEqual(enriched["blocks"][0]["ux_parity"]["overall"], "unverified")
        self.assertEqual(enriched["blocks"][1]["verified_evidence"]["status"], "unverified")
        self.assertNotIn("verified_evidence", inventory["blocks"][0])
        self.assertIsNone(enriched["evidence_summary"]["percentage"])

    def test_ambiguous_inventory_identity_remains_unverified(self):
        self.materialize()
        inventory = {
            "source": {"executable_sha256": GATES.APPLICATION["sha256"]},
            "blocks": [{"xml_type": "Synthetic", "control_type": "0"}] * 2,
        }
        enriched = GATES.enrich_inventory(inventory, [self.verify()])
        self.assertEqual(enriched["summary"]["gui_parity_verified"], 0)
        self.assertEqual(enriched["evidence_summary"]["unmatched_contracts"], 1)
        self.assertFalse(enriched["evidence_summary"]["complete"])
        self.assertTrue(enriched["evidence_summary"]["diagnostics"])

    def test_strict_inventory_gate_rejects_unmatched_supplied_contract(self):
        self.materialize()
        path = self.root / "inventory.json"
        self.write(path, {"source": {"executable_sha256": GATES.APPLICATION["sha256"]},
                         "blocks": [{"xml_type": "Unknown", "control_type": "1"}]})
        code, output = self.cli(["--inventory", str(path)])
        self.assertEqual(code, 1)
        self.assertFalse(output["gate"]["complete"])
        self.assertEqual(self.cli(["--inventory", str(path), "--report"])[0], 0)

    def test_invalid_inventory_identity_types_and_source_fail(self):
        self.materialize()
        for inventory in (
                {"source": {"executable_sha256": GATES.APPLICATION["sha256"]},
                 "blocks": [{"xml_type": 0, "control_type": "0"}]},
                {"source": {"executable_sha256": GATES.APPLICATION["sha256"]},
                 "blocks": [{"xml_type": "Synthetic", "control_type": True}]},
                {"source": {"executable_sha256": "invalid"}, "blocks": []},
                {"source": {"executable_sha256": GATES.APPLICATION["sha256"]},
                 "blocks": [], "summary": []}):
            with self.subTest(inventory=inventory), self.assertRaises(ValueError):
                GATES.enrich_inventory(inventory, [self.verify()])

    def test_stale_inventory_executable_is_rejected(self):
        self.materialize()
        inventory = {"source": {"executable_sha256": "a" * 64},
                     "blocks": [{"xml_type": "Synthetic", "control_type": "0"}]}
        with self.assertRaisesRegex(ValueError, "identities differ"):
            GATES.enrich_inventory(inventory, [self.verify()])

    def test_published_example_is_partial_and_not_real_evidence(self):
        report = GATES.verify_files(GATES.SCHEMAS / "synthetic-example.json", root=self.root)
        self.assertTrue(report["gate"]["input_valid"])
        self.assertFalse(report["gate"]["complete"])
        self.assertIsNone(report["overall_percentage"])

    def test_published_not_run_record_is_valid_but_never_application_evidence(self):
        contract_path = GATES.SCHEMAS / "synthetic-example.json"
        runner_path = GATES.SCHEMAS / "synthetic-runner-record.example.json"
        contract = GATES.load_json(contract_path)
        record = GATES.load_json(runner_path)
        GATES.validate_document(record, "ux-runner-record.schema.json")
        self.write(self.results_path, {
            "schema_version": 1, "contract_sha256": GATES.digest(contract_path),
            **{key: contract[key] for key in ("application", "block_type", "control_type")},
            "run": {
                **{key: record[key] for key in (
                    "id", "runner", "started_at", "finished_at", "application", "synthetic")},
                "record": {"path": "contracts\\synthetic-runner-record.example.json",
                           "sha256": GATES.digest(runner_path)},
            },
            "cases": record["observations"],
        })
        report = GATES.verify_files(contract_path, self.results_path, GATES.ROOT)
        self.assertTrue(report["gate"]["input_valid"], report["diagnostics"])
        self.assertFalse(report["gate"]["complete"])
        self.assertEqual(report["case_counts"]["not_run"], 1)
        self.assertEqual(report["case_counts"]["verified"], 0)

    def test_verifier_never_silently_ignores_new_schema_rules(self):
        with self.assertRaisesRegex(ValueError, "unsupported schema vocabulary"):
            GATES._schema_check("example", {"type": "string", "maxLength": 1},
                                "ux-contract.schema.json")

    def prepare_snapshot_inputs(self):
        original = copy.deepcopy(self.contract)
        originals = self.root / "contracts"
        originals.mkdir()
        original_path = originals / "energy-manager2.json"
        self.write(original_path, original)
        self.add_cli_proofs()
        self.observations = [row for row in self.observations if row["evidence_kind"] != "gui_oracle"]
        self.record_kind = "cli_execution"
        self.materialize()
        run = self.root / "run"
        run.mkdir()
        shutil.copyfile(self.record_path, run / "runner-record.json")
        definitions = {
            **{key: original[key] for key in ("application", "block_type", "control_type")},
            "capability_mappings": [{
                "capability_id": "scalar", "condition_scope": "Mocked narrow scalar operation.",
                "omissions": ["GUI behavior", "uncovered bounds"],
                "required_case_ids": ["scalar.implementation", "scalar.unit_test"],
            }],
            "cases": [case for case in self.contract["cases"] if case["evidence_kind"] != "gui_oracle"],
        }
        self.write(run / "definitions.json", definitions)
        self.write(run / "metadata.json", {
            "case_definitions": {"path": "run\\definitions.json", "sha256": GATES.digest(run / "definitions.json")},
            "source_artifacts": [],
        })
        return run, original_path, definitions

    def test_cli_snapshot_preserves_actual_observations_and_missing_gui_definitions(self):
        run, original_path, definitions = self.prepare_snapshot_inputs()
        original_digest = GATES.digest(original_path)
        record_digest = GATES.digest(run / "runner-record.json")
        contract_path, result_path = self.root / "derived.json", self.root / "derived-results.json"
        with patch.object(SNAPSHOT, "ROOT", self.root):
            report = SNAPSHOT.derive(run, contract_path, result_path)
        contract, result = GATES.load_json(contract_path), GATES.load_json(result_path)
        self.assertEqual(contract["scope"]["status"], "incomplete")
        self.assertEqual(contract["cases"][:2], definitions["cases"])
        self.assertEqual(contract["cases"][2]["id"], "roundtrip")
        self.assertEqual(result["cases"], GATES.load_json(run / "runner-record.json")["observations"])
        self.assertEqual(result["contract_sha256"], GATES.digest(contract_path))
        self.assertTrue(report["gate"]["input_valid"])
        self.assertFalse(report["gate"]["complete"])
        self.assertEqual(report["case_counts"]["missing"], 1)
        self.assertEqual(report["stages"]["implementation"]["verified"], 1)
        self.assertEqual(report["stages"]["unit_test"]["verified"], 1)
        self.assertEqual(report["stages"]["gui_oracle"]["verified"], 0)
        self.assertEqual(report["stages"]["complete"]["verified"], 0)
        self.assertEqual(GATES.digest(original_path), original_digest)
        self.assertEqual(GATES.digest(run / "runner-record.json"), record_digest)

    def test_cli_snapshot_rejects_stale_definitions_without_writing_outputs(self):
        run, _, _ = self.prepare_snapshot_inputs()
        with (run / "definitions.json").open("a", encoding="utf-8") as stream:
            stream.write("\n")
        contract_path, result_path = self.root / "derived.json", self.root / "derived-results.json"
        with patch.object(SNAPSHOT, "ROOT", self.root), self.assertRaisesRegex(ValueError, "SHA256 mismatch"):
            SNAPSHOT.derive(run, contract_path, result_path)
        self.assertFalse(contract_path.exists())
        self.assertFalse(result_path.exists())

    def test_cli_snapshot_rejects_duplicate_runner_artifacts_without_rewriting_record(self):
        run, _, _ = self.prepare_snapshot_inputs()
        path = run / "runner-record.json"
        record = GATES.load_json(path)
        record["observations"][0]["artifacts"].append(record["observations"][0]["artifacts"][0])
        self.write(path, record)
        before = GATES.digest(path)
        contract_path, result_path = self.root / "derived.json", self.root / "derived-results.json"
        with patch.object(SNAPSHOT, "ROOT", self.root), self.assertRaisesRegex(ValueError, "duplicate"):
            SNAPSHOT.derive(run, contract_path, result_path)
        self.assertEqual(GATES.digest(path), before)
        self.assertFalse(contract_path.exists())
        self.assertFalse(result_path.exists())


if __name__ == "__main__":
    unittest.main()
