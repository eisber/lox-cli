#!/usr/bin/env python3
"""Measure corpus export quality and join actual official-simulator run evidence."""

import argparse
from collections import Counter, defaultdict
from contextlib import closing
import hashlib
import importlib.util
import json
import math
from pathlib import Path, PurePosixPath, PureWindowsPath
import re
import sqlite3
import sys

from loxone_ux_transport import parse_json


class EvidenceError(ValueError):
    pass


def require(condition, message):
    if not condition:
        raise EvidenceError(message)


def digest(path):
    result = hashlib.sha256()
    with Path(path).open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            result.update(chunk)
    return result.hexdigest()


def read_json(path):
    value = parse_json(Path(path).read_text(encoding="utf-8-sig"))
    require(isinstance(value, dict), f"Expected a JSON object: {path}")
    return value


def object_field(record, key):
    value = record.get(key, {})
    require(isinstance(value, dict), f"Expected an object for {key}.")
    return value


def valid_hash(value):
    return isinstance(value, str) and re.fullmatch(r"[a-f0-9]{64}", value) is not None


def finite_scalar(value):
    if type(value) not in (bool, int, float):
        return False
    try:
        return math.isfinite(value)
    except OverflowError:
        return False


def clock_seconds(value):
    require(isinstance(value, str) and re.fullmatch(r"(?:[01][0-9]|2[0-3]):[0-5][0-9]:[0-5][0-9]", value),
            "A declared native clock window requires an actual HH:MM:SS reading.")
    hours, minutes, seconds = (int(part) for part in value.split(":"))
    return hours * 3600 + minutes * 60 + seconds


def baseline(latest_path):
    latest = read_json(latest_path)
    binary = latest["binaries"]["LoxoneConfig.exe"]
    index_path = Path(binary["export"]["index_path"])
    index = read_json(index_path)
    require(index["target_sha256"] == binary["export"]["target_sha256"], "Export target identities differ.")
    counts = Counter(item["status"] for item in index["functions"])
    written = [item for item in index["functions"] if item["status"] == "written"]
    require(all(item.get("body", {}).get("available") is True for item in written),
            "A written entry does not contain an available body.")
    database = Path(binary["database"]).resolve()
    with closing(sqlite3.connect(database.as_uri() + "?mode=ro", uri=True)) as db:
        row = db.execute(
            """SELECT count(*),sum(instr(body,'/* WARNING:')>0),
                      sum(instr(body,'Could not recover jumptable')>0),
                      sum(instr(body,'Treating indirect jump as call')>0)
               FROM functions"""
        ).fetchone()
    require(row[0] == len(written), "The index and research database contain different body counts.")
    local_entries = len(index["functions"]) - counts["external"]
    return {
        "schema_version": 1,
        "kind": "official-simulator-corpus-baseline",
        "application": {"sha256": index["target_sha256"]},
        "provenance": {
            "index_sha256": digest(index_path),
            "database_sha256": digest(database),
            "provider": index["provider"],
        },
        "export": {
            "scope": "whole recognized executable, not simulator-owned code",
            "recognized_entries": len(index["functions"]),
            "status_counts": dict(sorted(counts.items())),
            "available_bodies": len(written),
            "recognized_nonexternal_entries": local_entries,
            "body_export_fraction": len(written) / local_entries if local_entries else None,
        },
        "quality_flags": {
            "bodies_with_warning_comments": row[1] or 0,
            "bodies_with_unrecovered_jumptable_warning": row[2] or 0,
            "bodies_with_indirect_jump_warning": row[3] or 0,
            "interpretation": "Text-derived flags, including thunks; not a count or percentage of incorrect functions.",
        },
        "simulator_scope": {"status": "unknown", "function_denominator": None, "basic_block_denominator": None},
        "native_execution": {"status": "not_measured", "basic_block_percentage": None},
        "gate": {"engine_complete": False},
    }


def check_artifact(reference, root):
    require(isinstance(reference, dict) and {"path", "sha256"} <= reference.keys(), "Malformed artifact reference.")
    require(valid_hash(reference["sha256"]), "Malformed artifact hash.")
    name = reference["path"]
    require(isinstance(name, str) and name and "\0" not in name, "Malformed artifact path.")
    windows = PureWindowsPath(name)
    relative = PurePosixPath(name.replace("\\", "/"))
    require(not windows.drive and not windows.root and not relative.is_absolute()
            and ".." not in relative.parts, "Artifact paths must be relative and cannot escape their root.")
    path = (root / Path(*relative.parts)).resolve()
    require(path.is_relative_to(root) and path.is_file(), f"Missing or escaped artifact: {name}")
    require(digest(path) == reference["sha256"], f"Artifact hash mismatch: {name}")
    return path


def bound_artifact(reference, seen, root, message):
    if isinstance(reference, str):
        require(reference in seen, message)
        reference = {"path": reference, "sha256": seen[reference]}
    check_artifact(reference, root)
    require(seen.get(reference["path"]) == reference["sha256"], message)
    return reference


def validate_entry_observation(record, seen, root):
    logs = record.get("entry_logs", [])
    require(isinstance(logs, list) and len(logs) <= 1, "Expected at most one passive entry log.")
    references = [
        bound_artifact(reference, seen, root, "Entry log is not bound to the receipt.")
        for reference in logs
    ]
    require(not record.get("coverage_logs"), "Entry observations cannot stand in for basic-block coverage.")
    if not references:
        require(record["execution"]["status"] != "pass", "Passing observer run omitted its actual entry log.")
        return {"status": "not_admitted", "reason": "No actual entry log."}
    evidence = object_field(record, "entry_observation")
    sources = {}
    for name in ("profile", "configuration", "parser_source"):
        reference = bound_artifact(
            evidence.get(name), seen, root, "Observer profile/configuration/parser source is not bound."
        )
        sources[name] = check_artifact(reference, root)
    profile = read_json(sources["profile"])
    require(profile.get("kind") == "passive-entry-observer-profile"
            and type(profile.get("schema")) is int and profile["schema"] == 1
            and profile.get("format_schema") in (
                "passive-entry-observer-jsonl-v1", "passive-entry-observer-jsonl-v2"
            )
            and valid_hash(profile.get("client_sha256")),
            "Unsupported passive observer profile.")
    specification = importlib.util.spec_from_file_location(
        "scorecard_entry_parser", Path(__file__).with_name("native-sim-observe.py")
    )
    parser = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(parser)
    expected = parser.read_config(sources["configuration"])
    require(expected == evidence.get("expected"), "Observer expected identity differs from its actual configuration.")
    require(expected["target_sha256"] == record["application"]["sha256"]
            and expected["client_sha256"] == profile["client_sha256"],
            "Observer configuration belongs to a different application/client.")
    parsed = parser.parse_log(check_artifact(references[0], root), expected)
    schema = 1 if profile["format_schema"].endswith("-v1") else 2
    require(parsed["header"]["schema"] == schema, "Observer profile and log format differ.")
    if schema == 2:
        require(profile.get("backend") == parsed["header"].get("backend") == "drmgr-entry"
                and profile.get("supports_return") is False,
                "Observer backend does not match its profile.")
        if record["execution"]["status"] == "pass":
            require(profile.get("late_attach_verified") is True,
                    "Passing observer run used a backend without admitted late-attach transparency.")
    for name in ("max_events", "max_objects", "prefix_bytes"):
        require(parsed["header"][name] == expected[name], "Observer budget differs from its configuration.")
    entries = [event for event in parsed["events"] if event["type"] == "ENTRY" and event["context_ok"]]
    if record["execution"]["status"] == "pass":
        require(evidence.get("status") == "validated" and entries,
                "Passing observer run has no validated application-context entry.")
    return {
        "status": "validated" if entries else "not_admitted",
        "raw_log_sha256": parsed["sha256"],
        "entry_count": len(entries),
        "observed_site_ids": sorted({event["site"] for event in entries}),
        "missing_site_ids": sorted({site["id"] for site in parsed["sites"]} - {event["site"] for event in entries}),
        "dropped_events": parsed["summary"]["dropped_events"],
        "scope": "Bounded entry evidence, not basic-block coverage, complete call traces, or instance/UUID attribution.",
    }


def receipt(path, root):
    record = read_json(path)
    require(type(record.get("schema_version")) is int and record["schema_version"] == 1
            and record.get("kind") == "official-native-replay",
            "Unsupported native receipt schema.")
    application = object_field(record, "application")
    require(valid_hash(application.get("sha256")), "Native receipt omitted its application hash.")
    case = object_field(record, "case")
    require(isinstance(case.get("id"), str) and case["id"]
            and valid_hash(case.get("sha256")) and valid_hash(case.get("fixture_sha256")),
            "Native receipt omitted its case/fixture identity.")
    references = record.get("artifacts")
    require(isinstance(references, list) and references, "Native receipt has no actual artifacts.")
    seen = {}
    for reference in references:
        check_artifact(reference, root)
        name = reference["path"]
        require(name not in seen or seen[name] == reference["sha256"], "Conflicting artifact identity.")
        seen[name] = reference["sha256"]
    definition = case.get("definition")
    definition_path = check_artifact(definition, root)
    contract = read_json(definition_path)
    require(definition["sha256"] == case["sha256"], "Case definition bytes do not match the case hash.")
    require(contract.get("id") == case["id"], "Case definition and receipt IDs differ.")
    if "application" in contract:
        require(object_field(contract, "application").get("sha256") == application["sha256"],
                "Case definition belongs to a different application.")
    if "fixture_sha256" in contract:
        require(contract["fixture_sha256"] == case["fixture_sha256"],
                "Case definition belongs to a different fixture.")
    require(case["fixture_sha256"] in seen.values(), "Fixture bytes are not bound to the run artifacts.")
    execution = object_field(record, "execution")
    run_id = execution.get("run_id")
    require(isinstance(run_id, str) and run_id and run_id == run_id.strip(),
            "Native receipt omitted its original execution run ID.")
    require(execution.get("mode") in ("plain", "drcov", "entry-observer")
            and execution.get("status") in ("pass", "fail", "blocked"),
            "Native receipt has an unknown execution mode/status.")
    observations = record.get("observations", [])
    require(isinstance(observations, list), "Native observations must be an array.")
    phases = {}
    samples = {}
    for observation in observations:
        require(isinstance(observation, dict), "Native observation must be an object.")
        phase = observation.get("phase")
        require(isinstance(phase, str) and phase and phase not in phases, "Observation phases must be explicit and unique.")
        elapsed = observation.get("elapsed_ms")
        require(type(elapsed) in (int, float) and finite_scalar(elapsed) and elapsed >= 0,
                "Observation omitted its actual elapsed time.")
        outputs = observation.get("outputs")
        require(isinstance(outputs, dict) and outputs, "Observation has no actual output samples.")
        for name, value in outputs.items():
            require(isinstance(name, str) and name and finite_scalar(value),
                    "Observation output is not a finite scalar.")
        source = object_field(observation, "source")
        require(source.get("kind") in ("uia", "native-readable", "ocr"), "Output source is not an admitted native observation.")
        evidence = source.get("artifacts")
        require(isinstance(evidence, list) and evidence, "Output sample has no source evidence.")
        for reference in evidence:
            bound_artifact(reference, seen, root, "Output evidence is not bound to the receipt.")
        phases[phase] = outputs
        samples[phase] = observation
    if execution["status"] == "pass":
        require(execution.get("native_running_verified") is True and observations,
                "Passing receipt did not establish actual native execution and output.")
        if "expected_outputs" in contract:
            expected = object_field(contract, "expected_outputs")
            require(expected, "A passing case must declare at least one output assertion.")
            for phase, outputs in expected.items():
                require(isinstance(phase, str) and phase in phases and isinstance(outputs, dict) and outputs,
                        "Passing receipt omits a declared assertion phase.")
                for name, value in outputs.items():
                    require(isinstance(name, str) and finite_scalar(value)
                            and name in phases[phase] and phases[phase][name] == value,
                            "Passing receipt omits or contradicts a declared output assertion.")
        if "sampling_phases" in contract:
            windows = contract["sampling_phases"]
            require(isinstance(windows, list) and windows, "A timing case must declare sampling windows.")
            names = set()
            for window in windows:
                require(isinstance(window, dict) and isinstance(window.get("name"), str),
                        "Malformed declared sampling window.")
                name = window["name"]
                require(name not in names and name in samples, "Passing receipt omits a declared timing phase.")
                names.add(name)
                start, end = window.get("at_seconds"), window.get("latest_seconds")
                require(type(start) in (int, float) and type(end) in (int, float)
                        and finite_scalar(start) and finite_scalar(end) and 0 <= start <= end,
                        "Malformed declared sampling interval.")
                require(start * 1000 <= samples[name]["elapsed_ms"] <= end * 1000,
                        "Passing receipt sampled outside its declared timing window.")
                if "simulator_time_window" in window:
                    clock_window = window["simulator_time_window"]
                    require(isinstance(clock_window, list) and len(clock_window) == 2,
                            "Malformed native clock window.")
                    first, last = (clock_seconds(value) for value in clock_window)
                    actual = clock_seconds(samples[name].get("simulator_time"))
                    require(first <= actual <= last if first <= last else actual >= first or actual <= last,
                            "Passing receipt contradicts its declared native clock window.")
    coverage_logs = record.get("coverage_logs", [])
    require(isinstance(coverage_logs, list), "Coverage log references must be an array.")
    record["coverage_logs"] = [
        bound_artifact(reference, seen, root, "Coverage log is not bound to the receipt.")
        for reference in coverage_logs
    ]
    if execution["mode"] == "entry-observer":
        record["entry_validation"] = validate_entry_observation(record, seen, root)
    else:
        require(not record.get("entry_logs"), "Only entry-observer mode may declare passive entry logs.")
    return record, phases, digest(path)


def compare(receipt_paths, coverage_paths, artifact_root):
    root = Path(artifact_root).resolve()
    runs = []
    identities = set()
    executions = set()
    applications = set()
    groups = defaultdict(list)
    for path in receipt_paths:
        record, phases, identity = receipt(path, root)
        require(identity not in identities, "A duplicate receipt cannot count as an independent replay.")
        identities.add(identity)
        run_id = record["execution"]["run_id"]
        require(run_id not in executions, "Duplicate execution run ID: two analyses are not independent replays.")
        executions.add(run_id)
        applications.add(record["application"]["sha256"])
        key = (record["case"]["id"], record["case"]["sha256"], record["case"]["fixture_sha256"])
        item = {"record": record, "phases": phases, "sha256": identity}
        groups[key].append(item)
        runs.append({"sha256": identity, "case": key[0], "execution": record["execution"]})
    require(len(applications) == 1, "Compare runs of one exact application, not different native binaries.")
    application = next(iter(applications))
    analyses = []
    for path in coverage_paths:
        analysis = read_json(path)
        require(type(analysis.get("schema_version")) is int and analysis["schema_version"] == 1
                and analysis.get("kind") == "native-basic-block-coverage",
                "Unsupported coverage analysis schema.")
        require(object_field(analysis, "application").get("sha256") == application,
                "Coverage belongs to a different application.")
        references = analysis.get("artifacts")
        require(isinstance(references, list) and references, "Coverage has no actual artifacts.")
        for reference in references:
            check_artifact(reference, root)
        raw_log_hash = object_field(analysis, "measurement").get("raw_log_sha256")
        require(valid_hash(raw_log_hash) and any(ref["sha256"] == raw_log_hash for ref in references),
                "Coverage raw-log identity is not bound to an actual artifact.")
        scope = object_field(analysis, "scope")
        coverage = object_field(analysis, "coverage")
        observed = object_field(analysis, "observed")
        require(scope.get("status") in ("unknown", "reviewed_complete"),
                "Coverage did not state whether its code scope is known.")
        if scope["status"] == "unknown":
            require(coverage.get("basic_block_percentage") is None,
                    "Unknown static scope cannot have a basic-block coverage percentage.")
        for field in ("target_bb_count", "executed_function_count"):
            count = observed.get(field)
            require(type(count) is int and count >= 0, "Coverage omitted an actual nonnegative execution count.")
        analyses.append({"record": analysis, "sha256": digest(path)})
    outcomes = []
    for key, items in groups.items():
        plain = [item for item in items if item["record"]["execution"]["mode"] == "plain"]
        instrumented = [item for item in items if item["record"]["execution"]["mode"] == "drcov"]
        observed = [item for item in items if item["record"]["execution"]["mode"] == "entry-observer"]
        coverage_items = plain + instrumented
        passing = all(item["record"]["execution"]["status"] == "pass" for item in coverage_items)
        signatures = {
            json.dumps(item["phases"], sort_keys=True, separators=(",", ":"), allow_nan=False)
            for item in coverage_items
        }
        repeated = len(plain) >= 2
        plain_signatures = {
            json.dumps(item["phases"], sort_keys=True, separators=(",", ":"), allow_nan=False)
            for item in plain
        }
        plain_agrees = repeated and len(plain_signatures) == 1 and all(
            item["record"]["execution"]["status"] == "pass" for item in plain
        )
        agrees = bool(instrumented) and passing and len(signatures) == 1
        observer_signatures = {
            json.dumps(item["phases"], sort_keys=True, separators=(",", ":"), allow_nan=False)
            for item in plain + observed
        }
        observer_agrees = plain_agrees and bool(observed) and len(observer_signatures) == 1 and all(
            item["record"]["execution"]["status"] == "pass"
            and item["record"]["entry_validation"]["status"] == "validated" for item in observed
        )
        logs = {ref["sha256"] for item in instrumented for ref in item["record"].get("coverage_logs", [])}
        matching = [
            item for item in analyses
            if item["record"]["measurement"]["raw_log_sha256"] in logs
            and item["record"]["observed"]["target_bb_count"] > 0
            and item["record"]["observed"]["executed_function_count"] > 0
        ]
        outcomes.append({
            "case_id": key[0],
            "case_sha256": key[1],
            "fixture_sha256": key[2],
            "plain_runs": len(plain),
            "instrumented_runs": len(instrumented),
            "entry_observer_runs": len(observed),
            "repeat_plain": repeated,
            "plain_outputs_agree": plain_agrees,
            "sampled_outputs_agree": agrees,
            "mapped_native_coverage": bool(matching),
            "entry_observer_equivalence_established": observer_agrees,
            "entry_observations": [item["record"]["entry_validation"] for item in observed],
            "native_coverage_runs": [{
                "analysis_sha256": item["sha256"],
                "raw_log_sha256": item["record"]["measurement"]["raw_log_sha256"],
                "target_bb_count": item["record"]["observed"]["target_bb_count"],
                "executed_function_count": item["record"]["observed"]["executed_function_count"],
            } for item in matching],
            "loop_established": repeated and agrees and bool(matching),
        })
    return {
        "schema_version": 1,
        "kind": "official-native-feedback-scorecard",
        "application": {"sha256": application},
        "runs": runs,
        "cases": outcomes,
        "comparison_basis": "Exact output scalars at matching recorded case phases, enforcing declared sampling/native-clock windows when present; no full timing or trace equivalence is inferred.",
        "scope": {
            "engine_scope_status": "unknown",
            "native_basic_block_percentage": None,
            "recovered_behavior_percentage": None,
        },
        "gate": {
            "input_valid": True,
            "native_replay_established": bool(outcomes) and all(item["plain_outputs_agree"] for item in outcomes),
            "loop_established": bool(outcomes) and all(item["loop_established"] for item in outcomes),
            "entry_observer_equivalence_established": bool(outcomes) and all(
                item["entry_observer_equivalence_established"] for item in outcomes
            ),
            "engine_complete": False,
        },
        "trust_boundary": "Artifact hashes prove byte association, not that a dishonest producer really executed the GUI or recorder.",
    }


def save_new(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("x", encoding="utf-8") as output:
        output.write(json.dumps(value, indent=2, allow_nan=False) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    export = commands.add_parser("baseline")
    export.add_argument("--latest", type=Path, default=Path(".reverse-engineering") / "latest.json")
    export.add_argument("--output", type=Path, required=True)
    join = commands.add_parser("compare")
    join.add_argument("--receipt", action="append", type=Path, required=True)
    join.add_argument("--coverage", action="append", type=Path, default=[])
    join.add_argument("--artifact-root", type=Path, required=True)
    join.add_argument("--output", type=Path, required=True)
    join.add_argument("--gate", choices=("drcov-loop", "entry-observer"), default="drcov-loop",
                      help="Explicitly select entry equivalence instead of the default mapped-coverage loop")
    args = parser.parse_args()
    try:
        require(not args.output.exists(), "Output exists; preserve the earlier scorecard.")
        result = baseline(args.latest) if args.command == "baseline" else compare(args.receipt, args.coverage, args.artifact_root)
        save_new(args.output, result)
    except (OSError, ValueError, KeyError, TypeError, sqlite3.Error) as error:
        print(json.dumps({"ok": False, "error": str(error)}))
        return 2
    print(json.dumps({
        "ok": True, "output": str(args.output), "kind": result["kind"],
        "gate": result["gate"], "cases": result.get("cases"),
        "export": result.get("export"), "quality_flags": result.get("quality_flags"),
    }, allow_nan=False))
    if args.command == "baseline":
        return 0
    gate = "loop_established" if args.gate == "drcov-loop" else "entry_observer_equivalence_established"
    return 0 if result["gate"][gate] else 1


if __name__ == "__main__":
    sys.exit(main())
