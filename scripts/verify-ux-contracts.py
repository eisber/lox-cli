#!/usr/bin/env python3
"""Verify versioned block UX evidence; never execute Config or write to a server."""

import argparse
from collections import Counter
from datetime import datetime
import hashlib
import json
from pathlib import Path, PureWindowsPath
import re
import sys


ROOT = Path(__file__).resolve().parents[1]
SCHEMAS = ROOT / "contracts"
APPLICATION = {
    "version": "17.2.10.21",
    "sha256": "7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3",
}
STAGES = ("catalog", "native", "binding_known", "implementation",
          "unit_test", "gui_oracle", "complete")
REQUIRED_LEVELS = ("implementation", "unit_test", "gui_oracle")


def _object(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"duplicate JSON key: {key}")
        value[key] = item
    return value


def _nonfinite(value):
    raise ValueError(f"nonfinite JSON number: {value}")


def load_json(path):
    with Path(path).open(encoding="utf-8") as stream:
        return json.load(stream, object_pairs_hook=_object, parse_constant=_nonfinite)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def _diagnostic(report, code, path, message, invalid=False):
    report["diagnostics"].append({
        "code": code, "path": path, "message": message,
        "severity": "invalid" if invalid else "incomplete",
    })


def _schema_check(value, schema, name, path="$"):
    """Apply only the vocabulary used by our three fixed, versioned schemas."""
    unsupported = schema.keys() - {
        "$schema", "$id", "$defs", "$ref", "type", "additionalProperties",
        "required", "properties", "enum", "const", "minLength", "pattern",
        "minItems", "uniqueItems", "items", "minimum",
    }
    if unsupported:
        raise ValueError(f"{name}: unsupported schema vocabulary {sorted(unsupported)}")
    if "$ref" in schema:
        filename, fragment = schema["$ref"].split("#", 1)
        filename = filename or name
        if filename not in (
                "ux-contract.schema.json", "ux-results.schema.json",
                "ux-runner-record.schema.json"):
            raise ValueError("unsupported schema reference")
        target = load_json(SCHEMAS / filename)
        for component in fragment.strip("/").split("/"):
            if component:
                target = target[component]
        return _schema_check(value, target, filename, path)
    expected = schema.get("type")
    types = {
        "object": lambda item: type(item) is dict,
        "array": lambda item: type(item) is list,
        "string": lambda item: type(item) is str,
        "integer": lambda item: type(item) is int,
        "boolean": lambda item: type(item) is bool,
    }
    if expected and not types[expected](value):
        raise ValueError(f"{path}: expected {expected}, got {type(value).__name__}")
    if "const" in schema and value != schema["const"]:
        raise ValueError(f"{path}: expected {schema['const']!r}")
    if "enum" in schema and value not in schema["enum"]:
        raise ValueError(f"{path}: expected one of {schema['enum']!r}")
    if type(value) is dict:
        missing = set(schema.get("required", [])) - value.keys()
        extra = value.keys() - schema.get("properties", {}).keys()
        if missing:
            raise ValueError(f"{path}: missing fields {sorted(missing)}")
        if extra and schema.get("additionalProperties") is False:
            raise ValueError(f"{path}: unexpected fields {sorted(extra)}")
        for key, item in value.items():
            if key in schema.get("properties", {}):
                _schema_check(item, schema["properties"][key], name, f"{path}.{key}")
    elif type(value) is list:
        if len(value) < schema.get("minItems", 0):
            raise ValueError(f"{path}: requires at least {schema['minItems']} items")
        if schema.get("uniqueItems"):
            tokens = [json.dumps(item, sort_keys=True) for item in value]
            if len(tokens) != len(set(tokens)):
                raise ValueError(f"{path}: duplicate items")
        for index, item in enumerate(value):
            _schema_check(item, schema["items"], name, f"{path}[{index}]")
    elif type(value) is str:
        if len(value) < schema.get("minLength", 0):
            raise ValueError(f"{path}: empty string")
        if "pattern" in schema and not re.search(schema["pattern"], value):
            raise ValueError(f"{path}: string does not match {schema['pattern']}")
    elif type(value) is int and value < schema.get("minimum", value):
        raise ValueError(f"{path}: below minimum {schema['minimum']}")


def validate_document(value, name):
    _schema_check(value, load_json(SCHEMAS / name), name)


def _artifact_path(root, path):
    normalized = path.replace("\\", "/")
    parts = normalized.split("/")
    if (PureWindowsPath(path).drive or normalized.startswith("/")
            or any(part in ("", ".", "..") for part in parts)
            or any(":" in part for part in parts)):
        raise ValueError("artifact path must be root-relative without traversal")
    target = root.joinpath(*parts).resolve()
    if not target.is_relative_to(root):
        raise ValueError("artifact path escapes root")
    return target


def _artifacts(report, root, artifacts, label):
    valid, tokens = True, set()
    for artifact in artifacts:
        try:
            path = _artifact_path(root, artifact["path"])
            token = (str(path), artifact["sha256"])
            if any(existing[0] == token[0] for existing in tokens):
                raise ValueError("duplicate/conflicting artifact path")
            tokens.add(token)
            if digest(path) != artifact["sha256"]:
                raise ValueError("artifact SHA256 mismatch")
        except (OSError, ValueError) as error:
            _diagnostic(report, "artifact_invalid", label, str(error), invalid=True)
            valid = False
    return valid, tokens


def _unique(report, rows, label):
    indexed = {}
    for row in rows:
        if row["id"] in indexed:
            _diagnostic(report, "duplicate_id", label, row["id"], invalid=True)
        indexed[row["id"]] = row
    return indexed


def _sources(report, root, sources, label, gui_tokens):
    present = set()
    for index, source in enumerate(sources):
        valid, tokens = _artifacts(report, root, source["artifacts"], f"{label}[{index}]")
        kind = source["evidence_kind"]
        if kind == "gui_oracle":
            record_token, case_tokens = gui_tokens
            if not record_token or record_token not in tokens or not (tokens & case_tokens):
                _diagnostic(report, "gui_source_unverified", label,
                            "GUI source needs the actual runner record and a passed GUI artifact")
                valid = False
        if valid:
            present.add(kind)
    return present


def _runner(report, root, result):
    run = result["run"]
    valid, tokens = _artifacts(report, root, [run["record"]], "run.record")
    if not valid:
        return False, None
    try:
        record = load_json(_artifact_path(root, run["record"]["path"]))
        validate_document(record, "ux-runner-record.schema.json")
        for key in ("id", "runner", "started_at", "finished_at", "application", "synthetic"):
            if record[key] != run[key]:
                raise ValueError(f"runner record disagrees with run.{key}")
        start = datetime.fromisoformat(run["started_at"])
        finish = datetime.fromisoformat(run["finished_at"])
        if start.tzinfo is None or finish.tzinfo is None or finish < start:
            raise ValueError("run timestamps need timezone and nonnegative duration")
        observations = _unique(report, record["observations"], "run.record.observations")
        cases = _unique(report, result["cases"], "results.cases")
        if observations != cases:
            raise ValueError("runner observations disagree with result cases")
        if any(row["evidence_kind"] == "gui_oracle" and row["outcome"] != "not_run"
               for row in cases.values()):
            if record["kind"] != "gui_observation":
                raise ValueError("GUI cases require a gui_observation runner record")
    except (OSError, ValueError, TypeError, KeyError) as error:
        _diagnostic(report, "runner_invalid", "run.record", str(error), invalid=True)
        return False, None
    if run["synthetic"]:
        _diagnostic(report, "synthetic_run", "run.synthetic",
                    "Execution record is simulated/fabricated; safe synthetic input projects "
                    "are allowed for actual GUI runs with synthetic=false")
        return False, None
    return True, next(iter(tokens))


def _fraction(count, total):
    return {"verified": count, "declared": total,
            "percentage": round(100 * count / total, 2) if total else None}


def _finish(report):
    invalid = any(row["severity"] == "invalid" for row in report["diagnostics"])
    report["gate"] = {
        "complete": (not report["diagnostics"] and bool(report["capabilities"])
                     and all(capability["stages"]["complete"]
                             for capability in report["capabilities"])),
        "input_valid": not invalid,
    }
    return report


def verify_files(contract_path, results_path=None, root=ROOT, application=None):
    """Return a structured report using the same strict gates as the CLI."""
    report = {
        "schema_version": 1, "contract": str(contract_path), "application": None,
        "block_type": None, "control_type": None, "scope": None,
        "capabilities": [], "cases": [], "stages": {}, "overall_percentage": None,
        "diagnostics": [],
    }
    root = Path(root).resolve()
    try:
        contract = load_json(contract_path)
        validate_document(contract, "ux-contract.schema.json")
        result = load_json(results_path) if results_path else None
        if result is not None:
            validate_document(result, "ux-results.schema.json")
    except (OSError, ValueError) as error:
        _diagnostic(report, "input_invalid", "$", str(error), invalid=True)
        return _finish(report)
    report.update({key: contract[key] for key in (
        "application", "block_type", "control_type", "scope")})
    report["contract_sha256"] = digest(contract_path)
    report["results"] = str(results_path) if results_path else None
    report["run"] = result["run"] if result is not None else None
    target = application if application is not None else APPLICATION
    identity_valid = contract["application"] == target
    if not identity_valid:
        _diagnostic(report, "application_stale", "application",
                    "Contract does not identify the selected executable/version", invalid=True)
    capabilities = _unique(report, contract["capabilities"], "capabilities")
    definitions = _unique(report, contract["cases"], "cases")
    for case in definitions.values():
        for capability_id in case["capability_ids"]:
            if capability_id not in capabilities:
                _diagnostic(report, "unknown_capability", f"cases.{case['id']}",
                            capability_id, invalid=True)
    for cap in capabilities.values():
        binding = cap["binding"]
        if (binding["status"] == "known") != ("path" in binding):
            _diagnostic(report, "binding_invalid", f"capabilities.{cap['id']}.binding",
                        "Only known bindings must have a nonempty path", invalid=True)
        for case_id in cap["required_case_ids"]:
            if case_id not in definitions or cap["id"] not in definitions[case_id]["capability_ids"]:
                _diagnostic(report, "required_case_invalid", f"capabilities.{cap['id']}",
                            f"Missing or nonreciprocal case: {case_id}", invalid=True)
    if not capabilities:
        _diagnostic(report, "empty_denominator", "capabilities",
                    "Zero capabilities cannot establish complete parity")
    runner_valid, record_token = False, None
    observations = {}
    if result is not None:
        for key in ("application", "block_type", "control_type"):
            if result[key] != contract[key]:
                identity_valid = False
                _diagnostic(report, "result_identity_mismatch", key,
                            "Result and contract identities differ", invalid=True)
        if result["run"]["application"] != contract["application"]:
            identity_valid = False
            _diagnostic(report, "runner_application_stale", "run.application",
                        "Runner executed a different executable/version", invalid=True)
        if result["contract_sha256"] != digest(contract_path):
            identity_valid = False
            _diagnostic(report, "contract_stale", "contract_sha256",
                        "Result refers to different manifest bytes", invalid=True)
        runner_valid, record_token = _runner(report, root, result)
        observations = _unique(report, result["cases"], "results.cases")
        for case_id in observations.keys() - definitions.keys():
            _diagnostic(report, "unexpected_case", "results.cases", case_id, invalid=True)
    gui_artifacts = {}
    verified_cases = {}
    for case_id, case in definitions.items():
        observed = observations.get(case_id)
        verified, tokens = False, set()
        if observed:
            artifacts_valid, tokens = _artifacts(
                report, root, observed["artifacts"], f"results.cases.{case_id}.artifacts")
            if observed["evidence_kind"] != case["evidence_kind"]:
                _diagnostic(report, "evidence_kind_mismatch", f"results.cases.{case_id}",
                            "Metadata/unit evidence cannot replace the declared evidence kind",
                            invalid=True)
            if observed["outcome"] == "pass" and not observed["artifacts"]:
                _diagnostic(report, "pass_without_artifacts", f"results.cases.{case_id}",
                            "A pass requires actual outcome artifacts", invalid=True)
                artifacts_valid = False
            verified = (
                identity_valid and runner_valid and artifacts_valid
                and bool(tokens) and observed["outcome"] == "pass"
                and observed["evidence_kind"] == case["evidence_kind"])
        verified_cases[case_id] = verified
        if verified and case["evidence_kind"] == "gui_oracle":
            gui_artifacts[case_id] = tokens
        report["cases"].append({
            "id": case_id, "capability_ids": case["capability_ids"],
            "condition": case["condition"], "evidence_kind": case["evidence_kind"],
            "outcome": observed["outcome"] if observed else "missing",
            "actual": observed["actual"] if observed else None,
            "artifacts": observed["artifacts"] if observed else None, "verified": verified,
        })
    all_gui_tokens = set().union(*gui_artifacts.values()) if gui_artifacts else set()
    scope_sources = _sources(report, root, contract["scope"]["source_evidence"],
                             "scope.source_evidence", (record_token, all_gui_tokens))
    scope_verified = (identity_valid and contract["scope"]["status"] == "complete"
                      and "gui_oracle" in scope_sources)
    if not scope_verified:
        _diagnostic(report, "scope_unverified", "scope",
                    "Complete scope needs explicit complete status and verified GUI source")
    counts = Counter()
    for cap in capabilities.values():
        cap_cases = [case for case in definitions.values() if cap["id"] in case["capability_ids"]]
        cap_gui = set().union(*(gui_artifacts.get(case["id"], set()) for case in cap_cases))
        sources = _sources(report, root, cap["source_evidence"],
                           f"capabilities.{cap['id']}.source_evidence", (record_token, cap_gui))
        status = {"catalog": "catalog" in sources, "native": "native" in sources,
                  "binding_known": cap["binding"]["status"] == "known" and "path" in cap["binding"]}
        for kind in REQUIRED_LEVELS:
            kind_cases = [case for case in cap_cases if case["evidence_kind"] == kind]
            status[kind] = bool(kind_cases) and all(verified_cases[case["id"]] for case in kind_cases)
        required = cap["required_case_ids"]
        required_levels = {
            definitions[case_id]["evidence_kind"] for case_id in required if case_id in definitions
        }
        missing_levels = [kind for kind in REQUIRED_LEVELS if kind not in required_levels]
        has_gui = "gui_oracle" in required_levels
        status["complete"] = (
            identity_valid and bool(sources) and status["binding_known"] and bool(required)
            and not missing_levels and status["gui_oracle"]
            and all(verified_cases.get(case_id, False) for case_id in required))
        if not sources:
            _diagnostic(report, "source_unverified", f"capabilities.{cap['id']}",
                        "No verified source evidence")
        if not status["binding_known"]:
            _diagnostic(report, "binding_unverified", f"capabilities.{cap['id']}",
                        "Unknown/ambiguous storage binding prevents completion")
        if not required or not has_gui:
            _diagnostic(report, "gui_requirement_missing", f"capabilities.{cap['id']}",
                        "Nonempty requirements including a GUI case are mandatory")
        for kind in missing_levels:
            _diagnostic(report, f"required_{kind}_case_missing",
                        f"capabilities.{cap['id']}.required_case_ids",
                        f"CLI parity requires a required {kind} case, not optional or substituted evidence")
        if has_gui and not status["gui_oracle"]:
            _diagnostic(report, "gui_cases_unverified", f"capabilities.{cap['id']}",
                        "Not all declared GUI conditions have verified observations")
        for case_id in required:
            if not verified_cases.get(case_id, False):
                _diagnostic(report, "required_case_unverified", f"capabilities.{cap['id']}",
                            f"Required case is missing, failed, not run or unverified: {case_id}")
        counts.update(stage for stage, present in status.items() if present)
        report["capabilities"].append({
            "id": cap["id"], "kind": cap["kind"], "binding": cap["binding"],
            "source_evidence": cap["source_evidence"], "verified_source_kinds": sorted(sources),
            "required_case_ids": required, "missing_required_evidence_levels": missing_levels,
            "stages": status,
        })
    total = len(capabilities)
    if any(row["severity"] == "invalid" for row in report["diagnostics"]):
        for capability in report["capabilities"]:
            capability["stages"]["complete"] = False
        counts["complete"] = 0
        scope_verified = False
    report["stages"] = {stage: _fraction(counts[stage], total) for stage in STAGES}
    report["scope_evidence_verified"] = scope_verified
    report["case_counts"] = {
        "defined": len(definitions), "observed": len(definitions.keys() & observations.keys()),
        **{outcome: sum(case["outcome"] == outcome for case in report["cases"])
           for outcome in ("pass", "fail", "not_run", "missing")},
        "verified": sum(verified_cases.values()),
    }
    if scope_verified and total:
        report["overall_percentage"] = round(100 * counts["complete"] / total, 2)
    return _finish(report)


def enrich_inventory(inventory, reports):
    """Keep inventory metadata distinct; attach evidence only to exact identities."""
    if (type(inventory) is not dict or type(inventory.get("blocks")) is not list
            or type(inventory.get("source")) is not dict):
        raise ValueError("inventory needs source and blocks")
    if "summary" in inventory and type(inventory["summary"]) is not dict:
        raise ValueError("inventory summary must be an object")
    sha256 = inventory["source"].get("executable_sha256")
    if type(sha256) is not str or not re.fullmatch("[0-9a-f]{64}", sha256):
        raise ValueError("inventory needs a valid executable_sha256")
    for report in reports:
        if report["application"] and sha256 != report["application"]["sha256"]:
            raise ValueError("inventory and contract executable identities differ")
    result = json.loads(json.dumps(inventory))
    used = set()
    verified = 0
    diagnostics = []
    for row in result["blocks"]:
        if type(row) is not dict:
            raise ValueError("inventory block must be an object")
        if (row.get("xml_type") is not None and type(row["xml_type"]) is not str
                or row.get("control_type") is not None and type(row["control_type"]) is not str):
            raise ValueError("inventory xml_type/control_type must be strings or unknown")
        identity = (row.get("xml_type"), row.get("control_type"))
        matches = [report for report in reports if (
            report["block_type"], str(report["control_type"])) == identity]
        duplicates = sum((other.get("xml_type"), other.get("control_type")) == identity
                         for other in result["blocks"])
        row["verified_evidence"] = {"status": "unverified", "report": None}
        if len(matches) == 1 and duplicates == 1:
            report = matches[0]
            row["verified_evidence"] = {
                "status": "complete" if report["gate"]["complete"] else "partial",
                "report": report,
            }
            used.add(id(report))
            verified += report["gate"]["complete"]
        elif matches:
            diagnostics.append({"code": "inventory_identity_ambiguous",
                                "block_type": identity[0], "control_type": identity[1]})
    result.setdefault("summary", {})["gui_parity_verified"] = verified
    result["evidence_summary"] = {
        "matched_contracts": len(used), "unmatched_contracts": len(reports) - len(used),
        "verified_complete_blocks": verified, "catalog_records": len(result["blocks"]),
        "percentage": None,
        "denominator_status": "catalog inventory is not a GUI scope denominator",
        "diagnostics": diagnostics,
        "complete": False,
    }
    for report in reports:
        if id(report) not in used:
            diagnostics.append({"code": "contract_not_uniquely_matched",
                                "block_type": report["block_type"],
                                "control_type": report["control_type"]})
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--contract", type=Path, action="append", required=True)
    parser.add_argument("--results", type=Path, action="append", default=[])
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--inventory", type=Path)
    parser.add_argument("--application-version")
    parser.add_argument("--application-sha256")
    parser.add_argument("--report", action="store_true", help="Report valid partial evidence without failing the gate")
    args = parser.parse_args(argv)
    target = APPLICATION.copy()
    if bool(args.application_version) != bool(args.application_sha256):
        parser.error("specify both application identity flags")
    if args.application_version:
        target = {"version": args.application_version, "sha256": args.application_sha256}
    try:
        _schema_check(target, {"$ref": "#/$defs/application"}, "ux-contract.schema.json")
        results = {}
        for path in args.results:
            document = load_json(path)
            validate_document(document, "ux-results.schema.json")
            key = document["contract_sha256"]
            if key in results:
                raise ValueError("duplicate/conflicting results for one contract")
            results[key] = path
        reports, identities = [], set()
        for path in args.contract:
            report = verify_files(path, results.pop(digest(path), None), args.root, target)
            identity = (report["block_type"], report["control_type"])
            if identity in identities:
                raise ValueError("duplicate/conflicting contract block identity")
            identities.add(identity)
            reports.append(report)
        if results:
            raise ValueError("unmatched/stale results contract_sha256")
        output = {"schema_version": 1, "reports": reports}
        if args.inventory:
            output["inventory"] = enrich_inventory(load_json(args.inventory), reports)
        invalid = any(not report["gate"]["input_valid"] for report in reports)
        complete = bool(reports) and all(report["gate"]["complete"] for report in reports)
        if args.inventory and output["inventory"]["evidence_summary"]["diagnostics"]:
            complete = False
        output["gate"] = {"complete": complete, "input_valid": not invalid,
                          "scope": "provided_contracts_not_entire_catalog"}
        print(json.dumps(output, indent=2))
        return 2 if invalid else 0 if complete or args.report else 1
    except (OSError, ValueError, KeyError) as error:
        print(json.dumps({"schema_version": 1, "gate": {"complete": False, "input_valid": False},
                          "diagnostics": [{"code": "input_invalid", "message": str(error)}]}, indent=2))
        return 2


if __name__ == "__main__":
    sys.exit(main())
