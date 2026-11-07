#!/usr/bin/env python3
"""Manually check an explicitly owned automation-window.ps1 fixture and measure batching."""

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time


ROOT = Path(__file__).resolve().parent.parent
CLI = ROOT / "scripts" / "loxone-ux.py"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", required=True, type=Path)
    parser.add_argument("--pid", required=True, type=int)
    parser.add_argument("--expected-sha", required=True)
    parser.add_argument("--output", required=True, type=Path)
    args = parser.parse_args()
    if args.output.exists():
        parser.error("output already exists; preserve earlier measurements")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    prefix = [
        sys.executable, "-B", str(CLI), "--profile", str(args.profile),
        "--pid", str(args.pid), "--expected-sha", args.expected_sha,
    ]
    records = []

    def invoke(label, tail, expected=None, override=None):
        argv = prefix.copy()
        if override is not None:
            argv[argv.index("--expected-sha") + 1] = override
        start = time.monotonic()
        process = subprocess.run(argv + tail, capture_output=True, check=False)
        try:
            result = json.loads(process.stdout)
        except (ValueError, UnicodeError) as error:
            raise RuntimeError(f"{label}: client did not return JSON") from error
        error_code = (result.get("error") or {}).get("code")
        if expected is None:
            if process.returncode or result.get("ok") is not True:
                raise RuntimeError(f"{label}: unexpected failure {error_code}")
        elif process.returncode != 1 or error_code != expected:
            raise RuntimeError(f"{label}: expected {expected}, received {error_code}")
        record = {
            "label": label,
            "expected_error": expected,
            "wall_ms": round((time.monotonic() - start) * 1000, 3),
            "stdout_bytes": len(process.stdout),
            "response": result,
        }
        records.append(record)
        return result

    def selector(name, kind):
        return json.dumps({"name": name, "control_type": kind})

    report = {
        "kind": "owned_driver_check_not_gui_parity",
        "records": records,
        "local_source_sha256": {
            name: hashlib.sha256((ROOT / name).read_bytes()).hexdigest()
            for name in (
                "scripts/loxone-ux.py", "scripts/loxone_ux_transport.py",
                "scripts/loxone-ux.ps1", "tests/ux/fixtures/automation-window.ps1",
            )
        },
        "scope": "One owned synthetic WPF fixture; not Config behavior, model-token usage, or a statistical benchmark.",
    }
    try:
        batch = invoke("four-step-batch", [
            "run", str(ROOT / "tests" / "ux" / "recipes" / "fixture-roundtrip.json"),
        ])
        separate_start = len(records)
        for label, tail in [
            ("focus", ["focus", "--selector", selector("Lox UX owned fixture", "Window")]),
            ("set", ["set-value", "100% {literal}+ caf\u00e9", "--selector", selector("Fixture value", "Edit")]),
            ("read", ["inspect", "--selector", selector("Fixture value", "Edit"), "--limit", "1", "--depth", "0"]),
            ("record", ["invoke", "--selector", selector("Record value", "Button")]),
        ]:
            invoke(label, tail)
        separate = records[separate_start:]
        mismatch = invoke("stale-version", ["status"], "version_mismatch", "0" * 64)
        if mismatch["completed_steps"] != 0:
            raise RuntimeError("stale-version executed a step")
        ambiguous = invoke("stop-on-ambiguity", [
            "run", str(ROOT / "tests" / "ux" / "recipes" / "fixture-stop-on-ambiguity.json"),
        ], "selector_ambiguous")
        if ambiguous["completed_steps"] != 1 or ambiguous["failed_step"] != 1:
            raise RuntimeError("ambiguous input did not stop at the first failing step")
        matches = invoke("inspect-ambiguous-without-input", [
            "inspect", "--selector", selector("Ambiguous", "Button"), "--limit", "3", "--depth", "0",
        ])
        if len(matches["results"][0]["value"]["elements"]) != 2:
            raise RuntimeError("read-only inspection did not expose both exact matches")
        final = invoke("unchanged-after-failure", [
            "inspect", "--selector", selector("Fixture value", "Edit"), "--limit", "1", "--depth", "0",
        ])
        readings = [item for item in final["results"] if item["op"] == "inspect"]
        if not readings or readings[0].get("omitted"):
            raise RuntimeError("bounded readback was not available")
        if readings[0]["value"]["elements"][0]["value"] != "100% {literal}+ caf\u00e9":
            raise RuntimeError("the step after ambiguity changed the fixture")
        report["ok"] = True
        report["comparison"] = {
            "steps": 4,
            "batch": {
                "cli_invocations": 1,
                "mcp_tool_calls": batch["transport_metrics"]["mcp_tool_calls"],
                "stdout_bytes": records[0]["stdout_bytes"],
                "wall_ms": records[0]["wall_ms"],
            },
            "separate": {
                "cli_invocations": len(separate),
                "mcp_tool_calls": sum(item["response"]["transport_metrics"]["mcp_tool_calls"] for item in separate),
                "stdout_bytes": sum(item["stdout_bytes"] for item in separate),
                "wall_ms": round(sum(item["wall_ms"] for item in separate), 3),
            },
        }
    except (OSError, RuntimeError, KeyError) as error:
        report["ok"] = False
        report["error"] = str(error)
    try:
        with args.output.open("x", encoding="utf-8") as output:
            output.write(json.dumps(report, indent=2) + "\n")
    except OSError as error:
        report["ok"] = False
        report["error"] = f"Cannot persist the measurement without overwriting: {error}"
    print(json.dumps({key: value for key, value in report.items() if key != "records"}))
    return 0 if report["ok"] else 1


if __name__ == "__main__":
    sys.exit(main())
