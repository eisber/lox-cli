#!/usr/bin/env python3
"""Derive a separate, incomplete EnergyManager2 CLI evidence snapshot."""

import argparse
import copy
import importlib.util
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location(
    "verify_ux_contracts", Path(__file__).with_name("verify-ux-contracts.py")
)
GATES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(GATES)


def checked_artifacts(artifacts, label):
    report = {"diagnostics": []}
    valid, _ = GATES._artifacts(report, ROOT, artifacts, label)
    if not valid:
        raise ValueError(f"{label}: {report['diagnostics'][0]['message']}")


def derive(run_directory, contract_path, results_path):
    run_directory = Path(run_directory).resolve()
    original = GATES.load_json(ROOT / "contracts" / "energy-manager2.json")
    GATES.validate_document(original, "ux-contract.schema.json")
    record_path = run_directory / "runner-record.json"
    record = GATES.load_json(record_path)
    GATES.validate_document(record, "ux-runner-record.schema.json")
    metadata = GATES.load_json(run_directory / "metadata.json")
    checked_artifacts([metadata["case_definitions"], *metadata["source_artifacts"]], "source snapshots")
    definitions = GATES.load_json(
        GATES._artifact_path(ROOT, metadata["case_definitions"]["path"])
    )
    for key in ("application", "block_type", "control_type"):
        if definitions[key] != original[key]:
            raise ValueError(f"case definition identity differs: {key}")
    if (record["application"] != original["application"]
            or record["kind"] != "cli_execution" or record["synthetic"]):
        raise ValueError("expected genuine CLI execution for the recovered binding target")
    observed = {case["id"]: case for case in record["observations"]}
    defined = {case["id"]: case for case in definitions["cases"]}
    if (len(observed) != len(record["observations"]) or len(defined) != len(definitions["cases"])
            or observed.keys() != defined.keys()):
        raise ValueError("runner and definition IDs must match uniquely")
    for case_id, observation in observed.items():
        if (observation["evidence_kind"] not in ("implementation", "unit_test")
                or observation["evidence_kind"] != defined[case_id]["evidence_kind"]):
            raise ValueError(f"not an independently defined CLI/unit case: {case_id}")
        checked_artifacts(observation["artifacts"], case_id)
    mappings = {mapping["capability_id"]: mapping for mapping in definitions["capability_mappings"]}
    if len(mappings) != len(definitions["capability_mappings"]):
        raise ValueError("duplicate capability mapping")
    original_caps = {cap["id"]: cap for cap in original["capabilities"]}
    if mappings.keys() - original_caps.keys():
        raise ValueError("mapping targets unknown original capabilities")
    capabilities, gui_ids = [], set()
    for capability_id, mapping in mappings.items():
        cap = copy.deepcopy(original_caps[capability_id])
        gui_ids.update(cap["required_case_ids"])
        required = mapping["required_case_ids"]
        if ({defined[case_id]["evidence_kind"] for case_id in required}
                != {"implementation", "unit_test"}
                or any(defined[case_id]["capability_ids"] != [capability_id] for case_id in required)):
            raise ValueError(f"invalid narrow CLI/unit mapping: {capability_id}")
        cap["required_case_ids"].extend(required)
        cap["source_evidence"] = [
            source for source in cap["source_evidence"]
            if source["evidence_kind"] in ("catalog", "native")
        ]
        cap["binding"]["reason"] = (
            cap["binding"].get("reason", "") + " CLI snapshot condition scope: " + mapping["condition_scope"]
            + " CLI subset omissions: " + "; ".join(mapping["omissions"])
            + ". Original GUI recovery is separate and is not imported as this run's evidence."
        )
        capabilities.append(cap)
    gui_cases = [copy.deepcopy(case) for case in original["cases"] if case["id"] in gui_ids]
    if (len(gui_cases) != len(gui_ids)
            or any(not set(case["capability_ids"]).issubset(mappings) for case in gui_cases)):
        raise ValueError("original GUI case definitions cannot be retained verbatim in this subset")
    excluded = [cap["id"] for cap in original["capabilities"] if cap["id"] not in mappings]
    contract = {
        "schema_version": 1,
        **{key: original[key] for key in ("application", "block_type", "control_type")},
        "scope": {
            "status": "incomplete",
            "description": (
                "Separate CLI/unit snapshot of " + str(len(mappings))
                + " precisely mapped capabilities, not the full EnergyManager2 contract. "
                "Actual offline CLI/unit execution uses disposable independently authored input; "
                "the Config application identity is a recovered binding TARGET, not an executed GUI. "
                "Per-capability condition scopes and omissions are preserved in binding.reason. "
                "Original GUI requirements remain missing; no old GUI observations are imported. "
                "Excluded full-scope capabilities: " + ", ".join(excluded)
                + ". Current icon/deadline condition scopes and explicit omissions come "
                "from this run's definitions, not an assumed support level. Separate native GUI follow-ups are not imported. "
                "Runtime dependencies, numeric GUI filtering/overflow equivalence, structural CRUD, "
                "reference validation, migration and dirty-cancel feedback are not complete. "
                "Fresh combined GUI/CLI/unit proof is required before any parity claim."
            ),
            "source_evidence": [
                copy.deepcopy(source) for source in original["scope"]["source_evidence"]
                if source["evidence_kind"] in ("catalog", "native")
            ],
        },
        "capabilities": capabilities,
        "cases": copy.deepcopy(definitions["cases"]) + gui_cases,
    }
    GATES.validate_document(contract, "ux-contract.schema.json")
    for owner in [contract["scope"], *capabilities]:
        for source in owner["source_evidence"]:
            checked_artifacts(source["artifacts"], "catalog/native source")
    contract_path = Path(contract_path)
    results_path = Path(results_path)
    if contract_path.exists() or results_path.exists():
        raise ValueError("refusing to overwrite an existing contract/results snapshot")
    results_path.parent.mkdir(parents=True, exist_ok=True)
    contract_path.write_text(json.dumps(contract, indent=2) + "\n", encoding="utf-8", newline="\n")
    result = {
        "schema_version": 1, "contract_sha256": GATES.digest(contract_path),
        **{key: original[key] for key in ("application", "block_type", "control_type")},
        "run": {
            **{key: copy.deepcopy(record[key]) for key in (
                "id", "runner", "started_at", "finished_at", "application", "synthetic")},
            "record": {"path": str(record_path.relative_to(ROOT)), "sha256": GATES.digest(record_path)},
        },
        "cases": copy.deepcopy(record["observations"]),
    }
    GATES.validate_document(result, "ux-results.schema.json")
    results_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8", newline="\n")
    report = GATES.verify_files(contract_path, results_path, ROOT)
    if (not report["gate"]["input_valid"] or report["gate"]["complete"]
            or report["stages"]["implementation"]["verified"] != len(mappings)
            or report["stages"]["unit_test"]["verified"] != len(mappings)
            or report["stages"]["gui_oracle"]["verified"]
            or report["stages"]["complete"]["verified"]):
        raise ValueError(f"snapshot failed the expected partial gates: {report['diagnostics']}")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--run-directory", type=Path, required=True)
    parser.add_argument("--contract", type=Path, default=ROOT / "tests" / "ux" / "energy-manager2" / "cli-contract.json")
    parser.add_argument("--results", type=Path, required=True)
    args = parser.parse_args()
    try:
        report = derive(args.run_directory, args.contract, args.results)
        print(json.dumps({key: report[key] for key in ("gate", "case_counts", "stages")}, indent=2))
    except (OSError, ValueError, KeyError) as error:
        parser.exit(2, f"Energy CLI snapshot failed: {error}\n")


if __name__ == "__main__":
    main()
