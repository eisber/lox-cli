"""One bounded property roundtrip through the existing SDK, not a GUI authority.

The injected, privately pinned authority supplies live read-only UIA/OS facts
and verifies its existing policy/source/ownership receipts. It never supplies
input commands. All input here uses the normal SDK and its unchanged gates.
"""

import base64
from datetime import datetime, timedelta, timezone
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import time
import traceback
import xml.etree.ElementTree as ET

from loxone_ux_transport import parse_json


ADAPTER = "sdk-property-roundtrip-v1"
OUTCOMES = {"complete", "reject", "fail", "blocked", "UNKNOWN"}


class Stop(Exception):
    def __init__(self, outcome, code, details=None):
        super().__init__(code)
        self.outcome, self.code = outcome, code
        self.details = details


def utc():
    return datetime.now(timezone.utc)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=True,
                      separators=(",", ":"), allow_nan=False).encode("ascii")


def durable(path, raw):
    with Path(path).open("xb") as stream:
        stream.write(raw)
        stream.flush()
        os.fsync(stream.fileno())


def error_details(error, stage):
    return {
        "type": type(error).__name__, "code": getattr(error, "code", None),
        "stage": stage, "details": getattr(error, "details", None),
        "provider_artifact": getattr(error, "artifact", None),
        "traceback": traceback.format_exc(),
    }


def xml_nodes(raw):
    if b"<!DOCTYPE" in raw.upper() or b"<!ENTITY" in raw.upper():
        raise ValueError("DTD/entity-bearing XML is not a pilot fixture.")
    parser = ET.XMLParser(target=ET.TreeBuilder(insert_comments=True, insert_pis=True))
    root = ET.fromstring(raw, parser=parser)
    records = {}

    def tag_name(node):
        return node.tag if isinstance(node.tag, str) else (
            "#comment" if node.tag is ET.Comment else "#processing-instruction")

    def walk(node, path):
        records[path] = {
            "tag": tag_name(node), "attributes": [list(item) for item in node.attrib.items()],
            "text": node.text, "tail": node.tail,
            "children": [
                {"tag": tag_name(child),
                 "attributes": [list(item) for item in child.attrib.items()]}
                for child in node
            ],
        }
        for index, child in enumerate(node):
            walk(child, path + "/" + str(index))

    walk(root, "0")
    return root, records


def compare_xml(before, after):
    """Keep positional attributes, text/tails, attribute and child order."""
    _, a = xml_nodes(before)
    _, b = xml_nodes(after)
    paths = list(a) + [path for path in b if path not in a]
    return {
        "before_sha256": sha(before), "after_sha256": sha(after),
        "raw_bytes_equal": before == after,
        "changes": [
            {"path": path, "before": a.get(path), "after": b.get(path)}
            for path in paths if a.get(path) != b.get(path)
        ],
        "raw_artifacts_are_authoritative": True,
        "behavioral_equivalence": "UNPROVEN",
    }


def property_value(raw, target):
    root, _ = xml_nodes(raw)
    matches = [node for node in root.iter()
               if node.get("U") == target["uuid"] and node.get("Type") == target["type"]]
    if len(matches) != 1:
        raise ValueError("Expected exactly one UUID/type-bound XML target.")
    return matches[0].get(target["attribute"])


def validate_case(case):
    if case.get("adapter") != ADAPTER or case.get("phase") != "configuration":
        raise ValueError("Only the separately declared property pilot is supported.")
    target = case["property"]
    if (set(target) != {"uuid", "type", "attribute", "before", "after"}
            or target["attribute"] != "Title"
            or any(not isinstance(value, str) or not value or "\0" in value
                   for value in target.values())
            or target["before"] == target["after"]):
        raise ValueError("Pilot requires a single nonempty UUID/type-bound Title change.")
    if set(case["selectors"]) != {"target", "property", "value"} or any(
        not isinstance(selector, dict) or not selector
        or set(selector) - {"name", "automation_id", "control_type", "xml_uuid", "role"}
        or any(not isinstance(value, str) or not value for value in selector.values())
        for selector in case["selectors"].values()
    ):
        raise ValueError("Three declarative selector bindings are required, not coordinates.")
    if not re.fullmatch(r"[a-f0-9]{64}", case["fixture_sha256"]):
        raise ValueError("Raw fixture pin is required.")
    if not 1 <= case["timeout_seconds"] <= 600:
        raise ValueError("Pilot input deadline must be bounded.")
    return case


def pilot_catalog(campaign):
    """Five authored inputs, zero native executions; never modify mixed100."""
    cases = []
    for suffix, value in (
        ("ascii", "Energy ManagerPilot"),
        ("unicode", "Energy Manager Wärme"),
        ("spaces", "Energy Manager Pilot Room"),
        ("punctuation", "Energy Manager (Pilot-4)"),
        ("long", "Energy Manager Pilot Persistence Observation Five"),
    ):
        case = {
            "id": "pilot-title-" + suffix + "-20261109-v3",
            "phase": "configuration", "family": "EnergyManager2",
            "application": dict(campaign.APP), "adapter": ADAPTER,
            "executor_compatible": True,
            "canary_for": ADAPTER if suffix == "ascii" else None,
            "needed_capabilities": ["sdk-property-roundtrip", "guarded-native-input", "owned-capture"],
            "fixture_alias": "property-baseline",
            "fixture_sha256": "56df38960f6c6b27bb12bb00d7de78e552bb77588da2f380fcf15742711901b9",
            "timeout_seconds": 300, "field_bindings": ["@Title"],
            "property": {"uuid": "1ea20af3-0284-bf5d-ffff741afa8d2a84",
                         "type": "EnergyManager2", "attribute": "Title",
                         "before": "Energy Manager", "after": value},
            "selectors": {
                "target": {"role": "target-block",
                           "xml_uuid": "1ea20af3-0284-bf5d-ffff741afa8d2a84"},
                "property": {"role": "property-label", "name": "Bezeichnung"},
                "value": {"role": "property-value", "control_type": "Edit"},
            },
            "expectation": "Observe Title persistence only; all unrelated XML changes remain unexplained.",
        }
        case["definition_sha256"] = campaign.definition_hash(case)
        cases.append(case)
    return {
        "schema_version": 1, "kind": "ux-property-pilot",
        "id": "config-property-pilot-20261109-v3", "application": dict(campaign.APP),
        "mix": {"configuration": 5, "simulator": 0},
        "executor_sources": [campaign.source("scripts\\" + name) for name in (
            "ux_property_executor.py", "ux-experiment.py", "ux-campaign.py",
            "loxone-ux.py", "loxone_ux_transport.py")],
        "native_qualification": "UNKNOWN; private live authority and per-worker canary required.",
        "whole_product_behavior_percentage": None, "cases": cases,
    }


def load_authority(path, expected_sha):
    """Load explicitly trusted private code; a digest alone grants no authority."""
    path = Path(path).resolve()
    if sha(path.read_bytes()) != expected_sha:
        raise ValueError("Private authority source changed.")
    spec = importlib.util.spec_from_file_location("private_ux_authority", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def ps_literal(value):
    return "'" + value.replace("'", "''") + "'"


class Executor:
    def __init__(self, queue, run_id, handoff, client, authority, artifact_root):
        self.queue, self.run_id, self.handoff = queue, run_id, handoff
        self.client, self.authority = client, authority
        self.root = self.queue_tool().private_destination(artifact_root)
        self.run = queue.state()[1][run_id]
        self.case = validate_case(queue.cases[self.run["case_id"]])
        self.context = {key: self.run[key] for key in (
            "run_id", "worker", "physical_identity_sha256", "definition_sha256")}
        self.context["application"] = queue.catalog["application"]
        self.context["resources"] = queue.state()[0][self.run["worker"]]["binding"]["resources"]
        self.context["input_deadline_utc"] = (
            utc() + timedelta(seconds=self.case["timeout_seconds"])).isoformat()
        self.until = time.monotonic() + self.case["timeout_seconds"]
        self.artifacts, self.steps = [], []
        self.native_start = False
        self.pending = False
        self.origin = None
        self.old = self.fresh = None
        self.stage = "constructed"
        self.directory = self.root / ("property-" + run_id)

    def event(self, kind, **fields):
        with self.queue.locked():
            return self.queue.append(kind, run_id=self.run_id, **fields)

    def artifact(self, name, raw, kind):
        path = self.directory / name
        durable(path, raw)
        descriptor = {"path": str(path.relative_to(self.root)),
                      "sha256": sha(raw), "kind": kind}
        self.artifacts.append(descriptor)
        return descriptor

    def remaining(self):
        left = self.until - time.monotonic()
        if left <= 0:
            raise Stop("UNKNOWN" if self.pending else "blocked", "input-deadline")
        return min(left, 25)

    def tool(self, name, arguments, effect=False):
        self.stage = "sdk." + name
        timeout = self.remaining()
        intent = {"tool": name, "arguments": arguments,
                  "input_deadline_utc": self.context["input_deadline_utc"]}
        self.queue.execution_intent(self.run_id, step=len(self.steps), effect=effect, intent=intent)
        self.pending = True
        if effect and self.native_start is False:
            self.native_start = None
        reply = self.client.request("tools/call", {"name": name, "arguments": arguments},
                                    timeout=timeout)
        artifact = self.artifact(f"{len(self.steps):03d}-reply.json", canonical(reply), "sdk-reply")
        self.event("step-ack", step=len(self.steps), reply=artifact)
        self.pending = False
        self.steps.append({"tool": name, "reply": artifact})
        if not isinstance(reply, dict) or reply.get("isError") is not False:
            raise Stop("UNKNOWN", "sdk-error-or-missing-ack")
        if reply.get("completion") in {"UNKNOWN", "pending"}:
            raise Stop("UNKNOWN", "sdk-completion-unknown")
        return reply

    def structured(self, reply):
        value = reply.get("structuredContent")
        if not isinstance(value, dict):
            texts = [part["text"] for part in reply.get("content", [])
                     if part.get("type") == "text"]
            if len(texts) != 1:
                raise Stop("UNKNOWN", "missing-structured-receipt")
            value = parse_json(texts[0])
        return value

    def powershell(self, source, effect=False):
        reply = self.tool("PowerShell", {"command": source,
                                        "timeout": max(1, int(self.remaining()))}, effect)
        value = self.structured(reply).get("result")
        match = re.fullmatch(r"Response: (.*)\r?\nStatus Code:\s*(-?\d+)\s*",
                             value or "", re.DOTALL)
        if match is None or int(match.group(2)) != 0:
            raise Stop("UNKNOWN", "powershell-actual-status-not-zero")
        return parse_json(match.group(1))

    def facts(self, stage, expected=None, intent=None):
        control = self.structured(self.tool("ControlStatus", {}))
        if control.get("state") not in {"ready", "ai"}:
            raise Stop("blocked", "USER_CONTROL")
        display = self.structured(self.tool("DisplayInventory", {}))
        # This callback is the existing trusted authority's read-only observer,
        # not a caller-written admission flag. It must revalidate real policy,
        # identity, source guards and ownership and retain its native receipt.
        self.stage = "authority." + stage
        descriptor = self.authority.snapshot(
            self.client, dict(self.context), stage, expected, control, display, intent)
        receipt_path = self.queue_tool().checked_artifact(self.root, descriptor)
        raw = receipt_path.read_bytes()
        receipt = parse_json(raw)
        required = {**self.context, "stage": stage, "input": intent,
                    "display_inventory_sha256": sha(canonical(display))}
        if any(receipt.get(key) != value for key, value in required.items()):
            raise Stop("reject", "authority-case-worker-version-binding")
        observed = datetime.fromisoformat(receipt["source_utc"])
        if observed.tzinfo is None or not 0 <= (utc() - observed).total_seconds() <= 5:
            raise Stop("reject", "stale-live-authority-receipt")
        if (not receipt.get("source_guards") or not receipt.get("policy")
                or receipt.get("control_state") != control["state"]):
            raise Stop("reject", "missing-live-source-policy-control")
        if receipt.get("outcome") != "available":
            raise Stop("blocked", "external-authority-unavailable")
        if receipt.get("evidence_origin") not in {"native-config-ui", "synthetic-test"}:
            raise Stop("reject", "unknown-evidence-origin")
        if self.origin is not None and receipt["evidence_origin"] != self.origin:
            raise Stop("reject", "evidence-origin-changed")
        self.origin = receipt["evidence_origin"]
        retained = self.artifact(f"facts-{len(self.steps):03d}-{stage}.json",
                                 raw, "authority-receipt")
        self.event("checkpoint", stage=stage, facts=retained)
        facts = receipt["facts"]
        if expected is not None and facts.get("identity") != expected:
            raise Stop("reject", "target-process-or-root-changed")
        return facts

    @staticmethod
    def queue_tool():
        # Avoid an import cycle: queue loads this adapter only for pilot data.
        spec = importlib.util.spec_from_file_location(
            "property_campaign", Path(__file__).with_name("ux-campaign.py"))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def identity(self, facts):
        identity = facts.get("identity", {})
        if (type(identity.get("pid")) is not int or identity["pid"] <= 0
                or type(identity.get("root")) is not int or identity["root"] <= 0
                or not identity.get("start_identity") or not identity.get("session")
                or not identity.get("path")
                or identity.get("path") != self.handoff["executable_path"]
                or identity.get("application") != self.context["application"]
                or facts.get("target_visible") is not True):
            raise Stop("reject", "loaded-window-identity-required-not-splash")
        return identity

    def input(self, tool, arguments, identity, selector=None):
        facts = self.facts("guard", identity, {
            "tool": tool, "arguments": arguments, "selector": selector})
        foreground = facts.get("foreground")
        ambiguous = (
            facts.get("target_visible") is not True
            or facts.get("foreground_visible") is not True
            or (isinstance(foreground, dict) and foreground.get("pid") == identity["pid"]
                and foreground.get("root") != identity["root"])
        )
        if ambiguous:
            raise Stop("blocked", "AMBIGUOUS_WINDOW_ROOT", {
                "classification": "AMBIGUOUS",
                "reason": "Visibility or same-process root relationship is not proved.",
                "expected_identity": identity, "observed_foreground": foreground,
                "target_visible": facts.get("target_visible"),
                "foreground_visible": facts.get("foreground_visible"),
                "read_only_window_diagnostics": facts.get("window_relationships", {
                    "status": "UNAVAILABLE",
                    "required_live_observations": [
                        "Win32 owner and ancestor chains", "input focus",
                        "visible hit root", "expected PID and exact creation identity",
                        "window class and UI state",
                    ],
                }),
                "same_pid_is_not_authority": True,
                "hidden_owner_relationship": "UNPROVEN",
            })
        if foreground != identity and not (
            tool == "Click" and selector == "target" and isinstance(foreground, dict)
            and type(foreground.get("pid")) is int and foreground["pid"] > 0
            and type(foreground.get("root")) is int and foreground["root"] > 0
            and foreground.get("session") == identity["session"]
        ):
            raise Stop("blocked", "foreground-changed-no-forced-focus")
        if selector is not None:
            matches = facts.get("selectors", {}).get(selector, [])
            if (len(matches) != 1 or matches[0].get("binding") != self.case["selectors"][selector]
                    or not matches[0].get("selector") or matches[0].get("visible") is not True):
                raise Stop("reject", "uia-selector-not-unique")
            bounds = matches[0]["physical_bounds"]
            if (len(bounds) != 4 or any(type(v) not in (int, float)
                                      or not math.isfinite(v) for v in bounds)
                    or bounds[2] <= 0 or bounds[3] <= 0):
                raise Stop("reject", "invalid-dynamic-display-geometry")
            point = [round(bounds[0] + bounds[2] / 2), round(bounds[1] + bounds[3] / 2)]
            # Observer/SDK DPI mapping and physical hit-testing are supplied by
            # the live authority; neither a fixed scale nor an awareness setter.
            if facts.get("hit") != {"physical_point": point, "identity": identity}:
                raise Stop("reject", "physical-hit-root-changed")
            arguments = {**arguments, "loc": point}
        self.tool(tool, arguments, effect=True)

    def read_xml(self, stage):
        self.facts("guard", self.fresh or self.old)
        path = ps_literal(self.handoff["document_path"])
        value = self.powershell(
            "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);"
            f"$b=[IO.File]::ReadAllBytes({path});"
            "@{raw_base64=[Convert]::ToBase64String($b)}|ConvertTo-Json -Compress")
        raw = base64.b64decode(value["raw_base64"], validate=True)
        self.artifact(stage + ".Loxone", raw, stage + "-xml")
        return raw

    def screenshot(self, stage):
        reply = self.tool("Screenshot", {})
        images = [item for item in reply.get("content", [])
                  if item.get("type") == "image" and item.get("mimeType") == "image/png"]
        if len(images) != 1:
            raise Stop("UNKNOWN", "owned-screenshot-missing")
        raw = base64.b64decode(images[0]["data"], validate=True)
        if not raw.startswith(b"\x89PNG\r\n\x1a\n"):
            raise Stop("UNKNOWN", "invalid-screenshot")
        self.artifact(stage + ".png", raw, "screenshot")

    def launch(self, stage):
        facts = self.facts(stage)
        if facts.get("process_count") != 0:
            raise Stop("blocked", "full-normal-close-not-observed")
        exe = ps_literal(self.handoff["executable_path"])
        document = ps_literal('"' + self.handoff["document_path"] + '"')
        launched = self.powershell(
            "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);"
            f"$p=Start-Process -FilePath {exe} -ArgumentList {document} -PassThru;"
            "@{pid=$p.Id}|ConvertTo-Json -Compress", effect=True)
        if type(launched.get("pid")) is not int or launched["pid"] <= 0:
            raise Stop("UNKNOWN", "ordinary-launch-pid-missing")
        self.native_start = True
        loaded = self.facts("loaded")
        identity = self.identity(loaded)
        if (identity["pid"] != launched["pid"]
                or loaded.get("document_path") != self.handoff["document_path"]):
            raise Stop("reject", "ordinary-launch-document-or-pid-mismatch")
        self.screenshot(stage)
        return identity

    def perform(self):
        baseline_path = self.queue_tool().checked_artifact(self.root, self.handoff["before_xml"])
        before = baseline_path.read_bytes()
        if sha(before) != self.case["fixture_sha256"]:
            raise Stop("reject", "raw-fixture-pin-mismatch")
        if property_value(before, self.case["property"]) != self.case["property"]["before"]:
            raise Stop("reject", "baseline-property-mismatch")
        self.artifact("before.Loxone", before, "before-xml")
        self.old = self.launch("initial-open")
        if self.read_xml("opened") != before:
            raise Stop("reject", "opened-document-not-raw-baseline")
        self.input("Click", {"button": "left", "clicks": 1}, self.old, "target")
        self.input("Click", {"button": "left", "clicks": 1}, self.old, "property")
        self.input("Type", {"text": self.case["property"]["after"],
                            "clear": True, "press_enter": True}, self.old, "value")
        self.input("Shortcut", {"shortcut": "ctrl+s"}, self.old)
        saved = self.read_xml("saved")
        self.screenshot("saved")
        self.input("Shortcut", {"shortcut": "alt+f4"}, self.old)
        closed = self.facts("closed")
        if closed.get("process_count") != 0 or closed.get("original_identity_absent") != self.old:
            raise Stop("UNKNOWN", "full-normal-exit-not-proved")
        self.fresh = self.launch("fresh-reopen")
        if self.fresh["pid"] == self.old["pid"] or (
            self.fresh["start_identity"] == self.old["start_identity"]
        ):
            raise Stop("reject", "fresh-process-reopen-required")
        self.input("Click", {"button": "left", "clicks": 1}, self.fresh, "target")
        self.input("Click", {"button": "left", "clicks": 1}, self.fresh, "property")
        reopened = self.read_xml("reopened")
        value = self.facts("value", self.fresh)
        self.screenshot("reopened")
        diff = compare_xml(before, saved)
        self.artifact("xml-diff.json", canonical(diff), "xml-diff")
        self.artifact("reopen-diff.json", canonical(compare_xml(saved, reopened)), "xml-diff")
        if saved != reopened:
            raise Stop("fail", "saved-reopened-raw-hash-mismatch")
        if (property_value(saved, self.case["property"]) != self.case["property"]["after"]
                or property_value(reopened, self.case["property"]) != self.case["property"]["after"]
                or value.get("uia_value") != {
                    "binding": self.case["selectors"]["value"],
                    "value": self.case["property"]["after"], "identity": self.fresh,
                    "method": "read-only-ValuePattern"}):
            raise Stop("fail", "xml-or-readonly-uia-value-mismatch")
        self.input("Shortcut", {"shortcut": "alt+f4"}, self.fresh)
        final = self.facts("final-closed")
        if final.get("process_count") != 0 or final.get("original_identity_absent") != self.fresh:
            raise Stop("UNKNOWN", "final-normal-close-not-proved")
        return {"outcome": "complete", "code": "property-persisted",
                "before_sha256": sha(before), "saved_sha256": sha(saved),
                "reopened_sha256": sha(reopened), "old_identity": self.old,
                "fresh_identity": self.fresh, "xml_change_records": len(diff["changes"]),
                "behavioral_equivalence": "UNPROVEN", "semantic_pass": False}

    def execute(self):
        self.run, fd = self.queue.execution_start(self.run_id, self.handoff)
        try:
            try:
                self.directory.mkdir(parents=True, exist_ok=False)
                self.stage = "baseline"
                result = self.perform()
            except Stop as stop:
                result = {"outcome": stop.outcome, "code": stop.code, "stage": self.stage}
                if stop.details is not None:
                    result["blocked_diagnostics"] = stop.details
            except Exception as error:
                diagnostic = self.artifact("executor-error.json",
                                           canonical(error_details(error, self.stage)), "executor-error")
                result = {"outcome": "UNKNOWN", "code": getattr(error, "code", None)
                          or "UNEXPECTED_EXECUTOR_ERROR", "stage": self.stage,
                          "error_type": type(error).__name__, "diagnostic": diagnostic}
            result.update({
                "schema_version": 1, "kind": "ux-property-result", **self.context,
                "source_utc": utc().isoformat(), "actual_native_start": self.native_start,
                "evidence_origin": self.origin or "unknown", "automatic_retry": False,
                "steps_acknowledged": len(self.steps),
            })
            receipt = self.artifact("result.json", canonical(result), "native-receipt")
            capture = {
                "schema_version": 1, "kind": "ux-campaign-capture",
                **{key: self.run[key] for key in (
                    "run_id", "worker", "physical_identity_sha256", "case_id",
                    "definition_sha256", "purpose", "qualification_id",
                    "qualification_definition_sha256")},
                "evidence_origin": result["evidence_origin"],
                "handoff_sha256": sha(canonical(self.handoff)),
                "fixture_sha256": self.case["fixture_sha256"],
                "native_run_id": "property-" + self.run_id,
                "artifacts": self.artifacts, "result_sha256": receipt["sha256"],
                "atlas_reference": {"family": self.case["family"],
                                    "field": "@Title", "case_id": self.case["id"],
                                    "definition_sha256": self.case["definition_sha256"],
                                    "semantic_pass": False},
            }
            durable(self.directory / "capture.json", canonical(capture))
            self.event("execution-result", result=receipt, outcome=result["outcome"],
                       actual_native_start=self.native_start,
                       evidence_origin=result["evidence_origin"])
            return result, capture
        finally:
            os.close(fd)
            # Keep the physical reservation until collection, even for complete.
            # UNKNOWN/takeover requires separately verified quiescence.


def checked_acknowledgements(run, capture, result, root, events, queue_module, *, no_effect=False):
    intents = [event for event in events
               if event["type"] == "step-intent" and event.get("run_id") == run["run_id"]]
    acks = [event for event in events
            if event["type"] == "step-ack" and event.get("run_id") == run["run_id"]]
    if (len(intents) != len(acks) or (not intents and not no_effect)
            or [event["step"] for event in intents] != list(range(len(intents)))
            or [event["step"] for event in acks] != list(range(len(acks)))
            or any(intent["seq"] >= ack["seq"] for intent, ack in zip(intents, acks))
            or any(ack["seq"] >= following["seq"] for ack, following in zip(acks, intents[1:]))
            or result.get("steps_acknowledged") != len(acks)):
        raise ValueError("Unacknowledged or reordered dispatch cannot complete.")
    retained = {(item["path"], item["sha256"]) for item in capture["artifacts"]}
    for intent, ack in zip(intents, acks):
        request = intent["intent"]
        if (type(intent.get("effect")) is not bool
                or request.get("input_deadline_utc") != result["input_deadline_utc"]):
            raise ValueError("SDK intent protocol/deadline binding changed.")
        if no_effect and (intent["effect"] is not False
                          or request["tool"] not in {"ControlStatus", "DisplayInventory"}):
            raise ValueError("A definitely-no-effect result contains a possible effect.")
        descriptor = ack["reply"]
        if (descriptor["path"], descriptor["sha256"]) not in retained:
            raise ValueError("SDK acknowledgement is missing from capture.")
        reply = parse_json(queue_module.checked_artifact(root, descriptor).read_bytes())
        if reply.get("isError") is not False or reply.get("completion") in {"UNKNOWN", "pending"}:
            raise ValueError("Unknown SDK receipt cannot complete.")
        if request["tool"] == "PowerShell":
            value = reply.get("structuredContent", {}).get("result")
            match = re.fullmatch(r"Response: (.*)\r?\nStatus Code:\s*(-?\d+)\s*",
                                 value or "", re.DOTALL)
            if match is None or int(match[2]) != 0:
                raise ValueError("PowerShell actual status is not zero.")
    return intents, acks


def validate_capture(case, run, capture, root, events, queue_module):
    """Collector rechecks raw bytes; producer claims alone cannot complete."""
    if (not isinstance(capture, dict) or capture.get("schema_version") != 1
            or capture.get("kind") != "ux-campaign-capture"):
        raise ValueError("Invalid property capture envelope.")
    for key in ("run_id", "worker", "physical_identity_sha256", "case_id",
                "definition_sha256", "purpose", "qualification_id",
                "qualification_definition_sha256"):
        if capture.get(key) != run.get(key):
            raise ValueError("Property capture dispatch binding changed: " + key)
    handoffs = [event for event in events
                if event["type"] == "handoff" and event["run_id"] == run["run_id"]]
    if len(handoffs) != 1 or capture.get("handoff_sha256") != handoffs[0]["handoff_sha256"]:
        raise ValueError("Missing immutable property handoff.")
    if capture.get("fixture_sha256") != handoffs[0]["fixture_sha256"]:
        raise ValueError("Property capture fixture binding changed.")
    by_kind = {}
    for descriptor in capture["artifacts"]:
        path = queue_module.checked_artifact(root, descriptor)
        by_kind.setdefault(descriptor["kind"], []).append(path)
    if len(by_kind.get("native-receipt", [])) != 1:
        raise ValueError("Exactly one property result receipt is required.")
    receipt_raw = by_kind["native-receipt"][0].read_bytes()
    result = parse_json(receipt_raw)
    if sha(receipt_raw) != capture.get("result_sha256"):
        raise ValueError("Property result receipt hash changed.")
    for key in ("run_id", "worker", "physical_identity_sha256", "definition_sha256"):
        if result.get(key) != run[key]:
            raise ValueError("Property result identity changed.")
    if result.get("application") != case["application"] or result.get("outcome") not in OUTCOMES:
        raise ValueError("Property result application/outcome is invalid.")
    if (result.get("kind") != "ux-property-result" or result.get("schema_version") != 1
            or result.get("automatic_retry") is not False
            or not isinstance(result.get("code"), str)
            or result.get("evidence_origin") != capture.get("evidence_origin")):
        raise ValueError("Invalid property result protocol/origin.")
    starts = [event for event in events
              if event.get("run_id") == run["run_id"] and event["type"] == "execution-start"]
    recorded = [event for event in events
                if event.get("run_id") == run["run_id"] and event["type"] == "execution-result"]
    if (len(starts) != 1 or starts[0]["handoff_sha256"] != capture["handoff_sha256"]
            or len(recorded) != 1 or recorded[0]["result"]["sha256"] != sha(receipt_raw)
            or recorded[0]["outcome"] != result["outcome"]
            or recorded[0].get("actual_native_start") is not result.get("actual_native_start")
            or starts[0]["seq"] >= recorded[0]["seq"]):
        raise ValueError("Result is not bound to its durable execution start/closeout.")
    admissions = [event["binding"] for event in events
                  if event["type"] == "admitted" and event["binding"]["worker"] == run["worker"]]
    if len(admissions) != 1 or result.get("resources") != admissions[0]["resources"]:
        raise ValueError("Result resource binding changed.")
    for field in ("source_utc", "input_deadline_utc"):
        if datetime.fromisoformat(result[field]).tzinfo is None:
            raise ValueError("Result requires timezone-aware timestamps.")
    started = result.get("actual_native_start")
    if started is not None and type(started) is not bool:
        raise ValueError("Native start must be false, true or null (unknown).")
    outcome = result["outcome"]
    if outcome in {"blocked", "reject"} and started is False:
        if capture.get("evidence_origin") not in {"unknown", "native-config-ui"}:
            raise ValueError("Synthetic execution is not an authoritative no-effect closeout.")
        intents, acks = checked_acknowledgements(
            run, capture, result, root, events, queue_module, no_effect=True)
        if any(event["seq"] <= starts[0]["seq"] or event["seq"] >= recorded[0]["seq"]
               for event in intents + acks):
            raise ValueError("No-effect observations are outside the execution protocol.")
        return "blocked", False
    if capture.get("evidence_origin") != "native-config-ui":
        raise ValueError("Synthetic or unknown execution is not native evidence.")
    if outcome == "complete":
        if started is not True:
            raise ValueError("UNKNOWN native start cannot complete.")
        if result.get("evidence_origin") != "native-config-ui":
            raise ValueError("Unknown or synthetic result cannot complete.")
        required = {"before-xml", "saved-xml", "reopened-xml", "xml-diff",
                    "screenshot", "authority-receipt"}
        if required - by_kind.keys():
            raise ValueError("Complete property result is missing raw artifacts.")
        if any(len(by_kind[key]) != 1 for key in ("before-xml", "saved-xml", "reopened-xml")):
            raise ValueError("Ambiguous raw native XML artifacts.")
        before, saved, reopened = [
            by_kind[key][0].read_bytes() for key in ("before-xml", "saved-xml", "reopened-xml")]
        if (sha(before) != case["fixture_sha256"]
                or sha(before) != handoffs[0]["fixture_sha256"]
                or saved != reopened
                or sha(saved) != result["saved_sha256"]
                or sha(reopened) != result["reopened_sha256"]):
            raise ValueError("Native after hash mismatch.")
        if (property_value(before, case["property"]) != case["property"]["before"]
                or property_value(saved, case["property"]) != case["property"]["after"]):
            raise ValueError("Native property proof mismatch.")
        old, fresh = result["old_identity"], result["fresh_identity"]
        if (old["pid"] == fresh["pid"] or old["start_identity"] == fresh["start_identity"]
                or fresh["application"] != case["application"]):
            raise ValueError("Fresh process identity not proved.")
        expected_diff = compare_xml(before, saved)
        if not any(parse_json(path.read_bytes()) == expected_diff for path in by_kind["xml-diff"]):
            raise ValueError("Full XML comparison was changed or omitted.")
        facts = [parse_json(path.read_bytes()) for path in by_kind["authority-receipt"]]
        for item in facts:
            for key in ("run_id", "worker", "physical_identity_sha256", "definition_sha256",
                        "application", "input_deadline_utc"):
                if item.get(key) != result.get(key):
                    raise ValueError("Authority fact case/worker/version/deadline changed.")
            if (item.get("evidence_origin") != "native-config-ui"
                    or not item.get("source_guards") or not item.get("policy")):
                raise ValueError("Native authority facts missing.")
        closed = [item for item in facts if item.get("stage") == "closed"]
        final = [item for item in facts if item.get("stage") == "final-closed"]
        if (len(closed) != 1 or len(final) != 1
                or closed[0]["facts"].get("process_count") != 0
                or closed[0]["facts"].get("original_identity_absent") != old
                or final[0]["facts"].get("process_count") != 0
                or final[0]["facts"].get("original_identity_absent") != fresh):
            raise ValueError("Full normal close receipts missing.")
        values = [item for item in facts if item.get("stage") == "value"]
        if len(values) != 1 or values[0]["facts"].get("identity") != fresh or (
            values[0]["facts"].get("uia_value") != {
                "binding": case["selectors"]["value"], "value": case["property"]["after"],
                "identity": fresh, "method": "read-only-ValuePattern"}):
            raise ValueError("Fresh-process read-only UIA proof missing.")
        checked_acknowledgements(run, capture, result, root, events, queue_module)
        retained = {item["sha256"] for item in capture["artifacts"]}
        for event in events:
            if event.get("run_id") == run["run_id"] and event["type"] == "checkpoint":
                if event["facts"]["sha256"] not in retained:
                    raise ValueError("Authority checkpoint was replaced.")
                queue_module.checked_artifact(root, event["facts"])
        # This observes persistence only, never behavioral equivalence.
        return "observed", True
    return {"reject": "blocked", "fail": "verified-fail",
            "blocked": "blocked", "UNKNOWN": "UNKNOWN"}[outcome], started
