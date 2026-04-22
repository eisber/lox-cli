#!/usr/bin/env bash
# Loxone Config-as-Code Eval Harness v2
#
# Usage:
#   ./tests/eval/run-eval.sh <case-id> <modified-config.Loxone>    # eval single case
#   ./tests/eval/run-eval.sh --batch <results-dir>                  # eval all results in dir
#   ./tests/eval/run-eval.sh --list [--filter pattern]              # list cases
#   ./tests/eval/run-eval.sh --validate-fixture                     # check fixture
#   ./tests/eval/run-eval.sh --report <results.json>                # pretty-print report
#
# Results directory structure (for --batch):
#   results/
#     s01-piano-protection.Loxone    # each file named after case ID
#     s02-night-hallway-dim.Loxone
#     k001-threshold.Loxone
#     ...
#
# Exit codes: 0 = PASS, 1 = FAIL, 2 = usage error

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EVAL_DIR="$(dirname "$SCRIPT_DIR")"
FIXTURE="$EVAL_DIR/fixture.Loxone"
CASES_INDEX="$EVAL_DIR/cases-index.json"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
RESET='\033[0m'

usage() {
  echo "Usage: $0 <case-id> <modified-config.Loxone>"
  echo "       $0 --batch <results-dir>     Evaluate all results"
  echo "       $0 --list [--filter PATTERN]  List available cases"
  echo "       $0 --validate-fixture         Validate the fixture"
  echo "       $0 --report <results.json>    Pretty-print report"
  echo "       $0 --stats                    Show case statistics"
  exit 2
}

LOX_CMD="lox"
if ! command -v lox &>/dev/null; then
  LOX_CMD="cargo run --quiet --"
fi

list_cases() {
  local filter="${1:-}"
  python3 << PYEOF
import json
import sys
from pathlib import Path

eval_dir = Path('$EVAL_DIR')
index_path = eval_dir / 'cases-index.json'

with open(index_path) as f:
    index = json.load(f)

# Load all case files
all_cases = []
for category, meta in index['categories'].items():
    case_file = eval_dir / meta['file']
    with open(case_file) as f:
        cases = json.load(f)
        all_cases.extend(cases)

filt = "$filter".lower()

print(f"{'ID':40s} {'Diff':8s} {'Pattern':30s} Utterance")
print("─" * 140)

for c in all_cases:
    cid = c['id']
    diff = c.get('difficulty', 'medium')
    pattern = c.get('pattern', '')
    if isinstance(pattern, list):
        pattern = ', '.join(pattern)
    utt = c['utterance'][:60]
    if filt and filt not in cid.lower() and filt not in pattern.lower() and filt not in utt.lower():
        continue
    print(f"{cid:40s} {diff:8s} {pattern:30s} {utt}")

total = len(all_cases)
print(f"\n{total} cases total")
PYEOF
}

show_stats() {
  python3 << PYEOF
import json
from collections import Counter
from pathlib import Path

eval_dir = Path('$EVAL_DIR')
index_path = eval_dir / 'cases-index.json'

with open(index_path) as f:
    index = json.load(f)

# Load all case files and organize by category
categories = {}
all_cases = []
for category, meta in index['categories'].items():
    case_file = eval_dir / meta['file']
    with open(case_file) as f:
        cases = json.load(f)
        categories[category] = cases
        all_cases.extend(cases)

print("═══ Eval Set Statistics ═══\n")
print("  Cases by Category:")
for category, cases in sorted(categories.items()):
    print(f"    {category:20s}: {len(cases):3d} cases")
print(f"  {'Total':20s}: {len(all_cases):3d} cases\n")

# Difficulty
diff = Counter(c.get('difficulty', 'medium') for c in all_cases)
weights = {"easy": 1.0, "medium": 1.5, "hard": 2.0, "expert": 3.0}
total_weight = sum(diff[d] * weights.get(d, 1) for d in diff)
print("  Difficulty Distribution:")
for d in ["easy", "medium", "hard", "expert"]:
    n = diff.get(d, 0)
    bar = "█" * (n // 2)
    print(f"    {d:8s}: {n:3d}  {bar}")
print(f"    Weighted total: {total_weight:.0f} points\n")

# Patterns
pats = Counter()
for c in all_cases:
    pattern = c.get('pattern', '')
    if isinstance(pattern, list):
        for p in pattern:
            pats[p] += 1
    elif pattern:
        pats[pattern] += 1
print("  Pattern Coverage:")
for p, n in sorted(pats.items(), key=lambda x: -x[1])[:15]:
    bar = "█" * (n // 3)
    print(f"    {str(p):25s}: {n:3d}  {bar}")

# Blocks/wiring/params
total_blocks = sum(len(c.get('expected', {}).get('new_blocks', [])) for c in all_cases)
total_wires = sum(len(c.get('expected', {}).get('wiring', [])) for c in all_cases)
total_params = sum(len(c.get('expected', {}).get('params', [])) for c in all_cases)
print(f"\n  Expected Totals:")
print(f"    Blocks: {total_blocks}")
print(f"    Wires:  {total_wires}")
print(f"    Params: {total_params}")
PYEOF
}

validate_fixture() {
  echo -e "${CYAN}Validating fixture...${RESET}"
  $LOX_CMD config validate "$FIXTURE" 2>&1 || true
  $LOX_CMD config stats "$FIXTURE" 2>&1 | head -20
}

# Evaluate a single case: returns JSON metrics to stdout
eval_single() {
  local case_id="$1"
  local config="$2"

  python3 << 'PYEOF'
import json, sys, os
import xml.etree.ElementTree as ET
from collections import defaultdict

def load_case(eval_dir, case_id):
    """Load a case from the index and case files."""
    from pathlib import Path
    
    eval_path = Path(eval_dir)
    index_path = eval_path / 'cases-index.json'
    
    with open(index_path) as f:
        index = json.load(f)
    
    # Search in all category files
    for category, meta in index['categories'].items():
        case_file = eval_path / meta['file']
        with open(case_file) as f:
            cases = json.load(f)
            for c in cases:
                if c['id'] == case_id:
                    return c
    return None

import re
def strip_bracket(title):
    """Strip bracket room qualifier: 'Raumregler [Wohnzimmer]' → 'Raumregler'"""
    return re.sub(r'\s*\[.*\]', '', title) if title else title

def title_matches(selector, block_title):
    """Check if a selector matches a block title, handling bracket syntax."""
    if not selector:
        return True
    base = strip_bracket(selector)
    return base in block_title

# Equivalent block types (agent may use a valid alternative)
TYPE_EQUIVALENTS = {
    'Memory': {'Memory', 'AMemory'},
    'AMemory': {'Memory', 'AMemory'},
    'GreaterEqual': {'GreaterEqual', 'AnalogThresholdTrigger'},
    'AnalogThresholdTrigger': {'GreaterEqual', 'AnalogThresholdTrigger'},
}

def type_matches(expected_type, actual_type):
    """Check if block type matches, allowing equivalents."""
    if not expected_type:
        return True
    if expected_type == actual_type:
        return True
    return actual_type in TYPE_EQUIVALENTS.get(expected_type, set())

def collect_controls(root):
    """Recursively collect all control blocks with metadata."""
    INFRA_TYPES = {
        "Document", "Page", "Place", "Category", "Program",
        "User", "UserCaption", "LoxCaption", "VirtualInCaption",
        "LightscenesC", "LightsceneC", "Lightscene",
        "TreeDevice", "LoxAIRDevice", "NetworkDevice",
        "TreeAsensor", "TreeDsensor", "TreeAactuator", "TreeDactuator",
        "Co", "IoData", "Display", "HP", "Const", "Note", "SET",
    }

    controls = {}
    conn_map = defaultdict(list)  # connector uuid → [(block_uuid, block_title, block_type, conn_key)]

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
            controls[uuid] = {
                "type": etype, "title": title, "uuid": uuid,
                "page": page_title or "", "room_uuid": room_uuid,
                "elem": elem,
            }

        for child in elem:
            if child.tag == "Co" or child.get("Type") == "Co":
                cu = child.get("U", "")
                ck = child.get("K", "")
                if cu and uuid:
                    conn_map[cu].append((uuid, title, etype, ck))

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

    # Also detect UUID-sharing wiring: when two connectors from
    # different blocks share the same UUID, they are wired together.
    # Build uuid → [(block_uuid, conn_key)] map
    uuid_conns = defaultdict(list)
    def walk_conns(elem, block_uuid=None, block_type=None):
        etype = elem.get("Type", "")
        euuid = elem.get("U", "")
        if etype and euuid and etype not in ("Co", "IoData", "SET"):
            block_uuid = euuid
            block_type = etype
        for child in elem:
            if child.tag == "Co" or child.get("Type") == "Co":
                cu = child.get("U", "")
                if cu and block_uuid:
                    uuid_conns[cu].append(block_uuid)
            walk_conns(child, block_uuid, block_type)
    walk_conns(root)

    # If a UUID appears in connectors of 2+ different blocks, they're wired
    for cu, block_uuids in uuid_conns.items():
        unique_blocks = list(set(block_uuids))
        if len(unique_blocks) >= 2:
            # Each block's connector with this UUID is wired to all others
            for bu in unique_blocks:
                for other_bu in unique_blocks:
                    if bu != other_bu:
                        if cu not in wiring.get(cu, []):
                            wiring[cu].append(cu)

    return wiring

def find_param_value(elem, param_name):
    for child in elem:
        if (child.tag == "Co" or child.get("Type") == "Co") and child.get("K") == param_name:
            v = child.get("Def", "") or child.get("V", "")
            if v: return v
    for child in elem:
        if child.tag == "SET" or child.get("Type") == "SET":
            v = child.get(param_name, "")
            if v: return v
    v = elem.get(param_name, "")
    if v: return v
    return None

def main():
    fixture_file = os.environ.get("EVAL_FIXTURE", "")
    config_file = os.environ.get("EVAL_CONFIG", "")
    eval_dir = os.environ.get("EVAL_DIR", "")
    case_id = os.environ.get("EVAL_CASE_ID", "")

    case = load_case(eval_dir, case_id)
    if not case:
        print(json.dumps({"error": f"case '{case_id}' not found", "pass": False}))
        sys.exit(1)

    expected = case.get("expected", {})
    difficulty = case.get("difficulty", "medium")

    # Parse XMLs
    try:
        fix_tree = ET.parse(fixture_file)
        mod_tree = ET.parse(config_file)
    except Exception as e:
        print(json.dumps({"error": f"XML parse error: {e}", "pass": False}))
        sys.exit(1)

    fix_root = fix_tree.getroot()
    mod_root = mod_tree.getroot()

    rooms = collect_rooms(mod_root)
    rooms.update(collect_rooms(fix_root))

    fix_controls, fix_conns = collect_controls(fix_root)
    mod_controls, mod_conns = collect_controls(mod_root)

    new_uuids = set(mod_controls.keys()) - set(fix_controls.keys())
    new_blocks = {u: mod_controls[u] for u in new_uuids}

    mod_wiring = collect_wiring(mod_root)

    # ── Block Evaluation ──
    exp_blocks = expected.get("new_blocks", [])
    block_tp = 0  # true positives
    block_fn = 0  # false negatives (expected but not found)
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
            if exp_type and not type_matches(exp_type, info["type"]):
                continue
            if exp_title and not title_matches(exp_title, info["title"]):
                continue
            if exp_room:
                block_room = rooms.get(info["room_uuid"], "")
                if block_room != exp_room:
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
            block_details.append({"check": f"block:{exp_type}", "pass": False, "reason": "not found"})

    # Extra blocks (false positives)
    block_fp = len(new_blocks) - len(matched_uuids)

    block_precision = block_tp / (block_tp + block_fp) if (block_tp + block_fp) > 0 else (1.0 if len(exp_blocks) == 0 else 0.0)
    block_recall = block_tp / (block_tp + block_fn) if (block_tp + block_fn) > 0 else (1.0 if len(exp_blocks) == 0 else 0.0)
    block_f1 = 2 * block_precision * block_recall / (block_precision + block_recall) if (block_precision + block_recall) > 0 else 0.0

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

        # Find source connector candidates (from modified config AND fixture)
        src_conn_uuids = set()
        for conns in [mod_conns, fix_conns]:
            for cu, entries in conns.items():
                for (buuid, btitle, btype, ck) in entries:
                    if from_title and not title_matches(from_title, btitle):
                        continue
                    if from_type and not type_matches(from_type, btype):
                        continue
                    if from_conn and ck != from_conn:
                        continue
                    if from_room:
                        ctrl = mod_controls.get(buuid, fix_controls.get(buuid, {}))
                        br = rooms.get(ctrl.get("room_uuid", ""), "")
                        if br != from_room:
                            continue
                    src_conn_uuids.add(cu)

        # Find destination connector and check wiring
        found = False
        for cu, entries in mod_conns.items():
            for (buuid, btitle, btype, ck) in entries:
                if to_title and not title_matches(to_title, btitle):
                    continue
                if to_type and not type_matches(to_type, btype):
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

                # UUID-sharing wiring: dest connector UUID matches a source connector UUID
                if cu in src_conn_uuids:
                    found = True
                    break
            if found:
                break

        label = " → ".join(filter(None, [
            from_title or from_type,
            f".{from_conn}" if from_conn else "",
            to_title or to_type,
            f".{to_conn}" if to_conn else "",
        ]))

        if found:
            wire_correct += 1
            wire_details.append({"check": f"wire:{label}", "pass": True})
        else:
            wire_details.append({"check": f"wire:{label}", "pass": False, "reason": "not found"})

    wiring_accuracy = wire_correct / wire_total if wire_total > 0 else 1.0

    # ── Param Evaluation ──
    exp_params = expected.get("params", [])
    param_correct = 0
    param_total = len(exp_params)
    param_details = []

    for exp in exp_params:
        exp_type = exp.get("block_type", "")
        exp_param = exp.get("param", "")
        exp_value = str(exp.get("value", ""))

        search_controls = {**new_blocks}
        for uuid, info in mod_controls.items():
            if uuid not in fix_controls:
                continue
            if exp_type and not type_matches(exp_type, info["type"]):
                continue
            search_controls[uuid] = info

        found = False
        for uuid, info in search_controls.items():
            if exp_type and not type_matches(exp_type, info["type"]):
                continue
            val = find_param_value(info["elem"], exp_param)
            if val is not None:
                # Handle float comparison
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
            param_details.append({"check": f"param:{label}", "pass": False, "reason": "not found"})

    param_accuracy = param_correct / param_total if param_total > 0 else 1.0

    # ── Check Score (sub-element completeness via lox config check) ──
    import subprocess as sp
    lox_bin = os.environ.get("LOX_BIN", "./target/debug/lox")
    if not os.path.exists(lox_bin):
        lox_bin = "cargo run --quiet --"
    chk = sp.run(f"{lox_bin} config check {config_file}".split(),
                 capture_output=True, text=True, timeout=30)
    check_ok = chk.stdout.count('✓')
    check_warn = chk.stdout.count('⚠')
    check_err = chk.stdout.count('✗')
    check_total = check_ok + check_warn + check_err
    check_score = check_ok / check_total if check_total > 0 else 1.0

    # ── Trace Score (BFS reachability through wiring graph) ──
    from collections import deque as _deque

    def _build_block_adjacency(xml_root):
        """Build block-level directed adjacency from XML wiring."""
        _blocks = {}   # uuid → {type, title, room_uuid, connectors: {conn_uuid: key}}
        _c2b = {}       # connector_uuid → block_uuid

        def _walk_b(el, pg=""):
            et = el.get("Type", "")
            eu = el.get("U", "")
            tl = el.get("Title", "")
            if et == "Page":
                pg = tl
            ru = ""
            for ch in el:
                if ch.tag == "IoData" or ch.get("Type") == "IoData":
                    ru = ch.get("Pr", "")
            _skip = {
                "Document", "Page", "Place", "Category", "Program",
                "User", "UserCaption", "LoxCaption", "VirtualInCaption",
                "LightscenesC", "LightsceneC", "Lightscene",
                "TreeDevice", "LoxAIRDevice", "NetworkDevice",
                "TreeAsensor", "TreeDsensor", "TreeAactuator", "TreeDactuator",
                "Co", "IoData", "Display", "HP", "Const", "Note", "SET",
            }
            if et and et not in _skip and eu:
                conns = {}
                for ch in el:
                    if ch.tag == "Co" or ch.get("Type") == "Co":
                        cu = ch.get("U", "")
                        ck = ch.get("K", "")
                        if cu:
                            conns[cu] = ck
                            _c2b[cu] = eu
                _blocks[eu] = {"type": et, "title": tl, "room_uuid": ru, "connectors": conns}
            for ch in el:
                _walk_b(ch, pg)

        _walk_b(xml_root)

        adj = defaultdict(set)
        def _walk_w(el):
            for ch in el:
                if ch.tag == "Co" or ch.get("Type") == "Co":
                    dcu = ch.get("U", "")
                    db = _c2b.get(dcu, "")
                    for inp in ch:
                        if inp.tag == "In":
                            scu = inp.get("Input", "")
                            if scu and dcu:
                                sb = _c2b.get(scu, "")
                                if sb and db and sb != db:
                                    adj[sb].add(db)
                _walk_w(ch)
        _walk_w(xml_root)

        # UUID-sharing
        ub = defaultdict(list)
        for cu, bu in _c2b.items():
            ub[cu].append(bu)
        for cu, bus in ub.items():
            uq = list(set(bus))
            if len(uq) >= 2:
                for a in uq:
                    for b in uq:
                        if a != b:
                            adj[a].add(b)
        return _blocks, adj

    def _match_block(blks, title="", btype="", room_name="", room_map=None):
        hits = []
        for uid, info in blks.items():
            if btype:
                if not type_matches(btype, info["type"]):
                    continue
            if title:
                base = title.split("[")[0].strip()
                if base not in info["title"]:
                    continue
            if room_name and room_map:
                if room_map.get(info["room_uuid"], "") != room_name:
                    continue
            hits.append(uid)
        return hits

    def _bfs_reachable(adj, sources):
        visited = set(sources)
        q = _deque(sources)
        while q:
            cur = q.popleft()
            for nb in adj.get(cur, set()):
                if nb not in visited:
                    visited.add(nb)
                    q.append(nb)
        return visited - set(sources)

    trace_blocks, trace_adj = _build_block_adjacency(mod_root)
    trace_found = 0
    trace_total = 0
    trace_details = []
    for exp in exp_wiring:
        st = exp.get("from_title", "")
        sy = exp.get("from_type", "")
        sr = exp.get("from_room", "")
        dt = exp.get("to_title", "")
        dy = exp.get("to_type", "")
        dr = exp.get("to_room", "")
        src_sel = {k: v for k, v in [("title", st), ("type", sy), ("room", sr)] if v}
        dst_sel = {k: v for k, v in [("title", dt), ("type", dy), ("room", dr)] if v}
        if not src_sel or not dst_sel:
            continue
        trace_total += 1
        src_ids = _match_block(trace_blocks, title=st, btype=sy, room_name=sr, room_map=rooms)
        dst_ids = _match_block(trace_blocks, title=dt, btype=dy, room_name=dr, room_map=rooms)
        reachable = _bfs_reachable(trace_adj, src_ids) if src_ids else set()
        hit = bool(set(dst_ids) & reachable)
        if hit:
            trace_found += 1
        label = " → ".join(filter(None, [st or sy, dt or dy]))
        reason = ""
        if not hit:
            if not src_ids:
                reason = f"source block not found"
            elif not dst_ids:
                reason = f"dest block not found"
            else:
                reason = "no path"
        trace_details.append({"check": f"trace:{label}", "pass": hit,
                              **({"reason": reason} if reason else {})})

    trace_score = trace_found / trace_total if trace_total > 0 else 1.0

    # ── Simulation Score (if simulation spec exists in case) ──
    sim_spec = expected.get("simulation")
    sim_score = None
    sim_details = []
    if sim_spec:
        sim_inputs = sim_spec.get("inputs", {})
        sim_ticks = sim_spec.get("ticks", 10)
        sim_expected = sim_spec.get("expected_outputs", {})
        sim_dt = sim_spec.get("dt", 0.1)

        # Try to call lox-sim if available, otherwise skip gracefully
        sim_script_dir = os.path.dirname(os.path.abspath(cases_file))
        sim_check_py = os.path.join(sim_script_dir, "sim_check.py")

        # For simulation, we check if lox-sim binary exists
        lox_sim_bin = os.environ.get("LOX_SIM_BIN", "")
        if not lox_sim_bin:
            # Try common locations
            for candidate in [
                os.path.join(sim_script_dir, "../../lox-sim/target/debug/lox-sim"),
                os.path.join(sim_script_dir, "../../target/debug/lox-sim"),
            ]:
                if os.path.exists(candidate):
                    lox_sim_bin = candidate
                    break

        if lox_sim_bin and os.path.exists(lox_sim_bin):
            # Call lox-sim binary for full simulation
            sim_cmd = json.dumps({
                "config": config_file,
                "inputs": sim_inputs,
                "ticks": sim_ticks,
                "dt": sim_dt,
            })
            try:
                sim_run = sp.run([lox_sim_bin, "eval", "--json"],
                                input=sim_cmd, capture_output=True,
                                text=True, timeout=30)
                if sim_run.returncode == 0:
                    sim_out = json.loads(sim_run.stdout)
                    sim_pass = 0
                    sim_total_checks = 0
                    for output_key, assertion in sim_expected.items():
                        sim_total_checks += 1
                        actual_val = sim_out.get("outputs", {}).get(output_key)
                        if actual_val is None:
                            sim_details.append({"check": f"sim:{output_key}", "pass": False,
                                                "reason": "output not found"})
                            continue
                        ok = True
                        for op, thresh in assertion.items():
                            if op == ">" and not (actual_val > thresh):
                                ok = False
                            elif op == "<" and not (actual_val < thresh):
                                ok = False
                            elif op == ">=" and not (actual_val >= thresh):
                                ok = False
                            elif op == "<=" and not (actual_val <= thresh):
                                ok = False
                            elif op == "==" and not (abs(actual_val - thresh) < 0.001):
                                ok = False
                        if ok:
                            sim_pass += 1
                        sim_details.append({"check": f"sim:{output_key}", "pass": ok,
                                            "actual": actual_val})
                    sim_score = sim_pass / sim_total_checks if sim_total_checks > 0 else 1.0
            except Exception:
                sim_details.append({"check": "sim:run", "pass": False, "reason": "lox-sim error"})
                sim_score = 0.0
        else:
            # lox-sim not available — record as skipped, don't penalize
            sim_details.append({"check": "sim:run", "pass": True, "reason": "lox-sim not available, skipped"})
            sim_score = None  # Don't include in overall

    # ── Overall Score ──
    # Weights: block_f1=0.20, wiring=0.30, params=0.20, check=0.15, trace=0.15
    overall = (0.20 * block_f1 + 0.30 * wiring_accuracy + 0.20 * param_accuracy
               + 0.15 * check_score + 0.15 * trace_score)
    difficulty_weights = {"easy": 1.0, "medium": 1.5, "hard": 2.0, "expert": 3.0}
    weighted_score = overall * difficulty_weights.get(difficulty, 1.0)
    passed = overall >= 0.8

    result = {
        "case_id": case_id,
        "pass": passed,
        "difficulty": difficulty,
        "patterns": case.get("pattern", []),
        "utterance": case["utterance"],
        "overall_score": round(overall, 3),
        "weighted_score": round(weighted_score, 3),
        "metrics": {
            "blocks": {
                "precision": round(block_precision, 3),
                "recall": round(block_recall, 3),
                "f1": round(block_f1, 3),
                "true_positives": block_tp,
                "false_positives": block_fp,
                "false_negatives": block_fn,
            },
            "wiring": {
                "accuracy": round(wiring_accuracy, 3),
                "correct": wire_correct,
                "total": wire_total,
            },
            "params": {
                "accuracy": round(param_accuracy, 3),
                "correct": param_correct,
                "total": param_total,
            },
            "check": {
                "score": round(check_score, 3),
                "ok": check_ok,
                "warnings": check_warn,
                "errors": check_err,
            },
            "trace": {
                "score": round(trace_score, 3),
                "found": trace_found,
                "total": trace_total,
            },
            **({"simulation": {
                "score": round(sim_score, 3),
                "details": sim_details,
            }} if sim_score is not None else {}),
        },
        "details": block_details + wire_details + param_details + trace_details + sim_details,
        "new_blocks_found": [
            {"type": info["type"], "title": info["title"],
             "room": rooms.get(info["room_uuid"], "?"), "page": info["page"]}
            for info in new_blocks.values()
        ],
    }

    print(json.dumps(result, indent=2, ensure_ascii=False))
    sys.exit(0 if passed else 1)

main()
PYEOF
}

# Pretty-print a single eval result
print_result() {
  local json_result="$1"
  python3 << PYEOF
import json, sys

r = json.loads('''$json_result''')

case_id = r['case_id']
passed = r['pass']
score = r['overall_score']
diff = r['difficulty']
m = r['metrics']

if passed:
    status = "\033[32m✓ PASS\033[0m"
else:
    status = "\033[31m✗ FAIL\033[0m"

print(f"\n{status}  {case_id}  (score: {score:.1%}, difficulty: {diff})")
print(f"  Blocks:  P={m['blocks']['precision']:.0%}  R={m['blocks']['recall']:.0%}  F1={m['blocks']['f1']:.0%}  (TP={m['blocks']['true_positives']} FP={m['blocks']['false_positives']} FN={m['blocks']['false_negatives']})")
print(f"  Wiring:  {m['wiring']['accuracy']:.0%}  ({m['wiring']['correct']}/{m['wiring']['total']})")
print(f"  Params:  {m['params']['accuracy']:.0%}  ({m['params']['correct']}/{m['params']['total']})")
chk = m.get('check', {})
if chk:
    print(f"  Check:   {chk.get('score',0):.0%}  ({chk.get('ok',0)} ok, {chk.get('warnings',0)} warn, {chk.get('errors',0)} err)")
trc = m.get('trace', {})
if trc:
    print(f"  Trace:   {trc.get('score',0):.0%}  ({trc.get('found',0)}/{trc.get('total',0)} paths found)")
sim = m.get('simulation', {})
if sim:
    sd = sim.get('details', [])
    sp_count = sum(1 for d in sd if d.get('pass'))
    sf_count = len(sd) - sp_count
    label_parts = []
    for d in sd:
        ck = d.get('check', '').replace('sim:', '')
        if d.get('actual') is not None:
            label_parts.append(f"{ck}={d['actual']}")
    sim_label = ', '.join(label_parts[:3]) if label_parts else ''
    print(f"  Sim:     {sim.get('score',0):.0%}  ({sp_count} ok, {sf_count} err)" + (f"  ({sim_label})" if sim_label else ""))

# Show failures
failures = [d for d in r.get('details', []) if not d['pass']]
if failures:
    print(f"  Failures:")
    for f in failures:
        print(f"    ✗ {f['check']}: {f.get('reason', 'unknown')}")
PYEOF
}

# Batch evaluation
batch_eval() {
  local results_dir="$1"
  local report_file="${2:-/dev/stdout}"

  echo -e "${CYAN}${BOLD}═══ Loxone Eval Harness v2 ═══${RESET}"
  echo -e "${DIM}Fixture: $FIXTURE${RESET}"
  echo -e "${DIM}Cases:   $CASES${RESET}"
  echo ""

  local total=0
  local passed=0
  local failed=0
  local errors=0
  local results_json="[]"

  for config_file in "$results_dir"/*.Loxone; do
    [ -f "$config_file" ] || continue
    local base=$(basename "$config_file" .Loxone)

    export EVAL_FIXTURE="$FIXTURE"
    export EVAL_CONFIG="$config_file"
    export EVAL_DIR="$EVAL_DIR"
    export EVAL_CASE_ID="$base"

    total=$((total + 1))

    set +e
    result=$(eval_single "$base" "$config_file" 2>/dev/null)
    exit_code=$?
    set -e

    if [ -z "$result" ] || echo "$result" | python3 -c "import json,sys; json.load(sys.stdin)" 2>/dev/null; then
      if [ $exit_code -eq 0 ]; then
        passed=$((passed + 1))
        echo -e "${GREEN}✓${RESET} $base"
      else
        failed=$((failed + 1))
        echo -e "${RED}✗${RESET} $base"
        print_result "$result" 2>/dev/null || true
      fi

      results_json=$(python3 -c "
import json, sys
results = json.loads('$results_json') if '$results_json' != '[]' else []
try:
    result = json.loads('''$result''')
    results.append(result)
except:
    pass
print(json.dumps(results))
" 2>/dev/null || echo "$results_json")
    else
      errors=$((errors + 1))
      echo -e "${YELLOW}⚠${RESET} $base (error)"
    fi
  done

  echo ""
  echo -e "${BOLD}═══ Summary ═══${RESET}"
  echo -e "  Total:   $total"
  echo -e "  ${GREEN}Passed:  $passed${RESET}"
  echo -e "  ${RED}Failed:  $failed${RESET}"
  if [ $errors -gt 0 ]; then
    echo -e "  ${YELLOW}Errors:  $errors${RESET}"
  fi

  if [ $total -gt 0 ]; then
    local pct=$((passed * 100 / total))
    echo -e "  Score:   ${pct}%"
  fi

  # Generate JSON report
  if [ "$report_file" != "/dev/stdout" ]; then
    python3 << PYEOF
import json
from collections import Counter

results = json.loads('''$results_json''')

# Aggregate metrics
total = len(results)
passed = sum(1 for r in results if r['pass'])
difficulties = Counter(r['difficulty'] for r in results)
patterns = Counter()
for r in results:
    for p in r.get('patterns', []):
        patterns[p] += 1

avg_score = sum(r['overall_score'] for r in results) / total if total else 0
avg_weighted = sum(r['weighted_score'] for r in results) / total if total else 0
avg_block_f1 = sum(r['metrics']['blocks']['f1'] for r in results) / total if total else 0
avg_wiring = sum(r['metrics']['wiring']['accuracy'] for r in results) / total if total else 0
avg_params = sum(r['metrics']['params']['accuracy'] for r in results) / total if total else 0
avg_trace = sum(r['metrics']['trace']['score'] for r in results) / total if total else 0
sim_results = [r for r in results if 'simulation' in r['metrics']]
avg_sim = sum(r['metrics']['simulation']['score'] for r in sim_results) / len(sim_results) if sim_results else None

# Per-difficulty scores
diff_scores = {}
for diff in ['easy', 'medium', 'hard', 'expert']:
    subset = [r for r in results if r['difficulty'] == diff]
    if subset:
        diff_scores[diff] = {
            "count": len(subset),
            "passed": sum(1 for r in subset if r['pass']),
            "avg_score": round(sum(r['overall_score'] for r in subset) / len(subset), 3),
        }

# Per-pattern scores
pat_scores = {}
for pat in sorted(patterns.keys()):
    subset = [r for r in results if pat in r.get('patterns', [])]
    if subset:
        pat_scores[pat] = {
            "count": len(subset),
            "passed": sum(1 for r in subset if r['pass']),
            "avg_score": round(sum(r['overall_score'] for r in subset) / len(subset), 3),
        }

report = {
    "summary": {
        "total": total,
        "passed": passed,
        "failed": total - passed,
        "pass_rate": round(passed / total, 3) if total else 0,
        "avg_overall_score": round(avg_score, 3),
        "avg_weighted_score": round(avg_weighted, 3),
        "avg_block_f1": round(avg_block_f1, 3),
        "avg_wiring_accuracy": round(avg_wiring, 3),
        "avg_param_accuracy": round(avg_params, 3),
        "avg_trace_score": round(avg_trace, 3),
        **({"avg_sim_score": round(avg_sim, 3)} if avg_sim is not None else {}),
    },
    "by_difficulty": diff_scores,
    "by_pattern": pat_scores,
    "cases": results,
}

with open("$report_file", "w") as f:
    json.dump(report, f, indent=2, ensure_ascii=False)
print(f"Report written to $report_file")
PYEOF
  fi
}

print_report() {
  local report_file="$1"
  python3 << PYEOF
import json

with open("$report_file") as f:
    report = json.load(f)

s = report['summary']
print(f"\n{'═' * 60}")
print(f"  EVAL REPORT")
print(f"{'═' * 60}\n")
print(f"  Pass Rate:      {s['passed']}/{s['total']} ({s['pass_rate']:.0%})")
print(f"  Overall Score:  {s['avg_overall_score']:.1%}")
print(f"  Weighted Score: {s['avg_weighted_score']:.2f}")
print(f"  Block F1:       {s['avg_block_f1']:.1%}")
print(f"  Wiring:         {s['avg_wiring_accuracy']:.1%}")
print(f"  Params:         {s['avg_param_accuracy']:.1%}")
print(f"  Trace:          {s['avg_trace_score']:.1%}")
if 'avg_sim_score' in s:
    print(f"  Simulation:     {s['avg_sim_score']:.1%}")

print(f"\n  By Difficulty:")
for diff in ['easy', 'medium', 'hard', 'expert']:
    d = report.get('by_difficulty', {}).get(diff, {})
    if d:
        print(f"    {diff:8s}: {d['passed']}/{d['count']} passed, avg {d['avg_score']:.0%}")

print(f"\n  By Pattern:")
for pat, p in sorted(report.get('by_pattern', {}).items(), key=lambda x: -x[1]['count']):
    print(f"    {pat:25s}: {p['passed']}/{p['count']} passed, avg {p['avg_score']:.0%}")

# Show worst 5 cases
cases = report.get('cases', [])
worst = sorted(cases, key=lambda c: c['overall_score'])[:5]
if worst:
    print(f"\n  Worst 5 Cases:")
    for c in worst:
        status = "✓" if c['pass'] else "✗"
        print(f"    {status} {c['case_id']:40s} {c['overall_score']:.0%}  ({c['difficulty']})")
PYEOF
}

# ── Main ──
if [[ $# -lt 1 ]]; then usage; fi

case "$1" in
  --list)
    shift
    filter=""
    if [[ $# -gt 0 && "$1" == "--filter" ]]; then
      shift
      filter="${1:-}"
    fi
    list_cases "$filter"
    exit 0
    ;;
  --stats)
    show_stats
    exit 0
    ;;
  --validate-fixture)
    validate_fixture
    exit $?
    ;;
  --batch)
    shift
    if [[ $# -lt 1 ]]; then
      echo "Error: --batch requires a results directory" >&2
      exit 2
    fi
    results_dir="$1"
    report_file="${2:-eval-report.json}"
    batch_eval "$results_dir" "$report_file"
    exit 0
    ;;
  --report)
    shift
    if [[ $# -lt 1 ]]; then
      echo "Error: --report requires a report JSON file" >&2
      exit 2
    fi
    print_report "$1"
    exit 0
    ;;
  --help|-h)
    usage
    ;;
esac

if [[ $# -lt 2 ]]; then usage; fi

CASE_ID="$1"
CONFIG="$2"

if [[ ! -f "$CONFIG" ]]; then
  echo -e "${RED}Error: config file not found: $CONFIG${RESET}" >&2
  exit 2
fi

if [[ ! -f "$FIXTURE" ]]; then
  echo -e "${RED}Error: fixture not found: $FIXTURE${RESET}" >&2
  exit 2
fi

# Validate first
echo -e "${CYAN}━━━ Eval: $CASE_ID ━━━${RESET}"
echo -e "${CYAN}[1/3] Validating config...${RESET}"

VALIDATE_OUT=$($LOX_CMD config validate "$CONFIG" 2>&1) || true
ERRORS=$(echo "$VALIDATE_OUT" | grep -c '^✗' || true)

if [[ "$ERRORS" -gt 0 ]]; then
  echo "$VALIDATE_OUT"
  echo -e "${RED}FAIL: config has validation errors${RESET}"
  exit 1
fi
echo -e "${GREEN}  Validation OK${RESET}"

# Run eval
echo -e "${CYAN}[2/3] Checking expectations...${RESET}"

export EVAL_FIXTURE="$FIXTURE"
export EVAL_CONFIG="$CONFIG"
export EVAL_DIR="$EVAL_DIR"
export EVAL_CASE_ID="$CASE_ID"

set +e
RESULT=$(eval_single "$CASE_ID" "$CONFIG")
EXIT=$?
set -e

# Print result
echo -e "${CYAN}[3/3] Result:${RESET}"
print_result "$RESULT"

echo ""
if [[ $EXIT -eq 0 ]]; then
  echo -e "${GREEN}${BOLD}PASS${RESET}"
  exit 0
else
  echo -e "${RED}${BOLD}FAIL${RESET}"
  exit 1
fi
