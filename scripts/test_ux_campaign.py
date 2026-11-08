"""File-only queue contract tests. Test envelopes are NOT native UX evidence."""

import copy
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timedelta, timezone
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import unittest
from unittest.mock import patch
import uuid


sys.path.insert(0, str(Path(__file__).parent))
SPEC = importlib.util.spec_from_file_location("ux_campaign", Path(__file__).with_name("ux-campaign.py"))
UX = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(UX)


class CatalogTests(unittest.TestCase):
    def test_exact_concrete_versioned_mix_and_reproducibility(self):
        catalog = UX.validate_catalog(UX.read_json(UX.CATALOG))
        self.assertEqual(catalog, UX.generate_catalog())
        cases = catalog["cases"]
        self.assertEqual(len({case["definition_sha256"] for case in cases}), 100)
        self.assertEqual([case["phase"] for case in cases], ["configuration"] * 70 + ["simulator"] * 30)
        self.assertEqual(sum(case["executor_compatible"] for case in cases), 31)
        self.assertEqual(len({case["source"]["path"] for case in cases if case["phase"] == "simulator"}), 5)
        deadlines = [case for case in cases if case["id"].startswith("config-gui-deadline-")
                     and case["id"] != "config-gui-deadline-invalid-selector"]
        self.assertEqual([case["procedure"][2]["index"] for case in deadlines], list(range(28)))
        self.assertEqual(next(case for case in cases if case["id"] == "config-gui-minrt-int-overflow")
                         ["procedure"][2]["value"], "2147483648")
        for case in cases:
            with self.subTest(case=case["id"]):
                self.assertTrue(case["source"]["sha256"])
                self.assertTrue(case["procedure"] or case["source_case"])
                self.assertTrue(case["field_bindings"])
                self.assertTrue(case["input_binding"])
                if not case["executor_compatible"]:
                    self.assertTrue(case["missing_adapter"])
        rendered = json.dumps(catalog)
        self.assertNotIn("marcozo", rendered)
        self.assertNotIn("mcp_url", rendered)
        self.assertIsNone(catalog["whole_product_behavior_percentage"])

    def test_changed_definition_or_promoted_missing_adapter_fails(self):
        for mutation in ("definition", "adapter"):
            catalog = UX.generate_catalog()
            case = catalog["cases"][1]
            if mutation == "definition":
                case["procedure"] = [{"op": "keys", "value": "invented"}]
            else:
                case["executor_compatible"] = True
                case["definition_sha256"] = UX.definition_hash(
                    {key: value for key, value in case.items() if key != "definition_sha256"})
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                UX.validate_catalog(catalog)

    def test_existing_executor_help_is_local_and_arguments_exist(self):
        for tool, flags in (
            ("loxone-ux.py", ("--pid", "--deadline-seconds", "--verbose")),
            ("official-sim.py", ("--fixture", "--case", "--mode", "--transfer-profile")),
        ):
            result = subprocess.run([sys.executable, str(UX.ROOT / "scripts" / tool), "--help"],
                                    capture_output=True, text=True, timeout=15, check=True)
            for flag in flags:
                self.assertIn(flag, result.stdout)


class QueueTests(unittest.TestCase):
    def setUp(self):
        self.directory = UX.ROOT / ".reverse-engineering" / "ux-campaign-tests" / uuid.uuid4().hex
        self.directory.mkdir(parents=True)
        self.catalog = UX.generate_catalog()
        self.queue = UX.Queue(self.directory / "queue", self.catalog)
        self.queue.initialize()
        self.index = 0

    def tearDown(self):
        shutil.rmtree(self.directory)

    def artifact(self, kind, value, raw=False):
        self.index += 1
        path = self.directory / f"{self.index:03d}-{kind}.data"
        path.write_bytes(value if raw else UX.canonical(value))
        return {"path": path.name, "sha256": UX.digest(path), "kind": kind}

    def binding(self, alias="lab-a", physical="a", resource=None, capacity=3, capabilities=None):
        return {
            "schema_version": 1, "kind": "ux-campaign-worker-binding",
            "worker": alias, "physical_identity_sha256": physical * 64, "application": UX.APP,
            "admitted": True, "authority": "test-contract-fixture-not-a-readiness-oracle",
            "admission": self.artifact("external-admission", {"test_only": True}),
            "valid_until": (datetime.now(timezone.utc) + timedelta(hours=1)).isoformat(),
            "resources": {resource or "test-shared-host": capacity},
            "capabilities": capabilities or [
                "energy-dialog", "guarded-native-input", "owned-capture",
                "official-sim-plain", "native-energy-readback",
            ],
        }

    def admit(self, alias="lab-a", physical="a", **kwargs):
        binding = self.binding(alias, physical, **kwargs)
        self.queue.admit(binding, self.directory)
        return binding

    def handoff(self, run):
        fixture = self.directory / "baseline.Loxone"
        fixture.write_bytes(UX.repo_path(r"tests\ux\energy-manager2\synthetic.Loxone").read_bytes())
        return self.queue.handoff(
            run["run_id"],
            {"worker": run["worker"], "fixtures": {"energy-configuration": str(fixture)},
             "ux_profile": str(self.directory / "private-profile.json"), "pid": 4321},
            self.directory / ("handoff-" + run["run_id"]),
        )

    def capture(self, run, handoff):
        case = self.queue.cases[run["case_id"]]
        native_id = "C:\\OwnedArtifacts\\" + run["run_id"]
        guard = self.artifact("guard-receipt", {"test_contract_fixture": True, "error": None})
        fence = self.artifact("fence-proof", {"test_only": True, "scope": "owned-application"})
        identity = self.artifact("identity-proof", {
            "run_id": run["run_id"], "native_run_id": native_id,
            "application": UX.APP, "fixture_sha256": handoff["fixture_sha256"],
            "physical_identity_sha256": run["physical_identity_sha256"], "pid": 4321,
            "path": r"C:\OwnedTools\LoxoneConfig.exe", "start_time": "2026-11-08T07:00:00Z",
            "session": 1, "fence_proof_sha256": fence["sha256"],
            "guard_receipt_sha256": guard["sha256"],
        })
        receipt = self.artifact("native-receipt", {
            "schema_version": 1, "ok": True, "pid": 4321,
            "target": {"exe_sha256": UX.APP["sha256"], "version": UX.APP["version"]},
            "completed_steps": len(case["procedure"]), "failed_step": None, "error": None,
            "results": [], "elapsed_ms": 200, "artifacts": {}, "metrics": {},
        })
        request = self.artifact("native-request", {
            "schema_version": 1, "target": {"pid": 4321, "expected_exe_sha256": UX.APP["sha256"]},
            "artifact_dir": native_id, "steps": case["procedure"],
        })
        before = self.artifact("before-xml", (self.directory / "baseline.Loxone").read_bytes(), raw=True)
        after = self.artifact("after-xml", (self.directory / "baseline.Loxone").read_bytes(), raw=True)
        return {
            "schema_version": 1, "kind": "ux-campaign-capture", "evidence_origin": "native-config-ui",
            **{key: run[key] for key in ("run_id", "worker", "physical_identity_sha256",
                                        "case_id", "definition_sha256", "purpose",
                                        "qualification_id", "qualification_definition_sha256")},
            "handoff_sha256": UX.definition_hash(handoff), "fixture_sha256": handoff["fixture_sha256"],
            "native_run_id": native_id,
            "artifacts": [guard, fence, identity, receipt, request, before, after,
                          self.artifact("xml-diff", {"changes": []}),
                          self.artifact("observations", {"test_only": True}),
                          self.artifact("screenshot", b"\x89PNG\r\n\x1a\nTEST-ONLY-NOT-NATIVE", raw=True)],
        }

    def completed_canary(self):
        run = self.queue.claim("lab-a", canary=True)
        handoff = self.handoff(run)
        result = self.queue.collect(run["run_id"], self.capture(run, handoff), self.directory)
        self.assertEqual(result["status"], "observed")
        return run

    def edit_capture_artifact(self, capture, kind, edit):
        original = next(item for item in capture["artifacts"] if item["kind"] == kind)
        value = UX.read_json(UX.checked_artifact(self.directory, original))
        edit(value)
        replacement = self.artifact(kind, value)
        capture["artifacts"] = [replacement if item is original else item for item in capture["artifacts"]]
        return replacement

    def interrupted_capture(self, capture, code, declare_terminal=True):
        guard = self.edit_capture_artifact(capture, "guard-receipt",
                                           lambda value: value.update(error={"code": code}))
        self.edit_capture_artifact(capture, "identity-proof",
                                   lambda value: value.update(guard_receipt_sha256=guard["sha256"]))
        self.edit_capture_artifact(capture, "native-receipt", lambda value: value.update(
            ok=False, completed_steps=2, failed_step=2,
            error={"code": code, "message": "Native execution interrupted after two steps."}))
        if declare_terminal:
            capture["terminal_error"] = code
        return capture

    def new_config_capture(self, qualification=True):
        self.queue = UX.Queue(self.directory / ("regression-" + uuid.uuid4().hex), self.catalog)
        self.queue.initialize()
        self.admit()
        if not qualification:
            self.completed_canary()
        run = self.queue.claim("lab-a", canary=qualification)
        handoff = self.handoff(run)
        return run, handoff, self.capture(run, handoff)

    def test_internally_consistent_wrong_pid_cannot_replace_immutable_handoff_target(self):
        for terminal in (False, True):
            with self.subTest(terminal=terminal):
                run, handoff, capture = self.new_config_capture()
                handoff_event = next(event for event in self.queue.events() if event["type"] == "handoff")
                self.assertEqual(handoff_event["expected_target_pid"], 4321)
                self.assertEqual(handoff["expected_target_pid"], 4321)
                if terminal:
                    self.interrupted_capture(capture, "USER_CONTROL")
                self.edit_capture_artifact(capture, "identity-proof", lambda value: value.update(pid=9999))
                self.edit_capture_artifact(capture, "native-receipt", lambda value: value.update(pid=9999))
                self.edit_capture_artifact(capture, "native-request",
                                           lambda value: value["target"].update(pid=9999))
                self.assertEqual(capture["handoff_sha256"], handoff_event["handoff_sha256"])
                result = self.queue.collect(run["run_id"], capture, self.directory)
                self.assertEqual(result["status"], "UNKNOWN")
                self.assertIn("handoff PID", result["reason"])
                self.assertFalse(result["native_started"])
                workers, _, _, qualified = self.queue.state()
                self.assertTrue(workers["lab-a"]["parked"])
                self.assertEqual(workers["lab-a"]["active"], run["run_id"])
                self.assertEqual(qualified, set())
                self.assertEqual(self.queue.stats()["qualification"]["native_attempts"], 0)
                with self.assertRaises(ValueError):
                    self.queue.claim("lab-a", True)

    def test_terminal_partial_native_start_is_counted_with_or_without_redundant_terminal_field(self):
        for code in sorted(UX.TERMINAL_CONTROL):
            for qualification in (True, False):
                for declare_terminal in (True, False):
                    with self.subTest(code=code, qualification=qualification, declared=declare_terminal):
                        run, _, capture = self.new_config_capture(qualification)
                        qualified_before = self.queue.state()[3].copy()
                        self.interrupted_capture(capture, code, declare_terminal)
                        result = self.queue.collect(run["run_id"], capture, self.directory)
                        self.assertEqual(result["status"], "blocked")
                        self.assertTrue(result["native_started"])
                        workers, _, _, qualified = self.queue.state()
                        self.assertEqual(qualified, qualified_before)
                        self.assertTrue(workers["lab-a"]["parked"])
                        self.assertEqual(workers["lab-a"]["active"], run["run_id"])
                        stats = self.queue.stats()
                        relevant = stats["qualification"] if qualification else stats
                        self.assertEqual((relevant["native_attempts"], relevant["blocked"], relevant["completions"]), (1, 1, 0))
                        self.assertEqual(relevant["verified_pass"], 0)
                        self.assertEqual(stats["native_attempts"], 0 if qualification else 1)
                        with self.assertRaises(ValueError):
                            self.queue.claim("lab-a", True)

    def test_terminal_partial_receipt_without_identity_proof_is_unknown_not_invented_start(self):
        run, _, capture = self.new_config_capture()
        self.interrupted_capture(capture, "CONTROL_UNAVAILABLE")
        capture["artifacts"] = [item for item in capture["artifacts"]
                                if item["kind"] in {"guard-receipt", "native-receipt"}]
        result = self.queue.collect(run["run_id"], capture, self.directory)
        self.assertEqual(result["status"], "UNKNOWN")
        self.assertFalse(result["native_started"])
        self.assertTrue(self.queue.state()[0]["lab-a"]["parked"])
        self.assertEqual(self.queue.state()[3], set())
        self.assertEqual(self.queue.stats()["qualification"]["native_attempts"], 0)

    def test_legacy_handoff_without_expected_pid_cannot_qualify_or_be_migrated(self):
        run, handoff, _ = self.new_config_capture()
        del handoff["expected_target_pid"]
        events = self.queue.events()
        del events[0]["handoff_identity_version"]
        previous = None
        for event in events:
            del event["sha256"]
            if event["type"] == "handoff":
                del event["expected_target_pid"]
                event["handoff_sha256"] = UX.definition_hash(handoff)
            event["previous"] = previous
            event["sha256"] = UX.definition_hash(event)
            previous = event["sha256"]
        (self.queue.path / "journal.jsonl").write_bytes(b"".join(UX.canonical(event) + b"\n" for event in events))
        result = self.queue.collect(run["run_id"], self.capture(run, handoff), self.directory)
        self.assertEqual(result["status"], "UNKNOWN")
        self.assertIn("Legacy Config handoff", result["reason"])
        self.assertFalse(result["native_started"])
        self.assertTrue(self.queue.state()[0]["lab-a"]["parked"])
        self.assertEqual(self.queue.state()[3], set())
        with self.assertRaisesRegex(ValueError, "Legacy"):
            self.queue.qualification_plan()

    def simulator_capture(self, outputs=1, original_status="pass"):
        """A separate synthetic protocol harness, never the public native fixture."""
        native = UX.read_json(UX.repo_path(r"tests\ux\native-simulator\energy-storage-at-finite.json"))
        fixture = self.directory / "sim-test.Loxone"
        fixture.write_bytes(UX.repo_path(r"tests\ux\energy-manager2\synthetic.Loxone").read_bytes())
        native["fixture_sha256"] = UX.digest(fixture)
        native_source = self.directory / "test-native-definition.json"
        native_source.write_bytes(UX.canonical(native))
        catalog = copy.deepcopy(self.catalog)
        case = next(case for case in catalog["cases"]
                    if case["id"] == "sim-native-storage-at-finite-repeat-01")
        case["source"] = UX.source(str(native_source.relative_to(UX.ROOT)), native["id"])
        case["fixture_sha256"] = native["fixture_sha256"]
        case["needed_capabilities"] = ["test-storage-only"]
        case["definition_sha256"] = UX.definition_hash(
            {key: value for key, value in case.items() if key != "definition_sha256"})
        self.queue = UX.Queue(self.directory / ("sim-queue-" + uuid.uuid4().hex), catalog)
        self.queue.initialize()
        self.admit(capabilities=["test-storage-only"])
        run = self.queue.claim("lab-a", True)
        handoff = self.queue.handoff(
            run["run_id"],
            {"worker": "lab-a", "fixtures": {case["fixture_alias"]: str(fixture)},
             "sim_profile": "external-private-sim.json", "transfer_profile": "external-private-transfer.json",
             "artifact_root": str(self.directory)},
            self.directory / ("sim-handoff-" + run["run_id"]))
        # The real replay parser requires its --case under --artifact-root.
        staged_case = Path(handoff["argv"][handoff["argv"].index("--case") + 1])
        self.assertTrue(staged_case.is_relative_to(self.directory))
        self.assertEqual(UX.read_json(staged_case), native)
        (self.directory / "baseline.Loxone").write_bytes(fixture.read_bytes())
        capture = self.capture(run, handoff)
        native_id = "native-" + run["run_id"]
        capture["native_run_id"] = native_id
        old_identity = next(item for item in capture["artifacts"] if item["kind"] == "identity-proof")
        identity = UX.read_json(UX.checked_artifact(self.directory, old_identity))
        identity["native_run_id"] = native_id
        capture["artifacts"] = [item for item in capture["artifacts"]
                                if item["kind"] not in {"identity-proof", "native-receipt", "native-request", "observations"}]
        identity_artifact = self.artifact("identity-proof", identity)
        request = self.artifact("native-request", {
            "schema_version": 1, "run_id": native_id, "fixture_sha256": native["fixture_sha256"],
            "application": {**UX.APP, "path": identity["path"]}, "mode": "plain",
            "sampling_phases": [{"name": "after_first_calculation", "at_seconds": 70}],
        })
        definition = self.artifact("native-case-definition", native)
        observation = self.artifact("observations", {"test_contract_fixture": True})
        samples = [{
            "phase": "after_first_calculation", "elapsed_ms": 70001, "outputs": {"OL1": outputs},
            "source": {"kind": "native-readable", "artifacts": [observation["path"]]},
        }]
        receipt = self.artifact("native-receipt", {
            "schema_version": 1, "kind": "official-native-replay", "run_id": native_id,
            "application": UX.APP, "case": {
                "id": native["id"], "sha256": definition["sha256"],
                "definition": definition, "fixture_sha256": native["fixture_sha256"],
            },
            "execution": {
                "mode": "plain", "status": original_status, "run_id": native_id,
                "reason": "Actual original blocker" if original_status == "blocked" else None,
                "native_running_verified": True, "cleanup": {"status": "closed"},
            },
            "remote": {"target": {"pid": 4321, "start_utc": identity["start_time"]}},
            "observations": samples,
            "artifacts": [request, definition, observation],
        })
        capture["artifacts"].extend([identity_artifact, request, observation, receipt])
        return run, capture

    def test_native_simulator_rejudges_observations_not_self_declared_pass(self):
        for output, expected in ((1, "verified-pass"), (0, "verified-fail")):
            with self.subTest(output=output):
                run, capture = self.simulator_capture(output)
                result = self.queue.collect(run["run_id"], capture, self.directory)
                self.assertEqual(result["status"], expected)
                self.assertEqual(self.queue.stats()["qualification"]["native_attempts"], 1)
                self.assertEqual(self.queue.stats()["qualification"]["completions"], 1)
                self.assertEqual(self.queue.stats()["native_attempts"], 0)
                if expected == "verified-fail":
                    self.assertFalse(self.queue.state()[0]["lab-a"]["parked"])
                    with self.assertRaises(ValueError):
                        self.queue.claim("lab-a", True)

    def test_native_original_blocker_is_not_hidden_by_matching_outputs(self):
        run, capture = self.simulator_capture(original_status="blocked")
        result = self.queue.collect(run["run_id"], capture, self.directory)
        self.assertEqual(result["status"], "blocked")
        self.assertEqual(self.queue.stats()["verified_pass"], 0)
        with self.assertRaises(ValueError):
            self.queue.claim("lab-a", True)

    def test_native_simulator_original_case_definition_digest_is_checked(self):
        run, capture = self.simulator_capture()
        native_receipt = next(item for item in capture["artifacts"] if item["kind"] == "native-receipt")
        receipt = UX.read_json(UX.checked_artifact(self.directory, native_receipt))
        receipt["case"]["sha256"] = "0" * 64
        capture["artifacts"] = [item for item in capture["artifacts"] if item is not native_receipt]
        capture["artifacts"].append(self.artifact("native-receipt", receipt))
        self.assertEqual(self.queue.collect(run["run_id"], capture, self.directory)["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["completions"], 0)

    def test_plans_and_failed_readiness_are_not_native_attempts(self):
        for admitted in (False, True):
            binding = self.binding()
            binding["admitted"] = admitted
            if admitted:
                binding["valid_until"] = "2020-01-01T00:00:00+00:00"
            with self.assertRaises(ValueError):
                self.queue.admit(binding, self.directory)
        with self.assertRaises(ValueError):
            self.queue.claim("lab-a")
        stats = self.queue.stats()
        self.assertEqual((stats["dispatch_intents"], stats["native_attempts"], stats["completions"]), (0, 0, 0))
        self.assertEqual(stats["unattempted_cases"], 100)

    def test_catalog_is_reproducible_across_lf_and_crlf_source_checkouts(self):
        source = UX.repo_path(UX.GUI_SOURCE).read_bytes().replace(b"\r\n", b"\n")
        alternate = self.directory / "source-checkout.json"
        original_resolver = UX.repo_path

        def resolver(relative):
            return alternate if relative == UX.GUI_SOURCE else original_resolver(relative)

        alternate.write_bytes(source)
        with patch.object(UX, "repo_path", side_effect=resolver):
            lf_catalog = UX.generate_catalog()
        alternate.write_bytes(source.replace(b"\n", b"\r\n"))
        with patch.object(UX, "repo_path", side_effect=resolver):
            crlf_catalog = UX.generate_catalog()
        self.assertEqual(lf_catalog, crlf_catalog)
        self.assertEqual(UX.definition_hash(lf_catalog), UX.definition_hash(crlf_catalog))

    def test_canary_gate_and_bounded_no_input_batch(self):
        self.admit()
        with self.assertRaises(ValueError):
            self.queue.claim("lab-a")
        plan = self.queue.batch_plan("lab-a", 5)
        self.assertTrue(plan["canary_required"])
        self.assertEqual(plan["cases"], [UX.CANARY])
        self.assertEqual(self.queue.stats()["dispatch_intents"], 0)
        self.completed_canary()
        self.assertEqual(self.queue.stats()["qualification"]["observations"], 1)
        self.assertEqual(self.queue.stats()["observations"], 0)
        self.assertEqual(self.queue.stats()["verified_pass"], 0)
        experiment = self.queue.claim("lab-a")
        self.assertEqual(experiment["case_id"], UX.CANARY)
        self.assertEqual(experiment["purpose"], "experiment")
        self.assertIsNone(experiment["qualification_id"])
        handoff = self.handoff(experiment)
        self.queue.collect(experiment["run_id"], self.capture(experiment, handoff), self.directory)
        self.assertEqual(self.queue.stats()["observations"], 1)
        # Simulator is a different adapter: its own native canary is required.
        with self.assertRaises(ValueError):
            self.queue.claim("lab-a")
        next_plan = self.queue.batch_plan("lab-a", 5)
        self.assertTrue(next_plan["canary_required"])
        self.assertEqual(len(next_plan["cases"]), 1)
        with self.assertRaises(ValueError):
            self.queue.batch_plan("lab-a", 100)

    def test_three_workers_qualify_same_config_adapter_without_consuming_experiments(self):
        declared = {item["qualification_id"]: item for item in self.queue.qualification_plan()}
        self.assertEqual(len(declared), 6)
        runs, captures = [], []
        capabilities = ["energy-dialog", "guarded-native-input", "owned-capture"]
        for alias, physical in (("notebook", "c"), ("lab-a", "a"), ("lab-b", "b")):
            self.admit(alias, physical, capabilities=capabilities)
            run = self.queue.claim(alias, canary=True)
            expected = declared["qualification-" + alias + "-loxone-ux-recipe"]
            self.assertEqual(run["run_id"], expected["run_id"])
            self.assertEqual(run["qualification_definition_sha256"], expected["qualification_definition_sha256"])
            self.assertEqual(run["case_id"], UX.CANARY)
            handoff = self.handoff(run)
            self.assertEqual(handoff["purpose"], "qualification")
            capture = self.capture(run, handoff)
            result = self.queue.collect(run["run_id"], capture, self.directory)
            self.assertEqual(result["status"], "observed")
            runs.append(run)
            captures.append(capture)
        self.assertEqual(len({run["run_id"] for run in runs}), 3)
        self.assertEqual(len({capture["native_run_id"] for capture in captures}), 3)
        workers, _, used, qualified = self.queue.state()
        self.assertEqual(used, set())
        self.assertEqual(qualified, {(alias, "loxone-ux-recipe") for alias in workers})
        stats = self.queue.stats()
        self.assertEqual((stats["definitions"], stats["unattempted_cases"]), (100, 100))
        self.assertEqual((stats["native_attempts"], stats["completions"], stats["unique_native_cases"]), (0, 0, 0))
        self.assertEqual((stats["qualification"]["native_attempts"], stats["qualification"]["completions"]), (3, 3))
        self.assertEqual(stats["all_native_attempts"], 3)
        for alias in workers:
            with self.assertRaises(ValueError):
                self.queue.claim(alias, canary=True)
        experiment = self.queue.claim("notebook")
        self.assertEqual(experiment["case_id"], UX.CANARY)
        handoff = self.handoff(experiment)
        self.queue.collect(experiment["run_id"], self.capture(experiment, handoff), self.directory)
        stats = self.queue.stats()
        self.assertEqual((stats["native_attempts"], stats["completions"], stats["unattempted_cases"]), (1, 1, 99))
        self.assertEqual(stats["qualification"]["completions"], 3)
        self.assertEqual(stats["all_completions"], 4)
        with self.assertRaises(ValueError):
            self.queue.claim("lab-a")

    def test_failed_qualification_stays_owned_and_other_worker_uses_predeclared_identity_only(self):
        declared = copy.deepcopy(self.queue.qualification_plan())
        self.admit("lab-a", "a", resource="independent-lab", capacity=1)
        self.admit("notebook", "c", resource="independent-notebook", capacity=1)
        failed = self.queue.claim("lab-a", True)
        handoff = self.handoff(failed)
        capture = self.capture(failed, handoff)
        capture["terminal_error"] = "USER_CONTROL"
        capture["artifacts"] = [self.artifact("guard-receipt", {"error": {"code": "USER_CONTROL"}})]
        self.assertEqual(self.queue.collect(failed["run_id"], capture, self.directory)["status"], "blocked")
        with self.assertRaises(ValueError):
            self.queue.claim("lab-a", True)
        with self.assertRaises(ValueError):
            self.queue.claim("notebook")
        other = self.queue.claim("notebook", True)
        expected = next(item for item in declared if item["qualification_id"] == other["qualification_id"])
        self.assertEqual(other["run_id"], expected["run_id"])
        self.assertNotEqual(other["run_id"], failed["run_id"])
        self.assertNotEqual(other["qualification_id"], failed["qualification_id"])
        self.assertEqual(declared, self.queue.qualification_plan())
        workers, runs, used, _ = self.queue.state()
        self.assertTrue(workers["lab-a"]["parked"])
        self.assertEqual(workers["lab-a"]["active"], failed["run_id"])
        self.assertEqual(runs[failed["run_id"]]["status"], "blocked")
        self.assertEqual(used, set())
        self.assertEqual(self.queue.stats()["qualification"]["dispatch_intents"], 2)
        self.assertEqual(self.queue.stats()["dispatch_intents"], 0)

    def test_consumed_configuration_experiment_does_not_remove_later_worker_canary(self):
        capabilities = ["energy-dialog", "guarded-native-input", "owned-capture"]
        self.admit("lab-a", "a", capabilities=capabilities)
        self.completed_canary()
        experiment = self.queue.claim("lab-a")
        handoff = self.handoff(experiment)
        self.queue.collect(experiment["run_id"], self.capture(experiment, handoff), self.directory)
        self.admit("notebook", "c", capabilities=capabilities)
        plan = self.queue.batch_plan("notebook", 5)
        self.assertEqual(plan["purpose"], "qualification")
        self.assertEqual(plan["cases"], [UX.CANARY])
        run = self.queue.claim("notebook", True)
        self.assertEqual(run["run_id"], plan["predeclared_run_id"])
        handoff = self.handoff(run)
        self.assertEqual(self.queue.collect(run["run_id"], self.capture(run, handoff), self.directory)["status"], "observed")
        self.assertEqual(self.queue.stats()["qualification"]["completions"], 2)
        self.assertEqual(self.queue.stats()["completions"], 1)
        with self.assertRaises(ValueError):
            self.queue.claim("notebook")

    def test_legacy_queue_is_not_silently_reinterpreted_as_qualification(self):
        path = self.queue.path / "journal.jsonl"
        created = self.queue.events()[0]
        del created["qualification_canaries"]
        del created["sha256"]
        created["sha256"] = UX.definition_hash(created)
        path.write_bytes(UX.canonical(created) + b"\n")
        original = path.read_bytes()
        self.assertEqual(self.queue.stats()["qualification"]["predeclared_runs"], 0)
        self.assertEqual(self.queue.stats()["unattempted_cases"], 100)
        with self.assertRaisesRegex(ValueError, "Legacy"):
            self.queue.admit(self.binding(), self.directory)
        self.assertEqual(path.read_bytes(), original)

    def test_other_worker_cannot_replay_qualification_receipt_or_relabel_it_as_experiment(self):
        self.admit("lab-a", "a")
        self.admit("lab-b", "b")
        first = self.queue.claim("lab-a", True)
        capture = self.capture(first, self.handoff(first))
        self.queue.collect(first["run_id"], capture, self.directory)
        second = self.queue.claim("lab-b", True)
        replay = self.capture(second, self.handoff(second))
        replay["native_run_id"] = capture["native_run_id"]
        self.assertEqual(self.queue.collect(second["run_id"], replay, self.directory)["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["qualification"]["completions"], 1)
        self.assertEqual(self.queue.stats()["native_attempts"], 0)
        experiment = self.queue.claim("lab-a")
        relabeled = self.capture(experiment, self.handoff(experiment))
        relabeled["purpose"] = "qualification"
        relabeled["qualification_id"] = first["qualification_id"]
        self.assertEqual(self.queue.collect(experiment["run_id"], relabeled, self.directory)["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["completions"], 0)

    def test_one_active_per_distinct_desktop_maximum_three(self):
        self.admit("lab-a", "a")
        with self.assertRaises(ValueError):
            self.admit("lab-b", "a")
        self.admit("lab-b", "b")
        self.admit("notebook", "c")
        first = self.queue.claim("lab-a", True)
        second = self.queue.claim("lab-b", True)
        third = self.queue.claim("notebook", True)
        self.assertEqual(len({run["physical_identity_sha256"] for run in (first, second, third)}), 3)
        self.assertEqual({run["case_id"] for run in (first, second, third)}, {UX.CANARY})
        self.assertEqual(len({run["qualification_id"] for run in (first, second, third)}), 3)
        for alias in ("lab-a", "lab-b", "notebook"):
            with self.assertRaises(ValueError):
                self.queue.claim(alias, True)

    def test_competing_claims_cannot_overlap_on_one_desktop(self):
        self.admit()

        def claim():
            try:
                return self.queue.claim("lab-a", True)
            except ValueError:
                return None

        with ThreadPoolExecutor(max_workers=2) as pool:
            results = list(pool.map(lambda _: claim(), range(2)))
        self.assertEqual(sum(result is not None for result in results), 1)
        self.assertEqual(self.queue.stats()["qualification"]["dispatch_intents"], 1)

    def test_shared_host_gate_preserved_but_independent_notebook_not_starved(self):
        self.admit("lab-a", "a", resource="vm-host", capacity=1)
        self.admit("lab-b", "b", resource="vm-host", capacity=1)
        self.admit("notebook", "c", resource="independent-notebook", capacity=1)
        self.queue.claim("lab-a", True)
        with self.assertRaises(ValueError):
            self.queue.claim("lab-b", True)
        self.assertEqual(self.queue.claim("notebook", True)["worker"], "notebook")

    def test_unknown_has_no_ttl_reclaim_and_consumed_case_is_not_retried(self):
        self.admit()
        self.admit("notebook", "c", resource="independent-notebook", capacity=1)
        run = self.queue.claim("lab-a", True)
        self.queue.unknown(run["run_id"], "Timeout; remote completion UNKNOWN")
        for _ in range(2):
            with self.assertRaises(ValueError):
                self.queue.claim("lab-a", True)
        with patch.object(UX, "datetime", wraps=datetime) as clock:
            clock.now.return_value = datetime.now(timezone.utc) + timedelta(days=30)
            with self.assertRaises(ValueError):
                self.queue.claim("lab-a", True)
        other = self.queue.claim("notebook", True)
        self.assertEqual(run["case_id"], other["case_id"])
        self.assertNotEqual(run["qualification_id"], other["qualification_id"])
        self.assertNotEqual(run["run_id"], other["run_id"])
        workers, runs, _, _ = self.queue.state()
        self.assertEqual(workers["lab-a"]["active"], run["run_id"])
        self.assertEqual(runs[run["run_id"]]["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["native_attempts"], 0)
        self.assertEqual(self.queue.stats()["qualification"]["unknown_native_start_cases"], 1)

    def test_terminal_control_parks_worker_without_case_requeue(self):
        for code in sorted(UX.TERMINAL_CONTROL):
            with self.subTest(code=code):
                directory = self.directory / code
                queue = UX.Queue(directory, self.catalog)
                queue.initialize()
                queue.admit(self.binding(), self.directory)
                run = queue.claim("lab-a", True)
                # Materialize through this queue, not the parent test queue.
                original = self.queue
                self.queue = queue
                try:
                    handoff = self.handoff(run)
                    capture = self.capture(run, handoff)
                    capture["terminal_error"] = code
                    guard = self.artifact("guard-receipt", {"error": {"code": code}})
                    capture["artifacts"] = [guard]
                    result = queue.collect(run["run_id"], capture, self.directory)
                    self.assertEqual(result["status"], "blocked")
                    with self.assertRaises(ValueError):
                        queue.claim("lab-a", True)
                    self.assertEqual(queue.stats()["native_attempts"], 0)
                finally:
                    self.queue = original

    def test_receipt_digest_case_app_fixture_pid_and_origin_mismatch_quarantine(self):
        for mutation in ("digest", "case", "app", "fixture", "pid", "origin", "missing"):
            with self.subTest(mutation=mutation):
                queue = UX.Queue(self.directory / ("queue-" + mutation), self.catalog)
                queue.initialize()
                queue.admit(self.binding(), self.directory)
                original = self.queue
                self.queue = queue
                try:
                    run = queue.claim("lab-a", True)
                    handoff = self.handoff(run)
                    capture = self.capture(run, handoff)
                    if mutation == "digest":
                        capture["artifacts"][0]["sha256"] = "0" * 64
                    elif mutation == "case":
                        capture["case_id"] = "some-other-case"
                    elif mutation == "fixture":
                        capture["fixture_sha256"] = "0" * 64
                    elif mutation == "origin":
                        capture["evidence_origin"] = "synthetic-helper"
                    elif mutation == "missing":
                        capture["artifacts"] = []
                    else:
                        descriptor = next(item for item in capture["artifacts"] if item["kind"] == "native-request")
                        request = UX.read_json(UX.checked_artifact(self.directory, descriptor))
                        request["target"]["pid" if mutation == "pid" else "expected_exe_sha256"] = (
                            999 if mutation == "pid" else "0" * 64)
                        capture["artifacts"] = [item for item in capture["artifacts"] if item is not descriptor]
                        capture["artifacts"].append(self.artifact("native-request", request))
                    result = queue.collect(run["run_id"], capture, self.directory)
                    self.assertEqual(result["status"], "UNKNOWN")
                    self.assertEqual(queue.stats()["verified_pass"], 0)
                    with self.assertRaises(ValueError):
                        queue.claim("lab-a", True)
                finally:
                    self.queue = original

    def test_local_cli_status_pass_is_not_a_native_receipt(self):
        self.admit()
        run = self.queue.claim("lab-a", True)
        handoff = self.handoff(run)
        capture = self.capture(run, handoff)
        capture["kind"] = "lox-cli-unit-test"
        capture["status"] = "pass"
        self.assertEqual(self.queue.collect(run["run_id"], capture, self.directory)["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["completions"], 0)

    def test_missing_receipt_file_is_unknown_and_keeps_dispatch_quarantined(self):
        self.admit()
        run = self.queue.claim("lab-a", True)
        result = self.queue.collect_file(run["run_id"], self.directory / "missing.json", self.directory)
        self.assertEqual(result["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["native_attempts"], 0)
        self.assertEqual(self.queue.state()[0]["lab-a"]["active"], run["run_id"])
        with self.assertRaises(ValueError):
            self.queue.claim("lab-a", True)

    def test_existing_driver_schema_error_becomes_unknown_not_unhandled_or_success(self):
        self.admit()
        run = self.queue.claim("lab-a", True)
        capture = self.capture(run, self.handoff(run))
        descriptor = next(item for item in capture["artifacts"] if item["kind"] == "native-receipt")
        receipt = UX.read_json(UX.checked_artifact(self.directory, descriptor))
        receipt["completed_steps"] = -1
        capture["artifacts"] = [item for item in capture["artifacts"] if item is not descriptor]
        capture["artifacts"].append(self.artifact("native-receipt", receipt))
        self.assertEqual(self.queue.collect(run["run_id"], capture, self.directory)["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["verified_pass"], 0)

    def test_artifacts_cannot_escape_private_root_or_omit_digest(self):
        for descriptor in (
            {"path": "..\\outside.json", "sha256": "0" * 64},
            {"path": "C:\\outside.json", "sha256": "0" * 64},
            {"path": "proof.json"},
            {"path": 123, "sha256": "0" * 64},
        ):
            with self.subTest(descriptor=descriptor), self.assertRaises(ValueError):
                UX.checked_artifact(self.directory, descriptor)

    def test_handoff_matches_actual_recipe_and_is_exclusive_data_only(self):
        self.admit()
        run = self.queue.claim("lab-a", True)
        handoff = self.handoff(run)
        parsed = UX.load_tool("loxone-ux").make_parser().parse_args(handoff["argv"][2:])
        steps, pin = UX.load_tool("loxone-ux").command_steps(parsed)
        self.assertEqual(steps, self.queue.cases[run["case_id"]]["procedure"])
        self.assertEqual(pin, UX.APP["sha256"])
        self.assertEqual(self.queue.stats()["native_attempts"], 0)
        with self.assertRaises(ValueError):
            self.handoff(run)

    def test_batch_audit_is_idempotent_and_cannot_replace_terminal_result(self):
        self.admit()
        run = self.queue.claim("lab-a", True)
        handoff = self.handoff(run)
        capture = self.capture(run, handoff)
        first = self.queue.collect(run["run_id"], capture, self.directory)
        event_count = len(self.queue.events())
        second = self.queue.collect(run["run_id"], capture, self.directory)
        self.assertEqual(second, first)
        self.assertEqual(len(self.queue.events()), event_count)
        self.assertEqual(self.queue.stats()["qualification"]["completions"], 1)
        self.assertEqual(self.queue.stats()["completions"], 0)
        changed = copy.deepcopy(capture)
        changed["status"] = "pass"
        with self.assertRaises(ValueError):
            self.queue.collect(run["run_id"], changed, self.directory)

    def test_journal_catalog_and_partial_append_fail_closed(self):
        path = self.queue.path / "journal.jsonl"
        original = path.read_bytes()
        path.write_bytes(original.rstrip(b"\n"))
        with self.assertRaises(ValueError):
            self.queue.stats()
        path.write_bytes(original)
        other = copy.deepcopy(self.catalog)
        other["policy"] = "different history"
        with self.assertRaises(ValueError):
            UX.Queue(self.queue.path, other).stats()
        with self.assertRaises(FileExistsError):
            self.queue.initialize()

    def test_quiescence_releases_capacity_not_failed_case_or_worker(self):
        self.admit("lab-a", "a", resource="vm-host", capacity=1)
        self.admit("lab-b", "b", resource="vm-host", capacity=1)
        run = self.queue.claim("lab-a", True)
        self.queue.unknown(run["run_id"], "Uncertain native completion")
        with self.assertRaises(ValueError):
            self.queue.claim("lab-b", True)
        wrong = self.artifact("quiescence", {"kind": "unit-test"})
        with self.assertRaises(ValueError):
            self.queue.quiescent("lab-a", wrong, self.directory)
        proof = self.artifact("quiescence", {
            "kind": "ux-campaign-quiescence-binding", "run_id": run["run_id"],
            "physical_identity_sha256": "a" * 64, "quiescent": True,
            "guard_receipt": self.artifact("external-guard", {"test_only": True}),
        })
        self.queue.quiescent("lab-a", proof, self.directory)
        with self.assertRaises(ValueError):
            self.queue.claim("lab-a", True)
        other = self.queue.claim("lab-b", True)
        self.assertEqual(other["case_id"], run["case_id"])
        self.assertNotEqual(other["qualification_id"], run["qualification_id"])


if __name__ == "__main__":
    unittest.main()
