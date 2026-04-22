#!/usr/bin/env python3
"""Eval coverage report — block type, input connector, and output connector coverage.

Two coverage dimensions:

1. **Aggregate coverage**: Across ALL eval cases, what % of Loxone's 195 block
   types and 2806 connectors are exercised? Identifies gaps in the eval suite.

2. **Per-case spec coverage**: For each eval case, what % of the new blocks'
   connectors are pinned by wiring specs? Low coverage = eval is too loose,
   the agent could wire things wrong and still pass.

Usage:
    python3 tests/eval/coverage.py                    # summary (both)
    python3 tests/eval/coverage.py --detail            # per-type breakdown
    python3 tests/eval/coverage.py --cases             # per-case spec tightness
    python3 tests/eval/coverage.py --json              # machine-readable
    python3 tests/eval/coverage.py --uncovered         # list uncovered types
"""

import json
import glob
import os
import sys
from collections import Counter, defaultdict
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent.parent.parent
CONNECTOR_MAP = REPO / "docs" / "schemas" / "connector-map.json"
CASES_DIR = REPO / "tests" / "eval" / "cases"
REPORT_OUT = REPO / "tests" / "eval" / "reports" / "coverage-report.json"


def load_connector_schema():
    """Load full block type schema with all input/output connectors."""
    with open(CONNECTOR_MAP) as f:
        cm = json.load(f)

    schema = {}
    for typ, data in cm.items():
        t = data.get("t", {})
        inputs = sorted(k for k, v in t.items() if v == "I")
        outputs = sorted(k for k, v in t.items() if v == "O")
        schema[typ] = {"inputs": inputs, "outputs": outputs}
    return schema


def load_eval_cases():
    """Load all eval cases and extract block types + connector usage."""
    block_types = Counter()
    # type -> set of input connector names wired as target
    input_usage = defaultdict(set)
    # type -> set of output connector names wired as source
    output_usage = defaultdict(set)
    # type -> set of case IDs that use it
    type_cases = defaultdict(set)
    total_cases = 0

    for f in sorted(CASES_DIR.glob("*.json")):
        cases = json.load(open(f))
        for c in cases:
            cid = c.get("id", "?")
            total_cases += 1
            exp = c.get("expected", {})

            # Block types from new_blocks
            for b in exp.get("new_blocks", []):
                if isinstance(b, dict):
                    t = b.get("type", b.get("Type", ""))
                    if t:
                        block_types[t] += 1
                        type_cases[t].add(cid)

            # Wiring: extract connector usage
            for w in exp.get("wiring", []):
                if not isinstance(w, dict):
                    continue
                ft = w.get("from_type", "")
                fc = w.get("from_connector", w.get("from_conn", ""))
                tt = w.get("to_type", "")
                tc = w.get("to_connector", w.get("to_conn", ""))

                # Source side = output connector of from_type
                if ft and fc:
                    output_usage[ft].add(fc)
                    type_cases[ft].add(cid)
                elif ft:
                    type_cases[ft].add(cid)

                # Target side = input connector of to_type
                if tt and tc:
                    input_usage[tt].add(tc)
                    type_cases[tt].add(cid)
                elif tt:
                    type_cases[tt].add(cid)

    return {
        "block_types": block_types,
        "input_usage": input_usage,
        "output_usage": output_usage,
        "type_cases": type_cases,
        "total_cases": total_cases,
    }


def compute_coverage(schema, eval_data):
    """Compute coverage metrics."""
    total_types = len(schema)
    covered_types = set(eval_data["block_types"].keys())

    # Also count types that appear in wiring but not in new_blocks
    for t in list(eval_data["type_cases"].keys()):
        covered_types.add(t)

    # Remove types not in schema (custom/unknown)
    valid_covered = covered_types & set(schema.keys())
    unknown_types = covered_types - set(schema.keys())

    # Per-type connector coverage
    type_coverage = {}
    total_inputs_in_schema = 0
    total_outputs_in_schema = 0
    covered_inputs = 0
    covered_outputs = 0

    for typ, io in schema.items():
        n_in = len(io["inputs"])
        n_out = len(io["outputs"])
        total_inputs_in_schema += n_in
        total_outputs_in_schema += n_out

        used_in = eval_data["input_usage"].get(typ, set()) & set(io["inputs"])
        used_out = eval_data["output_usage"].get(typ, set()) & set(io["outputs"])
        covered_inputs += len(used_in)
        covered_outputs += len(used_out)

        in_pct = len(used_in) / n_in * 100 if n_in > 0 else None
        out_pct = len(used_out) / n_out * 100 if n_out > 0 else None

        type_coverage[typ] = {
            "exercised": typ in valid_covered,
            "instances": eval_data["block_types"].get(typ, 0),
            "case_count": len(eval_data["type_cases"].get(typ, set())),
            "total_inputs": n_in,
            "covered_inputs": len(used_in),
            "uncovered_inputs": sorted(set(io["inputs"]) - used_in) if used_in or typ in valid_covered else [],
            "input_pct": round(in_pct, 1) if in_pct is not None else None,
            "total_outputs": n_out,
            "covered_outputs": len(used_out),
            "uncovered_outputs": sorted(set(io["outputs"]) - used_out) if used_out or typ in valid_covered else [],
            "output_pct": round(out_pct, 1) if out_pct is not None else None,
        }

    # Categorize block types by importance tier
    tier1_types = {  # Core automation — must have coverage
        "And", "Or", "Not", "Xor", "FlipFlop", "SRFlipFlop", "RSFlipFlop",
        "GreaterEqual", "Greater", "Less", "LessEqual", "Equal", "NotEqual",
        "Memory", "AMemory", "Counter", "State", "StateV",
        "Monoflop", "OnPulseDelay", "OnDelay", "OffDelay", "OnOffDelay",
        "StairwayLS", "PulseGen", "PulseAt", "PulseBy", "DayTimer",
        "Add", "Sub", "Mult", "Div", "Mod", "Formula", "Average", "Avg",
        "LightController2", "JalousieUpDown2", "AutoJalousie",
        "HeatIRoomController2", "Heatmixer2", "Ventilation",
        "PushButton", "PushButton2", "EdgeDetection", "LongClick",
        "InputRef", "OutputRef", "EIBsensor", "EIBactor",
    }
    tier2_types = {  # Common controls
        "AnalogThresholdTrigger", "AnalogComparator", "AnalogScaler",
        "AnalogMultiplexer", "AnalogMultiplexer2", "Minmax", "AMinmax",
        "Alarm", "AlarmClock", "CentralLight", "CentralShade",
        "Presence", "PresenceController", "PresenceDetector",
        "Irrigation", "PoolController", "Sauna", "SaunaVapor",
        "EnergyManager", "EnergyManager2", "Energy",
        "Door", "Doorcontroller", "NfcCodeTouch",
        "PI", "PID", "PWM", "Ramp", "Rand", "RandomGen",
        "HVACController", "Heatmixer", "Heatcurve", "DewPoint",
        "Fan", "Fancoil", "FancoilFreshAir", "ToiletFan",
    }

    tier1_covered = len(tier1_types & valid_covered)
    tier2_covered = len(tier2_types & valid_covered)
    tier1_total = len(tier1_types & set(schema.keys()))
    tier2_total = len(tier2_types & set(schema.keys()))

    return {
        "summary": {
            "total_block_types": total_types,
            "covered_block_types": len(valid_covered),
            "block_type_coverage_pct": round(len(valid_covered) / total_types * 100, 1),
            "total_input_connectors": total_inputs_in_schema,
            "covered_input_connectors": covered_inputs,
            "input_coverage_pct": round(covered_inputs / total_inputs_in_schema * 100, 1)
            if total_inputs_in_schema > 0
            else 0,
            "total_output_connectors": total_outputs_in_schema,
            "covered_output_connectors": covered_outputs,
            "output_coverage_pct": round(covered_outputs / total_outputs_in_schema * 100, 1)
            if total_outputs_in_schema > 0
            else 0,
            "total_eval_cases": eval_data["total_cases"],
            "total_wiring_specs": sum(
                len(v) for v in eval_data["input_usage"].values()
            )
            + sum(len(v) for v in eval_data["output_usage"].values()),
            "tier1_coverage": f"{tier1_covered}/{tier1_total} ({round(tier1_covered/tier1_total*100)}%)"
            if tier1_total > 0
            else "N/A",
            "tier2_coverage": f"{tier2_covered}/{tier2_total} ({round(tier2_covered/tier2_total*100)}%)"
            if tier2_total > 0
            else "N/A",
        },
        "tier1_uncovered": sorted(
            (tier1_types & set(schema.keys())) - valid_covered
        ),
        "tier2_uncovered": sorted(
            (tier2_types & set(schema.keys())) - valid_covered
        ),
        "unknown_types": sorted(unknown_types),
        "type_coverage": type_coverage,
    }


def print_summary(cov):
    """Print human-readable coverage summary."""
    s = cov["summary"]
    print("=" * 65)
    print("              EVAL COVERAGE REPORT")
    print("=" * 65)
    print()
    bar = lambda pct: "█" * int(pct / 2.5) + "░" * (40 - int(pct / 2.5))
    print(f"  Block types:  {s['covered_block_types']:3d}/{s['total_block_types']}  "
          f"{bar(s['block_type_coverage_pct'])} {s['block_type_coverage_pct']:5.1f}%")
    print(f"  Input conns:  {s['covered_input_connectors']:3d}/{s['total_input_connectors']}  "
          f"{bar(s['input_coverage_pct'])} {s['input_coverage_pct']:5.1f}%")
    print(f"  Output conns: {s['covered_output_connectors']:3d}/{s['total_output_connectors']}  "
          f"{bar(s['output_coverage_pct'])} {s['output_coverage_pct']:5.1f}%")
    print()
    print(f"  Tier 1 (core):    {s['tier1_coverage']}")
    print(f"  Tier 2 (common):  {s['tier2_coverage']}")
    print(f"  Eval cases: {s['total_eval_cases']}  |  Wiring specs: {s['total_wiring_specs']}")
    print()

    if cov["tier1_uncovered"]:
        print("  ⚠ Tier 1 uncovered (core blocks needing eval cases):")
        for t in cov["tier1_uncovered"]:
            print(f"    - {t}")
        print()

    if cov["unknown_types"]:
        print(f"  ℹ Types in evals but not in connector-map: {cov['unknown_types']}")
        print()


def print_detail(cov):
    """Print per-type breakdown."""
    print()
    print(f"{'Type':<30s} {'Inst':>4s} {'Cases':>5s} "
          f"{'In':>3s}/{' Tot':>3s} {'InPct':>5s}  "
          f"{'Out':>3s}/{' Tot':>3s} {'OutPct':>6s}")
    print("-" * 85)

    tc = cov["type_coverage"]
    # Show exercised types first, sorted by instance count
    exercised = sorted(
        [(t, d) for t, d in tc.items() if d["exercised"]],
        key=lambda x: -x[1]["instances"],
    )
    uncovered = sorted(
        [(t, d) for t, d in tc.items() if not d["exercised"]],
        key=lambda x: x[0],
    )

    for t, d in exercised:
        in_s = f"{d['input_pct']:5.1f}%" if d["input_pct"] is not None else "  N/A"
        out_s = f"{d['output_pct']:5.1f}%" if d["output_pct"] is not None else "   N/A"
        print(f"  ✓ {t:<28s} {d['instances']:4d} {d['case_count']:5d} "
              f"{d['covered_inputs']:3d}/{d['total_inputs']:3d} {in_s}  "
              f"{d['covered_outputs']:3d}/{d['total_outputs']:3d} {out_s}")

    if "--uncovered" in sys.argv:
        print()
        print("--- Uncovered types ---")
        for t, d in uncovered:
            print(f"  ✗ {t:<28s}       "
                  f"      {d['total_inputs']:3d}         "
                  f"      {d['total_outputs']:3d}")


def compute_case_spec_coverage(schema):
    """Per-case spec coverage: how tightly is each eval case specified?

    For each case, counts how many connectors on the NEW blocks are pinned
    by explicit wiring specs (to_connector/from_connector) and how many
    params are checked. Returns coverage = pinned / total_surface.
    """
    results = []

    for f in sorted(CASES_DIR.glob("*.json")):
        category = f.stem
        for c in json.load(open(f)):
            cid = c.get("id", "?")
            exp = c.get("expected", {})
            blocks = exp.get("new_blocks", [])
            wiring = exp.get("wiring", [])
            params = exp.get("params", [])
            difficulty = c.get("difficulty", "?")

            # New block types + their total connectors
            block_type_list = []
            new_types = set()
            total_inputs = 0
            total_outputs = 0
            for b in blocks:
                t = b.get("type", "") if isinstance(b, dict) else ""
                block_type_list.append(t)
                new_types.add(t)
                if t in schema:
                    io = schema[t]
                    total_inputs += len(io["inputs"])
                    total_outputs += len(io["outputs"])

            # Count connector pins on new blocks specifically
            pinned_in = set()
            pinned_out = set()
            wires_to_new = 0  # wiring specs touching new blocks at all
            for w in wiring:
                ft = w.get("from_type", "")
                fc = w.get("from_connector", w.get("from_conn", ""))
                tt = w.get("to_type", "")
                tc = w.get("to_connector", w.get("to_conn", ""))

                touches_new = ft in new_types or tt in new_types
                if touches_new:
                    wires_to_new += 1

                if ft in new_types and fc:
                    pinned_out.add((ft, fc))
                if tt in new_types and tc:
                    pinned_in.add((tt, tc))

            n_params = len(params) if isinstance(params, list) else 0
            total_surface = total_inputs + total_outputs
            pinned = len(pinned_in) + len(pinned_out)

            if total_surface > 0:
                connector_pct = min(pinned / total_surface * 100, 100)
            elif len(blocks) == 0:
                connector_pct = 100  # no new blocks = nothing to check
            else:
                connector_pct = 0

            results.append({
                "id": cid,
                "category": category,
                "difficulty": difficulty,
                "blocks": len(blocks),
                "block_types": block_type_list,
                "total_inputs": total_inputs,
                "total_outputs": total_outputs,
                "total_surface": total_surface,
                "pinned_inputs": len(pinned_in),
                "pinned_outputs": len(pinned_out),
                "pinned_total": pinned,
                "wiring_specs": len(wiring),
                "wires_to_new_blocks": wires_to_new,
                "param_specs": n_params,
                "connector_coverage_pct": round(connector_pct, 1),
            })

    return results


def print_case_coverage(case_cov):
    """Print per-case spec coverage report."""
    case_cov.sort(key=lambda x: x["connector_coverage_pct"])
    total = len(case_cov)

    # Distribution
    z = sum(1 for r in case_cov if r["connector_coverage_pct"] == 0 and r["blocks"] > 0)
    lo = sum(1 for r in case_cov if 0 < r["connector_coverage_pct"] <= 25)
    md = sum(1 for r in case_cov if 25 < r["connector_coverage_pct"] <= 50)
    hi = sum(1 for r in case_cov if r["connector_coverage_pct"] > 50)
    na = sum(1 for r in case_cov if r["blocks"] == 0)
    avg = sum(r["connector_coverage_pct"] for r in case_cov) / total if total else 0

    bar = lambda pct: "█" * int(pct / 2.5) + "░" * (40 - int(pct / 2.5))

    print()
    print("=" * 65)
    print("         PER-CASE SPEC COVERAGE (connector tightness)")
    print("=" * 65)
    print()
    print(f"  Average:  {bar(avg)} {avg:5.1f}%")
    print()
    print(f"    0% (loose):  {z:3d} cases — wiring exists but no connector names")
    print(f"   1-25%:        {lo:3d} cases")
    print(f"  26-50%:        {md:3d} cases")
    print(f"  51-100%:       {hi:3d} cases — well specified")
    if na:
        print(f"  N/A (no blocks):{na:3d} cases")
    print()

    # By category
    cat_stats = defaultdict(list)
    for r in case_cov:
        cat_stats[r["category"]].append(r["connector_coverage_pct"])

    print("  By category:")
    for cat in sorted(cat_stats):
        vals = cat_stats[cat]
        a = sum(vals) / len(vals)
        print(f"    {cat:<20s}  avg={a:5.1f}%  n={len(vals)}")
    print()

    # Bottom 15 — most in need of tightening
    loose = [r for r in case_cov if r["connector_coverage_pct"] == 0 and r["blocks"] > 0]
    loose.sort(key=lambda x: -(x["total_surface"]))
    if loose:
        print(f"  Loosest cases (0% conn coverage, largest surface):")
        print(f"  {'ID':<42s} {'Blk':>3s} {'Surface':>7s} {'Wires':>5s} {'Params':>6s}")
        print(f"  {'-'*42} {'---':>3s} {'-------':>7s} {'-----':>5s} {'------':>6s}")
        for r in loose[:15]:
            print(f"  {r['id']:<42s} {r['blocks']:3d} "
                  f"{r['total_inputs']}i+{r['total_outputs']}o"
                  f"  {r['wiring_specs']:5d} {r['param_specs']:6d}")


def main():
    schema = load_connector_schema()
    eval_data = load_eval_cases()
    cov = compute_coverage(schema, eval_data)

    if "--json" in sys.argv:
        case_cov = compute_case_spec_coverage(schema)
        report = {
            "aggregate": {
                "summary": cov["summary"],
                "tier1_uncovered": cov["tier1_uncovered"],
                "tier2_uncovered": cov["tier2_uncovered"],
                "unknown_types": cov["unknown_types"],
                "type_coverage": {
                    t: {k: v for k, v in d.items()
                        if k not in ("uncovered_inputs", "uncovered_outputs")}
                    for t, d in cov["type_coverage"].items()
                    if d["exercised"]
                },
            },
            "per_case": case_cov,
        }
        with open(REPORT_OUT, "w") as f:
            json.dump(report, f, indent=2)
        print(f"Coverage report saved to {REPORT_OUT}")
    else:
        print_summary(cov)
        if "--detail" in sys.argv or "--uncovered" in sys.argv:
            print_detail(cov)
        case_cov = compute_case_spec_coverage(schema)
        print_case_coverage(case_cov)
        if "--cases" in sys.argv:
            # Full per-case table
            case_cov.sort(key=lambda x: x["connector_coverage_pct"])
            print()
            print(f"  {'ID':<42s} {'Cov%':>5s} {'Pin':>3s}/{' Srf':>3s} {'Wires':>5s} {'Par':>3s}")
            print(f"  {'-'*42} {'-----':>5s} {'---':>3s} {'---':>3s} {'-----':>5s} {'---':>3s}")
            for r in case_cov:
                if r["blocks"] == 0:
                    continue
                print(f"  {r['id']:<42s} {r['connector_coverage_pct']:5.1f} "
                      f"{r['pinned_total']:3d}/{r['total_surface']:3d} "
                      f"{r['wiring_specs']:5d} {r['param_specs']:3d}")


if __name__ == "__main__":
    main()
