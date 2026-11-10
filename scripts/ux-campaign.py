#!/usr/bin/env python3
"""Local, file-only native UX campaign queue. Never launches a GUI or transport."""

import argparse
from collections import Counter
from contextlib import contextmanager
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PureWindowsPath
import re
import sys
import uuid

from loxone_ux_transport import UxError, parse_json


ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "tests" / "ux" / "campaigns" / "config-17.2.10.21-mixed100.json"
APP = {"version": "17.2.10.21",
       "sha256": "7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3"}
GUI_SOURCE = r"tests\ux\energy-manager2\gui-cli-cases.json"
CANARY = "config-energy-dialog-cancel-canary"
TERMINAL_CONTROL = {"USER_CONTROL", "CONTROL_UNAVAILABLE"}


def canonical(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=True,
                      separators=(",", ":"), allow_nan=False).encode("ascii")


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def source_digest(path):
    # Repository text may be checked out with CRLF; native artifact hashes stay raw.
    return hashlib.sha256(Path(path).read_bytes().replace(b"\r\n", b"\n")).hexdigest()


def definition_hash(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def read_json(path):
    return parse_json(Path(path).read_text(encoding="utf-8-sig"))


def read_object(path):
    value = read_json(path)
    if not isinstance(value, dict):
        raise ValueError("Expected a JSON object: " + Path(path).name)
    return value


def repo_path(relative):
    path = (ROOT / Path(*relative.split("\\"))).resolve()
    path.relative_to(ROOT)
    return path


def load_tool(name):
    path = ROOT / "scripts" / (name + ".py")
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def source(path, case_id=None):
    result = {"path": path, "sha256": source_digest(repo_path(path)), "digest_kind": "sha256-lf-source-bytes"}
    if case_id is not None:
        result["case_id"] = case_id
    return result


def energy_input(original):
    """Concrete authored variants; these inputs do not assert native acceptance."""
    identifier = original["id"]
    if "selector" in original:
        return ["Load[1]/@tpR"], {"row": 1, "column": 8, "index": original["selector"]}
    if "column" in original:
        storage = {4: "Na", 5: "RT", 6: "MinRt", 7: "MDay", 8: "tpR", 9: "SwPwr", 10: "MaxPwr"}
        return [f"Load[{original['row']}]/@{storage[original['column']]}"], {
            "row": original["row"], "column": original["column"], "value": original["input"],
            "gui_unit": original.get("gui_unit"), "storage_unit": original.get("storage_unit"),
        }
    if "parameter" in original:
        return ["Co[@K='O']/@Def"], {
            "parameter": "O", "seed": original["seed"], "unit": original["unit"],
            "value": original.get("input", "9" * 310),
        }
    variants = {
        "gui-cli-roundtrip-baseline": (
            ["Load[1]/@Na", "Load[1]/@RT"],
            {"row": 1, "name": "UX campaign roundtrip", "preparation_seconds": 60}),
        "gui-dirty-cancel": (
            ["Load[1]/@Na", "Load[1]/@RT", "Load[1]/@MinRt", "Load[1]/@MDay", "Load[12]/@Na"],
            {"row": 1, "name": "UX canceled edit", "preparation_seconds": 60,
             "minimum_runtime_minutes": -1, "daily_runtime_minutes": 5,
             "unused_row": 12, "unused_name": "UX unused canceled", "finish": "cancel"}),
        "gui-marker-new": (
            ["Load[4]/@Na", "Load[4]/@An", "Load[4]/@Ncu", "Load[4]/@Anc"],
            {"row": 4, "name": "UX custom name", "mode_index": 1, "seed_markers": "absent"}),
        "gui-marker-noop": (
            ["Load[5]/@Na", "Load[5]/@An", "Load[5]/@Ncu", "Load[5]/@Anc"],
            {"row": 5, "name": "", "mode_index": 0, "seed_markers": "absent"}),
        "gui-marker-revert": (
            ["Load[6]/@Na", "Load[6]/@An", "Load[6]/@Ncu", "Load[6]/@Anc"],
            {"row": 6, "name_sequence": ["UX temporary", ""], "mode_sequence": [1, 0]}),
        "gui-icon-builtin-select": (
            ["Load[1]/@ico"],
            {"row": 1, "native_inventory_choice": "first distinct existing built-in icon",
             "maximum_choices_to_inspect": 8, "finish": "accept",
             "unbound": "Exact native picker selector/icon reference requires guarded inventory."}),
        "gui-icon-picker-cancel": (
            ["Load[1]/@ico"],
            {"row": 1, "native_inventory_choice": "first distinct existing built-in icon",
             "maximum_choices_to_inspect": 8, "finish": "cancel",
             "unbound": "Exact native picker selector/icon reference requires guarded inventory."}),
        "gui-unused-slot-reorder": (
            ["Load[@OIdx='11']", "Load/@Id", "Load/@OIdx", "Co/@U", "Co/@In"],
            {"source_priority": 12, "destination_priority": 2, "output_index": 11}),
        "gui-preserve-note-linebreaks": (
            ["C[@Type='Note']/@Text", "Load[1]/@Na"],
            {"authored_note_text": "Campaign line one\r\nCampaign line two\nCampaign line three",
             "row": 1, "name": "UX note preservation"}),
        "gui-structure-missing-slot": (["Load[12]"], {"variant": "remove-direct-child", "priority": 12}),
        "gui-structure-extra-slot": (
            ["Load[13]"], {"variant": "append-direct-child", "OIdx": 12, "RT": 60,
                           "Id": "30000000-0000-0000-000000000000000d"}),
        "gui-duplicate-load-id": (
            ["Load[2]/@Id"], {"variant": "copy-attribute", "from_priority": 1, "to_priority": 2, "attribute": "Id"}),
        "gui-duplicate-output-index": (
            ["Load[2]/@OIdx"], {"variant": "set-attribute", "priority": 2, "attribute": "OIdx", "value": "0"}),
        "gui-malformed-load-id": (
            ["Load[1]/@Id"], {"variant": "set-attribute", "priority": 1, "attribute": "Id", "value": "not-a-uuid"}),
        "gui-icon-dangling-reference": (
            ["Load[1]/@ico"], {"variant": "set-attribute", "priority": 1, "attribute": "ico",
                               "value": "40000000-0000-0000-00000000000000ff", "require_reference_absent": True}),
        "gui-icon-malformed-reference": (
            ["Load[1]/@ico"], {"variant": "set-attribute", "priority": 1, "attribute": "ico", "value": "not-a-uuid"}),
        "gui-deadline-invalid-selector": (
            ["Load[1]/@tpR"], {"variant": "set-attribute", "priority": 1, "attribute": "tpR", "value": "28"}),
    }
    return variants[identifier]


def generate_catalog():
    """Keep source procedures intact; unavailable adapters are explicit, not guessed."""
    cases = []

    def add(case):
        case["application"] = dict(APP)
        case["definition_sha256"] = definition_hash(case)
        cases.append(case)

    recipe_path = r"tests\ux\recipes\energy-edit-cancel-check.json"
    add({
        "id": CANARY, "phase": "configuration", "family": "EnergyManager2",
        "class_binding": "CEnergyManagerConfigurationDlg",
        "fixture_alias": "energy-configuration", "fixture_sha256": None,
        "fixture_policy": "Externally hash-bound disposable native-loadable offline baseline; selected Energy block.",
        "source": source(recipe_path), "procedure": read_json(repo_path(recipe_path))["steps"],
        "field_bindings": ["Load[1].Na", "Load[1].tpR"],
        "input_binding": {"row": 1, "name": "UX scripted test", "deadline_index": 27, "finish": "cancel"},
        "expectation": {"kind": "exploratory",
                        "scope": "Native dialog edit/cancel/reopen observations only; UIA cell text is non-authoritative."},
        "adapter": "loxone-ux-recipe", "executor_compatible": True,
        "needed_capabilities": ["energy-dialog", "guarded-native-input", "owned-capture"],
        "timeout_seconds": 300, "effect_scope": "owned-disposable-dialog-cancel",
        "canary_for": "loxone-ux-recipe",
    })
    for original in read_json(repo_path(GUI_SOURCE))["cases"]:
        case_id = original["id"]
        fields, input_binding = energy_input(original)
        procedure = []
        if "selector" in original:
            procedure = [
                {"op": "energy.open"},
                {"op": "focus", "window_class": "CEnergyManagerConfigurationDlg"},
                {"op": "energy.select", "row": 1, "column": 8, "index": original["selector"]},
                {"op": "energy.accept"},
            ]
        elif "column" in original:
            procedure = [
                {"op": "energy.open"},
                {"op": "focus", "window_class": "CEnergyManagerConfigurationDlg"},
                {"op": "energy.set", "row": original["row"], "column": original["column"],
                 "value": original["input"]},
                {"op": "energy.accept"},
            ]
        add({
            "id": "config-" + case_id, "phase": "configuration", "family": "EnergyManager2",
            "class_binding": "CEnergyManagerConfigurationDlg",
            "fixture_alias": "energy-configuration", "fixture_sha256": None,
            "fixture_policy": "Fresh disposable baseline; malformed cases require separately pinned variant.",
            "source": source(GUI_SOURCE, case_id), "source_case": original,
            "binding_source": source(r"tests\ux\energy-manager2\field-bindings.json"),
            "procedure": procedure, "field_bindings": fields, "input_binding": input_binding,
            "input_origin": "Concrete independently authored variant of the imported procedure; not a native acceptance oracle.",
            "expectation": {"kind": "exploratory",
                            "scope": "Retain actual native XML normalization and dialog feedback; no CLI-derived oracle."},
            "adapter": "needs-executor", "executor_compatible": False,
            "available_stage": "loxone-ux-recipe" if procedure else None,
            "missing_adapter": (
                "Guarded native save-as/close/reopen lifecycle with retrieved before/after XML, "
                "semantic diff and capture; plus version-calibrated widget/variant binding where procedure is empty."
            ),
            "needed_capabilities": ["energy-dialog", "native-save-close-reopen", "xml-diff", "owned-capture"],
            "timeout_seconds": 300, "effect_scope": "owned-disposable-save-close-reopen",
        })
    lighting_path = r"tests\ux\light-controller2\gui-protocol.json"
    lighting = read_json(repo_path(lighting_path))
    for execution in lighting["executions"]:
        add({
            "id": "config-" + execution["id"], "phase": "configuration", "family": "LightController2",
            "class_binding": "LightController2; native dialog selector not calibrated",
            "fixture_alias": "lighting-slice-" + execution["id"],
            "fixture_sha256": execution.get("input_sha256", lighting["fixture_preparation"]["project_sha256"])
            if execution["id"] != "lighting-04-cancel-noop" else None,
            "source": source(lighting_path, execution["id"]), "source_case": execution,
            "procedure": execution["steps"], "field_bindings": ["Mood/SID", "Mood/Q2", "Mood/Q10"],
            "input_binding": {"edit": execution.get("edit", "No-op or guarded discovered-field cancel, as source procedure."),
                              "fixture_dependency": "Fresh native baseline from lighting-01" if execution["id"] == "lighting-04-cancel-noop" else None},
            "expectation": {"kind": "exploratory", "scope": execution["purpose"]},
            "adapter": "needs-executor", "executor_compatible": False,
            "missing_adapter": "Version-calibrated nested mood/channel GUI selectors and guarded save-close-reopen/XML diff.",
            "needed_capabilities": ["lighting-mood-dialog", "native-save-close-reopen", "owned-capture"],
            "timeout_seconds": 300, "effect_scope": "owned-disposable-lighting-roundtrip",
        })
    for family, directory in (("JalousieUpDown2", "shading"), ("HeatIRoomController2", "hvac")):
        path = f"tests\\ux\\{directory}\\protocol.json"
        protocol = read_json(repo_path(path))
        admission = protocol["cases"][0]
        add({
            "id": "config-" + admission["id"].replace(".", "-"), "phase": "configuration",
            "family": family, "class_binding": family + "; native dialog uncalibrated",
            "fixture_alias": directory + "-minimum",
            "fixture_sha256": protocol["private_fixture"]["sha256"],
            "source": source(path, admission["id"]), "source_case": admission,
            "procedure": [admission["procedure"]],
            "field_bindings": list(protocol["generation"]["authored_parameter_inputs"]),
            "input_binding": {"authored_parameters": protocol["generation"]["authored_parameter_inputs"],
                              "runtime_inputs": [], "scope": "Admission only; no movement or temperature injection."},
            "expectation": {"kind": "exploratory", "scope": admission["limitations"]},
            "adapter": "needs-executor", "executor_compatible": False,
            "missing_adapter": (
                "Native fixture/connector admission and version-calibrated save-close-reopen adapter; "
                + ("HVAC 56-versus-52 core connector mismatch remains unresolved."
                   if directory == "hvac" else "Shading timing units and dialog binding remain uncalibrated.")
            ),
            "needed_capabilities": [directory + "-native-admission", "native-save-close-reopen"],
            "timeout_seconds": 120 if directory == "hvac" else 90,
            "effect_scope": "owned-disposable-admission-no-runtime-input",
        })
    simulator_paths = [
        r"tests\ux\native-simulator\energy-multi-original-reviewed.json",
        r"tests\ux\native-simulator\energy-multi-third-first-reviewed.json",
        r"tests\ux\native-simulator\energy-storage-at-finite.json",
        r"tests\ux\native-simulator\energy-storage-below-finite.json",
        r"tests\ux\native-simulator\energy-daily-observer-baseline.json",
    ]
    for repeat in range(1, 7):
        for path in simulator_paths:
            native = read_json(repo_path(path))
            add({
                "id": f"sim-{native['id']}-repeat-{repeat:02d}", "phase": "simulator",
                "family": "EnergyManager2", "class_binding": "Existing native Energy pin/Or.I1 readback only",
                "fixture_alias": Path(path.replace("\\", os.sep)).stem,
                "fixture_sha256": native["fixture_sha256"], "source": source(path, native["id"]),
                "procedure": {"mode": "plain", "repeat": repeat,
                              "inputs": native.get("inputs", {}), "settings": native.get("settings", {}),
                              "sampling_phases": native.get("sampling_phases"),
                              "wait_seconds": native.get("wait_seconds"),
                              "simulator_start_time": native.get("simulator_start_time")},
                "field_bindings": native.get("channels", ["OL1"]),
                "input_binding": {"source_protocol": native["id"], "repeat": repeat, "mode": "plain",
                                  "fixture_sha256": native["fixture_sha256"]},
                "expectation": {"kind": native.get("purpose", "assertion"),
                                "outputs": native["expected_outputs"],
                                "scope": "Exact source fixture and sample windows only; repeats measure reproducibility, not new behaviors."},
                "adapter": "official-sim-plain", "executor_compatible": True,
                "needed_capabilities": ["official-sim-plain", "native-energy-readback", "owned-capture"],
                "timeout_seconds": 600, "effect_scope": "owned-disconnected-simulator-plain",
                "canary_for": "official-sim-plain" if repeat == 1 else None,
            })
    return {
        "schema_version": 1, "id": "config-17.2.10.21-mixed100", "application": dict(APP),
        "executor_sources": [source("scripts\\" + name) for name in (
            "loxone-ux.py", "loxone-ux.ps1", "loxone_ux_transport.py",
            "official-sim.py", "official-sim.ps1", "official_sim_runner.py")],
        "mix": {"configuration": 70, "simulator": 30},
        "behavior_protocols": 5, "plain_repetitions_per_protocol": 6,
        "policy": "Configuration first among compatible unattempted cases. Unsupported definitions stay queued-needs-executor.",
        "native_qualification": "UNKNOWN; no worker is qualified by this catalog.",
        "whole_product_behavior_percentage": None, "cases": cases,
    }


def validate_catalog(catalog):
    if catalog.get("kind") == "ux-property-pilot":
        from ux_property_executor import validate_case, ADAPTER
        if (catalog.get("schema_version") != 1 or catalog.get("application") != APP
                or not catalog.get("cases") or len(catalog["cases"]) > 5
                or catalog.get("mix") != {"configuration": len(catalog["cases"]), "simulator": 0}):
            raise ValueError("Invalid separate bounded property pilot.")
        if len({case["id"] for case in catalog["cases"]}) != len(catalog["cases"]):
            raise ValueError("Pilot identities must be unique.")
        expected_sources = {"scripts\\" + name for name in (
            "ux_property_executor.py", "ux-experiment.py", "ux-campaign.py",
            "loxone-ux.py", "loxone_ux_transport.py")}
        if {item["path"] for item in catalog["executor_sources"]} != expected_sources:
            raise ValueError("Pilot must pin all actual executor sources.")
        for descriptor in catalog["executor_sources"]:
            if (descriptor.get("digest_kind") != "sha256-lf-source-bytes"
                    or source_digest(repo_path(descriptor["path"])) != descriptor["sha256"]):
                raise ValueError("Pilot executor source drift.")
        for case in catalog["cases"]:
            validate_case(case)
            body = {key: value for key, value in case.items() if key != "definition_sha256"}
            if (case["definition_sha256"] != definition_hash(body) or case["application"] != APP
                    or not case["id"].startswith("pilot-title-")
                    or case["executor_compatible"] is not True
                    or case["canary_for"] not in (None, ADAPTER)):
                raise ValueError("Pilot definition/application binding changed.")
        return catalog
    if catalog.get("schema_version") != 1 or catalog.get("application") != APP:
        raise ValueError("Unsupported campaign schema/application pin.")
    cases = catalog.get("cases", [])
    if len(cases) != 100 or len({case["id"] for case in cases}) != 100:
        raise ValueError("Campaign must contain 100 unique concrete definitions.")
    if Counter(case["phase"] for case in cases) != {"configuration": 70, "simulator": 30}:
        raise ValueError("Expected the documented 70 configuration / 30 simulator mix.")
    if any(case["phase"] == "configuration" for case in cases[70:]):
        raise ValueError("Configuration definitions must precede simulator definitions.")
    gui = load_tool("loxone-ux")
    for executor in catalog["executor_sources"]:
        if source_digest(repo_path(executor["path"])) != executor["sha256"]:
            raise ValueError("Existing native executor source drift: " + executor["path"])
    for case in cases:
        body = {key: value for key, value in case.items() if key != "definition_sha256"}
        if definition_hash(body) != case.get("definition_sha256"):
            raise ValueError("Case definition digest mismatch: " + case["id"])
        if case["application"] != APP or source_digest(repo_path(case["source"]["path"])) != case["source"]["sha256"]:
            raise ValueError("Source/application drift: " + case["id"])
        if "binding_source" in case and source_digest(repo_path(case["binding_source"]["path"])) != case["binding_source"]["sha256"]:
            raise ValueError("Native field-binding source drift: " + case["id"])
        if not case["procedure"] and "source_case" not in case:
            raise ValueError("Case has no concrete procedure: " + case["id"])
        if not case["field_bindings"] or not 1 <= case["timeout_seconds"] <= 600:
            raise ValueError("Case lacks field binding or bounded timeout.")
        if case["executor_compatible"] != (case["adapter"] in {"loxone-ux-recipe", "official-sim-plain"}):
            raise ValueError("Unsupported adapters cannot be runnable.")
        if case["adapter"] == "loxone-ux-recipe":
            gui.validate_recipe({"schema_version": 1, "name": case["id"],
                                 "requires": {"exe_sha256": APP["sha256"]}, "steps": case["procedure"]})
        elif case["adapter"] == "official-sim-plain":
            native = read_json(repo_path(case["source"]["path"]))
            from official_sim_runner import native_channels, sampling_plan
            native_channels(native)
            sampling_plan(native)
            if native["fixture_sha256"] != case["fixture_sha256"]:
                raise ValueError("Simulator fixture mismatch.")
    return catalog


def now():
    return datetime.now(timezone.utc).isoformat()


def private_destination(path):
    resolved = Path(path).resolve()
    if resolved.is_relative_to(ROOT) and resolved.relative_to(ROOT).parts[:1] != (".reverse-engineering",):
        raise ValueError("Keep campaign raw data under ignored .reverse-engineering or an external private root.")
    return resolved


def checked_artifact(root, descriptor):
    if (not isinstance(descriptor, dict) or not isinstance(descriptor.get("path"), str)
            or not descriptor["path"] or PureWindowsPath(descriptor["path"]).anchor
            or not re.fullmatch(r"[a-f0-9]{64}", descriptor.get("sha256", ""))):
        raise ValueError("Artifacts require relative paths and exact SHA-256 digests.")
    relative = descriptor["path"]
    # Accept both native Windows receipt paths and portable collector paths.
    path = (Path(root) / Path(*relative.replace("\\", "/").split("/"))).resolve()
    path.relative_to(Path(root).resolve())
    if digest(path) != descriptor["sha256"]:
        raise ValueError("Artifact digest mismatch: " + relative)
    return path


def check_binding(binding, artifact_root):
    if not isinstance(binding, dict):
        raise ValueError("Expected an external scheduling binding object.")
    if binding.get("schema_version") != 1 or binding.get("kind") != "ux-campaign-worker-binding":
        raise ValueError("Expected an externally issued campaign scheduling binding.")
    if binding.get("worker") not in {"lab-a", "lab-b", "notebook"}:
        raise ValueError("Use only opaque public worker aliases.")
    identity = binding.get("physical_identity_sha256", "")
    if not re.fullmatch(r"[a-f0-9]{64}", identity):
        raise ValueError("A verified opaque physical desktop identity is required.")
    admission = checked_artifact(artifact_root, binding["admission"])
    if binding.get("admitted") is not True:
        raise ValueError("Worker is readiness-blocked; no case was attempted.")
    if binding.get("application") != APP or not binding.get("capabilities"):
        raise ValueError("Worker application/capabilities are unqualified.")
    if datetime.fromisoformat(binding["valid_until"]) <= datetime.now(timezone.utc):
        raise ValueError("External scheduling admission is stale; it does not expire/reclaim any active job.")
    # This file binds an existing authority's decision; it cannot grant input authority.
    if not binding.get("authority") or not admission.is_file():
        raise ValueError("The separately guarded authority receipt is required.")
    resources = binding.get("resources")
    if not isinstance(resources, dict) or not resources or any(
        not re.fullmatch(r"[a-z0-9-]{1,64}", key) or type(value) is not int or not 1 <= value <= 3
        for key, value in resources.items()
    ):
        raise ValueError("Declare verified opaque shared-host resources and capacity 1..3.")
    return binding


def qualification_definitions(catalog):
    """Declare one non-retryable qualification identity per worker/adapter."""
    templates = {}
    for case in catalog["cases"]:
        adapter = case.get("canary_for")
        if adapter and case["executor_compatible"]:
            templates.setdefault(adapter, []).append({
                "case_id": case["id"], "definition_sha256": case["definition_sha256"],
            })
    definitions = []
    for worker in ("lab-a", "lab-b", "notebook"):
        for adapter, candidates in templates.items():
            definition = {
                "qualification_id": "qualification-" + worker + "-" + adapter,
                "purpose": "qualification", "worker": worker, "adapter": adapter,
                "templates": candidates, "maximum_dispatches": 1,
            }
            definition["qualification_definition_sha256"] = definition_hash(definition)
            definitions.append(definition)
    return definitions


class Queue:
    def __init__(self, path, catalog):
        self.path = Path(path)
        self.catalog = validate_catalog(catalog)
        self.cases = {case["id"]: case for case in catalog["cases"]}

    @contextmanager
    def locked(self):
        lock = self.path / "writer.lock"
        try:
            fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
        except FileExistsError:
            raise ValueError("Queue writer lock exists; separately verify quiescence, never expire/reclaim it.") from None
        try:
            os.write(fd, str(os.getpid()).encode("ascii"))
            os.fsync(fd)
            yield
        finally:
            os.close(fd)
            lock.unlink()

    def initialize(self):
        private_destination(self.path)
        self.path.mkdir(parents=True, exist_ok=False)
        with (self.path / "journal.jsonl").open("x", encoding="ascii", newline="\n") as stream:
            event = {"seq": 0, "time": now(), "type": "created",
                     "catalog_sha256": definition_hash(self.catalog), "previous": None,
                     "handoff_identity_version": 2,
                     "qualification_canaries": [
                         {**item, "run_id": uuid.uuid4().hex}
                         for item in qualification_definitions(self.catalog)
                     ]}
            event["sha256"] = definition_hash(event)
            stream.write(canonical(event).decode("ascii") + "\n")
            stream.flush()
            os.fsync(stream.fileno())

    def events(self):
        result = []
        previous = None
        raw = (self.path / "journal.jsonl").read_bytes().decode("ascii")
        if not raw.endswith("\n"):
            raise ValueError("Incomplete journal append; quarantine, never silently repair.")
        for line in raw.splitlines():
            event = json.loads(line)
            body = {key: value for key, value in event.items() if key != "sha256"}
            if (event["seq"] != len(result) or event["previous"] != previous
                    or event["sha256"] != definition_hash(body)):
                raise ValueError("Journal is truncated, reordered or has changed; quarantine this queue.")
            result.append(event)
            previous = event["sha256"]
        if not result or result[0]["catalog_sha256"] != definition_hash(self.catalog):
            raise ValueError("Queue catalog binding changed.")
        declared = result[0].get("qualification_canaries")
        if declared is not None:
            bodies = [{key: value for key, value in item.items() if key != "run_id"} for item in declared]
            if (bodies != qualification_definitions(self.catalog)
                    or len({item["run_id"] for item in declared}) != len(declared)
                    or any(not re.fullmatch(r"[a-f0-9]{32}", item["run_id"]) for item in declared)):
                raise ValueError("Predeclared qualification identities/source bindings changed.")
        return result

    def qualification_plan(self):
        created = self.events()[0]
        declared = created.get("qualification_canaries")
        if declared is None or created.get("handoff_identity_version") != 2:
            raise ValueError("Legacy queue is audit-only; verified quiescence and a new queue are required, not journal migration.")
        return declared

    def append(self, kind, **fields):
        events = self.events()
        event = {"seq": len(events), "time": now(), "type": kind,
                 "previous": events[-1]["sha256"], **fields}
        event["sha256"] = definition_hash(event)
        with (self.path / "journal.jsonl").open("a", encoding="ascii", newline="\n") as stream:
            stream.write(canonical(event).decode("ascii") + "\n")
            stream.flush()
            os.fsync(stream.fileno())
        return event

    def state(self):
        workers, runs, used, canaries = {}, {}, set(), set()
        for event in self.events()[1:]:
            kind = event["type"]
            if kind == "admitted":
                workers[event["binding"]["worker"]] = {
                    "binding": event["binding"], "parked": False, "active": None}
            elif kind == "dispatched":
                runs[event["run_id"]] = {**event, "status": "dispatched", "native_started": False}
                workers[event["worker"]]["active"] = event["run_id"]
                if event.get("purpose", "experiment") == "experiment":
                    used.add(event["case_id"])
            elif kind in {"result", "unknown"}:
                run = runs[event["run_id"]]
                run.update(status=event["status"], native_started=event.get("native_started", False),
                           completion=event)
                worker = workers[run["worker"]]
                if kind == "unknown" or event.get("park"):
                    worker["parked"] = True
                    # Preserve active intent and resource quarantine until verified quiescence.
                else:
                    worker["active"] = None
                    if (run.get("purpose") == "qualification"
                            and event["status"] in {"observed", "verified-pass"}):
                        canaries.add((run["worker"], self.cases[run["case_id"]]["adapter"]))
            elif kind == "quiescent":
                worker = workers[event["worker"]]
                worker["active"] = None
                worker["parked"] = True
        return workers, runs, used, canaries

    def admit(self, binding, artifact_root):
        check_binding(binding, artifact_root)
        with self.locked():
            self.qualification_plan()
            workers, _, _, _ = self.state()
            alias = binding["worker"]
            if alias in workers:
                raise ValueError("Worker bindings are immutable for this queue; use a fresh queue for readmission.")
            if len(workers) >= 3 or any(
                worker["binding"]["physical_identity_sha256"] == binding["physical_identity_sha256"]
                for worker in workers.values()
            ):
                raise ValueError("At most three distinct verified physical desktops; aliases are not capacity.")
            for worker in workers.values():
                for resource, capacity in binding["resources"].items():
                    if resource in worker["binding"]["resources"] and worker["binding"]["resources"][resource] != capacity:
                        raise ValueError("Conflicting shared-host capacity binding.")
            return self.append("admitted", binding=binding)

    def claim(self, alias, canary=False):
        with self.locked():
            workers, runs, used, canaries = self.state()
            if alias not in workers:
                raise ValueError("Worker is readiness-blocked; admit external guard binding first.")
            worker = workers[alias]
            if worker["parked"] or worker["active"]:
                raise ValueError("Worker is active/parked; no automatic retry or UNKNOWN reclaim.")
            active = [workers[run["worker"]]["binding"] for run in runs.values()
                      if workers[run["worker"]]["active"] == run["run_id"]]
            binding = worker["binding"]
            if datetime.fromisoformat(binding["valid_until"]) <= datetime.now(timezone.utc):
                raise ValueError("External scheduling admission is stale; no input authority was inferred.")
            if len(active) >= 3 or any(
                sum(resource in other["resources"] for other in active) >= capacity
                for resource, capacity in binding["resources"].items()
            ):
                raise ValueError("Verified shared-host capacity is busy/quarantined.")
            qualifications = {(item["worker"], item["adapter"]): item
                              for item in self.qualification_plan()}
            available = []
            for case in self.catalog["cases"]:
                if not case["executor_compatible"] or not set(case["needed_capabilities"]) <= set(binding["capabilities"]):
                    continue
                if canary:
                    if not case.get("canary_for") or (alias, case["adapter"]) in canaries:
                        continue
                    qualification = qualifications.get((alias, case["adapter"]))
                    if (qualification is None or qualification["run_id"] in runs
                            or {"case_id": case["id"], "definition_sha256": case["definition_sha256"]}
                            not in qualification["templates"]):
                        continue
                elif case["id"] in used or (alias, case["adapter"]) not in canaries:
                    continue
                available.append(case)
            if not available:
                raise ValueError("No compatible case: canary/admission/adapter needed; nothing was attempted.")
            case = available[0]
            qualification = qualifications[(alias, case["adapter"])] if canary else None
            return self.append("dispatched",
                               run_id=qualification["run_id"] if canary else uuid.uuid4().hex, worker=alias,
                               physical_identity_sha256=binding["physical_identity_sha256"],
                               case_id=case["id"], definition_sha256=case["definition_sha256"],
                               canary=canary, status="dispatched",
                               purpose="qualification" if canary else "experiment",
                               qualification_id=qualification["qualification_id"] if canary else None,
                               qualification_definition_sha256=(
                                   qualification["qualification_definition_sha256"] if canary else None))

    def handoff(self, run_id, private, destination):
        """Create argv data only. Private worker/executor retains all launch/input authority."""
        with self.locked():
            return self._handoff(run_id, private, destination)

    def _handoff(self, run_id, private, destination):
        self.qualification_plan()
        workers, runs, _, _ = self.state()
        run = runs[run_id]
        if run["status"] != "dispatched" or workers[run["worker"]]["parked"]:
            raise ValueError("Only a live dispatch intent may produce a handoff.")
        if datetime.fromisoformat(workers[run["worker"]]["binding"]["valid_until"]) <= datetime.now(timezone.utc):
            raise ValueError("External scheduling admission expired before handoff; intent remains retained.")
        case = self.cases[run["case_id"]]
        fixture = private["fixtures"][case["fixture_alias"]]
        fixture_sha = digest(fixture)
        if case["fixture_sha256"] is not None and fixture_sha != case["fixture_sha256"]:
            raise ValueError("Private fixture does not match the existing native protocol.")
        if private["worker"] != run["worker"]:
            raise ValueError("Private handoff binding targets another worker.")
        if any(event["type"] == "handoff" and event["run_id"] == run_id for event in self.events()):
            raise ValueError("Handoff already retained; no duplicate dispatch request.")
        destination = private_destination(destination)
        destination.mkdir(parents=True, exist_ok=False)
        expected_target_pid = None
        if case["adapter"] == "loxone-ux-recipe":
            load_tool("loxone-ux").integer(private["pid"], 1, 2147483647, "Owned PID")
            expected_target_pid = private["pid"]
            recipe = {"schema_version": 1, "name": case["id"],
                      "requires": {"exe_sha256": APP["sha256"]}, "steps": case["procedure"]}
            recipe_file = destination / "recipe.json"
            write_new(recipe_file, recipe)
            argv = [sys.executable, str(ROOT / "scripts" / "loxone-ux.py"),
                    "--profile", private["ux_profile"], "--pid", str(private["pid"]),
                    "--expected-sha", APP["sha256"], "--deadline-seconds", str(case["timeout_seconds"]),
                    "--verbose", "run", str(recipe_file)]
            load_tool("loxone-ux").make_parser().parse_args(argv[2:])
        elif case["adapter"] == "sdk-property-roundtrip-v1":
            gui = load_tool("loxone-ux")
            for key in ("executable_path", "document_path"):
                gui.windows_path(private[key], key)
            before_xml = checked_artifact(private["artifact_root"], private["before_xml"])
            if digest(before_xml) != fixture_sha:
                raise ValueError("Pilot baseline descriptor does not match fixture.")
            if digest(private["authority_module"]) != private["authority_source_sha256"]:
                raise ValueError("Private authority source pin changed.")
            argv = [sys.executable, str(ROOT / "scripts" / "ux-experiment.py"),
                    "--queue", str(self.path), "--catalog", str(private["catalog_path"]),
                    "--run-id", run_id, "--handoff", str(destination / "handoff.json"),
                    "--profile", private["ux_profile"], "--authority", private["authority_module"],
                    "--authority-sha256", private["authority_source_sha256"],
                    "--artifact-root", private["artifact_root"]]
        else:
            # official-sim requires --case to reside below its artifact root.
            native_case_dir = (Path(private["artifact_root"]) / ".reverse-engineering" /
                               "ux-campaign" / run_id)
            native_case_dir.mkdir(parents=True, exist_ok=False)
            native_case_file = native_case_dir / "case.json"
            write_new(native_case_file, read_json(repo_path(case["source"]["path"])))
            argv = [sys.executable, str(ROOT / "scripts" / "official-sim.py"),
                    "--profile", private["sim_profile"], "--transfer-profile", private["transfer_profile"],
                    "--case", str(native_case_file), "--fixture", fixture,
                    "--mode", "plain", "--artifact-root", private["artifact_root"]]
            if private.get("ocr_profile"):
                argv.extend(["--ocr-profile", private["ocr_profile"]])
        handoff = {
            "schema_version": 1, "kind": "ux-campaign-handoff", "run_id": run_id,
            "worker": run["worker"], "physical_identity_sha256": run["physical_identity_sha256"],
            "case_id": case["id"], "definition_sha256": case["definition_sha256"],
            "purpose": run["purpose"], "qualification_id": run["qualification_id"],
            "qualification_definition_sha256": run["qualification_definition_sha256"],
            "fixture_sha256": fixture_sha, "argv": argv, "timeout_seconds": case["timeout_seconds"],
            "expected_target_pid": expected_target_pid,
            "authority": "DATA ONLY. Existing guarded worker must independently admit every input/lifecycle action.",
            "collection": "Retain actual native receipt/request, guard+PID/path/hash/start/session/fence proof, XML/diff, owned screenshots and observations.",
        }
        if case["adapter"] == "sdk-property-roundtrip-v1":
            handoff.update({key: private[key] for key in (
                "executable_path", "document_path", "before_xml", "authority_source_sha256")})
        write_new(destination / "handoff.json", handoff)
        self.append("handoff", run_id=run_id, handoff_sha256=definition_hash(handoff),
                    fixture_sha256=fixture_sha, expected_target_pid=expected_target_pid)
        return handoff

    def unknown(self, run_id, reason):
        with self.locked():
            return self._unknown(run_id, reason)

    def _unknown(self, run_id, reason):
        _, runs, _, _ = self.state()
        if runs[run_id]["status"] != "dispatched":
            raise ValueError("Dispatch already terminal; no retry/requeue.")
        started = False
        if self.cases[runs[run_id]["case_id"]]["adapter"] == "sdk-property-roundtrip-v1":
            own = [event for event in self.events() if event.get("run_id") == run_id]
            results = [event for event in own if event["type"] == "execution-result"]
            started = (results[-1].get("actual_native_start") if results else
                       None if any(event["type"] == "step-intent" and event.get("effect")
                                   for event in own) else False)
            if results and started is True and results[-1].get("evidence_origin") != "native-config-ui":
                started = None
        return self.append("unknown", run_id=run_id, status="UNKNOWN", reason=reason,
                           native_started=started, park=True)

    def _execution_run(self, run_id):
        workers, runs, _, _ = self.state()
        run = runs[run_id]
        worker = workers[run["worker"]]
        if (run["status"] != "dispatched" or worker["parked"]
                or worker["active"] != run_id):
            raise ValueError("Current dispatch is terminal, quarantined or not owned; no execution.")
        if datetime.fromisoformat(worker["binding"]["valid_until"]) <= datetime.now(timezone.utc):
            raise ValueError("Current worker admission expired; no execution.")
        return run

    def execution_start(self, run_id, handoff):
        with self.locked():
            run = self._execution_run(run_id)
            own = [event for event in self.events() if event.get("run_id") == run_id]
            if any(event["type"] == "execution-start" for event in own):
                self._unknown(run_id, "Executor was previously entered; inspect, never replay.")
                raise ValueError("Recovery is audit-only; prior execution is never replayed.")
            handoffs = [event for event in own if event["type"] == "handoff"]
            handoff_sha = definition_hash(handoff)
            if (handoff.get("run_id") != run_id or len(handoffs) != 1
                    or handoffs[0]["handoff_sha256"] != handoff_sha):
                raise ValueError("Immutable execution handoff mismatch.")
            lock = self.path / ("desktop-" + run["physical_identity_sha256"] + ".lock")
            fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY, 0o600)
            try:
                os.write(fd, run_id.encode("ascii"))
                os.fsync(fd)
                self.append("execution-start", run_id=run_id, handoff_sha256=handoff_sha)
            except BaseException:
                os.close(fd)
                raise
            return run, fd

    def execution_intent(self, run_id, **fields):
        with self.locked():
            self._execution_run(run_id)
            return self.append("step-intent", run_id=run_id, **fields)

    def collect(self, run_id, capture, artifact_root):
        with self.locked():
            workers, runs, _, _ = self.state()
            run = runs[run_id]
            if run["status"] != "dispatched":
                previous = run.get("completion", {})
                if previous.get("type") == "result" and previous.get("capture_sha256") == definition_hash(capture):
                    for item in capture["artifacts"]:
                        checked_artifact(artifact_root, item)
                    return previous  # Read-only batch audit, not another attempt or promotion.
                raise ValueError("Dispatch already terminal; append-only evidence cannot retry it.")
            case = self.cases[run["case_id"]]
            pilot = case["adapter"] == "sdk-property-roundtrip-v1"
            try:
                status, started = validate_capture(case, run, capture, artifact_root, self.events())
                if pilot and status == "observed":
                    lock = self.path / ("desktop-" + run["physical_identity_sha256"] + ".lock")
                    if lock.read_text(encoding="ascii") != run_id:
                        raise ValueError("Physical execution reservation changed.")
            except (ValueError, KeyError, OSError, TypeError, UxError) as error:
                return self.append("unknown", run_id=run_id, status="UNKNOWN", reason=str(error),
                                   native_started=None if pilot else False, park=True,
                                   rejected_capture_sha256=definition_hash(capture))
            error = capture.get("terminal_error")
            result = self.append("result", run_id=run_id, status=status, native_started=started,
                               park=(error in TERMINAL_CONTROL or status in {"UNKNOWN", "blocked"}
                                     or (pilot and status != "observed")),
                               terminal_error=error, capture_sha256=definition_hash(capture),
                               native_run_id=capture.get("native_run_id"),
                               native_receipt_sha256=next(
                                   (item["sha256"] for item in capture["artifacts"] if item["kind"] == "native-receipt"), None),
                               artifacts=capture["artifacts"], worker=run["worker"],
                               physical_identity_sha256=workers[run["worker"]]["binding"]["physical_identity_sha256"])
            if pilot and status == "observed":
                lock = self.path / ("desktop-" + run["physical_identity_sha256"] + ".lock")
                if lock.read_text(encoding="ascii") == run_id:
                    lock.unlink()
            return result

    def collect_file(self, run_id, capture_path, artifact_root):
        try:
            capture = read_json(capture_path)
        except (OSError, ValueError) as error:
            return self.unknown(run_id, "Native capture is missing/unreadable: " + str(error))
        return self.collect(run_id, capture, artifact_root)

    def quiescent(self, alias, proof, root):
        record = read_json(checked_artifact(root, proof))
        with self.locked():
            workers, _, _, _ = self.state()
            if not workers[alias]["parked"] or not workers[alias]["active"]:
                raise ValueError("Only quarantined resources can be released.")
            if (record.get("kind") != "ux-campaign-quiescence-binding"
                    or record.get("run_id") != workers[alias]["active"]
                    or record.get("physical_identity_sha256") != workers[alias]["binding"]["physical_identity_sha256"]
                    or record.get("quiescent") is not True):
                raise ValueError("Separately verified quiescence does not bind this quarantined desktop/run.")
            checked_artifact(root, record["guard_receipt"])
            return self.append("quiescent", worker=alias, proof=proof,
                               note="External guard authority separately verified quiescence; worker stays parked.")

    def batch_plan(self, alias, limit):
        if not 1 <= limit <= 20:
            raise ValueError("Batch plans are bounded to 1..20 sequential cases.")
        workers, _, used, canaries = self.state()
        if alias not in workers or workers[alias]["parked"] or workers[alias]["active"]:
            raise ValueError("Batch requires an independently admitted idle worker.")
        worker = workers[alias]["binding"]
        if datetime.fromisoformat(worker["valid_until"]) <= datetime.now(timezone.utc):
            raise ValueError("Batch scheduling admission is stale.")
        cases = [case for case in self.catalog["cases"]
                 if case["executor_compatible"]
                 and (case["id"] not in used or (alias, case["adapter"]) not in canaries)
                 and set(case["needed_capabilities"]) <= set(worker["capabilities"])]
        if not cases:
            raise ValueError("No compatible unattempted cases; blocked definitions remain unattempted.")
        adapter = cases[0]["adapter"]
        gated = (alias, adapter) not in canaries
        if gated:
            cases = [case for case in cases if case.get("canary_for") == adapter][:1]
        else:
            cases = [case for case in cases if case["adapter"] == adapter][:limit]
        qualification = next((item for item in self.qualification_plan()
                              if item["worker"] == alias and item["adapter"] == adapter), None) if gated else None
        if gated and (qualification is None or qualification["run_id"] in self.state()[1]):
            raise ValueError("This worker/adapter qualification was already attempted; no alternate-template retry.")
        return {
            "kind": "ux-campaign-sequential-batch-plan", "worker": alias,
            "cases": [case["id"] for case in cases], "canary_required": gated,
            "purpose": "qualification" if gated else "experiment",
            "qualification_id": qualification["qualification_id"] if gated else None,
            "predeclared_run_id": qualification["run_id"] if gated else None,
            "maximum_batch_seconds": sum(case["timeout_seconds"] for case in cases),
            "reservation": False, "native_attempts": 0,
            "protocol": [
                "Claim one case atomically; materialize its exact argv handoff.",
                "Existing guarded worker independently authorizes input and captures actual artifacts.",
                "Collect receipt before claiming the next case; never overlap jobs on this desktop.",
                "Stop batch on USER_CONTROL, CONTROL_UNAVAILABLE, timeout or missing receipt; no automatic retry.",
            ],
            "baseline_policy": "Disposable file copy per case; existing VM baseline per batch, no VM reset/reboot per click.",
            "review_policy": "Review retained capture group at batch level; this plan grants neither readiness nor native pass.",
        }

    def stats(self):
        workers, runs, used, _ = self.state()
        experiments = [run for run in runs.values() if run.get("purpose", "experiment") == "experiment"]
        qualification_runs = [run for run in runs.values() if run.get("purpose") == "qualification"]

        def summarize(selected):
            counts = Counter(run["status"] for run in selected)
            native = [run for run in selected if run["native_started"]]
            completed = [run for run in native if run["status"] in {"observed", "verified-pass", "verified-fail"}]
            times = [datetime.fromisoformat(run["time"]) for run in native]
            ends = [datetime.fromisoformat(run["completion"]["time"]) for run in native]
            hours = (max(ends) - min(times)).total_seconds() / 3600 if times else 0
            return {
                "dispatch_intents": len(selected), "native_attempts": len(native), "completions": len(completed),
                "statuses": dict(counts), "observations": counts["observed"],
                "verified_pass": counts["verified-pass"], "verified_fail": counts["verified-fail"],
                "blocked": counts["blocked"], "unknown": counts["UNKNOWN"],
                "unknown_native_start_cases": sum(run["status"] == "UNKNOWN" and not run["native_started"]
                                                  for run in selected),
                "runs_per_hour": round(len(native) / hours, 3) if hours > 0 else None,
            }

        experiment_stats = summarize(experiments)
        qualification_stats = summarize(qualification_runs)
        per_worker = {}
        for alias in workers:
            per_worker[alias] = {
                **summarize([run for run in experiments if run["worker"] == alias]),
                "qualification": summarize([run for run in qualification_runs if run["worker"] == alias]),
                "parked": workers[alias]["parked"],
            }
        native_experiments = [run for run in experiments if run["native_started"]]
        return {
            "definitions": len(self.cases), "executor_compatible": sum(c["executor_compatible"] for c in self.cases.values()),
            "needs_executor": sum(not c["executor_compatible"] for c in self.cases.values()),
            "unattempted_cases": len(self.cases) - len(used),
            **experiment_stats,
            "qualification": {"predeclared_runs": len(self.events()[0].get("qualification_canaries", [])),
                              **qualification_stats},
            "all_native_attempts": experiment_stats["native_attempts"] + qualification_stats["native_attempts"],
            "all_completions": experiment_stats["completions"] + qualification_stats["completions"],
            "unique_native_cases": len({run["case_id"] for run in native_experiments}),
            "unique_control_fields": len({(self.cases[run["case_id"]]["family"], field)
                                          for run in native_experiments for field in self.cases[run["case_id"]]["field_bindings"]}),
            "workers": per_worker, "whole_product_behavior_percentage": None,
        }


def validate_capture(case, run, capture, root, events):
    if case.get("adapter") == "sdk-property-roundtrip-v1":
        from ux_property_executor import validate_capture as validate_property_capture
        # The adapter's collector is file-only; it never constructs a client.
        return validate_property_capture(case, run, capture, root, events,
                                         sys.modules[__name__] if __name__ in sys.modules
                                         else load_tool("ux-campaign"))
    if not isinstance(capture, dict):
        raise ValueError("Expected an actual native capture envelope object.")
    if capture.get("kind") != "ux-campaign-capture" or capture.get("schema_version") != 1:
        raise ValueError("Local CLI/unit/helper results are not native UX receipts.")
    if capture.get("evidence_origin") != "native-config-ui":
        raise ValueError("CLI/unit/synthetic-helper evidence cannot qualify a native UX experiment.")
    for key in ("run_id", "worker", "physical_identity_sha256", "case_id", "definition_sha256"):
        if capture.get(key) != run[key]:
            raise ValueError("Capture dispatch binding mismatch: " + key)
    for key in ("purpose", "qualification_id", "qualification_definition_sha256"):
        if key in run and capture.get(key) != run[key]:
            raise ValueError("Capture experiment/qualification binding mismatch: " + key)
    handoffs = [event for event in events if event["type"] == "handoff" and event["run_id"] == run["run_id"]]
    if len(handoffs) != 1 or capture.get("handoff_sha256") != handoffs[0]["handoff_sha256"]:
        raise ValueError("Capture requires exactly its retained dispatch handoff.")
    if capture.get("fixture_sha256") != handoffs[0]["fixture_sha256"]:
        raise ValueError("Capture fixture mismatch.")
    expected_target_pid = handoffs[0].get("expected_target_pid")
    if case["adapter"] == "loxone-ux-recipe" and (
        type(expected_target_pid) is not int or not 1 <= expected_target_pid <= 2147483647
    ):
        raise ValueError("Legacy Config handoff lacks its immutable expected PID; verified quiescence and a new queue are required.")
    artifacts = capture.get("artifacts", [])
    by_kind = {}
    for item in artifacts:
        checked_artifact(root, item)
        by_kind.setdefault(item["kind"], []).append(item)
    terminal = capture.get("terminal_error") in TERMINAL_CONTROL
    if terminal:
        if "guard-receipt" not in by_kind:
            raise ValueError("Terminal control requires the actual guarded executor error receipt.")
        guard = read_object(checked_artifact(root, by_kind["guard-receipt"][0]))
        if guard.get("error", {}).get("code") != capture["terminal_error"]:
            raise ValueError("Terminal control code is not present in the guarded receipt.")
        if set(by_kind) == {"guard-receipt"}:
            return "blocked", False
    if not capture.get("native_run_id") or any(
        event["type"] == "result" and event.get("native_run_id") == capture["native_run_id"]
        for event in events
    ):
        raise ValueError("Native execution identity is missing or already collected; no receipt reuse.")
    required = {"guard-receipt", "fence-proof", "identity-proof", "native-receipt", "observations", "screenshot",
                "before-xml", "after-xml", "xml-diff"}
    if not required <= by_kind.keys():
        raise ValueError("Missing native artifacts: " + ", ".join(sorted(required - by_kind.keys())))
    identity = read_object(checked_artifact(root, by_kind["identity-proof"][0]))
    if case["adapter"] == "loxone-ux-recipe" and identity.get("pid") != expected_target_pid:
        raise ValueError("Native identity PID differs from the immutable Config handoff PID.")
    if (identity.get("run_id") != run["run_id"] or identity.get("native_run_id") != capture["native_run_id"]
            or identity.get("application") != APP or identity.get("fixture_sha256") != capture["fixture_sha256"]
            or identity.get("physical_identity_sha256") != run["physical_identity_sha256"]
            or type(identity.get("pid")) is not int or identity["pid"] <= 0
            or type(identity.get("session")) is not int or identity["session"] <= 0
            or not identity.get("start_time") or not PureWindowsPath(identity.get("path", "")).is_absolute()
            or not re.fullmatch(r"[a-f0-9]{64}", identity.get("fence_proof_sha256", ""))
            or identity["fence_proof_sha256"] != by_kind["fence-proof"][0]["sha256"]
            or identity.get("guard_receipt_sha256") != by_kind["guard-receipt"][0]["sha256"]):
        raise ValueError("Native PID/path/hash/start/session/fence/fixture identity proof is incomplete.")
    if digest(checked_artifact(root, by_kind["before-xml"][0])) != capture["fixture_sha256"]:
        raise ValueError("Before XML is not the bound disposable fixture.")
    receipt = read_object(checked_artifact(root, by_kind["native-receipt"][0]))
    if case["adapter"] == "loxone-ux-recipe":
        if "native-request" not in by_kind:
            raise ValueError("GUI receipt requires its actual backend request.")
        request = read_object(checked_artifact(root, by_kind["native-request"][0]))
        if (request.get("steps") != case["procedure"] or request.get("target", {}).get("pid") != expected_target_pid
                or request["target"].get("pid") != identity["pid"]
                or request["target"].get("expected_exe_sha256") != APP["sha256"]
                or request.get("artifact_dir") != capture["native_run_id"]):
            raise ValueError("GUI request case/application/PID binding mismatch.")
        load_tool("loxone-ux").validate_reply(receipt, request)
        if receipt["target"].get("exe_sha256") != APP["sha256"]:
            raise ValueError("GUI application pin mismatch.")
        return ("observed" if receipt["ok"] and not terminal else "blocked"), receipt["completed_steps"] > 0
    if (receipt.get("schema_version") != 1 or receipt.get("kind") != "official-native-replay"
            or receipt.get("application") != APP):
        raise ValueError("Not the existing official native simulator receipt/application.")
    if receipt.get("run_id") != capture["native_run_id"]:
        raise ValueError("Native simulator execution identity mismatch.")
    native = read_json(repo_path(case["source"]["path"]))
    if (receipt.get("case", {}).get("id") != native["id"]
            or receipt["case"].get("fixture_sha256") != capture["fixture_sha256"]
            or native["fixture_sha256"] != capture["fixture_sha256"]):
        raise ValueError("Native simulator case/fixture mismatch.")
    declared = {item["path"]: item["sha256"] for item in receipt.get("artifacts", [])}
    for item in receipt.get("artifacts", []):
        checked_artifact(root, item)
    case_artifact = receipt["case"]["definition"]
    if (declared.get(case_artifact["path"]) != case_artifact["sha256"]
            or receipt["case"]["sha256"] != case_artifact["sha256"]
            or read_json(checked_artifact(root, case_artifact)) != native):
        raise ValueError("Native simulator definition/source digest binding mismatch.")
    target = receipt.get("remote", {}).get("target", {})
    if target.get("pid") != identity["pid"] or target.get("start_utc") != identity["start_time"]:
        raise ValueError("Native simulator target PID mismatch.")
    if "native-request" not in by_kind:
        raise ValueError("Native simulator receipt requires its actual retained request.")
    request_artifact = by_kind["native-request"][0]
    request = read_object(checked_artifact(root, request_artifact))
    expected_application = {**APP, "path": identity["path"]}
    from official_sim_runner import sampling_plan
    if (request.get("run_id") != capture["native_run_id"]
            or request.get("fixture_sha256") != capture["fixture_sha256"]
            or request.get("application") != expected_application
            or request.get("mode") != "plain"
            or request.get("sampling_phases") != sampling_plan(native)
            or request_artifact["sha256"] not in declared.values()):
        raise ValueError("Native simulator request/run/application/fixture/sample binding mismatch.")
    execution = receipt["execution"]
    if (execution.get("mode") != "plain" or execution.get("status") not in {"pass", "fail", "blocked"}
            or execution.get("run_id") != capture["native_run_id"]):
        raise ValueError("Only the existing plain native protocols are admitted by this campaign.")
    for sample in receipt.get("observations", []):
        for path in sample.get("source", {}).get("artifacts", []):
            if path not in declared:
                raise ValueError("Native readback source is not in the artifact manifest.")
    report = {"error": ({"message": execution.get("reason") or "Original native executor was blocked."}
                        if execution.get("status") == "blocked" else None),
              "native_running_verified": execution.get("native_running_verified"),
              "cleanup": execution.get("cleanup"), "observations": receipt["observations"]}
    simulator = load_tool("official-sim")
    calibration_case = copy_for_calibration(native)
    simulator.validate_readback_calibration(calibration_case, Path(root).resolve())
    status, _ = simulator.judge(native, report)
    if execution.get("status") == "fail" and status == "pass":
        raise ValueError("Rejudged observations contradict the original native failure; no success fallback.")
    return ("blocked" if terminal else {"pass": "verified-pass", "fail": "verified-fail", "blocked": "blocked"}[status]), (
        execution.get("native_running_verified") is True)


def copy_for_calibration(native):
    # Existing native validator uses local paths; normalize only its file references.
    result = json.loads(json.dumps(native))
    calibration = result.get("readback", {}).get("calibration", {})
    for item in [calibration.get("image", {}), calibration.get("header_anchor", {}),
                 *calibration.get("evidence", [])]:
        if "path" in item:
            item["path"] = str(Path(*item["path"].replace("\\", "/").split("/")))
    return result


def write_new(path, value):
    with Path(path).open("x", encoding="ascii", newline="\n") as stream:
        stream.write(json.dumps(value, ensure_ascii=True, indent=2, allow_nan=False) + "\n")


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, default=CATALOG)
    sub = parser.add_subparsers(dest="command", required=True)
    build = sub.add_parser("catalog", help="Deterministically generate a new catalog, never overwrite")
    build.add_argument("--write", type=Path)
    build.add_argument("--pilot", action="store_true", help="Separate five-input property pilot, not mixed100")
    sub.add_parser("validate")
    sub.add_parser("plan")
    for name in ("init", "stats", "admit", "claim", "handoff", "collect", "collect-batch",
                 "batch-plan", "unknown", "quiescent"):
        child = sub.add_parser(name)
        child.add_argument("--queue", type=Path, required=True)
        if name in {"admit", "collect", "collect-batch", "quiescent"}:
            child.add_argument("--artifact-root", type=Path, required=True)
        if name == "admit":
            child.add_argument("--binding", type=Path, required=True)
        if name in {"claim", "quiescent", "batch-plan"}:
            child.add_argument("--worker", choices=("lab-a", "lab-b", "notebook"), required=True)
        if name == "claim":
            child.add_argument("--canary", action="store_true")
        if name in {"handoff", "collect", "unknown"}:
            child.add_argument("--run-id", required=True)
        if name == "handoff":
            child.add_argument("--private-binding", type=Path, required=True)
            child.add_argument("--output", type=Path, required=True)
        if name == "collect":
            child.add_argument("--capture", type=Path, required=True)
        if name == "collect-batch":
            child.add_argument("--captures", type=Path, required=True)
        if name == "batch-plan":
            child.add_argument("--limit", type=int, default=5)
        if name == "unknown":
            child.add_argument("--reason", required=True)
        if name == "quiescent":
            child.add_argument("--proof", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.command == "catalog":
            if args.pilot:
                from ux_property_executor import pilot_catalog
                catalog = validate_catalog(pilot_catalog(load_tool("ux-campaign")))
            else:
                catalog = validate_catalog(generate_catalog())
            if args.write:
                args.write.parent.mkdir(parents=True, exist_ok=True)
                write_new(args.write, catalog)
            result = {"definitions": len(catalog["cases"]), "mix": catalog["mix"],
                      "executor_compatible": sum(c["executor_compatible"] for c in catalog["cases"]),
                      "catalog_sha256": definition_hash(catalog)}
        else:
            catalog = validate_catalog(read_json(args.catalog))
            if args.command in {"plan", "validate"}:
                result = {"definitions": len(catalog["cases"]), "mix": catalog["mix"],
                          "executor_compatible": sum(c["executor_compatible"] for c in catalog["cases"]),
                          "native_executed": 0, "qualification": "external-guard-required",
                          "first_canary": next(
                              case["id"] for case in catalog["cases"] if case.get("canary_for")),
                          "qualification_canaries": qualification_definitions(catalog),
                          "whole_product_behavior_percentage": None}
            else:
                queue = Queue(args.queue, catalog)
                if args.command == "init":
                    queue.initialize()
                    result = queue.stats()
                elif args.command == "stats":
                    result = queue.stats()
                elif args.command == "admit":
                    result = queue.admit(read_json(args.binding), args.artifact_root)
                elif args.command == "claim":
                    result = queue.claim(args.worker, args.canary)
                elif args.command == "handoff":
                    result = queue.handoff(args.run_id, read_json(args.private_binding), args.output)
                elif args.command == "collect":
                    result = queue.collect_file(args.run_id, args.capture, args.artifact_root)
                elif args.command == "collect-batch":
                    captures = read_json(args.captures)
                    if not isinstance(captures, list) or not 1 <= len(captures) <= 20:
                        raise ValueError("A capture group must contain 1..20 actual case envelopes.")
                    result = {"results": [queue.collect(item["run_id"], item, args.artifact_root)
                                          for item in captures], "stats": queue.stats()}
                elif args.command == "batch-plan":
                    result = queue.batch_plan(args.worker, args.limit)
                elif args.command == "unknown":
                    result = queue.unknown(args.run_id, args.reason)
                else:
                    result = queue.quiescent(args.worker, read_json(args.proof), args.artifact_root)
        print(json.dumps({"ok": True, **result}, ensure_ascii=True, allow_nan=False))
        return 0
    except (ValueError, KeyError, OSError, TypeError, UxError) as error:
        print(json.dumps({"ok": False, "error": {"code": "ux_campaign", "message": str(error)}}))
        return 1


if __name__ == "__main__":
    sys.exit(main())
