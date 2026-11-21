"""Synthetic protocol fixtures, not real selection, grants or native credit."""

import copy
from datetime import datetime, timedelta
import io
import importlib.util
import json
from pathlib import Path
import shutil
import unittest
import uuid
from unittest.mock import patch

try:
    from PIL import Image
except ImportError:
    Image = None

import ux_property_executor as EX
import ux_hybrid_selection as HY
from test_ux_property_executor import Q, SyntheticSDK, SyntheticAuthority


def png(source):
    output = io.BytesIO()
    source.save(output, format="PNG")
    return output.getvalue()


class HybridAuthority(SyntheticAuthority):
    def __init__(self, test):
        super().__init__(test)
        self.expected_seen = []
        self.modify = None
        self.mutate = self.hybrid_facts

    def preflight(self, context, binding):
        return self.test.save("preflight", {**context, "execution_binding": binding,
                              "source_utc": EX.utc().isoformat(), "outcome": "available",
                              "test_only_not_a_grant": True})

    def hybrid_facts(self, receipt):
        test = self.test
        facts, intent = receipt["facts"], receipt["input"]
        identity = facts["identity"]
        expected = receipt["expected_title"]
        self.expected_seen.append((receipt["stage"], identity["pid"], expected))
        after = expected == test.case["property"]["after"]
        frame = test.frame_after if after else test.frame_before
        source = test.save("observed-image", png(frame))
        row = {"class_name": "ActualSyntheticRow", "runtime_id": [3, 10],
               "physical_bounds": [110, 10, 100, 30]}
        editor = {
            "selector": {"automation_id": "duplicated-Title", "control_type": "Edit"},
            "runtime_id": [3, 11], "ancestors": [row, {"name": "Allgemein"}],
            "physical_bounds": [150, 10, 50, 30], "visible": True, "enabled": True,
            "read_only": False, "has_keyboard_focus": True,
            "method": "read-only-ValuePattern", "value": expected,
        }
        mapping = {
            "identity": identity,
            "display_inventory_sha256": receipt["display_inventory_sha256"],
            "pixel_to_physical": [1, 0, 0, 0, 1, 0],
            "calibration": {"method": "measured-owned-image-v1",
                            "image_pixels": [[0, 0], [220, 80]],
                            "physical_points": [[0, 0], [220, 80]]},
        }
        capture_ack = test.save("actual-shaped-capture-ack", {
            "isError": False, "structuredContent": {
                "result": "Response: " + json.dumps({
                    "synthetic_not_native": True,
                    "capture": {"method": "PrintWindow", "flags": 2, "returned": True,
                                "raw_png_sha256": source["sha256"], "width": 220, "height": 80,
                                "pixel_to_physical_screen": [1, 0, 0, 0, 1, 0]},
                    **{phase: {"pid": identity["pid"], "root": identity["root"],
                              "start": identity["start_identity"], "session": identity["session"],
                              "executable": identity["path"], "exe_sha256": Q.APP["sha256"],
                              "project": test.handoff["document_path"], "calibration_ok": True,
                              "physical_rect": [0, 0, 220, 80]}
                       for phase in ("before", "after")},
                }) + "\nStatus Code: 0"},
        })
        capture = {
            "identity": identity, "mapping": mapping, "image": source,
            "capture_source_sha256": "c" * 64, "native_method": "PrintWindow2",
            "native_success": True, "window_bounds": [0, 0, 220, 80],
            "capture_ack": capture_ack,
        }
        facts["owned_capture"] = {**capture, "method": "reviewed-owned-window-png-v1"}
        facts["hybrid"] = {
            **capture,
            "method": HY.METHOD, "expected_title": expected, "identity": identity,
            "baseline_sha256": test.case["fixture_sha256"],
            "capture_source_sha256": "c" * 64, "mapping": mapping, "image": source,
            "hit_canvas_runtime_id": [3, 20],
            "canvas": {"selector": {"class_name": "ActualSyntheticCanvas"},
                       "ancestors": [{"control_type": "Pane"}],
                       "runtime_id": [3, 20], "visible": True,
                       "physical_bounds": [0, 0, 100, 70]},
        }
        facts["inspector"] = {
            **capture,
            "identity": identity, "expected_title": expected, "mapping": mapping,
            "selected_headers": [{"selector": {"name": "Actual native Energy header"},
                                  "ancestors": [{"control_type": "Pane"}]}],
            "editors": [editor, {**editor, "value": "", "runtime_id": [3, 99],
                                "ancestors": [{"class_name": "VisualizationNameRow"},
                                              {"name": "Allgemein"}]}],
            "label_row_runtime_id": row["runtime_id"],
            "label_image": test.label, "label_source_image": source,
            "label_crop": [111, 15, 117, 18],
        }
        facts["input_focus"] = {
            "identity": identity, "hwnd": 123, "thread_active_root": identity["root"],
            "visible": True, "enabled": True,
        }
        for role in ("property", "value"):
            facts["selectors"][role] = [{
                "binding": test.case["selectors"][role], "selector": editor["selector"],
                "runtime_id": editor["runtime_id"], "visible": True,
                "physical_bounds": editor["physical_bounds"],
            }]
        if intent and intent["selector"]:
            point = [22, 32] if intent["selector"] == "target" else [175, 25]
            facts["hit"] = {"identity": identity, "physical_point": point}
        if self.modify:
            self.modify(receipt)


@unittest.skipIf(Image is None, "Hybrid image tests require the optional Pillow dependency.")
class HybridSelectionTests(unittest.TestCase):
    def setUp(self):
        self.root = Q.ROOT / ".reverse-engineering" / "hybrid-tests" / uuid.uuid4().hex
        self.root.mkdir(parents=True)
        self.index = 0
        self.catalog = EX.hybrid_catalog(Q)
        self.before = (b'<R><C Type="EnergyManager2" U="1ea20af3-0284-bf5d-ffff741afa8d2a84"'
                       b' Title="Energy Manager" 12hTF="keep"/></R>')
        self.after = self.before.replace(b'Energy Manager"', b'Energy ManagerPilot"')
        for case in self.catalog["cases"]:
            case["fixture_sha256"] = EX.sha(self.before)
            case["definition_sha256"] = Q.definition_hash(
                {key: value for key, value in case.items() if key != "definition_sha256"})
        self.case = self.catalog["cases"][0]
        self.baseline = self.root / "before.Loxone"
        self.baseline.write_bytes(self.before)
        self.frame_before = Image.new("RGB", (220, 80), "white")
        for y in range(3):
            for x in range(4):
                self.frame_before.putpixel((20 + x, 30 + y), (x * 41, y * 57, 91))
        for y in range(3):
            for x in range(6):
                self.frame_before.putpixel((111 + x, 15 + y), (117, x * 21, y * 48))
        self.frame_before.paste((50, 60, 70), (20, 5, 50, 8))
        self.frame_after = self.frame_before.copy()
        self.frame_after.paste((170, 80, 60), (20, 5, 70, 8))
        original = self.save("original", png(self.frame_before))
        body = self.save("body", png(self.frame_before.crop((20, 30, 24, 33))))
        self.label = self.save("label", png(self.frame_before.crop((111, 15, 117, 18))))
        baseline_label = self.save("title-label", png(self.frame_before.crop((20, 5, 50, 8))))
        evidence = {
            "method": HY.METHOD, "strategy": HY.STRATEGY, "mutable_title_excluded": True,
            "capture_source_sha256": "c" * 64,
            "capture_methods": ["PrintWindow2"],
            "baseline_canvas_crop": [0, 0, 100, 70],
            "native_result_contract": {
                "version": "printwindow-native-result-v1", "native_method": "PrintWindow2",
                "start_encoding": "identity-object",
                "capture": {key: ["capture", value] for key, value in {
                    "sha256": "raw_png_sha256", "returned": "returned", "method": "method",
                    "flags": "flags", "width": "width", "height": "height",
                    "pixel_to_physical": "pixel_to_physical_screen"}.items()},
                **{phase: {key: [phase, key] for key in (
                    "pid", "root", "start", "session", "executable", "exe_sha256",
                    "project", "calibration_ok", "physical_rect")}
                   for phase in ("before", "after")},
            },
            "body_template": {"source_image": original, "template": body, "crop": [20, 30, 24, 33],
                              "safe_click_box": [0, 0, 4, 3], "safe_click_offset": [2, 2]},
            "baseline_label": {"source_image": original, "template": baseline_label,
                               "crop": [20, 5, 50, 8]},
            "inspector_label_template": {"source_image": original, "template": self.label,
                                         "crop": [111, 15, 117, 18]},
            "canvas": {"selector": {"class_name": "ActualSyntheticCanvas"},
                       "ancestors": [{"control_type": "Pane"}]},
            "inspector": {
                "row_ancestor_index": 0, "label_semantic": "Bezeichnung",
                "native_locale": "de-DE", "xml_attribute": "Title",
                "selected_header": {"selector": {"name": "Actual native Energy header"},
                                    "ancestors": [{"control_type": "Pane"}]},
                "xml_type": "EnergyManager2",
                "editor": {"selector": {"automation_id": "duplicated-Title",
                                        "control_type": "Edit"},
                           "ancestors": [{"class_name": "ActualSyntheticRow"},
                                         {"name": "Allgemein"}]},
            },
        }
        self.evidence = evidence
        self.queue = Q.Queue(self.root / "queue", self.catalog)
        self.queue.initialize()
        self.queue.admit({
            "schema_version": 1, "kind": "ux-campaign-worker-binding",
            "worker": "notebook", "physical_identity_sha256": "a" * 64,
            "application": Q.APP, "authority": "synthetic-only-not-permission",
            "admitted": True, "valid_until": (EX.utc() + timedelta(hours=1)).isoformat(),
            "admission": self.save("synthetic-admission", {"test_only": True}),
            "resources": {"synthetic": 1}, "capabilities": self.case["needed_capabilities"],
        }, self.root)
        self.run = self.queue.claim("notebook", canary=True)
        scope = {key: self.run[key] for key in (
            "run_id", "worker", "physical_identity_sha256", "definition_sha256")}
        scope.update(application=Q.APP)
        grant = self.save("synthetic-grant", {
            **scope, "resources": {"synthetic": 1}, "scope": "native-property-execution",
            "valid_until": (EX.utc() + timedelta(hours=1)).isoformat(), "test_only": True,
        })
        selectors = self.save("evidence", evidence)
        self.authority_path = Path(__file__).resolve()
        runtime = self.save("runtime", {
            **scope, "method": HY.METHOD, "effect_authority": grant,
            "selector_evidence": selectors, "authority_source_sha256": Q.digest(self.authority_path),
            "executable_path": r"C:\Owned\Config.exe", "document_path": r"C:\Owned\Pilot.Loxone",
        })
        self.handoff = self.queue.handoff(self.run["run_id"], {
            "worker": "notebook", "fixtures": {"property-baseline": str(self.baseline)},
            "artifact_root": str(self.root), "before_xml": {
                "path": self.baseline.name, "sha256": Q.digest(self.baseline)},
            "executable_path": r"C:\Owned\Config.exe", "document_path": r"C:\Owned\Pilot.Loxone",
            "authority_module": str(self.authority_path),
            "authority_source_sha256": Q.digest(self.authority_path),
            "catalog_path": str(self.root / "catalog.json"), "ux_profile": str(self.root / "profile.json"),
            "execution_binding": {"effect_authority": grant, "runtime_binding": runtime,
                                  "selector_evidence": selectors},
        }, self.root / "handoff")
        self.sdk = SyntheticSDK(self.before, self.after)
        self.authority = HybridAuthority(self)

    def save(self, label, value):
        self.index += 1
        path = self.root / (str(self.index) + "-" + label)
        path.write_bytes(value if isinstance(value, bytes) else EX.canonical(value))
        return {"path": path.name, "sha256": Q.digest(path)}

    def tearDown(self):
        shutil.rmtree(self.root)

    def execute(self):
        return EX.Executor(self.queue, self.run["run_id"], self.handoff,
                           self.sdk, self.authority, self.root).execute()

    def test_exact_body_match_handles_changed_title_and_fresh_after_inspector(self):
        result, capture = self.execute()
        self.assertEqual(result["outcome"], "complete", result)
        self.assertNotEqual(png(self.frame_before), png(self.frame_after))
        self.assertIn(("guard", 202, self.case["property"]["after"]), self.authority.expected_seen)
        clicks = [call for call in self.sdk.calls if call["name"] == "Click"]
        self.assertEqual(clicks[0]["arguments"]["loc"], [22, 32])
        self.assertEqual(clicks[2]["arguments"]["loc"], [22, 32])
        self.assertFalse(any(call["name"] == "Screenshot" for call in self.sdk.calls))
        self.assertEqual(self.queue.collect(self.run["run_id"], capture, self.root)["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)

    def test_native_shaped_collector_contract_recomputes_without_recording_native_credit(self):
        self.authority.origin = "native-config-ui"
        result, capture = self.execute()
        self.assertEqual(result["outcome"], "complete", result)
        self.assertEqual(EX.validate_capture(self.case, self.run, capture, self.root,
                                             self.queue.events(), Q), ("observed", True))
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)

    def test_exact_match_rejects_zero_and_multiple_candidates(self):
        template = self.frame_before.crop((20, 30, 24, 33))
        blank = Image.new("RGB", (100, 70), "white")
        self.assertEqual(HY.exact_matches(blank, template, [0, 0, 100, 70]), [])
        blank.paste(template, (20, 30))
        blank.paste(template, (50, 40))
        self.assertEqual(len(HY.exact_matches(blank, template, [0, 0, 100, 70])), 2)

    def test_root_calibration_and_display_mismatch_refuse(self):
        for mutation in ("root", "mapping", "display"):
            with self.subTest(mutation=mutation):
                identity = self.authority.identity(1)
                metadata = {"identity": identity, "mapping": {
                    "identity": identity, "display_inventory_sha256": "display",
                    "pixel_to_physical": [1, 0, 0, 0, 1, 0],
                    "calibration": {"method": "measured-owned-image-v1",
                                    "image_pixels": [[0, 0], [220, 80]],
                                    "physical_points": [[0, 0], [220, 80]]}}}
                if mutation == "root":
                    metadata["identity"] = {**identity, "root": 999}
                elif mutation == "mapping":
                    metadata["mapping"]["calibration"]["physical_points"][1][0] += 1
                else:
                    metadata["mapping"]["display_inventory_sha256"] = "stale"
                with self.assertRaises(ValueError):
                    HY.mapped_frame(metadata, identity, "display", (220, 80))

    def test_wrong_pre_type_value_cannot_reach_type_or_save(self):
        def mutate(receipt):
            if receipt["input"] and receipt["input"]["tool"] == "Type":
                receipt["facts"]["inspector"]["editors"][0]["value"] = "Wrong block"
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertFalse(any(call["name"] in {"Type", "Shortcut"} for call in self.sdk.calls))

    def test_duplicate_qualified_ancestor_or_wrong_type_refuses_before_project_edit(self):
        def mutate(receipt):
            if receipt["input"] and receipt["input"]["selector"] == "property":
                editor = copy.deepcopy(receipt["facts"]["inspector"]["editors"][0])
                receipt["facts"]["inspector"]["editors"].append(editor)
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertFalse(any(call["name"] == "Type" for call in self.sdk.calls))

    def test_duplicate_title_ids_and_ancestor_classes_are_resolved_by_actual_row_label_pixels(self):
        def mutate(receipt):
            receipt["facts"]["inspector"]["editors"][1]["ancestors"] = [
                {"class_name": "ActualSyntheticRow", "runtime_id": [3, 99],
                 "physical_bounds": [110, 45, 100, 30]}, {"name": "Allgemein"}]
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "complete", result)

    def test_changed_effect_authority_refuses_before_launch(self):
        descriptor = self.handoff["execution_binding"]["effect_authority"]
        path = Q.checked_artifact(self.root, descriptor)
        grant = Q.read_json(path)
        grant["valid_until"] = (EX.utc() - timedelta(seconds=1)).isoformat()
        path.write_bytes(EX.canonical(grant))
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "blocked")
        self.assertFalse(any(call["name"] == "PowerShell" for call in self.sdk.calls))
        self.assertFalse(result["actual_native_start"])

    def test_mutable_handoff_repin_cannot_replace_journal_binding(self):
        changed = copy.deepcopy(self.handoff)
        changed["execution_binding"]["effect_authority"]["sha256"] = "0" * 64
        executor = EX.Executor(self.queue, self.run["run_id"], changed, self.sdk,
                               self.authority, self.root)
        with self.assertRaises(ValueError):
            executor.execute()
        self.assertEqual(self.sdk.calls, [])

    def test_user_control_stops_before_launch_and_real_ai_is_valid(self):
        self.sdk.control = "user"
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "blocked")
        self.assertFalse(result["actual_native_start"])
        self.assertFalse(any(call["name"] in {"Click", "Type", "PowerShell"}
                             for call in self.sdk.calls))

    def test_real_ai_control_state_is_not_treated_as_user_preemption(self):
        self.sdk.control = "ai"
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "complete", result)

    def test_foreign_foreground_is_observable_and_target_pointer_acquires_normally(self):
        def mutate(receipt):
            if not receipt["input"] or receipt["input"]["selector"] == "target":
                receipt["facts"]["foreground"] = {"pid": 909, "root": 808, "session": 1}
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "complete", result)
        self.assertFalse(any("SetForegroundWindow" in str(call) for call in self.sdk.calls))

    def test_wrong_physical_hit_refuses_before_rendered_target_click(self):
        def mutate(receipt):
            if receipt["input"] and receipt["input"]["selector"] == "target":
                receipt["facts"]["hit"]["physical_point"][0] += 1
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertFalse(any(call["name"] == "Click" for call in self.sdk.calls))

    def test_missing_measured_calibration_refuses_before_canvas_click(self):
        def mutate(receipt):
            receipt["facts"]["hybrid"]["mapping"].pop("calibration", None)
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertFalse(any(call["name"] == "Click" for call in self.sdk.calls))

    def test_expensive_image_processing_cannot_extend_live_observation_freshness(self):
        executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                               self.sdk, self.authority, self.root)
        original = executor.hybrid.select
        def delayed(*args, **kwargs):
            point = original(*args, **kwargs)
            executor.last_observation_utc = EX.utc() - timedelta(seconds=6)
            return point
        with patch.object(executor.hybrid, "select", side_effect=delayed):
            result, _ = executor.execute()
        self.assertEqual(result["code"], "hybrid-live-observation-expired-before-input")
        self.assertFalse(any(call["name"] == "Click" for call in self.sdk.calls))

    def test_missing_real_preflight_verifier_is_not_replaced_by_source_pin(self):
        self.authority.preflight = None
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "blocked")
        self.assertEqual(self.sdk.calls, [])

    def test_owned_capture_status_one_is_not_success_even_with_iserror_false(self):
        bad = self.save("failed-real-shaped-capture", {
            "isError": False, "structuredContent": {
                "result": "Response: helper refused before capture\nStatus Code: 1"},
        })
        def mutate(receipt):
            receipt["facts"]["owned_capture"]["capture_ack"] = bad
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertFalse(any(call["name"] in {"Click", "Type", "Screenshot"}
                             for call in self.sdk.calls))

    def test_owned_capture_unknown_completion_never_becomes_a_real_image_proof(self):
        unknown = self.save("unknown-shaped-capture", {
            "isError": False, "completion": "UNKNOWN", "structuredContent": {
                "result": 'Response: {"unknown":true}\nStatus Code: 0'},
        })
        def mutate(receipt):
            receipt["facts"]["owned_capture"]["capture_ack"] = unknown
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertFalse(any(call["name"] in {"Click", "Type", "Screenshot"}
                             for call in self.sdk.calls))

    def test_user_handoff_does_not_resume_a_terminal_run_after_pointer_acquisition(self):
        original = self.sdk.request
        def request(method, params, timeout):
            if params["name"] == "ControlStatus" and any(
                call["name"] == "Click" for call in self.sdk.calls
            ):
                self.sdk.control = "user"
            return original(method, params, timeout)
        self.sdk.request = request
        result, capture = self.execute()
        self.assertEqual(result["outcome"], "blocked")
        self.assertFalse(any(call["name"] == "Type" for call in self.sdk.calls))
        self.queue.collect(self.run["run_id"], capture, self.root)
        self.sdk.control = "ready"
        count = len(self.sdk.calls)
        with self.assertRaises(ValueError):
            self.execute()
        self.assertEqual(len(self.sdk.calls), count)

    def test_changed_baseline_image_binding_refuses_before_canvas_click(self):
        def mutate(receipt):
            receipt["facts"]["hybrid"]["baseline_sha256"] = "0" * 64
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertFalse(any(call["name"] == "Click" for call in self.sdk.calls))

    def test_entry_cannot_rebind_private_authority_path_from_sealed_argv(self):
        spec = importlib.util.spec_from_file_location(
            "hybrid_entry_contract", Path(__file__).with_name("ux-experiment.py"))
        entry = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(entry)
        catalog = self.root / "catalog.json"
        catalog.write_bytes(EX.canonical(self.catalog))
        with patch.object(entry, "MCPClient") as client, patch("builtins.print") as output:
            status = entry.main([
                "--queue", str(self.queue.path), "--catalog", str(catalog),
                "--handoff", str(self.root / "handoff" / "handoff.json"),
                "--profile", str(self.root / "profile.json"),
                "--authority", str(self.root / "another-provider.py"),
                "--authority-sha256", Q.digest(self.authority_path),
                "--artifact-root", str(self.root), "--run-id", self.run["run_id"],
            ])
        self.assertEqual(status, 1)
        client.assert_not_called()
        self.assertEqual(EX.parse_json(output.call_args.args[0])["error"]["stage"], "handoff")

    def test_stale_grant_and_mutable_binding_self_repin_are_refused(self):
        executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                               self.sdk, self.authority, self.root)
        descriptor = self.handoff["execution_binding"]["effect_authority"]
        grant = Q.read_json(Q.checked_artifact(self.root, descriptor))
        expired_at = datetime.fromisoformat(grant["valid_until"]) + timedelta(seconds=1)
        with self.assertRaisesRegex(ValueError, "stale"):
            HY.read_bound(executor, at=expired_at)
        executor.handoff["execution_binding"]["effect_authority"] = {
            "path": descriptor["path"], "sha256": "0" * 64}
        with self.assertRaisesRegex(ValueError, "journal anchor"):
            HY.read_bound(executor)

    def test_global_title_cardinality_and_type_are_required_even_with_expected_uuid(self):
        executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                               self.sdk, self.authority, self.root)
        for raw in (
            self.before.replace(b"</R>", b'<C Type="Other" U="other" Title="Energy Manager"/></R>'),
            self.before.replace(b'Type="EnergyManager2"', b'Type="Other"'),
        ):
            with self.subTest(raw=raw), self.assertRaises(EX.Stop):
                executor.correlate_xml(raw, self.case["property"]["before"])

    def test_fresh_inspector_cannot_reuse_before_value_after_title_changed(self):
        def mutate(receipt):
            if receipt["facts"]["identity"]["pid"] == 202:
                receipt["facts"]["inspector"]["editors"][0]["value"] = self.case["property"]["before"]
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertEqual(sum(call["name"] == "Type" for call in self.sdk.calls), 1)

    def test_actual_header_ancestor_writable_label_and_focus_checks_fail_closed(self):
        executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                               self.sdk, self.authority, self.root)
        executor.before_xml = self.before
        self.sdk.launches = 1
        display = {"test_only": True}
        descriptor = self.authority.snapshot(
            self.sdk, {**executor.context, "expected_title": self.case["property"]["before"],
                       "selection_method": HY.METHOD}, "inspector", None,
            {"state": "ready"}, display, None)
        facts = Q.read_json(Q.checked_artifact(self.root, descriptor))["facts"]
        executor.display_hash = EX.sha(EX.canonical(display))
        identity = self.authority.identity(1)
        mutations = [
            lambda value: value["selected_headers"][0]["selector"].update(name="Wrong type"),
            lambda value: value["selected_headers"].append(copy.deepcopy(value["selected_headers"][0])),
            lambda value: value["editors"][0]["ancestors"][0].update(class_name="OtherRow"),
            lambda value: value["editors"][0].update(read_only=True),
            lambda value: value["editors"][0].update(has_keyboard_focus=False),
            lambda value: value.update(label_row_runtime_id=[999]),
            lambda value: value.update(label_crop=[20, 30, 24, 33]),
        ]
        for mutation in mutations:
            altered = copy.deepcopy(facts)
            mutation(altered["inspector"])
            with self.subTest(mutation=mutation), self.assertRaises((ValueError, EX.Stop)):
                HY.inspector(executor, altered, identity, focused=True, retain=False)

    def test_old_v4_catalog_is_preserved_and_refuses_new_source_checkout(self):
        old = Q.ROOT / "tests" / "ux" / "campaigns" / "config-property-pilot-20261110-v4.json"
        with self.assertRaisesRegex(ValueError, "source drift"):
            Q.validate_catalog(Q.read_json(old))

    def test_bad_raw_native_image_hash_refuses_before_graph_or_type(self):
        def mutate(receipt):
            for role in ("owned_capture", "hybrid", "inspector"):
                metadata = receipt["facts"][role]
                ack = Q.read_json(Q.checked_artifact(self.root, metadata["capture_ack"]))
                native = EX.parse_json(ack["structuredContent"]["result"].split("Response: ", 1)[1]
                                       .rsplit("\nStatus Code:", 1)[0])
                native["capture"]["raw_png_sha256"] = "0" * 64
                metadata["capture_ack"] = self.save("bad-hash-ack", {
                    "isError": False, "structuredContent": {
                        "result": "Response: " + json.dumps(native) + "\nStatus Code: 0"}})
        self.authority.modify = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertFalse(any(call["name"] in {"Click", "Type"} for call in self.sdk.calls))

    def test_native_ack_identity_geometry_method_and_success_cannot_be_normalized_away(self):
        executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                               self.sdk, self.authority, self.root)
        self.sdk.launches = 1
        display = {"test_only": True}
        descriptor = self.authority.snapshot(
            self.sdk, {**executor.context, "expected_title": self.case["property"]["before"],
                       "selection_method": HY.METHOD}, "capture", None,
            {"state": "ready"}, display, None)
        facts = Q.read_json(Q.checked_artifact(self.root, descriptor))["facts"]
        executor.display_hash = EX.sha(EX.canonical(display))
        metadata = facts["owned_capture"]
        ack = Q.read_json(Q.checked_artifact(self.root, metadata["capture_ack"]))
        original = EX.parse_json(ack["structuredContent"]["result"].split("Response: ", 1)[1]
                                 .rsplit("\nStatus Code:", 1)[0])
        mutations = [
            lambda value: value["capture"].update(raw_png_sha256="0" * 64),
            lambda value: value["capture"].update(returned=False),
            lambda value: value["capture"].update(method="Other"),
            lambda value: value["capture"].update(flags=0),
            lambda value: value["capture"].update(width=219),
            lambda value: value["capture"].update(height=79),
            lambda value: value["capture"].update(pixel_to_physical_screen=[1, 0, 9, 0, 1, 0]),
            lambda value: value["before"].update(pid=999),
            lambda value: value["after"].update(root=999),
            lambda value: value["after"].update(start={"kind": "kernel-100ns", "value": "999"}),
            lambda value: value["before"].update(project=r"C:\Other\Wrong.Loxone"),
            lambda value: value["after"].update(calibration_ok=False),
            lambda value: value["after"].update(physical_rect=[0, 0, 219, 80]),
        ]
        for mutation in mutations:
            altered = copy.deepcopy(original)
            mutation(altered)
            bad = {**metadata, "capture_ack": self.save("contradictory-ack", {
                "isError": False, "structuredContent": {
                    "result": "Response: " + json.dumps(altered) + "\nStatus Code: 0"}})}
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                HY.owned_image(executor, bad, self.authority.identity(1), self.evidence)

    def test_collector_rejects_actual_click_arguments_different_from_recomputed_proof(self):
        self.authority.origin = "native-config-ui"
        result, capture = self.execute()
        self.assertEqual(result["outcome"], "complete", result)
        events = self.queue.events()
        clicks = [index for index, event in enumerate(events)
                  if event["type"] == "step-intent" and event["intent"]["tool"] == "Click"]
        for index in clicks[:2]:
            changed = copy.deepcopy(events)
            changed[index]["intent"]["arguments"]["loc"][0] += 1
            with self.subTest(index=index), self.assertRaisesRegex(ValueError, "dispatched input arguments"):
                EX.validate_capture(self.case, self.run, capture, self.root, changed, Q)

    def test_body_and_baseline_label_require_same_frame_and_canvas_association(self):
        executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                               self.sdk, self.authority, self.root)
        different = copy.deepcopy(self.evidence)
        different["baseline_label"]["source_image"] = self.save("other-frame", png(self.frame_after))
        with self.assertRaisesRegex(ValueError, "same reviewed frame"):
            HY.template_provenance(executor, different)
        outside = copy.deepcopy(self.evidence)
        outside["baseline_canvas_crop"] = [19, 20, 100, 70]
        with self.assertRaisesRegex(ValueError, "outside"):
            HY.template_provenance(executor, outside)

    def test_safe_click_is_template_relative_and_transformed_not_full_template_center(self):
        specification = {"safe_click_box": [55, 38, 90, 59], "safe_click_offset": [72, 48]}
        self.assertEqual(HY.click_point([865, 280, 100, 68], specification, (1, 1, 0, 0)),
                         [937, 328])
        self.assertEqual(HY.click_point([20, 30, 100, 68], specification, (1.5, 1.5, 10, 20)),
                         [148, 137])
        for invalid in ({}, {"safe_click_box": [-1, 0, 10, 10], "safe_click_offset": [2, 2]},
                        {"safe_click_box": [0, 0, 10, 10], "safe_click_offset": [99, 99]}):
            with self.subTest(invalid=invalid), self.assertRaises(ValueError):
                HY.click_point([0, 0, 100, 68], invalid, (1, 1, 0, 0))


class OptionalImageDependencyTests(unittest.TestCase):
    def test_missing_optional_decoder_refuses_in_preflight_before_launch(self):
        from types import SimpleNamespace
        original_import = __import__
        def without_pillow(name, *args, **kwargs):
            if name == "PIL" or name.startswith("PIL."):
                raise ImportError("Optional image decoder intentionally unavailable")
            return original_import(name, *args, **kwargs)
        launch = unittest.mock.Mock()
        with patch("builtins.__import__", side_effect=without_pillow), (
            self.assertRaisesRegex(ValueError, "Pillow.*before hybrid launch")
        ):
            HY.preflight(SimpleNamespace(launch=launch))
        launch.assert_not_called()


if __name__ == "__main__":
    unittest.main()
