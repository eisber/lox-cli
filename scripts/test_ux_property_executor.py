"""Self-contained synthetic protocol tests; NEVER native execution evidence."""

import base64
import copy
from datetime import timedelta
import importlib.util
import json
from pathlib import Path
import shutil
import sys
import unittest
from unittest.mock import patch
import uuid

sys.path.insert(0, str(Path(__file__).parent))
import ux_property_executor as EX

SPEC = importlib.util.spec_from_file_location("property_test_campaign",
                                             Path(__file__).with_name("ux-campaign.py"))
Q = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(Q)


class SyntheticSDK:
    def __init__(self, before, after):
        self.before, self.after = before, after
        self.launches = 0
        self.calls = []
        self.reopened = after
        self.control = "ready"
        self.failure = None
        self.ps_status = 0

    def request(self, method, params, timeout):
        assert method == "tools/call" and 0 < timeout <= 25
        tool, arguments = params["name"], params["arguments"]
        self.calls.append(copy.deepcopy(params))
        if self.failure == tool:
            raise TimeoutError("synthetic transport timeout")
        if tool == "ControlStatus":
            value = {"state": self.control}
        elif tool == "DisplayInventory":
            value = {"displays": [{"physical_bounds": [0, 0, 2400, 1600], "dpi": 144}]}
        elif tool == "PowerShell":
            command = arguments["command"]
            if "Start-Process" in command:
                self.launches += 1
                result = {"pid": self.launches * 101}
            else:
                reads = sum(call["name"] == "PowerShell" and "ReadAllBytes" in
                            call["arguments"]["command"] for call in self.calls)
                raw = self.before if reads == 1 else self.after if reads == 2 else self.reopened
                result = {"raw_base64": base64.b64encode(raw).decode("ascii")}
            value = {"result": "Response: " + json.dumps(result) +
                     "\nStatus Code: " + str(self.ps_status)}
        elif tool == "Screenshot":
            return {"isError": False, "content": [{
                "type": "image", "mimeType": "image/png",
                "data": base64.b64encode(b"\x89PNG\r\n\x1a\nSYNTHETIC-NOT-NATIVE").decode("ascii"),
            }]}
        else:
            value = {"result": "synthetic acknowledgement"}
        return {"isError": False, "structuredContent": value}


class SyntheticAuthority:
    def __init__(self, test):
        self.test = test
        self.index = 0
        self.mutate = None
        self.origin = "synthetic-test"

    def identity(self, launch):
        return {"pid": launch * 101, "root": launch * 1001,
                "start_identity": {"kind": "kernel-100ns", "value": str(launch * 100000000)},
                "session": 1, "path": r"C:\Owned\Config.exe", "application": Q.APP}

    def snapshot(self, client, context, stage, expected, control, display, intent):
        current = self.identity(client.launches)
        selectors = {
            key: [{"binding": value, "selector": {"control_type": "Edit", "value": "synthetic"},
                   "physical_bounds": [10 + index * 100 + client.launches, 20, 40, 30],
                   "visible": True}]
            for index, (key, value) in enumerate(self.test.case["selectors"].items())
        }
        facts = {"identity": current, "foreground": current, "selectors": selectors,
                 "target_visible": True, "foreground_visible": True}
        if stage in {"initial-open", "fresh-reopen", "closed", "final-closed"}:
            facts["process_count"] = 0
        if stage in {"closed", "final-closed"}:
            facts["original_identity_absent"] = current
        if stage == "loaded":
            facts["document_path"] = self.test.handoff["document_path"]
        if intent and intent["selector"]:
            bounds = selectors[intent["selector"]][0]["physical_bounds"]
            facts["hit"] = {"physical_point": [
                round(bounds[0] + bounds[2] / 2), round(bounds[1] + bounds[3] / 2)],
                "identity": current}
        if stage == "value":
            facts["uia_value"] = {
                "binding": self.test.case["selectors"]["value"],
                "value": self.test.case["property"]["after"],
                "identity": current, "method": "read-only-ValuePattern"}
        receipt = {
            **context, "stage": stage, "input": intent,
            "display_inventory_sha256": EX.sha(EX.canonical(display)),
            "source_utc": EX.utc().isoformat(),
            "source_guards": {"test-only": "synthetic-no-runtime-proof"},
            "policy": {"test-only": "no-policy-change"},
            "control_state": control["state"], "outcome": "available",
            "evidence_origin": self.origin, "facts": facts,
        }
        if self.mutate:
            self.mutate(receipt)
        self.index += 1
        path = self.test.root / f"authority-{self.index}.json"
        EX.durable(path, EX.canonical(receipt))
        return {"path": path.name, "sha256": Q.digest(path)}


class PropertyExecutorTests(unittest.TestCase):
    def setUp(self):
        self.root = Q.ROOT / ".reverse-engineering" / "property-tests" / uuid.uuid4().hex
        self.root.mkdir(parents=True)
        self.catalog = EX.pilot_catalog(Q)
        target = self.catalog["cases"][0]["property"]
        self.before = (f'<Loxone><C Type="{target["type"]}" U="{target["uuid"]}"'
                       ' Title="Energy Manager"><Co><In A="1"/><In A="2"/></Co></C></Loxone>').encode()
        for case in self.catalog["cases"]:
            case["fixture_sha256"] = EX.sha(self.before)
            case["definition_sha256"] = Q.definition_hash(
                {key: value for key, value in case.items() if key != "definition_sha256"})
        self.case = self.catalog["cases"][0]
        self.after = self.before.replace(b'Energy Manager"', b'Energy ManagerPilot"')
        self.after = self.after.replace(b'<In A="1"/><In A="2"/>', b'<In A="2"/><In A="1"/>')
        self.baseline = self.root / "before.Loxone"
        EX.durable(self.baseline, self.before)
        self.queue = Q.Queue(self.root / "queue", self.catalog)
        self.queue.initialize()
        admission = self.root / "admission.json"
        EX.durable(admission, EX.canonical({"test_only": True}))
        self.queue.admit({
            "schema_version": 1, "kind": "ux-campaign-worker-binding",
            "worker": "notebook", "physical_identity_sha256": "a" * 64,
            "application": Q.APP, "authority": "synthetic-test-only",
            "admitted": True, "valid_until": (EX.utc() + timedelta(hours=1)).isoformat(),
            "admission": {"path": admission.name, "sha256": Q.digest(admission)},
            "resources": {"synthetic-desktop": 1},
            "capabilities": self.case["needed_capabilities"],
        }, self.root)
        self.run = self.queue.claim("notebook", canary=True)
        authority_path = Path(__file__).resolve()
        self.handoff = self.queue.handoff(self.run["run_id"], {
            "worker": "notebook", "fixtures": {"property-baseline": str(self.baseline)},
            "artifact_root": str(self.root), "before_xml": {
                "path": self.baseline.name, "sha256": Q.digest(self.baseline)},
            "executable_path": r"C:\Owned\Config.exe", "document_path": r"C:\Owned\Pilot.Loxone",
            "authority_module": str(authority_path),
            "authority_source_sha256": Q.digest(authority_path),
            "catalog_path": str(self.root / "catalog.json"),
            "ux_profile": str(self.root / "private-profile.json"),
        }, self.root / "handoff")
        self.sdk = SyntheticSDK(self.before, self.after)
        self.authority = SyntheticAuthority(self)

    def tearDown(self):
        shutil.rmtree(self.root)

    def execute(self):
        return EX.Executor(self.queue, self.run["run_id"], self.handoff,
                           self.sdk, self.authority, self.root).execute()

    def test_synthetic_end_to_end_and_dynamic_sdk_inputs_are_not_native_credit(self):
        result, capture = self.execute()
        self.assertEqual(result["outcome"], "complete")
        self.assertTrue(result["actual_native_start"])
        self.assertFalse(result["semantic_pass"])
        self.assertNotEqual(result["old_identity"]["pid"], result["fresh_identity"]["pid"])
        inputs = [call for call in self.sdk.calls if call["name"] in {"Click", "Type", "Shortcut"}]
        self.assertEqual([call["name"] for call in inputs],
                         ["Click", "Click", "Type", "Shortcut", "Shortcut", "Click", "Click", "Shortcut"])
        self.assertEqual(inputs[0]["arguments"]["loc"], [31, 35])
        self.assertEqual(inputs[5]["arguments"]["loc"], [32, 35])
        collected = self.queue.collect(self.run["run_id"], capture, self.root)
        self.assertEqual(collected["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)
        self.assertTrue(self.queue.state()[0]["notebook"]["parked"])

    def test_timeout_keeps_intent_and_never_replays(self):
        self.sdk.failure = "Type"
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "UNKNOWN")
        self.assertTrue(result["actual_native_start"])
        self.assertEqual(result["stage"], "sdk.Type")
        diagnostic = Q.read_json(Q.checked_artifact(self.root, result["diagnostic"]))
        self.assertEqual(diagnostic["type"], "TimeoutError")
        self.assertEqual(diagnostic["stage"], "sdk.Type")
        intents = [event for event in self.queue.events() if event["type"] == "step-intent"]
        acks = [event for event in self.queue.events() if event["type"] == "step-ack"]
        self.assertEqual(len(intents), len(acks) + 1)
        count = len(self.sdk.calls)
        with self.assertRaises(ValueError):
            self.execute()
        self.assertEqual(len(self.sdk.calls), count)
        self.assertTrue(self.queue.state()[0]["notebook"]["parked"])
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)

    def test_constructed_executor_cannot_enter_after_current_dispatch_becomes_unknown(self):
        executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                               self.sdk, self.authority, self.root)
        self.queue.unknown(self.run["run_id"], "Actual external quarantine before execute")
        with patch.object(executor, "perform") as perform:
            with self.assertRaises(ValueError):
                executor.execute()
        perform.assert_not_called()
        self.assertEqual(self.sdk.calls, [])
        self.assertFalse(any(event["type"] == "execution-start" for event in self.queue.events()))

    def test_execution_admission_and_start_share_existing_writer_lock(self):
        executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                               self.sdk, self.authority, self.root)
        original_guard, original_append = self.queue._execution_run, self.queue.append
        checked = []
        def guarded(run_id):
            self.assertTrue((self.queue.path / "writer.lock").exists())
            checked.append("current-state")
            return original_guard(run_id)
        def append(kind, **fields):
            if kind == "execution-start":
                self.assertTrue((self.queue.path / "writer.lock").exists())
                checked.append("start")
            return original_append(kind, **fields)
        with patch.object(self.queue, "_execution_run", side_effect=guarded), (
            patch.object(self.queue, "append", side_effect=append)
        ), patch.object(executor, "perform", side_effect=EX.Stop("reject", "local-precondition")):
            result, capture = executor.execute()
        self.assertEqual(checked, ["current-state", "start"])
        self.assertEqual(result["outcome"], "reject")
        self.assertEqual(self.queue.collect(self.run["run_id"], capture, self.root)["status"], "blocked")

    def test_human_takeover_before_start_is_blocked_not_native(self):
        self.sdk.control = "user"
        result, capture = self.execute()
        self.assertEqual(result["outcome"], "blocked")
        self.assertIs(result["actual_native_start"], False)
        self.assertFalse(any(call["name"] in {"Click", "Type", "Shortcut", "PowerShell"}
                             for call in self.sdk.calls))
        collected = self.queue.collect(self.run["run_id"], capture, self.root)
        self.assertEqual(collected["status"], "blocked")
        self.assertIs(collected["native_started"], False)
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)
        self.assertEqual(self.queue.stats()["all_completions"], 0)
        self.assertTrue(self.queue.state()[0]["notebook"]["parked"])

    def test_synthetic_pre_effect_block_is_not_authoritative_closeout(self):
        self.authority.mutate = lambda receipt: receipt["facts"].update(process_count=1)
        result, capture = self.execute()
        self.assertEqual(result["outcome"], "blocked")
        self.assertEqual(result["evidence_origin"], "synthetic-test")
        collected = self.queue.collect(self.run["run_id"], capture, self.root)
        self.assertEqual(collected["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)

    def test_no_effect_closeout_rejects_unacknowledged_effect_intent(self):
        self.sdk.control = "user"
        _, capture = self.execute()
        with self.queue.locked():
            self.queue.append("step-intent", run_id=self.run["run_id"], step=1,
                              effect=True, intent={"tool": "Click", "arguments": {}})
        collected = self.queue.collect(self.run["run_id"], capture, self.root)
        self.assertEqual(collected["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)

    def test_definitely_no_effect_claim_cannot_hide_acknowledged_effect(self):
        executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                               self.sdk, self.authority, self.root)
        def malformed_producer():
            executor.tool("Shortcut", {"shortcut": "ctrl+s"}, effect=True)
            executor.native_start = False
            raise EX.Stop("blocked", "false-no-effect-claim")
        with patch.object(executor, "perform", side_effect=malformed_producer):
            _, capture = executor.execute()
        self.assertEqual(self.queue.collect(self.run["run_id"], capture, self.root)["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)

    def test_no_effect_closeout_rejects_malformed_protocol_binding(self):
        self.sdk.control = "user"
        _, capture = self.execute()
        capture["definition_sha256"] = "0" * 64
        self.assertEqual(self.queue.collect(self.run["run_id"], capture, self.root)["status"], "UNKNOWN")
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)

    def test_nonzero_powershell_status_cannot_open_or_complete(self):
        self.sdk.ps_status = 1
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "UNKNOWN")
        self.assertIsNone(result["actual_native_start"])
        self.assertFalse(any(call["name"] == "Click" for call in self.sdk.calls))

    def test_saved_reopened_raw_mismatch_is_failure_and_both_preserved(self):
        self.sdk.reopened += b"\r\n"
        result, capture = self.execute()
        self.assertEqual(result["outcome"], "fail")
        for kind, raw in (("saved-xml", self.after), ("reopened-xml", self.sdk.reopened)):
            artifact = next(item for item in capture["artifacts"] if item["kind"] == kind)
            self.assertEqual(Q.checked_artifact(self.root, artifact).read_bytes(), raw)

    def test_stale_authority_facts_rejected_before_launch(self):
        self.authority.mutate = lambda receipt: receipt.update(
            source_utc=(EX.utc() - timedelta(seconds=10)).isoformat())
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertIs(result["actual_native_start"], False)

    def test_worker_version_and_case_bindings_are_each_required(self):
        for key, value in (("worker", "lab-a"), ("application", {}),
                           ("definition_sha256", "0" * 64)):
            with self.subTest(key=key):
                executor = EX.Executor(self.queue, self.run["run_id"], self.handoff,
                                       self.sdk, self.authority, self.root)
                executor.directory = self.root / key
                executor.directory.mkdir(exist_ok=True)
                self.authority.mutate = lambda receipt, k=key, v=value: receipt.update({k: v})
                with self.assertRaises(EX.Stop) as raised:
                    executor.facts("initial-open")
                self.assertEqual(raised.exception.code, "authority-case-worker-version-binding")

    def test_changed_process_root_rejected_before_next_input(self):
        def mutate(receipt):
            if receipt["stage"] == "guard":
                receipt["facts"]["identity"]["root"] += 1
        self.authority.mutate = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertFalse(any(call["name"] in {"Click", "Type"} for call in self.sdk.calls))

    def test_invisible_foreground_root_blocks_before_any_input(self):
        def mutate(receipt):
            if receipt["stage"] == "guard":
                receipt["facts"]["foreground_visible"] = False
        self.authority.mutate = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "blocked")
        self.assertEqual(result["code"], "AMBIGUOUS_WINDOW_ROOT")
        self.assertEqual(result["blocked_diagnostics"]["classification"], "AMBIGUOUS")
        self.assertFalse(any(call["name"] in {"Click", "Type"} for call in self.sdk.calls))

    def test_same_pid_hidden_root_is_ambiguous_and_retains_readonly_diagnostics(self):
        relations = {
            "evidence_origin": "synthetic-test", "owner_chain": [9001, 1001],
            "ancestor_chain": [9001], "input_focus": 9002,
            "visible_hit_root": 1001, "window_class": "SyntheticHiddenOwner",
            "ui_state": "owner-relationship-not-proved",
        }
        def mutate(receipt):
            if receipt["stage"] == "guard" and receipt["input"]:
                receipt["facts"]["foreground"] = {**receipt["facts"]["identity"], "root": 9001}
                receipt["facts"]["foreground_visible"] = False
                receipt["facts"]["window_relationships"] = relations
        self.authority.mutate = mutate
        result, _ = self.execute()
        self.assertEqual(result["code"], "AMBIGUOUS_WINDOW_ROOT")
        self.assertEqual(result["outcome"], "blocked")
        self.assertTrue(result["actual_native_start"])
        self.assertEqual(result["blocked_diagnostics"]["read_only_window_diagnostics"], relations)
        self.assertFalse(any(call["name"] in {"Click", "Type", "Shortcut"} for call in self.sdk.calls))

    def test_same_pid_different_visible_root_cannot_be_authorized_by_owner_boolean(self):
        def mutate(receipt):
            if receipt["stage"] == "guard" and receipt["input"]:
                receipt["facts"]["foreground"] = {**receipt["facts"]["identity"], "root": 9001}
                receipt["facts"]["window_relationships"] = {"verified_owner_relation": True}
        self.authority.mutate = mutate
        result, _ = self.execute()
        self.assertEqual(result["code"], "AMBIGUOUS_WINDOW_ROOT")
        self.assertFalse(any(call["name"] == "Click" for call in self.sdk.calls))

    def test_uia_hittest_guard_cannot_be_replaced_by_coordinates(self):
        def mutate(receipt):
            if receipt["stage"] == "guard" and receipt["input"]:
                receipt["facts"]["hit"]["identity"]["root"] += 1
        self.authority.mutate = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")

    def test_normal_focus_click_allowed_with_observed_other_foreground_not_forced_api(self):
        def mutate(receipt):
            if receipt["stage"] == "guard" and receipt["input"] and (
                receipt["input"]["selector"] == "target"
            ):
                receipt["facts"]["foreground"] = {"pid": 909, "root": 808, "session": 1}
        self.authority.mutate = mutate
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "complete")
        self.assertFalse(any("SetForegroundWindow" in str(call) for call in self.sdk.calls))

    def test_same_pid_reopen_rejected(self):
        original = self.authority.identity
        def identity(launch):
            value = original(launch)
            if launch == 2:
                value["pid"] = 101
            return value
        self.authority.identity = identity
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "reject")
        self.assertIn(result["code"], {"ordinary-launch-document-or-pid-mismatch",
                                      "fresh-process-reopen-required"})

    def test_full_xml_comparison_preserves_text_attribute_and_record_order(self):
        before = b'<R a="1" b="2"> before <In X="1"/><In X="2"/></R>'
        after = b'<R b="2" a="1"> after <In X="2"/><In X="1"/></R>'
        diff = EX.compare_xml(before, after)
        self.assertEqual(len(diff["changes"]), 3)
        self.assertEqual(diff["changes"][0]["before"]["attributes"], [["a", "1"], ["b", "2"]])
        self.assertEqual(diff["changes"][0]["after"]["attributes"], [["b", "2"], ["a", "1"]])
        self.assertEqual(diff["behavioral_equivalence"], "UNPROVEN")
        self.assertFalse(EX.compare_xml(b"<R/>\r\n", b"<R/>\n")["raw_bytes_equal"])

    def test_digit_prefixed_attributes_preserve_unknown_data_order_and_values(self):
        raw = (b'\xef\xbb\xbf<?xml version="1.0" encoding="utf-8"?>'
               b'<R z="first" 12hTF="1" unknown="12hTF &amp; &lt;C 12hTF=&quot;x&quot;&gt;">'
               b'<!-- <C 12hTF="comment"/> -->'
               b'<![CDATA[<C 12hTF="cdata"/>]]><?keep 12hTF="pi"?>'
               b'<C 9custom="unchanged" last="z"/></R>')
        root, records = EX.xml_nodes(raw)
        self.assertEqual(list(root.attrib), ["z", "12hTF", "unknown"])
        self.assertEqual(root.get("unknown"), '12hTF & <C 12hTF="x">')
        self.assertEqual(records["0/0"]["text"], ' <C 12hTF="comment"/> ')
        self.assertEqual(records["0/0"]["tail"], '<C 12hTF="cdata"/>')
        self.assertEqual(records["0/1"]["text"], 'keep 12hTF="pi"')
        self.assertEqual(list(root[-1].attrib.items()), [("9custom", "unchanged"), ("last", "z")])
        changed = raw.replace(b'12hTF="1"', b'12hTF="2"')
        diff = EX.compare_xml(raw, changed)
        self.assertEqual(len(diff["changes"]), 1)
        self.assertEqual(diff["before_sha256"], EX.sha(raw))
        self.assertEqual(diff["changes"][0]["before"]["attributes"][1], ["12hTF", "1"])
        self.assertEqual(diff["changes"][0]["after"]["attributes"][1], ["12hTF", "2"])

    def test_digit_attribute_adapter_rejects_collisions_entities_and_invalid_markup(self):
        for raw in (
            b'<R 12hTF="1" __lox_digit_attribute_12hTF="2"/>',
            b'<R 12hTF="1" 12hTF="2"/>',
            b'<!DOCTYPE R [<!ENTITY x "test">]><R 12hTF="1">&x;</R>',
            b'<R 12hTF="1" broken/>',
            b'<12Tag x="1"/>',
        ):
            with self.subTest(raw=raw), self.assertRaises((ValueError, EX.ET.ParseError)):
                EX.xml_nodes(raw)
        self.assertEqual(EX.xml_nodes(b'<R a="&lt;!DOCTYPE harmless"/>')[0].get("a"),
                         "<!DOCTYPE harmless")

    def test_existing_energy_dialect_fixture_is_readable_without_dropping_sentinel(self):
        raw = (Q.ROOT / "tests" / "ux" / "energy-manager2" / "synthetic.Loxone").read_bytes()
        root, _ = EX.xml_nodes(raw)
        sentinel = next(node for node in root.iter() if node.get("Type") == "Note")
        self.assertEqual(sentinel.get("12hTF"), "true")
        self.assertEqual(sentinel.get("Text"), "Keep me unchanged")
        target = next(node for node in root.iter() if node.get("Type") == "EnergyManager2")
        self.assertEqual(EX.property_value(raw, {
            "uuid": target.get("U"), "type": target.get("Type"), "attribute": "Title",
        }), target.get("Title"))
        self.assertEqual(EX.compare_xml(raw, raw)["changes"], [])

    def test_authority_import_uses_same_verified_bytes_even_if_path_changes(self):
        path = self.root / "private-authority.py"
        verified = b'VALUE = "reviewed"\n'
        path.write_bytes(verified)
        original_read = Path.read_bytes
        reads = []
        def mutate_after_read(source):
            raw = original_read(source)
            if source == path:
                reads.append(raw)
                source.write_bytes(b'VALUE = "unreviewed"\n')
            return raw
        with patch.object(Path, "read_bytes", mutate_after_read):
            authority = EX.load_authority(path, EX.sha(verified))
        self.assertEqual(authority.VALUE, "reviewed")
        self.assertEqual(reads, [verified])

    def test_authority_pin_refuses_before_import_side_effect(self):
        path = self.root / "private-authority.py"
        marker = self.root / "imported"
        path.write_text(f"from pathlib import Path\nPath({str(marker)!r}).touch()\n")
        with self.assertRaises(ValueError):
            EX.load_authority(path, "0" * 64)
        self.assertFalse(marker.exists())

    def test_unknown_receipt_cannot_complete_even_with_native_origin_label(self):
        _, capture = self.execute()
        capture["evidence_origin"] = "native-config-ui"
        descriptor = next(item for item in capture["artifacts"] if item["kind"] == "native-receipt")
        path = Q.checked_artifact(self.root, descriptor)
        result = json.loads(path.read_bytes())
        result.update(actual_native_start=None, evidence_origin="native-config-ui")
        path.write_bytes(EX.canonical(result))
        descriptor["sha256"] = Q.digest(path)
        capture["result_sha256"] = descriptor["sha256"]
        with self.assertRaises(ValueError):
            EX.validate_capture(self.case, self.run, capture, self.root, self.queue.events(), Q)

    def test_native_schema_contract_only_rechecks_complete_without_recording_native_credit(self):
        # A mocked native-shaped envelope tests parsing, never native execution.
        self.authority.origin = "native-config-ui"
        _, capture = self.execute()
        self.assertEqual(EX.validate_capture(
            self.case, self.run, capture, self.root, self.queue.events(), Q), ("observed", True))
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)

    def test_collector_rejects_replaced_native_after_bytes(self):
        self.authority.origin = "native-config-ui"
        _, capture = self.execute()
        item = next(item for item in capture["artifacts"] if item["kind"] == "reopened-xml")
        path = Q.checked_artifact(self.root, item)
        path.write_bytes(path.read_bytes() + b"\n")
        item["sha256"] = Q.digest(path)
        with self.assertRaisesRegex(ValueError, "Native after hash mismatch"):
            EX.validate_capture(self.case, self.run, capture, self.root, self.queue.events(), Q)
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)

    def test_raw_native_pin_never_uses_source_lf_normalization(self):
        self.baseline.write_bytes(self.before + b"\r\n")
        result, _ = self.execute()
        self.assertEqual(result["outcome"], "UNKNOWN")
        self.assertIs(result["actual_native_start"], False)
        self.assertEqual(self.sdk.calls, [])

    def test_existing_profile_pin_conflict_parks_before_transport_initialization(self):
        spec = importlib.util.spec_from_file_location(
            "property_entry_test", Path(__file__).with_name("ux-experiment.py"))
        entry = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(entry)
        token = self.root / "token"
        token.write_text("synthetic-no-credentials", encoding="ascii")
        catalog = self.root / "catalog.json"
        catalog.write_bytes(EX.canonical(self.catalog))
        profile = self.root / "profile.json"
        profile.write_bytes(EX.canonical({
            "schema_version": 1, "mcp_url": "https://localhost",
            "token_file": str(token), "backend_path": r"C:\Owned\backend.ps1",
            "remote_artifact_root": r"C:\Owned\Artifacts",
            "expected_exe_sha256": "0" * 64,
        }))
        with patch.object(entry, "MCPClient") as client, patch("builtins.print") as output:
            result = entry.main([
                "--queue", str(self.queue.path), "--catalog", str(catalog),
                "--handoff", str(self.root / "handoff" / "handoff.json"),
                "--profile", str(profile), "--authority", str(Path(__file__)),
                "--authority-sha256", Q.digest(Path(__file__)),
                "--artifact-root", str(self.root), "--run-id", self.run["run_id"],
            ])
        self.assertEqual(result, 1)
        client.assert_not_called()
        self.assertEqual(self.queue.state()[1][self.run["run_id"]]["status"], "UNKNOWN")
        response = json.loads(output.call_args.args[0])
        self.assertEqual(response["error"], {"type": "ValueError", "code": None, "stage": "profile"})
        details = Q.read_json(Q.checked_artifact(self.root, response["diagnostic"]))
        self.assertIn("ValueError", details["traceback"])

    def test_entry_retains_typed_failure_and_journal_closeout_failure(self):
        spec = importlib.util.spec_from_file_location(
            "property_entry_closeout_test", Path(__file__).with_name("ux-experiment.py"))
        entry = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(entry)
        catalog = self.root / "catalog.json"
        catalog.write_bytes(EX.canonical(self.catalog))
        with patch.object(entry.Executor, "queue_tool", return_value=Q), (
            patch.object(Q.Queue, "unknown", side_effect=OSError("synthetic writer failure"))
        ), patch.object(entry, "MCPClient") as client, patch("builtins.print") as output:
            result = entry.main([
                "--queue", str(self.queue.path), "--catalog", str(catalog),
                "--handoff", str(self.root / "handoff" / "handoff.json"),
                "--profile", str(self.root / "missing-profile.json"),
                "--authority", str(Path(__file__)), "--authority-sha256", Q.digest(Path(__file__)),
                "--artifact-root", str(self.root), "--run-id", self.run["run_id"],
            ])
        self.assertEqual(result, 1)
        client.assert_not_called()
        response = json.loads(output.call_args.args[0])
        details = Q.read_json(Q.checked_artifact(self.root, response["diagnostic"]))
        self.assertEqual(details["type"], "FileNotFoundError")
        self.assertEqual(details["stage"], "profile")
        self.assertEqual(details["journal_closeout_error"]["type"], "OSError")

    def test_pilot_catalog_is_separate_and_predeclares_five_inputs(self):
        catalog = EX.pilot_catalog(Q)
        self.assertEqual(len(catalog["cases"]), 5)
        self.assertEqual(len({case["id"] for case in catalog["cases"]}), 5)
        self.assertTrue(all(case["id"].startswith("pilot-title-") for case in catalog["cases"]))
        self.assertEqual(Q.validate_catalog(catalog), catalog)
        self.assertEqual(len(self.queue.qualification_plan()), 3)
        self.assertEqual(self.queue.stats()["all_native_attempts"], 0)
        path = Q.ROOT / "tests" / "ux" / "campaigns" / "config-property-pilot-20261110-v4.json"
        if path.exists():
            self.assertEqual(Q.read_json(path), catalog)


if __name__ == "__main__":
    unittest.main()
