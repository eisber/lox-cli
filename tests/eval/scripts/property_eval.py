#!/usr/bin/env python3
"""
Property-based evaluation engine for Loxone circuits.

Verifies circuit behavior using type+room matching instead of hardcoded block
names. This handles the ~28% name-mismatch failures by checking that ANY block
of the expected type in the expected room satisfies the property condition.

Usage:
    # Standalone
    python3 property_eval.py --config path/to/file.Loxone --property '{"when":...}'

    # From eval-agent.py
    from property_eval import evaluate_property, spec_to_property, PropertyEvaluator
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

SCRIPT_DIR = Path(__file__).parent
EVAL_DIR = SCRIPT_DIR.parent
REPO_ROOT = EVAL_DIR.parent.parent


# ── Data model ───────────────────────────────────────────────

@dataclass
class Property:
    """A behavioral property to verify against a circuit.

    Attributes:
        when: Input conditions to inject (sensor_name → value).
        then_some: At least ONE block of this type in this room must satisfy.
        then_none: NO block of this type in this room should satisfy.
        name: Human-readable description of this property.
    """
    when: dict[str, float]
    then_some: list[dict] = field(default_factory=list)
    then_none: list[dict] = field(default_factory=list)
    name: str = ""

    def to_dict(self) -> dict:
        return {
            "name": self.name,
            "when": self.when,
            "then_some": self.then_some,
            "then_none": self.then_none,
        }

    @classmethod
    def from_dict(cls, d: dict) -> "Property":
        return cls(
            when=d.get("when", {}),
            then_some=d.get("then_some", []),
            then_none=d.get("then_none", []),
            name=d.get("name", ""),
        )


# ── Helpers ──────────────────────────────────────────────────

def _find_lox_binary() -> str:
    """Locate the lox CLI binary."""
    lox = shutil.which("lox")
    if lox:
        return lox
    candidate = REPO_ROOT / "target" / "release" / "lox"
    if candidate.exists():
        return str(candidate)
    return "lox"


def _compare(actual: float, op: str, expected: float) -> bool:
    """Evaluate a comparison operator."""
    if op == ">":
        return actual > expected
    elif op == ">=":
        return actual >= expected
    elif op == "<":
        return actual < expected
    elif op == "<=":
        return actual <= expected
    elif op == "==" or op == "=":
        return abs(actual - expected) < 1e-6
    elif op == "!=":
        return abs(actual - expected) >= 1e-6
    return False


_TRACE_KEY_RE = re.compile(
    r"^(?P<name>.+?)(?:\s+\[(?P<room>[^\]]+)\])?\.(?P<connector>[^.]+)$"
)


def parse_trace_key(key: str) -> dict[str, str]:
    """Parse a trace key like 'Jalousie 1 [Wohnzimmer].InputTriggerDown'.

    Returns: {"name": "Jalousie 1", "room": "Wohnzimmer", "connector": "InputTriggerDown"}
    """
    m = _TRACE_KEY_RE.match(key)
    if m:
        return {
            "name": m.group("name"),
            "room": m.group("room") or "",
            "connector": m.group("connector"),
        }
    # Fallback: no room qualifier
    if "." in key:
        block_part, connector = key.rsplit(".", 1)
        return {"name": block_part, "room": "", "connector": connector}
    return {"name": key, "room": "", "connector": ""}


# ── Block index ──────────────────────────────────────────────

def build_block_index(config_path: str) -> dict[str, dict[str, str]]:
    """Build a name → {type, room} mapping using `lox config controls`.

    Returns a dict like:
        {"Jalousie 1": {"type": "JalousieUpDown2", "room": "Wohnzimmer"},
         "Temperatur Threshold": {"type": "GreaterEqual", "room": "Wohnzimmer"}}
    """
    lox_bin = _find_lox_binary()
    try:
        r = subprocess.run(
            [lox_bin, "config", "controls", config_path, "-o", "json",
             "--limit", "9999"],
            capture_output=True, text=True, timeout=30,
        )
        if r.returncode != 0:
            return {}
        controls = json.loads(r.stdout)
        # Handle both list and dict-with-controls-key formats
        if isinstance(controls, dict):
            controls = controls.get("controls", [])
    except (subprocess.TimeoutExpired, json.JSONDecodeError, FileNotFoundError):
        return {}

    index: dict[str, dict[str, str]] = {}
    for c in controls:
        title = c.get("title", "")
        room = c.get("room", "")
        block_type = c.get("control_type", c.get("type", ""))
        if title:
            info = {"type": block_type, "room": room}
            # Bare name — last one wins (may be wrong room)
            index[title] = info
            # Room-qualified name — unambiguous
            if room:
                index[f"{title} [{room}]"] = info
    return index


# ── Simulation runner ────────────────────────────────────────

def run_sim_with_trace(config_path: str, inputs: dict[str, float],
                       ticks: int = 10, dt: float = 0.1) -> dict[str, float]:
    """Run the SPS simulator with trace=true and return all output values.

    Returns a dict of trace keys → values, e.g.:
        {"Jalousie 1 [Wohnzimmer].InputTriggerDown": 1.0, ...}
    """
    lox_bin = _find_lox_binary()

    sim_spec = json.dumps({
        "steps": [
            # Phase 1: baseline (zero all inputs)
            {"inputs": {k: 0.0 for k in inputs}, "ticks": 2, "dt": 0.1},
            # Phase 2: fast response (catch edge triggers)
            {"inputs": inputs, "ticks": 3, "dt": 0.1},
            # Phase 3: slow response (catch timers up to 10 min)
            {"inputs": inputs, "ticks": ticks, "dt": dt},
        ],
        "trace": True,
    }, ensure_ascii=False)

    try:
        r = subprocess.run(
            [lox_bin, "sim", "run", config_path, "--sim", sim_spec],
            capture_output=True, text=True, timeout=30,
        )
    except (subprocess.TimeoutExpired, FileNotFoundError):
        return {}

    # Parse JSON from stdout (skip WARNING lines on stderr)
    for line in r.stdout.strip().split("\n"):
        line = line.strip()
        if line.startswith("{"):
            try:
                data = json.loads(line)
                return data.get("trace", {})
            except json.JSONDecodeError:
                continue
    return {}


# ── Property evaluation ──────────────────────────────────────

def evaluate_property(config_path: str, prop: Property) -> dict[str, Any]:
    """Evaluate a single property against a config file.

    Steps:
        1. Run sim with prop.when inputs and get trace
        2. Build block index (name → type+room)
        3. Parse trace keys and match against type+room criteria
        4. For then_some: check if ANY matching block satisfies condition
        5. For then_none: check if ALL matching blocks DON'T satisfy

    Returns:
        {"pass": bool, "matched_blocks": [...], "evidence": "...",
         "details": {"then_some": [...], "then_none": [...]}}
    """
    # Step 1: run simulation
    trace = run_sim_with_trace(config_path, prop.when)
    if not trace:
        return {
            "pass": False,
            "matched_blocks": [],
            "evidence": "Simulation returned no trace data",
            "details": {"then_some": [], "then_none": []},
        }

    # Step 2: build block index
    block_index = build_block_index(config_path)

    # Step 3: parse trace and resolve types
    resolved_trace: list[dict] = []
    for key, value in trace.items():
        parsed = parse_trace_key(key)
        name = parsed["name"]
        room = parsed["room"]
        connector = parsed["connector"]

        # Look up type from block index
        block_info = block_index.get(name, {})
        block_type = block_info.get("type", "")
        # If room wasn't in trace key, get it from block index
        if not room:
            room = block_info.get("room", "")

        resolved_trace.append({
            "key": key,
            "name": name,
            "type": block_type,
            "room": room,
            "connector": connector,
            "value": value,
        })

    # Step 4: evaluate then_some conditions
    all_pass = True
    matched_blocks: list[str] = []
    evidence_parts: list[str] = []
    then_some_details: list[dict] = []
    then_none_details: list[dict] = []

    for condition in prop.then_some:
        cond_type = condition.get("type", "")
        cond_room = condition.get("room", "")
        cond_connector = condition.get("connector", "")
        cond_op = condition.get("op", ">")
        cond_value = condition.get("value", 0.5)

        # Find all blocks matching type+room
        candidates = [
            t for t in resolved_trace
            if (not cond_type or t["type"] == cond_type)
            and (not cond_room or t["room"] == cond_room)
            and (not cond_connector or t["connector"] == cond_connector)
        ]

        # Check if ANY candidate satisfies the condition
        satisfied = False
        candidate_details = []
        for c in candidates:
            result = _compare(c["value"], cond_op, cond_value)
            candidate_details.append({
                "block": c["name"],
                "type": c["type"],
                "room": c["room"],
                "connector": c["connector"],
                "actual": c["value"],
                "expected": f"{cond_op} {cond_value}",
                "satisfied": result,
            })
            if result:
                satisfied = True
                matched_blocks.append(c["key"])

        then_some_details.append({
            "condition": condition,
            "satisfied": satisfied,
            "candidates_found": len(candidates),
            "candidates": candidate_details,
        })

        if not satisfied:
            all_pass = False
            if not candidates:
                evidence_parts.append(
                    f"then_some FAIL: no blocks of type={cond_type} "
                    f"room={cond_room} connector={cond_connector} found in trace"
                )
            else:
                evidence_parts.append(
                    f"then_some FAIL: {len(candidates)} blocks of type={cond_type} "
                    f"room={cond_room} found, none satisfy {cond_connector} {cond_op} {cond_value}"
                )
        else:
            evidence_parts.append(
                f"then_some PASS: block satisfies {cond_connector} {cond_op} {cond_value}"
            )

    # Step 5: evaluate then_none conditions
    for condition in prop.then_none:
        cond_type = condition.get("type", "")
        cond_room = condition.get("room", "")
        cond_connector = condition.get("connector", "")
        cond_op = condition.get("op", ">")
        cond_value = condition.get("value", 0.5)

        # Find all blocks matching type+room
        candidates = [
            t for t in resolved_trace
            if (not cond_type or t["type"] == cond_type)
            and (not cond_room or t["room"] == cond_room)
            and (not cond_connector or t["connector"] == cond_connector)
        ]

        # Check that NO candidate satisfies the condition
        any_violates = False
        candidate_details = []
        for c in candidates:
            result = _compare(c["value"], cond_op, cond_value)
            candidate_details.append({
                "block": c["name"],
                "type": c["type"],
                "room": c["room"],
                "connector": c["connector"],
                "actual": c["value"],
                "forbidden": f"{cond_op} {cond_value}",
                "violates": result,
            })
            if result:
                any_violates = True

        then_none_details.append({
            "condition": condition,
            "satisfied": not any_violates,
            "candidates_found": len(candidates),
            "candidates": candidate_details,
        })

        if any_violates:
            all_pass = False
            evidence_parts.append(
                f"then_none FAIL: found block of type={cond_type} "
                f"room={cond_room} where {cond_connector} {cond_op} {cond_value}"
            )
        else:
            evidence_parts.append(
                f"then_none PASS: no blocks violate {cond_connector} {cond_op} {cond_value}"
            )

    return {
        "pass": all_pass,
        "matched_blocks": matched_blocks,
        "evidence": "; ".join(evidence_parts),
        "details": {
            "then_some": then_some_details,
            "then_none": then_none_details,
        },
    }


# ── Convert existing sim specs to properties ─────────────────

def spec_to_property(case: dict) -> list[Property]:
    """Convert existing case simulation specs to Property objects.

    Parses expected_outputs like:
        {"Jalousie 1 [Wohnzimmer].InputTriggerDown": {">": 0.5}}

    And converts to type+room based properties using the trace key format.
    The block name ("Jalousie 1") will be resolved to its type at evaluation
    time via the block index.
    """
    sims = case.get("expected", {}).get("simulation", [])
    if not sims:
        return []

    properties: list[Property] = []
    for sim in sims:
        name = sim.get("name", "unnamed")
        inputs = sim.get("inputs", {})
        expected_outputs = sim.get("expected_outputs", {})

        then_some: list[dict] = []
        then_none: list[dict] = []

        for key, checks in expected_outputs.items():
            parsed = parse_trace_key(key)
            block_name = parsed["name"]
            room = parsed["room"]
            connector = parsed["connector"]

            # We store the block name for later resolution against the index.
            # At eval time, build_block_index resolves name → type.
            for op, value in checks.items():
                entry = {
                    "block_name": block_name,
                    "room": room,
                    "connector": connector,
                    "op": op,
                    "value": value,
                }
                # Conditions expecting high values are "something should happen"
                # Conditions expecting low values are "nothing should happen"
                if op in (">", ">="):
                    then_some.append(entry)
                elif op in ("<", "<="):
                    then_none.append(entry)
                elif op in ("==", "="):
                    then_some.append(entry)
                elif op == "!=":
                    then_none.append(entry)

        if then_some or then_none:
            properties.append(Property(
                when=inputs,
                then_some=then_some,
                then_none=then_none,
                name=name,
            ))

    return properties


def resolve_property_types(prop: Property, block_index: dict[str, dict[str, str]]) -> Property:
    """Resolve block_name references in a property to type+room using the block index.

    This converts properties created by spec_to_property (which have block_name)
    into fully type-based properties suitable for evaluate_property.
    """
    def _resolve_conditions(conditions: list[dict]) -> list[dict]:
        resolved = []
        for cond in conditions:
            block_name = cond.get("block_name", "")
            if block_name and block_name in block_index:
                info = block_index[block_name]
                resolved.append({
                    "type": info["type"],
                    "room": cond.get("room") or info.get("room", ""),
                    "connector": cond.get("connector", ""),
                    "op": cond.get("op", ">"),
                    "value": cond.get("value", 0.5),
                })
            else:
                # Keep as-is if we can't resolve (fallback to room+connector match)
                resolved.append({
                    "type": cond.get("type", ""),
                    "room": cond.get("room", ""),
                    "connector": cond.get("connector", ""),
                    "op": cond.get("op", ">"),
                    "value": cond.get("value", 0.5),
                })
        return resolved

    return Property(
        when=prop.when,
        then_some=_resolve_conditions(prop.then_some),
        then_none=_resolve_conditions(prop.then_none),
        name=prop.name,
    )


# ── PropertyEvaluator: high-level orchestration ──────────────

class PropertyEvaluator:
    """Evaluates properties against a config, caching the block index."""

    def __init__(self, config_path: str):
        self.config_path = config_path
        self._block_index: dict[str, dict[str, str]] | None = None

    @property
    def block_index(self) -> dict[str, dict[str, str]]:
        if self._block_index is None:
            self._block_index = build_block_index(self.config_path)
        return self._block_index

    def evaluate(self, prop: Property) -> dict[str, Any]:
        """Evaluate a property, resolving block names to types first."""
        resolved = resolve_property_types(prop, self.block_index)
        return evaluate_property(self.config_path, resolved)

    def evaluate_case(self, case: dict) -> dict[str, Any]:
        """Convert a case's sim specs to properties and evaluate all of them.

        Returns aggregate result:
            {"pass": bool, "properties": [...], "passed": N, "total": N}
        """
        properties = spec_to_property(case)
        if not properties:
            return {
                "pass": True,
                "properties": [],
                "passed": 0,
                "total": 0,
                "note": "No simulation specs to convert",
            }

        results = []
        passed = 0
        for prop in properties:
            result = self.evaluate(prop)
            result["property_name"] = prop.name
            results.append(result)
            if result["pass"]:
                passed += 1

        return {
            "pass": passed == len(properties),
            "properties": results,
            "passed": passed,
            "total": len(properties),
        }


# ── Integration hook for eval-agent.py ───────────────────────

def evaluate_case_by_property(config_path: str, case: dict) -> dict[str, Any]:
    """Drop-in replacement for sim-spec evaluation using property matching.

    Called from eval-agent.py when --property-eval is set.
    """
    evaluator = PropertyEvaluator(config_path)
    return evaluator.evaluate_case(case)


# ── Example properties ───────────────────────────────────────

EXAMPLE_PROPERTIES = [
    # Case s01: piano protection
    Property(
        name="s01-piano-blinds-close-hot-sunny",
        when={"Außentemperatur": 30, "Sonnenschein": 1},
        then_some=[{
            "type": "JalousieUpDown2",
            "room": "Wohnzimmer",
            "connector": "InputTriggerDown",
            "op": ">",
            "value": 0.5,
        }],
    ),
    # Case s01 negative: blinds stay when cold
    Property(
        name="s01-piano-blinds-stay-cold",
        when={"Außentemperatur": 15, "Sonnenschein": 0},
        then_none=[{
            "type": "JalousieUpDown2",
            "room": "Wohnzimmer",
            "connector": "InputTriggerDown",
            "op": ">",
            "value": 0.5,
        }],
    ),
    # Bathroom humidity → ventilation
    Property(
        name="bathroom-humidity-ventilation",
        when={"Raumfeuchtigkeit Bad": 80},
        then_some=[{
            "type": "VirtualOut",
            "room": "Bad",
            "connector": "I1",
            "op": ">",
            "value": 0.5,
        }],
    ),
    # Wind protection: high wind disables blinds
    Property(
        name="wind-protection-disable-blinds",
        when={"Windgeschwindigkeit": 60},
        then_some=[{
            "type": "JalousieUpDown2",
            "room": "",
            "connector": "InputDisable",
            "op": ">",
            "value": 0.5,
        }],
    ),
    # Rain → close roof window
    Property(
        name="rain-closes-roof-window",
        when={"Regen": 1},
        then_some=[{
            "type": "JalousieUpDown2",
            "room": "",
            "connector": "InputTriggerDown",
            "op": ">",
            "value": 0.5,
        }],
    ),
]


# ── CLI ──────────────────────────────────────────────────────

def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Property-based circuit evaluation engine"
    )
    parser.add_argument("--config", help="Path to .Loxone config file")
    parser.add_argument("--property", help="Property as JSON string")
    parser.add_argument("--case-file", help="Case JSON file to convert and evaluate")
    parser.add_argument("--case-id", help="Case ID within the case file")
    parser.add_argument(
        "--examples", action="store_true",
        help="Run example properties against a config"
    )
    parser.add_argument(
        "--config-dir",
        help="Directory of .Loxone files to test against",
    )
    parser.add_argument("--verbose", "-v", action="store_true")
    parser.add_argument("--json", action="store_true", help="JSON output")
    args = parser.parse_args()

    if args.examples and args.config:
        # Run example properties
        print(f"Evaluating {len(EXAMPLE_PROPERTIES)} example properties against {args.config}")
        evaluator = PropertyEvaluator(args.config)
        results = []
        for prop in EXAMPLE_PROPERTIES:
            result = evaluator.evaluate(prop)
            result["property_name"] = prop.name
            results.append(result)
            status = "✓ PASS" if result["pass"] else "✗ FAIL"
            print(f"  {status}  {prop.name}: {result['evidence'][:80]}")

        passed = sum(1 for r in results if r["pass"])
        print(f"\n{passed}/{len(results)} properties passed")
        if args.json:
            print(json.dumps(results, indent=2, ensure_ascii=False))
        return

    if args.config_dir:
        # Run example properties against all configs in directory
        config_dir = Path(args.config_dir)
        configs = sorted(config_dir.glob("*.Loxone"))[:5]
        if not configs:
            print(f"No .Loxone files found in {config_dir}", file=sys.stderr)
            sys.exit(1)

        print(f"Testing {len(EXAMPLE_PROPERTIES)} properties against {len(configs)} configs\n")
        all_results = []
        for cfg in configs:
            print(f"── {cfg.name} ──")
            evaluator = PropertyEvaluator(str(cfg))
            cfg_results = []
            for prop in EXAMPLE_PROPERTIES:
                result = evaluator.evaluate(prop)
                result["property_name"] = prop.name
                result["config"] = cfg.name
                cfg_results.append(result)
                status = "✓" if result["pass"] else "✗"
                if args.verbose:
                    print(f"  {status} {prop.name}: {result['evidence'][:100]}")
                else:
                    print(f"  {status} {prop.name}")
            passed = sum(1 for r in cfg_results if r["pass"])
            print(f"  → {passed}/{len(cfg_results)} passed\n")
            all_results.extend(cfg_results)

        total_passed = sum(1 for r in all_results if r["pass"])
        print(f"Overall: {total_passed}/{len(all_results)} property checks passed")
        if args.json:
            print(json.dumps(all_results, indent=2, ensure_ascii=False))
        return

    if args.property and args.config:
        # Single property evaluation
        prop_data = json.loads(args.property)
        prop = Property.from_dict(prop_data)
        result = evaluate_property(args.config, prop)
        if args.json:
            print(json.dumps(result, indent=2, ensure_ascii=False))
        else:
            status = "✓ PASS" if result["pass"] else "✗ FAIL"
            print(f"{status}: {result['evidence']}")
        return

    if args.case_file and args.config:
        # Evaluate a case file
        with open(args.case_file) as f:
            cases_data = json.load(f)
        cases = cases_data if isinstance(cases_data, list) else cases_data.get("cases", [])
        if args.case_id:
            cases = [c for c in cases if c.get("id") == args.case_id]

        for case in cases:
            result = evaluate_case_by_property(args.config, case)
            case_id = case.get("id", "unknown")
            status = "✓ PASS" if result["pass"] else "✗ FAIL"
            print(f"{status}  {case_id}  ({result['passed']}/{result['total']} properties)")
            if args.verbose:
                for prop_result in result.get("properties", []):
                    p_status = "✓" if prop_result["pass"] else "✗"
                    print(f"    {p_status} {prop_result.get('property_name', '?')}: "
                          f"{prop_result['evidence'][:80]}")
        return

    parser.print_help()
    sys.exit(2)


if __name__ == "__main__":
    main()
