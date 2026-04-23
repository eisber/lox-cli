#!/usr/bin/env python3
"""
Loxone Eval Agent Runner

Runs an AI agent against eval cases, capturing:
- Correctness metrics (block F1, wiring accuracy, param accuracy)
- Token usage (input + output)
- CLI invocation count
- Retry count (validation failures → retry cycles)
- Validation pass/fail
- XML validity
- UX correctness (blocks on correct page with Px/Py)
- Wiring precision (extra unwanted wires)

Usage:
  python3 tests/eval/agent_runner.py --case s01-piano-protection
  python3 tests/eval/agent_runner.py --all --output eval-report.json
  python3 tests/eval/agent_runner.py --all --filter threshold --output report.json
  python3 tests/eval/agent_runner.py --report eval-report.json
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import xml.etree.ElementTree as ET
from collections import Counter, defaultdict
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
EVAL_DIR = SCRIPT_DIR.parent
FIXTURE = EVAL_DIR / "fixture.Loxone"
CASES_INDEX = EVAL_DIR / "cases-index.json"
SKILL_FILE = EVAL_DIR.parent.parent / "skills" / "loxone-automation" / "SKILL.md"
PATTERNS_FILE = EVAL_DIR.parent.parent / "skills" / "loxone-automation" / "references" / "PATTERNS.md"

# ── Agent System Prompt ──────────────────────────────────────

def build_agent_prompt(case, fixture_path):
    """Build the prompt sent to the agent."""
    skill_text = SKILL_FILE.read_text() if SKILL_FILE.exists() else ""
    patterns_text = PATTERNS_FILE.read_text() if PATTERNS_FILE.exists() else ""

    # Get fixture stats
    lox_cmd = _find_lox()
    stats = subprocess.run(
        lox_cmd + ["config", "stats", str(fixture_path)],
        capture_output=True, text=True, timeout=30
    ).stdout

    controls = subprocess.run(
        lox_cmd + ["config", "describe", str(fixture_path)],
        capture_output=True, text=True, timeout=30
    ).stdout

    prompt = f"""You are a Loxone home automation configuration agent. You edit .Loxone config files using the `lox` CLI tool.

## Skill Reference
{skill_text}

## Automation Patterns
{patterns_text}

## Current Config
File: {fixture_path}

Stats:
{stats}

Controls:
{controls}

## Task
Apply this automation request to the config file:

"{case['utterance']}"

## Instructions
1. Use `lox config` CLI commands to add blocks, set parameters, and wire connectors.
2. Always specify --page and --room when adding blocks.
3. Get connector UUIDs with `lox config control describe` before wiring.
4. Use bracket syntax for ambiguous names: "Name [Room]"
5. Run `lox config validate {fixture_path}` after each change.
6. The file is at: {fixture_path}

Execute the CLI commands now. Output each command and its result.
When done, output: DONE
"""
    return prompt


# ── CLI Tracking ─────────────────────────────────────────────

class CLITracker:
    """Tracks CLI invocations during agent execution."""

    def __init__(self):
        self.invocations = []
        self.validation_runs = 0
        self.validation_errors = 0
        self.retries = 0

    def run(self, args, cwd=None):
        """Run a lox CLI command and track it."""
        start = time.monotonic()
        result = subprocess.run(
            args, capture_output=True, text=True, cwd=cwd, timeout=60
        )
        elapsed = time.monotonic() - start

        cmd_str = " ".join(args)
        self.invocations.append({
            "command": cmd_str,
            "exit_code": result.returncode,
            "stdout_len": len(result.stdout),
            "stderr_len": len(result.stderr),
        })

        if "validate" in cmd_str:
            self.validation_runs += 1
            if result.returncode != 0 or "✗" in result.stdout:
                self.validation_errors += 1

        return result

    def summary(self):
        return {
            "total_invocations": len(self.invocations),
            "validation_runs": self.validation_runs,
            "validation_errors": self.validation_errors,
            "retries": self.retries,
            "commands": self.invocations,
        }


# ── Correctness Evaluation ──────────────────────────────────

INFRA_TYPES = {
    "Document", "Page", "Place", "Category", "Program",
    "User", "UserCaption", "LoxCaption", "VirtualInCaption",
    "LightscenesC", "LightsceneC", "Lightscene",
    "TreeDevice", "LoxAIRDevice", "NetworkDevice",
    "TreeAsensor", "TreeDsensor", "TreeAactuator", "TreeDactuator",
    "Co", "IoData", "Display", "HP", "Const", "Note", "SET",
}


def collect_controls(root):
    controls = {}
    conn_map = {}

    def walk(elem, page_title=None):
        etype = elem.get("Type", "")
        uuid = elem.get("U", "")
        title = elem.get("Title", "")

        if etype == "Page":
            page_title = title

        room_uuid = ""
        for child in elem:
            if child.tag == "IoData" or child.get("Type") == "IoData":
                room_uuid = child.get("Pr", "")

        if etype and etype not in INFRA_TYPES and uuid:
            px = elem.get("Px", "")
            py = elem.get("Py", "")
            controls[uuid] = {
                "type": etype, "title": title, "uuid": uuid,
                "page": page_title or "", "room_uuid": room_uuid,
                "px": px, "py": py, "elem": elem,
            }

        for child in elem:
            if child.tag == "Co" or child.get("Type") == "Co":
                cu = child.get("U", "")
                ck = child.get("K", "")
                if cu and uuid:
                    conn_map[cu] = (uuid, title, etype, ck)

        for child in elem:
            walk(child, page_title)

    walk(root)
    return controls, conn_map


def collect_rooms(root):
    rooms = {}
    for elem in root.iter("C"):
        if elem.get("Type") == "Place":
            rooms[elem.get("U", "")] = elem.get("Title", "")
    return rooms


def collect_wiring(root):
    wiring = defaultdict(list)
    def walk(elem):
        for child in elem:
            if child.tag == "Co" or child.get("Type") == "Co":
                cu = child.get("U", "")
                for inp in child:
                    if inp.tag == "In":
                        src = inp.get("Input", "")
                        if src and cu:
                            wiring[cu].append(src)
            walk(child)
    walk(root)
    return wiring


def find_param_value(elem, param_name):
    for child in elem:
        if (child.tag == "Co" or child.get("Type") == "Co") and child.get("K") == param_name:
            v = child.get("Def", "") or child.get("V", "")
            if v:
                return v
    for child in elem:
        if child.tag == "SET" or child.get("Type") == "SET":
            v = child.get(param_name, "")
            if v:
                return v
    v = elem.get(param_name, "")
    if v:
        return v
    return None


def evaluate_correctness(fixture_path, result_path, case):
    """Evaluate a result config against expected outcomes."""
    expected = case.get("expected", {})
    difficulty = case.get("difficulty", "medium")

    # Parse XMLs
    try:
        fix_root = ET.parse(str(fixture_path)).getroot()
    except Exception as e:
        return {"error": f"fixture parse error: {e}", "pass": False, "overall_score": 0}

    try:
        mod_root = ET.parse(str(result_path)).getroot()
        xml_valid = True
    except Exception as e:
        return {
            "error": f"result XML parse error: {e}",
            "pass": False, "overall_score": 0,
            "xml_valid": False,
        }

    rooms = collect_rooms(mod_root)
    rooms.update(collect_rooms(fix_root))

    fix_controls, fix_conns = collect_controls(fix_root)
    mod_controls, mod_conns = collect_controls(mod_root)

    new_uuids = set(mod_controls.keys()) - set(fix_controls.keys())
    new_blocks = {u: mod_controls[u] for u in new_uuids}

    mod_wiring = collect_wiring(mod_root)
    fix_wiring = collect_wiring(fix_root)

    # ── Block Evaluation ──
    exp_blocks = expected.get("new_blocks", [])
    block_tp = 0
    block_fn = 0
    block_details = []
    matched_uuids = set()

    for exp in exp_blocks:
        exp_type = exp.get("type", "")
        exp_title = exp.get("title_contains", "")
        exp_room = exp.get("room", "")
        exp_page = exp.get("page", "")

        found = False
        for uuid, info in new_blocks.items():
            if uuid in matched_uuids:
                continue
            if exp_type and info["type"] != exp_type:
                continue
            if exp_title and exp_title not in info["title"]:
                continue
            if exp_room:
                br = rooms.get(info["room_uuid"], "")
                if br != exp_room:
                    continue
            if exp_page and info["page"] != exp_page:
                continue
            found = True
            matched_uuids.add(uuid)
            break

        if found:
            block_tp += 1
            block_details.append({"check": f"block:{exp_type}", "pass": True})
        else:
            block_fn += 1
            block_details.append({"check": f"block:{exp_type}", "pass": False})

    block_fp = len(new_blocks) - len(matched_uuids)
    block_prec = block_tp / (block_tp + block_fp) if (block_tp + block_fp) > 0 else (1.0 if not exp_blocks else 0.0)
    block_rec = block_tp / (block_tp + block_fn) if (block_tp + block_fn) > 0 else (1.0 if not exp_blocks else 0.0)
    block_f1 = 2 * block_prec * block_rec / (block_prec + block_rec) if (block_prec + block_rec) > 0 else 0.0

    # ── Wiring Evaluation ──
    exp_wiring = expected.get("wiring", [])
    wire_correct = 0
    wire_total = len(exp_wiring)
    wire_details = []

    for exp in exp_wiring:
        from_title = exp.get("from_title", "")
        from_type = exp.get("from_type", "")
        from_conn = exp.get("from_connector", "")
        from_room = exp.get("from_room", "")
        to_title = exp.get("to_title", "")
        to_type = exp.get("to_type", "")
        to_conn = exp.get("to_connector", "")
        to_room = exp.get("to_room", "")

        src_conn_uuids = set()
        for cu, (buuid, btitle, btype, ck) in mod_conns.items():
            if from_title and from_title not in btitle:
                continue
            if from_type and btype != from_type:
                continue
            if from_conn and ck != from_conn:
                continue
            if from_room:
                ctrl = mod_controls.get(buuid, {})
                br = rooms.get(ctrl.get("room_uuid", ""), "")
                if br != from_room:
                    continue
            src_conn_uuids.add(cu)

        found = False
        for cu, (buuid, btitle, btype, ck) in mod_conns.items():
            if to_title and to_title not in btitle:
                continue
            if to_type and btype != to_type:
                continue
            if to_conn and ck != to_conn:
                continue
            if to_room:
                ctrl = mod_controls.get(buuid, {})
                br = rooms.get(ctrl.get("room_uuid", ""), "")
                if br != to_room:
                    continue

            wired_from = mod_wiring.get(cu, [])
            if not src_conn_uuids:
                if wired_from:
                    found = True
                    break
            else:
                if set(wired_from) & src_conn_uuids:
                    found = True
                    break

        label = " → ".join(filter(None, [
            from_title or from_type, to_title or to_type,
        ]))
        if found:
            wire_correct += 1
            wire_details.append({"check": f"wire:{label}", "pass": True})
        else:
            wire_details.append({"check": f"wire:{label}", "pass": False})

    wiring_accuracy = wire_correct / wire_total if wire_total > 0 else 1.0

    # ── Wiring Precision (extra unwanted wires) ──
    # Count wires in result that don't exist in fixture
    new_wires = 0
    for cu, sources in mod_wiring.items():
        fix_sources = fix_wiring.get(cu, [])
        for src in sources:
            if src not in fix_sources:
                new_wires += 1

    expected_new_wires = wire_total
    extra_wires = max(0, new_wires - expected_new_wires)
    wiring_precision = (new_wires - extra_wires) / new_wires if new_wires > 0 else 1.0

    # ── Param Evaluation ──
    exp_params = expected.get("params", [])
    param_correct = 0
    param_total = len(exp_params)
    param_details = []

    for exp in exp_params:
        exp_type = exp.get("block_type", "")
        exp_param = exp.get("param", "")
        exp_value = str(exp.get("value", ""))

        search = {**new_blocks}
        for uuid, info in mod_controls.items():
            if uuid not in fix_controls:
                continue
            if exp_type and info["type"] != exp_type:
                continue
            search[uuid] = info

        found = False
        for uuid, info in search.items():
            if exp_type and info["type"] != exp_type:
                continue
            val = find_param_value(info["elem"], exp_param)
            if val is not None:
                try:
                    if float(val) == float(exp_value):
                        found = True
                        break
                except (ValueError, TypeError):
                    pass
                if str(val).strip() == exp_value:
                    found = True
                    break

        label = f"{exp_param}={exp_value}" + (f" on {exp_type}" if exp_type else "")
        if found:
            param_correct += 1
            param_details.append({"check": f"param:{label}", "pass": True})
        else:
            param_details.append({"check": f"param:{label}", "pass": False})

    param_accuracy = param_correct / param_total if param_total > 0 else 1.0

    # ── UX Correctness ──
    ux_issues = []
    for uuid in new_uuids:
        info = mod_controls.get(uuid, {})
        if not info.get("page"):
            ux_issues.append(f"{info.get('type','?')} '{info.get('title','?')}' has no page")
        if not info.get("px") and not info.get("py"):
            ux_issues.append(f"{info.get('type','?')} '{info.get('title','?')}' has no position (Px/Py)")

    ux_score = 1.0 - (len(ux_issues) / max(len(new_uuids), 1)) if new_uuids else 1.0

    # ── Validation ──
    lox_cmd = _find_lox()
    val_result = subprocess.run(
        lox_cmd + ["config", "validate", str(result_path)],
        capture_output=True, text=True, timeout=30
    )
    val_errors = val_result.stdout.count("✗")
    validation_pass = val_errors == 0

    # ── Overall Score ──
    overall = 0.3 * block_f1 + 0.4 * wiring_accuracy + 0.3 * param_accuracy
    difficulty_weights = {"easy": 1.0, "medium": 1.5, "hard": 2.0, "expert": 3.0}
    weighted_score = overall * difficulty_weights.get(difficulty, 1.0)
    passed = overall >= 0.8

    return {
        "pass": passed,
        "overall_score": round(overall, 3),
        "weighted_score": round(weighted_score, 3),
        "xml_valid": xml_valid,
        "validation_pass": validation_pass,
        "validation_errors": val_errors,
        "metrics": {
            "blocks": {
                "precision": round(block_prec, 3),
                "recall": round(block_rec, 3),
                "f1": round(block_f1, 3),
                "true_positives": block_tp,
                "false_positives": block_fp,
                "false_negatives": block_fn,
            },
            "wiring": {
                "accuracy": round(wiring_accuracy, 3),
                "precision": round(wiring_precision, 3),
                "correct": wire_correct,
                "total": wire_total,
                "extra": extra_wires,
            },
            "params": {
                "accuracy": round(param_accuracy, 3),
                "correct": param_correct,
                "total": param_total,
            },
            "ux": {
                "score": round(ux_score, 3),
                "issues": ux_issues,
            },
        },
        "details": block_details + wire_details + param_details,
        "new_blocks_found": [
            {"type": info["type"], "title": info["title"],
             "room": rooms.get(info["room_uuid"], "?"), "page": info["page"]}
            for info in new_blocks.values()
        ],
    }


# ── Agent Execution ──────────────────────────────────────────

def _find_lox():
    """Find the lox command."""
    if shutil.which("lox"):
        return ["lox"]
    return ["cargo", "run", "--quiet", "--"]


def execute_agent_for_case(case, work_dir):
    """
    Execute lox CLI commands as the 'agent' for a given case.

    This is a deterministic rule-based agent that translates the expected
    blocks/wiring/params into CLI commands. It serves as a baseline agent
    to validate the harness itself and as a template for LLM agent integration.

    Returns: (result_path, cli_tracker, token_estimate)
    """
    lox_cmd = _find_lox()
    tracker = CLITracker()
    expected = case.get("expected", {})
    config_path = work_dir / f"{case['id']}.Loxone"

    # Copy fixture
    shutil.copy2(str(FIXTURE), str(config_path))

    token_input = 0
    token_output = 0

    # Step 1: Understand config
    r = tracker.run(lox_cmd + ["config", "stats", str(config_path)])
    token_output += len(r.stdout)

    r = tracker.run(lox_cmd + ["config", "describe", str(config_path)])
    token_output += len(r.stdout)

    # Step 2: Create blocks
    created_blocks = {}  # type_index → uuid
    type_counter = Counter()

    for block in expected.get("new_blocks", []):
        btype = block.get("type", "")
        if not btype:
            continue

        title = block.get("title_contains", f"{btype}")
        room = block.get("room", "")
        page = block.get("page", "")

        # Pick defaults if not specified
        if not room:
            room = "Wohnzimmer"
        if not page:
            page = room if room else "Wohnzimmer"

        # Make title unique
        type_counter[btype] += 1
        if type_counter[btype] > 1:
            title = f"{title} {type_counter[btype]}"

        cmd = lox_cmd + [
            "config", "add",
            "--type", btype,
            "--title", title,
            "--room", room,
            "--page", page,
            str(config_path),
        ]

        r = tracker.run(cmd)
        token_output += len(r.stdout)
        token_input += len(" ".join(cmd))

        # Extract UUID from output
        uuid_match = re.search(r'UUID: ([0-9a-f-]+)', r.stdout)
        if uuid_match:
            key = f"{btype}:{type_counter[btype]}"
            created_blocks[key] = uuid_match.group(1)

    # Step 3: Set parameters
    for param in expected.get("params", []):
        btype = param.get("block_type", "")
        pname = param.get("param", "")
        pvalue = param.get("value", "")

        if not pname or not pvalue:
            continue

        # Find the selector for the block
        selector = None
        if btype:
            # Find created block of this type
            for key, uuid in created_blocks.items():
                if key.startswith(f"{btype}:"):
                    selector = f"uuid:{uuid}"
                    break

        if not selector:
            # Try to find by type in the config
            r = tracker.run(lox_cmd + ["config", "controls", str(config_path), "-t", btype])
            token_output += len(r.stdout)
            # Parse first matching line
            for line in r.stdout.splitlines():
                uuid_m = re.search(r'([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{16})', line)
                if uuid_m:
                    selector = f"uuid:{uuid_m.group(1)}"
                    break

        if selector:
            cmd = lox_cmd + ["config", "set-param", str(config_path), selector, pname, pvalue]
            r = tracker.run(cmd)
            token_output += len(r.stdout)
            token_input += len(" ".join(cmd))

    # Step 4: Wire connectors
    for wire in expected.get("wiring", []):
        from_title = wire.get("from_title", "")
        from_type = wire.get("from_type", "")
        from_conn = wire.get("from_connector", "")
        from_room = wire.get("from_room", "")
        to_title = wire.get("to_title", "")
        to_type = wire.get("to_type", "")
        to_conn = wire.get("to_connector", "")
        to_room = wire.get("to_room", "")

        # Find source UUID
        source_uuid = None
        if from_title:
            # Find by title
            src_selector = from_title
            if from_room:
                src_selector = f"{from_title} [{from_room}]"
        elif from_type:
            # Find created block of this type
            for key, uuid in created_blocks.items():
                if key.startswith(f"{from_type}:"):
                    src_selector = f"uuid:{uuid}"
                    break
            else:
                src_selector = None
        else:
            continue

        if not src_selector:
            continue

        # Get source connector UUID via describe
        r = tracker.run(lox_cmd + ["config", "control", "describe", str(config_path), src_selector])
        token_output += len(r.stdout)

        src_conn_key = from_conn if from_conn else "Q"
        # Also try AQ for analog outputs
        if not from_conn:
            if "AQ →" in r.stdout or "AQ ->" in r.stdout:
                src_conn_key = "AQ"

        source_uuid = None
        for line in r.stdout.splitlines():
            if f"{src_conn_key} →" in line or f"{src_conn_key} ->" in line:
                uuid_m = re.search(r'([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{16})', line)
                if uuid_m:
                    source_uuid = uuid_m.group(1)
                    break

        if not source_uuid:
            continue

        # Build target selector
        if to_title:
            target_sel = to_title
            if to_room:
                target_sel = f"{to_title} [{to_room}]"
        elif to_type:
            for key, uuid in created_blocks.items():
                if key.startswith(f"{to_type}:"):
                    target_sel = f"uuid:{uuid}"
                    break
            else:
                continue
        else:
            continue

        target_connector = to_conn if to_conn else "I1"
        wire_target = f"{target_sel}.{target_connector}"

        cmd = lox_cmd + ["config", "wire-connector", str(config_path), wire_target, source_uuid]
        r = tracker.run(cmd)
        token_output += len(r.stdout)
        token_input += len(" ".join(cmd))

        if r.returncode != 0:
            tracker.retries += 1

    # Step 5: Validate
    r = tracker.run(lox_cmd + ["config", "validate", str(config_path)])
    token_output += len(r.stdout)

    # Estimate tokens (chars / 4 ≈ tokens)
    token_estimate = {
        "input_chars": token_input,
        "output_chars": token_output,
        "input_tokens_est": token_input // 4,
        "output_tokens_est": token_output // 4,
    }

    return config_path, tracker, token_estimate


# ── Report Generation ────────────────────────────────────────

def generate_report(results):
    """Generate aggregate report from individual case results."""
    total = len(results)
    if total == 0:
        return {"summary": {"total": 0}, "cases": []}

    # Primary metric: sim_pass
    sim_results = [r for r in results if r.get("sim_pass") is not None]
    sim_passed = sum(1 for r in sim_results if r.get("sim_pass"))
    sim_total = len(sim_results)

    # Secondary metric: validation_pass
    valid_xml = sum(1 for r in results if r.get("xml_valid", True))
    valid_config = sum(1 for r in results if r.get("validation_pass", False))

    # Overall pass: sim_pass if sim specs exist, else validation_pass
    passed = sum(
        1 for r in results
        if (r.get("sim_pass") if r.get("sim_pass") is not None else r.get("validation_pass", False))
    )

    # Averages
    avg = lambda key: sum(r.get(key, 0) for r in results) / total
    avg_metric = lambda *keys: sum(
        r.get("metrics", {}).get(keys[0], {}).get(keys[1], 0) for r in results
    ) / total

    # Per-difficulty
    diff_groups = defaultdict(list)
    for r in results:
        diff_groups[r.get("difficulty", "?")].append(r)

    by_difficulty = {}
    for diff in ["easy", "medium", "hard", "expert"]:
        group = diff_groups.get(diff, [])
        if group:
            g_sim = [r for r in group if r.get("sim_pass") is not None]
            g_sim_passed = sum(1 for r in g_sim if r.get("sim_pass"))
            g_passed = sum(
                1 for r in group
                if (r.get("sim_pass") if r.get("sim_pass") is not None else r.get("validation_pass", False))
            )
            by_difficulty[diff] = {
                "count": len(group),
                "passed": g_passed,
                "sim_passed": g_sim_passed,
                "sim_total": len(g_sim),
                "avg_score": round(sum(r["overall_score"] for r in group) / len(group), 3),
                "avg_block_f1": round(sum(r["metrics"]["blocks"]["f1"] for r in group) / len(group), 3),
                "avg_params": round(sum(r["metrics"]["params"]["accuracy"] for r in group) / len(group), 3),
                "avg_cli_invocations": round(sum(r.get("cli_invocations", 0) for r in group) / len(group), 1),
                "avg_tokens": round(sum(r.get("tokens", {}).get("output_tokens_est", 0) for r in group) / len(group), 0),
            }

    # Per-pattern
    pat_groups = defaultdict(list)
    for r in results:
        for p in r.get("patterns", []):
            pat_groups[p].append(r)

    by_pattern = {}
    for pat, group in sorted(pat_groups.items(), key=lambda x: -len(x[1])):
        g_sim = [r for r in group if r.get("sim_pass") is not None]
        g_sim_passed = sum(1 for r in g_sim if r.get("sim_pass"))
        by_pattern[pat] = {
            "count": len(group),
            "passed": sum(
                1 for r in group
                if (r.get("sim_pass") if r.get("sim_pass") is not None else r.get("validation_pass", False))
            ),
            "sim_passed": g_sim_passed,
            "sim_total": len(g_sim),
            "avg_score": round(sum(r["overall_score"] for r in group) / len(group), 3),
        }

    # Totals
    total_cli = sum(r.get("cli_invocations", 0) for r in results)
    total_retries = sum(r.get("retries", 0) for r in results)
    total_tokens_in = sum(r.get("tokens", {}).get("input_tokens_est", 0) for r in results)
    total_tokens_out = sum(r.get("tokens", {}).get("output_tokens_est", 0) for r in results)

    return {
        "summary": {
            "total": total,
            "passed": passed,
            "failed": total - passed,
            "pass_rate": round(passed / total, 3),
            "sim_passed": sim_passed,
            "sim_total": sim_total,
            "sim_rate": round(sim_passed / sim_total, 3) if sim_total else None,
            "xml_valid": valid_xml,
            "validation_pass": valid_config,
            "avg_overall_score": round(avg("overall_score"), 3),
            "avg_weighted_score": round(avg("weighted_score"), 3),
            "avg_block_f1": round(avg_metric("blocks", "f1"), 3),
            "avg_wiring_accuracy": round(avg_metric("wiring", "accuracy"), 3),
            "avg_wiring_precision": round(avg_metric("wiring", "precision"), 3),
            "avg_param_accuracy": round(avg_metric("params", "accuracy"), 3),
            "avg_ux_score": round(avg_metric("ux", "score"), 3),
            "total_cli_invocations": total_cli,
            "total_retries": total_retries,
            "total_tokens_input": total_tokens_in,
            "total_tokens_output": total_tokens_out,
            "avg_cli_per_case": round(total_cli / total, 1),
            "avg_tokens_per_case": round((total_tokens_in + total_tokens_out) / total, 0),
        },
        "by_difficulty": by_difficulty,
        "by_pattern": by_pattern,
        "cases": [
            {k: v for k, v in r.items() if k != "details" and k != "new_blocks_found"}
            for r in results
        ],
    }


def print_report(report):
    """Pretty-print a report."""
    s = report["summary"]

    print(f"\n{'═' * 70}")
    print(f"  LOXONE EVAL REPORT")
    print(f"{'═' * 70}\n")

    print(f"  Cases:          {s['total']}")
    if s['total'] == 0:
        print("  No cases were evaluated.")
        return

    # Primary: sim pass rate
    if s.get("sim_total"):
        print(f"  Sim Pass Rate:  {s['sim_passed']}/{s['sim_total']} ({s['sim_rate']:.0%})")
    print(f"  Pass Rate:      {s['passed']}/{s['total']} ({s['pass_rate']:.0%})")
    print()
    print(f"  ── Primary: Simulation ──")
    if s.get("sim_total"):
        print(f"  Sim Passed:        {s['sim_passed']}/{s['sim_total']}")
    else:
        print(f"  Sim Passed:        (no sim specs)")
    print()
    print(f"  ── Secondary: Validation ──")
    print(f"  XML Valid:         {s['xml_valid']}/{s['total']}")
    print(f"  Validation Pass:   {s['validation_pass']}/{s['total']}")
    print()
    print(f"  ── Info: Structural Scores ──")
    print(f"  Block F1:          {s['avg_block_f1']:.1%}")
    print(f"  Param Accuracy:    {s['avg_param_accuracy']:.1%}")
    print(f"  UX Score:          {s['avg_ux_score']:.1%}")
    print()
    print(f"  ── Efficiency ──")
    print(f"  CLI Invocations:   {s['total_cli_invocations']} total, {s['avg_cli_per_case']:.1f}/case")
    print(f"  Retries:           {s['total_retries']}")
    print(f"  Tokens (est):      {s['total_tokens_input']+s['total_tokens_output']} total, {s['avg_tokens_per_case']:.0f}/case")
    print(f"    Input:           {s['total_tokens_input']}")
    print(f"    Output:          {s['total_tokens_output']}")

    print(f"\n  ── By Difficulty ──")
    for diff in ["easy", "medium", "hard", "expert"]:
        d = report.get("by_difficulty", {}).get(diff, {})
        if d:
            sim_info = f"sim={d.get('sim_passed', 0)}/{d.get('sim_total', 0)}" if d.get('sim_total') else "no-sim"
            bar = "█" * int(d['avg_score'] * 20)
            print(f"    {diff:8s}: {d['passed']:3d}/{d['count']:3d} passed  "
                  f"{sim_info}  F1={d['avg_block_f1']:.0%}  "
                  f"param={d['avg_params']:.0%}  "
                  f"cli={d['avg_cli_invocations']:.0f}  tok={d['avg_tokens']:.0f}  {bar}")

    print(f"\n  ── By Pattern ──")
    for pat, p in sorted(report.get("by_pattern", {}).items(), key=lambda x: -x[1]["count"]):
        sim_info = f"sim={p.get('sim_passed', 0)}/{p.get('sim_total', 0)}" if p.get('sim_total') else "no-sim"
        bar = "█" * int(p["avg_score"] * 20)
        print(f"    {str(pat):25s}: {p['passed']:3d}/{p['count']:3d} passed  "
              f"{sim_info}  avg={p['avg_score']:.0%}  {bar}")

    # Worst 10
    cases = report.get("cases", [])
    worst = sorted(cases, key=lambda c: (
        0 if c.get("sim_pass") else 1,
        0 if c.get("validation_pass") else 1,
        c.get("overall_score", 0),
    ))[:10]
    if worst:
        print(f"\n  ── Worst 10 Cases ──")
        for c in worst:
            sim = c.get("sim_pass")
            if sim is True:
                status = "✓"
            elif sim is False:
                status = "✗"
            else:
                status = "✓" if c.get("validation_pass") else "✗"
            print(f"    {status} {c['case_id']:40s} {c.get('overall_score',0):.0%}  "
                  f"({c.get('difficulty','?')})  {c.get('utterance','')[:50]}")


# ── Main ─────────────────────────────────────────────────────

def load_cases(filter_str=None, section=None, max_cases=None):
    """Load eval cases with optional filtering."""
    with open(CASES_INDEX) as f:
        index = json.load(f)
    
    cases = []
    for category, meta in index['categories'].items():
        if section and category != section:
            continue
        case_file = EVAL_DIR / meta['file']
        with open(case_file) as f:
            category_cases = json.load(f)
            for c in category_cases:
                if filter_str:
                    pattern = c.get('pattern', '')
                    if isinstance(pattern, list):
                        pattern = ' '.join(pattern)
                    searchable = f"{c['id']} {pattern} {c['utterance']} {c.get('difficulty', '')}".lower()
                    if filter_str.lower() not in searchable:
                        continue
                c['_section'] = category
                cases.append(c)
    
    if max_cases:
        cases = cases[:max_cases]
    
    return cases


def main():
    parser = argparse.ArgumentParser(description="Loxone Eval Agent Runner")
    parser.add_argument("--case", help="Run a single case by ID")
    parser.add_argument("--all", action="store_true", help="Run all cases")
    parser.add_argument("--filter", help="Filter cases by pattern/difficulty/keyword")
    parser.add_argument("--section", choices=["synthetic", "reference"], help="Only run synthetic or reference cases")
    parser.add_argument("--max", type=int, help="Max cases to run")
    parser.add_argument("--output", default="eval-report.json", help="Output report file")
    parser.add_argument("--report", help="Pretty-print an existing report")
    parser.add_argument("--work-dir", help="Working directory for result configs")
    parser.add_argument("--keep", action="store_true", help="Keep working directory after run")
    args = parser.parse_args()

    if args.report:
        with open(args.report) as f:
            report = json.load(f)
        print_report(report)
        return

    if not args.case and not args.all:
        parser.print_help()
        sys.exit(2)

    # Load cases
    if args.case:
        cases = load_cases()
        cases = [c for c in cases if c["id"] == args.case]
        if not cases:
            print(f"Case '{args.case}' not found", file=sys.stderr)
            sys.exit(2)
    else:
        cases = load_cases(args.filter, args.section, args.max)

    print(f"Running {len(cases)} eval cases...")
    print(f"Fixture: {FIXTURE}")
    print()

    # Work directory
    if args.work_dir:
        work_dir = Path(args.work_dir)
        work_dir.mkdir(parents=True, exist_ok=True)
    else:
        work_dir = Path(tempfile.mkdtemp(prefix="lox-eval-"))

    results = []
    passed = 0
    failed = 0

    for i, case in enumerate(cases):
        case_id = case["id"]
        sys.stdout.write(f"  [{i+1}/{len(cases)}] {case_id:42s} ")
        sys.stdout.flush()

        try:
            result_path, tracker, tokens = execute_agent_for_case(case, work_dir)

            # Evaluate correctness
            eval_result = evaluate_correctness(FIXTURE, result_path, case)

            # Merge all data
            eval_result["case_id"] = case_id
            eval_result["difficulty"] = case.get("difficulty", "medium")
            eval_result["patterns"] = case.get("pattern", [])
            eval_result["utterance"] = case["utterance"]
            eval_result["cli_invocations"] = tracker.summary()["total_invocations"]
            eval_result["retries"] = tracker.summary()["retries"]
            eval_result["validation_runs"] = tracker.summary()["validation_runs"]
            eval_result["tokens"] = tokens

            results.append(eval_result)

            if eval_result["pass"]:
                passed += 1
                print(f"\033[32m✓ PASS\033[0m  {eval_result['overall_score']:.0%}")
            else:
                failed += 1
                m = eval_result["metrics"]
                print(f"\033[31m✗ FAIL\033[0m  {eval_result['overall_score']:.0%}  "
                      f"B={m['blocks']['f1']:.0%} W={m['wiring']['accuracy']:.0%} P={m['params']['accuracy']:.0%}")

        except Exception as e:
            failed += 1
            results.append({
                "case_id": case_id,
                "pass": False,
                "overall_score": 0,
                "error": str(e),
                "difficulty": case.get("difficulty", "medium"),
                "patterns": case.get("pattern", []),
                "utterance": case["utterance"],
                "metrics": {
                    "blocks": {"precision": 0, "recall": 0, "f1": 0, "true_positives": 0, "false_positives": 0, "false_negatives": 0},
                    "wiring": {"accuracy": 0, "precision": 0, "correct": 0, "total": 0, "extra": 0},
                    "params": {"accuracy": 0, "correct": 0, "total": 0},
                    "ux": {"score": 0, "issues": []},
                },
                "cli_invocations": 0,
                "retries": 0,
                "tokens": {"input_tokens_est": 0, "output_tokens_est": 0},
            })
            print(f"\033[33m⚠ ERROR\033[0m  {e}")

    # Generate report
    report = generate_report(results)

    with open(args.output, "w") as f:
        json.dump(report, f, indent=2, ensure_ascii=False)

    print()
    print_report(report)
    print(f"\nReport saved to: {args.output}")

    if not args.keep and not args.work_dir:
        shutil.rmtree(work_dir, ignore_errors=True)

    # Exit code
    sys.exit(0 if failed == 0 else 1)


if __name__ == "__main__":
    main()
