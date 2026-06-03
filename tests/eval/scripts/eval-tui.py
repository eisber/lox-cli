#!/usr/bin/env python3
"""Beautiful terminal UI for inspecting Loxone eval results."""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import termios
import tty
from collections import defaultdict
from dataclasses import dataclass, field
from pathlib import Path
from rich.console import Console
from rich.layout import Layout
from rich.live import Live
from rich.panel import Panel
from rich.table import Table
from rich.text import Text

SCRIPT_DIR = Path(__file__).resolve().parent
REPO = SCRIPT_DIR.parent.parent.parent
DEFAULT_REPORT = REPO / "tests/eval/reports/llm-report.json"
DEFAULT_CASES = REPO / "tests/eval/cases"
DEFAULT_CONFIGS = Path("/tmp/lox-eval-agent-z98dvmgk")
REPORTS_DIR = REPO / "tests/eval/reports"
LOX_BIN = REPO / "target/release/lox"
DIFF_COLORS = {"easy": "green", "medium": "yellow", "hard": "red", "expert": "magenta"}
SORT_KEYS = ["case_id", "difficulty", "sim", "block_f1", "wiring", "tokens"]
DIFF_ORDER = {"easy": 0, "medium": 1, "hard": 2, "expert": 3}

HELP_DASHBOARD = Text.assemble(
    ("[↑↓]", "bold"), " Navigate  ", ("[Enter]", "bold"), " Detail  ",
    ("[f]", "bold"), " Filter  ", ("[d]", "bold"), " Difficulty  ",
    ("[s]", "bold"), " Sort  ", ("[R]", "bold"), " Reports  ",
    ("[/]", "bold"), " Search  ", ("[q]", "bold"), " Quit")

HELP_DETAIL = Text.assemble(
    ("[1-3]", "bold"), " Tabs  ", ("[n/p]", "bold"), " Fails  ",
    ("[c]", "bold"), " Copy  ", ("[t]", "bold"), " Trace  ",
    ("[r]", "bold"), " Re-run  ", ("[Esc]", "bold"), " Back  ",
    ("[?]", "bold"), " Help")

HELP_REPORTS = Text.assemble(
    ("[↑↓]", "bold"), " Navigate  ", ("[Enter]", "bold"), " Select  ",
    ("[Esc]", "bold"), " Back  ", ("[q]", "bold"), " Quit")

HELP_FULL = {
    "dashboard": [
        ("↑ / ↓", "Navigate cases"),
        ("Enter", "Open case detail"),
        ("f", "Cycle filter: all → pass → fail"),
        ("d", "Cycle difficulty: all → easy → medium → hard → expert"),
        ("s", "Cycle sort: case_id → difficulty → sim → block → tokens"),
        ("r", "Reverse sort order"),
        ("/ ", "Search by text (case ID, utterance, pattern)"),
        ("g / G", "Jump to first / last case"),
        ("R", "Open report picker (switch eval runs)"),
        ("?  h", "Show this help"),
        ("q", "Quit"),
    ],
    "detail": [
        ("↑ / ↓", "Scroll content"),
        ("PgUp / PgDn", "Scroll one page"),
        ("g / G", "Jump to top / bottom"),
        ("1 / 2 / 3", "Switch tab: Circuit / Conversation / Commands"),
        ("n", "Jump to next failing case"),
        ("p", "Jump to previous failing case"),
        ("c", "Copy case summary to clipboard"),
        ("r", "Re-run simulation live"),
        ("t", "Open sim trace stepper"),
        ("Esc  q", "Back to dashboard"),
        ("?  h", "Show this help"),
    ],
    "sim_trace": [
        ("↑ / ↓", "Scroll content"),
        ("PgUp / PgDn", "Scroll one page"),
        ("← / →", "Previous / next step"),
        ("[ / ]", "Previous / next scenario"),
        ("g / G", "Jump to top / bottom"),
        ("Esc  q", "Back to detail view"),
        ("?  h", "Show this help"),
    ],
    "sim_rerun": [
        ("Esc  q", "Back to detail view"),
        ("?  h", "Show this help"),
    ],
    "report_picker": [
        ("↑ / ↓", "Navigate reports"),
        ("Enter", "Load selected report"),
        ("g / G", "Jump to first / last report"),
        ("Esc  q", "Quit"),
        ("?  h", "Show this help"),
    ],
}


def readkey():
    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    try:
        tty.setraw(fd)
        ch = sys.stdin.read(1)
        if ch == "\x1b":
            ch2 = sys.stdin.read(1)
            if ch2 == "[":
                ch3 = sys.stdin.read(1)
                if ch3 in ("5", "6"):
                    sys.stdin.read(1)  # consume trailing '~'
                    return "page_up" if ch3 == "5" else "page_down"
                return {"A": "up", "B": "down", "C": "right", "D": "left"}.get(ch3, "")
            return "esc"
        return ch
    finally:
        termios.tcsetattr(fd, termios.TCSADRAIN, old)


def load_cases(cases_dir):
    specs = {}
    d = Path(cases_dir)
    if not d.exists():
        return specs
    for f in sorted(d.glob("*.json")):
        try:
            data = json.loads(f.read_text())
            for c in (data if isinstance(data, list) else [data]):
                if "id" in c:
                    specs[c["id"]] = c
        except (json.JSONDecodeError, KeyError):
            pass
    return specs


def load_report(path):
    if not path:
        return None
    p = Path(path)
    if not p.exists():
        return None
    try:
        return json.loads(p.read_text())
    except json.JSONDecodeError:
        return None


def _make_row(rc, spec):
    """Build a unified row dict from report case + spec."""
    sim = rc.get("simulation", {})
    tok = rc.get("tokens", {})
    ti, to = tok.get("input_tokens_est", 0), tok.get("output_tokens_est", 0)
    m = rc.get("metrics", {})
    return {
        "case_id": rc.get("case_id", ""), "pass": rc.get("pass", False),
        "difficulty": rc.get("difficulty", spec.get("difficulty", "?")),
        "patterns": rc.get("patterns", spec.get("pattern", [])),
        "utterance": rc.get("utterance", spec.get("utterance", "")),
        "sim_passed": sim.get("passed_count", 0), "sim_total": sim.get("total_count", 0),
        "sim_pass": rc.get("sim_pass", False),
        "block_f1": m.get("blocks", {}).get("f1", 0),
        "wiring_acc": m.get("wiring", {}).get("accuracy", 0),
        "param_acc": m.get("params", {}).get("accuracy", 0),
        "tokens_in": ti, "tokens_out": to, "tokens_total": ti + to,
        "cli_invocations": rc.get("cli_invocations", 0), "retries": rc.get("retries", 0),
        "simulation_detail": sim, "metrics": m, "spec": spec,
    }


def merge_data(report, specs, configs_dir):
    """Merge report data with case specs. If no report, run sims on saved configs."""
    if report and "cases" in report:
        return [_make_row(rc, specs.get(rc.get("case_id", ""), {})) for rc in report["cases"]]

    # Fallback: scan saved configs and run sims to populate results
    rows = []
    configs_path = Path(configs_dir) if configs_dir else DEFAULT_CONFIGS
    lox_bin = None
    for candidate in ["./target/release/lox", "lox"]:
        if Path(candidate).exists() or shutil.which(candidate):
            lox_bin = candidate
            break

    for cid, spec in sorted(specs.items()):
        config = configs_path / f"{cid}.Loxone"
        rc = {"case_id": cid}
        if config.exists() and lox_bin:
            sims = spec.get("expected", {}).get("simulation", [])
            if sims:
                try:
                    r = subprocess.run(
                        [lox_bin, "sim", "run", str(config), "--sim", json.dumps(sims)],
                        capture_output=True, text=True, timeout=60,
                    )
                    if r.stdout.strip():
                        result = json.loads(r.stdout)
                        rc["sim_pass"] = result.get("pass", False)
                        rc["pass"] = result.get("pass", False)
                        rc["simulation"] = {
                            "passed_count": result.get("passed", 0),
                            "total_count": result.get("total", 0),
                            "scenarios": result.get("scenarios", []),
                        }
                except Exception:
                    pass
            rc["_has_config"] = True
        rows.append(_make_row(rc, spec))
    return rows


def scan_reports(reports_dir):
    """Scan a directory for report JSON files and return metadata sorted newest first."""
    results = []
    d = Path(reports_dir)
    if not d.exists():
        return results
    for f in sorted(d.glob("*.json")):
        try:
            data = json.loads(f.read_text())
            if not isinstance(data, dict):
                continue
            cases = data.get("cases", [])
            meta = data.get("meta", {})
            total = len(cases)
            passed = sum(1 for c in cases if c.get("pass"))
            rate = (passed / total * 100) if total else 0
            results.append({
                "path": str(f),
                "filename": f.name,
                "timestamp": meta.get("timestamp", ""),
                "mtime": f.stat().st_mtime,
                "cases": total,
                "passed": passed,
                "pass_rate": rate,
                "model": meta.get("model", ""),
                "work_dir": meta.get("work_dir", ""),
            })
        except (json.JSONDecodeError, KeyError, OSError):
            pass
    results.sort(key=lambda r: r["mtime"], reverse=True)
    return results


def build_wiring_dag(dump):
    """Build a DAG from dump data. Returns (blocks, edges)."""
    if not dump:
        return [], []
    blocks = [b for b in dump.get("blocks", []) if b.get("type", "") not in SKIP_TYPES]
    cid_to_block = {}
    for i, blk in enumerate(blocks):
        for inp in blk.get("inputs", []):
            cid_to_block[inp["cid"]] = (i, inp["key"], "in")
        for out in blk.get("outputs", []):
            cid_to_block[out["cid"]] = (i, out["key"], "out")

    edges = set()
    for i, blk in enumerate(blocks):
        for inp in blk.get("inputs", []):
            wf = inp.get("wired_from")
            if wf and wf in cid_to_block:
                src_idx, src_key, _ = cid_to_block[wf]
                if src_idx != i:
                    edges.add((src_idx, i, src_key, inp["key"]))
        for out in blk.get("outputs", []):
            for tgt_cid in out.get("wired_to", []):
                if tgt_cid in cid_to_block:
                    dst_idx, dst_key, _ = cid_to_block[tgt_cid]
                    if i != dst_idx:
                        edges.add((i, dst_idx, out["key"], dst_key))
    return blocks, list(edges)


def _topo_sort(n_nodes, edges):
    """Topological sort via Kahn's algorithm. Returns sorted indices or None if cyclic."""
    adj = defaultdict(list)
    in_deg = [0] * n_nodes
    for s, d, _, _ in edges:
        adj[s].append(d)
        in_deg[d] += 1
    queue = sorted([i for i in range(n_nodes) if in_deg[i] == 0])
    result = []
    while queue:
        node = queue.pop(0)
        result.append(node)
        for nb in sorted(adj[node]):
            in_deg[nb] -= 1
            if in_deg[nb] == 0:
                queue.append(nb)
    return result if len(result) == n_nodes else None


def render_wiring_diagram(dump, max_width=70):
    """Render an ASCII wiring diagram. Returns list of Text lines, or None if too complex."""
    blocks, edges = build_wiring_dag(dump)
    if not blocks or not edges:
        return None
    wired = set()
    for s, d, _, _ in edges:
        wired.add(s)
        wired.add(d)
    if len(wired) > 8:
        return None

    order = _topo_sort(len(blocks), edges)
    if not order:
        return None

    ordered_wired = [i for i in order if i in wired]
    if not ordered_wired:
        return None

    in_edges = defaultdict(list)
    out_edges = defaultdict(list)
    for s, d, sk, dk in edges:
        in_edges[d].append((s, sk, dk))
        out_edges[s].append((d, sk, dk))

    lines = []
    for pos, idx in enumerate(ordered_wired):
        blk = blocks[idx]
        name = blk.get("name", "?")
        btype = blk.get("type", "?")
        room = blk.get("room", "")

        incoming = in_edges.get(idx, [])
        if incoming and pos > 0:
            for src_idx, src_key, dst_key in incoming:
                src_name = blocks[src_idx].get("name", "?")
                lines.append(Text(f"  {src_name}.{src_key}", style="dim"))
            lines.append(Text("                    │", style="dim"))
            lines.append(Text("                    ▼", style="bold"))

        label = name
        type_str = f"({btype})"
        room_str = f"[{room}]" if room else ""
        box_w = max(len(label) + 2, len(type_str) + 2, len(room_str) + 2 if room_str else 0, 15)
        box_w = min(box_w, max_width - 16)
        pad = " " * 12

        lines.append(Text(f"{pad}┌{'─' * box_w}┐"))
        lines.append(Text.assemble((f"{pad}│", ""), (label.center(box_w), "bold"), ("│", "")))
        lines.append(Text(f"{pad}│{type_str.center(box_w)}│", style="dim"))
        if room:
            lines.append(Text(f"{pad}│{room_str.center(box_w)}│", style="dim"))
        lines.append(Text(f"{pad}└{'─' * (box_w // 2)}┬{'─' * (box_w - box_w // 2 - 1)}┘"))

        outgoing = out_edges.get(idx, [])
        if outgoing and pos < len(ordered_wired) - 1:
            out_keys = sorted(set(sk for _, sk, _ in outgoing))
            lines.append(Text(f"{pad}{' ' * (box_w // 2)}.{','.join(out_keys)}  │", style="dim"))

    return lines if lines else None


SKIP_TYPES = {"VirtualInCaption", "WeatherServer", "LightscenesC", "LightsceneC"}

_SIG_RE = re.compile(r"^(.+?)(?:\s+\[(.+?)\])?\.([A-Za-z0-9_]+)$")


@dataclass
class BlockState:
    name: str
    block_type: str
    room: str
    inputs: dict = field(default_factory=dict)   # connector_key → value
    outputs: dict = field(default_factory=dict)   # connector_key → value
    sources: dict = field(default_factory=dict)   # input_key → "SourceBlock.OutputKey"
    targets: dict = field(default_factory=dict)   # output_key → "TargetBlock.InputKey"
    checked: dict = field(default_factory=dict)   # output_key → check dict
    injected: bool = False


def _parse_signal_key(sig_key):
    """Parse 'Name [Room].Connector' or 'Name.Connector' → (name, room, connector)."""
    m = _SIG_RE.match(sig_key)
    if m:
        return m.group(1), m.group(2) or "", m.group(3)
    # Fallback: split on last '.'
    dot = sig_key.rfind(".")
    if dot > 0:
        return sig_key[:dot], "", sig_key[dot + 1:]
    return sig_key, "", ""


def _build_block_states(dump, signals, checks, injected_keys, injected_values=None):
    """Build topologically ordered BlockState list from dump + trace signals.

    Args:
        dump: Block graph from `lox sim dump --json`
        signals: Dict of signal_key → value from trace
        checks: List of check dicts with 'output', 'pass', etc.
        injected_keys: Set of injected input keys (e.g. "Sensor.AQ")
        injected_values: Dict of injected input keys → values
    Returns:
        List of BlockState in topological order, filtered to active blocks.
    """
    if injected_values is None:
        injected_values = {}
    if not dump:
        return []

    blocks = dump.get("blocks", [])
    if not blocks:
        return []

    # Build cid→(block_idx, key, direction) and cid→"Name.Key" maps
    cid_to_info = {}
    cid_to_label = {}
    for i, blk in enumerate(blocks):
        bname = blk.get("name", "?")
        broom = blk.get("room", "")
        for inp in blk.get("inputs", []):
            cid_to_info[inp["cid"]] = (i, inp["key"], "in")
            cid_to_label[inp["cid"]] = f"{bname}.{inp['key']}"
        for out in blk.get("outputs", []):
            cid_to_info[out["cid"]] = (i, out["key"], "out")
            cid_to_label[out["cid"]] = f"{bname}.{out['key']}"

    # Build block name/room → block_idx lookup
    name_to_idx = {}
    for i, blk in enumerate(blocks):
        bname = blk.get("name", "?")
        broom = blk.get("room", "")
        name_to_idx[(bname, broom)] = i
        name_to_idx[(bname, "")] = i  # allow room-less match

    # Map signals to blocks: signal_key → (block_idx, connector_key, value)
    sig_map = []
    for sig_key, val in signals.items():
        sname, sroom, sconn = _parse_signal_key(sig_key)
        idx = name_to_idx.get((sname, sroom)) or name_to_idx.get((sname, ""))
        if idx is not None:
            sig_map.append((idx, sconn, val))

    # Build check output → check map
    check_map = {}
    for ch in checks:
        co = ch.get("output", "")
        if co:
            check_map[co] = ch

    # Build injected block names
    injected_names = set()
    for ik in injected_keys:
        iname, _, _ = _parse_signal_key(ik)
        injected_names.add(iname)

    # Build BlockState for each block
    block_states = []
    active_indices = set()
    for i, blk in enumerate(blocks):
        if blk.get("type", "") in SKIP_TYPES:
            continue
        bname = blk.get("name", "?")
        btype = blk.get("type", "?")
        broom = blk.get("room", "")

        bs = BlockState(name=bname, block_type=btype, room=broom)
        bs.injected = bname in injected_names

        # For injected blocks, populate their output values from injected_values
        if bs.injected and injected_values:
            for ik, iv in injected_values.items():
                ik_name, _, ik_conn = _parse_signal_key(ik)
                if ik_name == bname:
                    if ik_conn:
                        bs.outputs[ik_conn] = iv
                    else:
                        # Bare name injection → set first output
                        for out in blk.get("outputs", [])[:1]:
                            bs.outputs[out["key"]] = iv

        # Populate connector values from signals
        for idx, conn, val in sig_map:
            if idx == i:
                # Determine if input or output
                in_keys = {inp["key"] for inp in blk.get("inputs", [])}
                out_keys = {out["key"] for out in blk.get("outputs", [])}
                if conn in out_keys:
                    bs.outputs[conn] = val
                elif conn in in_keys:
                    bs.inputs[conn] = val
                else:
                    # Default to output
                    bs.outputs[conn] = val

        # Resolve source names for inputs (wired_from)
        for inp in blk.get("inputs", []):
            wf = inp.get("wired_from")
            if wf and wf in cid_to_label:
                bs.sources[inp["key"]] = cid_to_label[wf]

        # Resolve target names for outputs (wired_to)
        for out in blk.get("outputs", []):
            for tgt_cid in out.get("wired_to", []):
                if tgt_cid in cid_to_label:
                    bs.targets[out["key"]] = cid_to_label[tgt_cid]

        # Attach check annotations
        for out_key in bs.outputs:
            full_key = f"{bname}.{out_key}"
            if full_key in check_map:
                bs.checked[out_key] = check_map[full_key]
            # Also match partial (check output may omit room)
            for co, ch in check_map.items():
                cname, _, cconn = _parse_signal_key(co)
                if cname == bname and cconn == out_key:
                    bs.checked[out_key] = ch

        # Track active (has non-zero signals)
        has_nonzero = any(v != 0 and v != 0.0 for v in bs.inputs.values()) or \
                      any(v != 0 and v != 0.0 for v in bs.outputs.values())
        if has_nonzero or bs.injected:
            active_indices.add(i)

        block_states.append((i, bs))

    # Expand active set: include blocks wired to/from active blocks
    expanded = set(active_indices)
    for i, blk in enumerate(blocks):
        if i in expanded:
            continue
        for inp in blk.get("inputs", []):
            wf = inp.get("wired_from")
            if wf and wf in cid_to_info:
                src_idx = cid_to_info[wf][0]
                if src_idx in active_indices:
                    expanded.add(i)
                    break
        if i not in expanded:
            for out in blk.get("outputs", []):
                for tgt_cid in out.get("wired_to", []):
                    if tgt_cid in cid_to_info:
                        dst_idx = cid_to_info[tgt_cid][0]
                        if dst_idx in active_indices:
                            expanded.add(i)
                            break
                if i in expanded:
                    break

    # Filter to active blocks
    filtered = [(i, bs) for i, bs in block_states if i in expanded]
    if not filtered:
        return []

    # Topological sort
    idx_set = {i for i, _ in filtered}
    idx_remap = {old: new for new, old in enumerate(sorted(idx_set))}
    edges = []
    for i, blk in enumerate(blocks):
        if i not in idx_set:
            continue
        for inp in blk.get("inputs", []):
            wf = inp.get("wired_from")
            if wf and wf in cid_to_info:
                src_idx = cid_to_info[wf][0]
                if src_idx in idx_set and src_idx != i:
                    edges.append((idx_remap[src_idx], idx_remap[i], "", ""))

    order = _topo_sort(len(idx_remap), edges)
    if order is None:
        # Cycle: just use original order
        return [bs for _, bs in filtered]

    remap_to_orig = {v: k for k, v in idx_remap.items()}
    orig_order = [remap_to_orig[o] for o in order]
    idx_to_bs = {i: bs for i, bs in filtered}
    return [idx_to_bs[i] for i in orig_order if i in idx_to_bs]


class EvalTUI:
    _dump_cache = {}

    def __init__(self, report_path, cases_dir, configs_dir):
        self.console = Console(highlight=False)
        self.specs = load_cases(cases_dir)
        self.report = load_report(report_path)
        self.report_path = report_path
        self.cases_dir = cases_dir
        # Resolve configs dir: prefer report meta.work_dir if it exists on disk
        meta_wd = None
        if self.report:
            meta_wd = self.report.get("meta", {}).get("work_dir")
        if meta_wd and Path(meta_wd).exists():
            self.configs_dir = Path(meta_wd)
        else:
            self.configs_dir = Path(configs_dir) if configs_dir else DEFAULT_CONFIGS

        # If the default report has very few cases and no report was explicitly given,
        # use fallback mode (live sims on saved configs) for current results.
        # User can press R to load historical reports.
        report_cases = self.report.get("cases", []) if self.report else []
        if len(report_cases) < 5 and report_path is None:
            self.report = None
            self.report_path = None

        self.all_rows = merge_data(self.report, self.specs, str(self.configs_dir))
        self.rows = list(self.all_rows)
        self.view = "dashboard"
        self.cursor = self.scroll_offset = 0
        self.selected = None
        self.sort_key, self.sort_rev = "case_id", False
        self.filt_status, self.filt_diff, self.filt_pat = "all", None, ""
        self.sim_output = ""
        self.detail_tab = 1
        self._status_msg = ""
        self._help_from = ""
        self.detail_scroll = 0
        # Sim trace state
        self.trace_data = None
        self.trace_scenario_idx = 0
        self.trace_step_idx = 0
        self.trace_scroll = 0
        # Report picker state
        self.reports_dir = str(REPORTS_DIR)
        self.available_reports = []
        self.report_cursor = 0
        self.report_scroll = 0
        self._apply_sort()

    def _get_term_width(self):
        """Get current terminal width."""
        try:
            return os.get_terminal_size().columns
        except (ValueError, OSError):
            return self.console.size.width

    def _find_best_report(self):
        """Find the report with the most cases (preferring recent ones)."""
        if not hasattr(self, "available_reports") or not self.available_reports:
            return None
        # Pick the report with the most cases; break ties by date (newest)
        return max(self.available_reports, key=lambda r: (r.get("cases", 0), r.get("date", "")))

    def _apply_filters(self):
        self.rows = list(self.all_rows)
        if self.filt_status == "pass":
            self.rows = [r for r in self.rows if r["pass"]]
        elif self.filt_status == "fail":
            self.rows = [r for r in self.rows if not r["pass"]]
        if self.filt_diff:
            self.rows = [r for r in self.rows if r["difficulty"] == self.filt_diff]
        if self.filt_pat:
            p = self.filt_pat.lower()
            self.rows = [r for r in self.rows if p in r["case_id"].lower()
                         or p in r["utterance"].lower()
                         or any(p in t.lower() for t in r["patterns"])]
        self._apply_sort()
        self.cursor = min(self.cursor, max(0, len(self.rows) - 1))

    def _apply_sort(self):
        km = {"case_id": lambda r: r["case_id"], "difficulty": lambda r: DIFF_ORDER.get(r["difficulty"], 9),
              "sim": lambda r: r["sim_passed"] / max(r["sim_total"], 1), "block_f1": lambda r: r["block_f1"],
              "wiring": lambda r: r["wiring_acc"], "tokens": lambda r: r["tokens_total"]}
        self.rows.sort(key=km.get(self.sort_key, km["case_id"]), reverse=self.sort_rev)

    def _load_report(self, report_info):
        """Load a report from picker selection and refresh data."""
        self.report_path = report_info["path"]
        self.report = load_report(self.report_path)
        meta_wd = None
        if self.report:
            meta_wd = self.report.get("meta", {}).get("work_dir")
        if meta_wd and Path(meta_wd).exists():
            self.configs_dir = Path(meta_wd)
        self.all_rows = merge_data(self.report, self.specs, str(self.configs_dir))
        self.rows = list(self.all_rows)
        self.cursor = self.scroll_offset = 0
        self.selected = None
        self.filt_status, self.filt_diff, self.filt_pat = "all", None, ""
        self._apply_sort()

    def render_report_picker(self):
        """Render report selector view."""
        if not self.available_reports:
            self.available_reports = scan_reports(self.reports_dir)
        reports = self.available_reports
        t = Table(show_header=True, header_style="bold cyan", expand=True, padding=(0, 1))
        t.add_column("", width=2, justify="center")
        t.add_column("Report", ratio=3, no_wrap=True)
        t.add_column("Date", width=20)
        t.add_column("Cases", width=7, justify="right")
        t.add_column("Pass%", width=7, justify="right")
        t.add_column("Model", width=12)

        vis = max(5, self.console.size.height - 8)
        if self.report_cursor < self.report_scroll:
            self.report_scroll = self.report_cursor
        elif self.report_cursor >= self.report_scroll + vis:
            self.report_scroll = self.report_cursor - vis + 1

        for i, r in enumerate(reports):
            if i < self.report_scroll or i >= self.report_scroll + vis:
                continue
            sel = i == self.report_cursor
            ts = r["timestamp"][:19] if r["timestamp"] else "—"
            rate_s = f"{r['pass_rate']:.1f}%"
            rate_style = "green" if r["pass_rate"] >= 80 else "yellow" if r["pass_rate"] >= 50 else "red"
            style = "on dark_blue" if sel else ""
            t.add_row(
                "▸" if sel else " ",
                Text(r["filename"], style="bold" if sel else ""),
                ts, str(r["cases"]),
                Text(rate_s, style=rate_style),
                r["model"] or "—",
                style=style)

        return Panel(Layout(t, name="t"),
                     title=f"[bold]Select Report — {len(reports)} available[/bold]",
                     subtitle=HELP_REPORTS, border_style="bright_blue")

    def render_dashboard(self):
        total, passed = len(self.all_rows), sum(1 for r in self.all_rows if r["pass"])
        rate = (passed / total * 100) if total else 0
        fp = []
        if self.filt_status != "all":
            fp.append(f"status={self.filt_status}")
        if self.filt_diff:
            fp.append(f"diff={self.filt_diff}")
        if self.filt_pat:
            fp.append(f"/{self.filt_pat}/")
        filt = ("  │  " + " ".join(fp)) if fp else ""
        sort_s = f"sort={self.sort_key}{'↓' if self.sort_rev else '↑'}"

        width = self._get_term_width()
        narrow = width < 80

        t = Table(show_header=True, header_style="bold cyan", expand=True, show_lines=False, padding=(0, 1))
        t.add_column("", width=2, justify="center")
        t.add_column("Case ID", ratio=3, no_wrap=True)
        t.add_column("Diff", width=7, justify="center")
        t.add_column("Sim", width=7, justify="center")
        t.add_column("Block", width=6, justify="right")
        if not narrow:
            t.add_column("Wire", width=6, justify="right")
            t.add_column("Param", width=6, justify="right")
        t.add_column("Tokens", width=8, justify="right")

        vis = max(5, self.console.size.height - 8)
        if self.cursor < self.scroll_offset:
            self.scroll_offset = self.cursor
        elif self.cursor >= self.scroll_offset + vis:
            self.scroll_offset = self.cursor - vis + 1

        for i, r in enumerate(self.rows):
            if i < self.scroll_offset or i >= self.scroll_offset + vis:
                continue
            sel = i == self.cursor
            dc = DIFF_COLORS.get(r["difficulty"], "white")
            partial = r["sim_pass"] and r["block_f1"] < 0.8
            st = ("on dark_green" if sel else "green") if r["pass"] else \
                 ("on dark_goldenrod" if sel else "yellow") if partial else \
                 ("on dark_red" if sel else "red")
            row_data = [
                "▸" if sel else " ",
                Text(r["case_id"], style="bold" if sel else ""),
                Text(r["difficulty"], style=f"bold {dc}"),
                f"{r['sim_passed']}/{r['sim_total']}",
                f"{r['block_f1']:.0%}",
            ]
            if not narrow:
                row_data.extend([f"{r['wiring_acc']:.0%}", f"{r['param_acc']:.0%}"])
            row_data.append(f"{r['tokens_total']:,}" if r["tokens_total"] else "—")
            t.add_row(*row_data, style=st)

        rname = ""
        if self.report_path:
            rname = f"  │  {Path(self.report_path).name}"
        hdr = f"Eval Dashboard — {passed}/{total} pass ({rate:.1f}%){filt}  │  {sort_s}{rname}"
        return Panel(Layout(t, name="t"), title=f"[bold]{hdr}[/bold]  showing {len(self.rows)}/{total}",
                     subtitle=HELP_DASHBOARD, border_style="bright_blue")

    def render_detail(self):
        r = self.selected
        if not r:
            return Panel("No case selected", border_style="red")
        cid = r["case_id"]
        utt = (r["utterance"][:77] + "…") if len(r["utterance"]) > 80 else r["utterance"]

        # Tab bar
        tabs = []
        for num, label in [(1, "Circuit"), (2, "Conversation"), (3, "Commands")]:
            if num == self.detail_tab:
                tabs.append(Text.assemble(("[", "dim"), (f"{num}", "bold"), (f"] {label}", "bold underline")))
            else:
                tabs.append(Text.assemble(("[", "dim"), (f"{num}", ""), (f"] {label}", "dim")))
        tab_bar = Text("  ").join(tabs)

        if self.detail_tab == 2:
            content = self._render_conversation_tab(r)
        elif self.detail_tab == 3:
            content = self._render_commands_tab(r)
        else:
            content = self._render_circuit_tab(r)

        P = [tab_bar, Text("")]
        P.extend(content)

        # Apply scroll offset — slice visible lines to fit terminal
        try:
            vis_height = os.get_terminal_size().lines - 6  # panel border + title + subtitle + tab bar
        except (ValueError, OSError):
            vis_height = 30
        total_lines = len(P)
        if total_lines > vis_height:
            self.detail_scroll = max(0, min(self.detail_scroll, total_lines - vis_height))
            P = P[self.detail_scroll:self.detail_scroll + vis_height]
            scroll_info = f"  [{self.detail_scroll + 1}–{min(self.detail_scroll + vis_height, total_lines)}/{total_lines}]"
        else:
            self.detail_scroll = 0
            scroll_info = ""

        subtitle = HELP_DETAIL
        if self._status_msg:
            subtitle = Text.assemble(("✓ ", "green"), (self._status_msg, "green bold"), ("  │  ", "dim")) + HELP_DETAIL
            self._status_msg = ""
        return Panel(Text("\n").join(P), title=f"[bold]{cid}[/bold] — {utt}{scroll_info}",
                     subtitle=subtitle, border_style="bright_blue")

    def _render_circuit_tab(self, r):
        """Render the circuit/blocks detail tab."""
        expected = r.get("spec", {}).get("expected", {})
        cid = r["case_id"]
        dc = DIFF_COLORS.get(r["difficulty"], "white")
        ss = "bold green" if r["sim_pass"] else "bold red"
        si = "✓" if r["sim_pass"] else "✗"
        P = []
        P.append(Text.assemble(("Difficulty: ", "dim"), (r["difficulty"], f"bold {dc}"),
                               ("  Patterns: ", "dim"), (", ".join(r["patterns"]) or "—", "")))
        P.append(Text.assemble(("Sim: ", "dim"), (f"{si} {r['sim_passed']}/{r['sim_total']}", ss),
                               ("  Blocks: ", "dim"), (f"{r['block_f1']:.0%}", ""),
                               ("  Wiring: ", "dim"), (f"{r['wiring_acc']:.0%}", ""),
                               ("  Params: ", "dim"), (f"{r['param_acc']:.0%}", "")))
        P.append(Text(""))
        # Expected
        blks, wires = expected.get("new_blocks", []), expected.get("wiring", [])
        if blks or wires:
            P.append(Text("─── Expected ───", style="bold cyan"))
            for i, b in enumerate(blks, 1):
                title = b.get("title") or b.get("title_contains", "")
                rm = f" [{b['room']}]" if b.get("room") else ""
                P.append(Text(f"  {i}. Add {b['type']} '{title}'{rm}"))
            for w in wires:
                s = w.get("from_title") or w.get("from_type", "?")
                d = w.get("to_title") or w.get("to_type", "?")
                dr = f" [{w['to_room']}]" if w.get("to_room") else ""
                P.append(Text(f"  → {s}.{w.get('from_connector','?')} → {d}{dr}.{w.get('to_connector','?')}"))
            P.append(Text(""))
        # Agent-built: try wiring diagram first, then text fallback
        dump = self._get_dump(cid)
        diagram = render_wiring_diagram(dump) if dump else None
        if diagram:
            P.append(Text("─── Agent Built (Wiring Diagram) ───", style="bold green"))
            P.extend(diagram)
            P.append(Text(""))
        elif dump:
            cid_map = {}
            for blk in dump.get("blocks", []):
                for io in ("inputs", "outputs"):
                    for c in blk.get(io, []):
                        rm = f" [{blk['room']}]" if blk.get("room") else ""
                        cid_map[c["cid"]] = f"{blk['name']}{rm}.{c['key']}"

            P.append(Text("─── Agent Built ───", style="bold green"))
            cnt = 0
            for blk in dump.get("blocks", []):
                if blk["type"] in SKIP_TYPES or (not blk["inputs"] and not blk["outputs"]):
                    continue
                has_w = any(i.get("wired_from") for i in blk["inputs"]) or \
                        any(o.get("wired_to") for o in blk["outputs"])
                if not has_w:
                    continue
                rm = f" [{blk['room']}]" if blk.get("room") else ""
                P.append(Text.assemble(("  ", ""), (blk["name"], "bold"), (f" ({blk['type']}){rm}", "dim")))
                unwired = []
                for inp in blk["inputs"]:
                    if inp.get("wired_from"):
                        src = cid_map.get(inp["wired_from"], f"cid:{inp['wired_from']}")
                        P.append(Text(f"    in:  {inp['key']} ← {src}", style="dim"))
                    else:
                        unwired.append(inp["key"])
                if unwired and any(i.get("wired_from") for i in blk["inputs"]):
                    P.append(Text(f"    ⚠ unwired: {', '.join(unwired[:5])}", style="red"))
                elif unwired and not any(i.get("wired_from") for i in blk["inputs"]):
                    P.append(Text("    ⚠ ALL inputs unwired!", style="bold red"))
                for out in blk["outputs"]:
                    for tgt in out.get("wired_to", []):
                        dst = cid_map.get(tgt, f"cid:{tgt}")
                        P.append(Text(f"    out: {out['key']} → {dst}", style="dim"))
                cnt += 1
                if cnt >= 15:
                    P.append(Text("    … and more", style="dim"))
                    break
            P.append(Text(""))
        # Sim results
        scenarios = r.get("simulation_detail", {}).get("scenarios", [])
        if scenarios:
            P.append(Text("─── Simulation Results ───", style="bold yellow"))
            for sc in scenarios:
                ic = "✓" if sc["pass"] else "✗"
                P.append(Text(f"  {ic} {sc['name']}", style="green" if sc["pass"] else "red"))
                for ch in sc.get("checks", []):
                    ci = "✓" if ch["pass"] else "✗"
                    P.append(Text(f"    {ci} {ch['output']}: {ch['actual']}  "
                                  f"(expected {ch['comparator']} {ch['expected']})",
                                  style="green" if ch["pass"] else "red"))
        # Efficiency
        ti = f"{r['tokens_in']:,}" if r["tokens_in"] else "—"
        to = f"{r['tokens_out']:,}" if r["tokens_out"] else "—"
        P.append(Text(""))
        P.append(Text("─── Efficiency ───", style="bold dim"))
        P.append(Text(f"  CLI: {r['cli_invocations']} cmds  Retries: {r['retries']}  "
                      f"Tokens: {r['tokens_total']:,} (in: {ti}, out: {to})"))
        return P

    def _render_conversation_tab(self, r):
        """Render the conversation tab showing LLM messages."""
        P = []
        rc = self._find_report_case(r["case_id"])
        messages = rc.get("conversation", rc.get("messages", [])) if rc else []

        if not messages:
            P.append(Text(""))
            # Check if there's an error message
            error = rc.get("error", "") if rc else ""
            if error:
                P.append(Text("⚠ Case errored during execution:", style="bold red"))
                P.append(Text(f"  {error}", style="red"))
                P.append(Text(""))
            else:
                P.append(Text("Not available — re-run with latest agent to capture.", style="dim italic"))
                P.append(Text(""))
            return P

        turn = 0
        for i, msg in enumerate(messages):
            role = msg.get("role", "unknown")
            content = msg.get("content", "")
            label_text = msg.get("label", "")

            if role in ("user", "human"):
                turn += 1
                turn_label = "initial prompt" if turn == 1 else f"retry {turn - 1}"
                icon, label, style = "🧑", f" USER ({turn_label})", "bold cyan"
            elif role in ("assistant", "ai"):
                icon, label, style = "🤖", f" ASSISTANT (turn {turn})", "bold green"
            else:
                icon, label, style = "📎", f" {role.upper()}", "bold"

            P.append(Text(f"{icon}{label}", style=style))
            lines = content.split("\n") if isinstance(content, str) else [str(content)]
            term_w = self._get_term_width() - 8  # panel borders + padding
            box_w = min(max((len(ln) for ln in lines[:50]), default=40) + 2, term_w)
            P.append(Text(f"  ┌{'─' * box_w}┐"))
            max_lines = 80 if role in ("user", "human") else 40
            for line in lines[:max_lines]:
                padded = line[:box_w].ljust(box_w)
                P.append(Text(f"  │{padded}│", style="dim"))
            if len(lines) > max_lines:
                trunc = f"… {len(lines) - max_lines} more lines"
                P.append(Text(f"  │{trunc.ljust(box_w)}│", style="dim"))
            P.append(Text(f"  └{'─' * box_w}┘"))
            P.append(Text(""))
        return P

    def _render_commands_tab(self, r):
        """Render the commands tab showing CLI invocations."""
        P = []
        rc = self._find_report_case(r["case_id"])
        commands = rc.get("commands", rc.get("cli_commands", [])) if rc else []

        if not commands:
            P.append(Text(""))
            P.append(Text("Not available — re-run with latest agent to capture.", style="dim italic"))
            P.append(Text(""))
            P.append(Text("The commands log requires the eval agent to record", style="dim"))
            P.append(Text("CLI invocations in the report JSON under each case's", style="dim"))
            P.append(Text("'commands' or 'cli_commands' key.", style="dim"))
            return P

        total = len(commands)
        retries = sum(1 for i, c in enumerate(commands)
                      if i > 0 and "config check" in c.get("command", c.get("cmd", "")))
        P.append(Text(f"Commands ({total} total)", style="bold"))
        P.append(Text(""))
        for cmd in commands:
            cmd_str = cmd.get("command", cmd.get("cmd", ""))
            exit_code = cmd.get("exit_code", cmd.get("returncode", "?"))
            stdout = cmd.get("output", cmd.get("stdout", ""))
            stderr = cmd.get("stderr", "")
            P.append(Text(f"  $ {cmd_str}", style="bold"))
            if exit_code == 0:
                summary = stdout.split("\n")[0][:80] if stdout else ""
                P.append(Text(f"    → exit 0: ✓ {summary}", style="green"))
            else:
                # Show stderr on failure (that's where the actual error is)
                err_msg = stderr.split("\n")[0][:80] if stderr else ""
                out_msg = stdout.split("\n")[0][:80] if stdout else ""
                msg = err_msg or out_msg or "(no output)"
                P.append(Text(f"    → exit {exit_code}: ✗ {msg}", style="red"))
            P.append(Text(""))
        return P

    def _find_report_case(self, case_id):
        """Find the raw report case dict for a given case_id."""
        if not self.report or "cases" not in self.report:
            return None
        for rc in self.report["cases"]:
            if rc.get("case_id") == case_id:
                return rc
        return None

    def render_sim(self):
        r = self.selected
        if not r:
            return Panel("No case", border_style="red")
        body = Text(self.sim_output) if self.sim_output else Text("Running simulation…", style="bold yellow")
        nav = Text.assemble(("[Esc]", "bold"), " Back  ", ("[q]", "bold"), " Quit")
        return Panel(body, title=f"[bold]Sim Re-run: {r['case_id']}[/bold]",
                     subtitle=nav, border_style="bright_yellow")

    def render_help(self):
        """Render a help overlay showing all keybindings for the previous view."""
        view = self._help_from or "dashboard"
        bindings = HELP_FULL.get(view, HELP_FULL["dashboard"])
        t = Table(show_header=False, box=None, padding=(0, 2))
        t.add_column("Key", style="bold cyan", min_width=12)
        t.add_column("Action")
        for key, desc in bindings:
            t.add_row(key, desc)
        subtitle = Text.assemble(("Press any key to close", "dim"))
        return Panel(t, title=f"[bold]Help — {view}[/bold]",
                     subtitle=subtitle, border_style="bright_cyan")

    def _run_trace(self, case_id):
        """Run sim with trace=true for each scenario, return structured trace data."""
        spec = self.selected.get("spec", {})
        sim_specs = spec.get("expected", {}).get("simulation", [])
        cfg = self.configs_dir / f"{case_id}.Loxone"
        if not cfg.exists():
            return None
        if not sim_specs:
            return None

        dump = self._get_dump(case_id)

        scenarios = []
        for sc in sim_specs:
            traced = dict(sc, trace=True)
            steps = sc.get("steps", [sc] if "inputs" in sc else [])
            scenario_result = {"name": sc.get("name", "unnamed"), "steps": [],
                               "raw_output": "", "dump": dump}
            try:
                res = subprocess.run(
                    [str(LOX_BIN), "sim", "run", str(cfg), "--sim", json.dumps(traced)],
                    capture_output=True, text=True, timeout=60,
                )
                scenario_result["raw_output"] = res.stdout

                parsed = None
                for ln in res.stdout.splitlines():
                    ln = ln.strip()
                    if ln.startswith("{"):
                        try:
                            parsed = json.loads(ln)
                            break
                        except json.JSONDecodeError:
                            pass

                parsed_scenarios = parsed.get("scenarios", []) if parsed else []
                parsed_sc = parsed_scenarios[0] if parsed_scenarios else {}
                traces = parsed_sc.get("traces", [])
                checks = parsed_sc.get("checks", [])

                for i, step in enumerate(steps):
                    step_data = {
                        "index": i,
                        "inputs": step.get("inputs", {}),
                        "ticks": step.get("ticks", 10),
                        "dt": step.get("dt", 0.1),
                        "signals": {},
                        "checks": [],
                    }
                    if i < len(traces):
                        step_data["signals"] = traces[i] if isinstance(traces[i], dict) else {}
                    # Attach checks to the last step
                    if i == len(steps) - 1:
                        step_data["checks"] = checks
                    scenario_result["steps"].append(step_data)

                # If no steps were parsed but we have trace data, create a single step
                if not scenario_result["steps"] and traces:
                    step_data = {
                        "index": 0,
                        "inputs": sc.get("inputs", {}),
                        "ticks": sc.get("ticks", 10),
                        "dt": sc.get("dt", 0.1),
                        "signals": traces[0] if isinstance(traces[0], dict) else {},
                        "checks": checks,
                    }
                    scenario_result["steps"].append(step_data)
                elif not scenario_result["steps"]:
                    # Fallback: show raw step info even without trace
                    for i, step in enumerate(steps):
                        step_data = {
                            "index": i,
                            "inputs": step.get("inputs", {}),
                            "ticks": step.get("ticks", 10),
                            "dt": step.get("dt", 0.1),
                            "signals": {},
                            "checks": checks if i == len(steps) - 1 else [],
                        }
                        scenario_result["steps"].append(step_data)

            except subprocess.TimeoutExpired:
                scenario_result["steps"] = [{"index": 0, "inputs": {}, "ticks": 0, "dt": 0,
                                             "signals": {}, "checks": [],
                                             "error": "Simulation timed out"}]
            except Exception as e:
                scenario_result["steps"] = [{"index": 0, "inputs": {}, "ticks": 0, "dt": 0,
                                             "signals": {}, "checks": [],
                                             "error": str(e)}]
            scenarios.append(scenario_result)

        return scenarios if scenarios else None

    def render_trace(self):
        """Render the debugger-style block inspector view."""
        r = self.selected
        if not r:
            return Panel("No case selected", border_style="red")
        cid = r["case_id"]

        if self.trace_data is None:
            return Panel(Text("Running sim with trace…", style="bold yellow"),
                         title=f"[bold]Sim Debugger: {cid}[/bold]",
                         border_style="bright_magenta")

        if not self.trace_data:
            return Panel(Text("No simulation specs or config not found.", style="red"),
                         title=f"[bold]Sim Debugger: {cid}[/bold]",
                         border_style="red")

        sc_idx = max(0, min(self.trace_scenario_idx, len(self.trace_data) - 1))
        sc = self.trace_data[sc_idx]
        sc_name = sc.get("name", "unnamed")
        steps = sc.get("steps", [])
        dump = sc.get("dump")

        if not steps:
            return Panel(Text("No steps in scenario.", style="dim"),
                         title=f"[bold]Sim Debugger: {cid}[/bold]  │  {sc_name}",
                         subtitle=self._trace_nav_bar(), border_style="bright_magenta")

        step_idx = max(0, min(self.trace_step_idx, len(steps) - 1))
        step = steps[step_idx]

        ticks = step.get("ticks", 10)
        dt = step.get("dt", 0.1)
        total_t = ticks * dt

        P = []
        # Scenario info (if multiple)
        if len(self.trace_data) > 1:
            P.append(Text(f"Scenario {sc_idx + 1}/{len(self.trace_data)}: {sc_name}",
                          style="bold"))
            P.append(Text(""))

        # Timeline bar showing all steps with current highlighted
        if len(steps) > 1:
            timeline_parts = []
            cumulative_t = 0.0
            for si, s in enumerate(steps):
                s_ticks = s.get("ticks", 10)
                s_dt = s.get("dt", 0.1)
                s_total = s_ticks * s_dt
                # Build label
                inj_keys = list(s.get("inputs", {}).keys())
                label = f"t={cumulative_t:.0f}s"
                if inj_keys:
                    short = inj_keys[0].split(".")[-1][:8]
                    label += f" {short}={s.get('inputs',{}).get(inj_keys[0],'')}"
                has_check = bool(s.get("expected_outputs", s.get("checks")))
                if has_check:
                    label += " ✓?"

                if si == step_idx:
                    timeline_parts.append(("▶ ", "bold yellow"))
                    timeline_parts.append((f"[{label}]", "bold yellow underline"))
                else:
                    timeline_parts.append(("  ", ""))
                    timeline_parts.append((f"[{label}]", "dim"))
                timeline_parts.append((" ─── ", "dim"))
                cumulative_t += s_total
            P.append(Text.assemble(*timeline_parts))
            P.append(Text(""))

        if step.get("error"):
            P.append(Text(f"  ⚠ {step['error']}", style="bold red"))
            P.append(Text(""))
        else:
            # Inject line
            inputs = step.get("inputs", {})
            if inputs:
                inj = ", ".join(f"{k}={v}" for k, v in inputs.items())
                P.append(Text(f" ── Inject: {inj} ──", style="bold cyan"))
                P.append(Text(""))

            signals = step.get("signals", {})
            checks = step.get("checks", [])
            injected_keys = set(inputs.keys())

            # Build block states from dump + signals
            block_states = _build_block_states(dump, signals, checks, injected_keys, inputs)

            if block_states:
                self._render_block_boxes(P, block_states)
            else:
                # Fallback: flat signal list when no dump available
                self._render_flat_signals(P, signals, checks)

            # Checks summary at bottom
            if checks:
                P.append(Text(""))
                for ch in checks:
                    icon = "✓" if ch.get("pass") else "✗"
                    style = "green" if ch.get("pass") else "red"
                    out = ch.get("output", "?")
                    actual = ch.get("actual", "?")
                    comp = ch.get("comparator", "?")
                    exp = ch.get("expected", "?")
                    P.append(Text(f" ── Check: {icon} {out} = {actual} (expected {comp} {exp}) ──",
                                  style=style))

        # Scrolling
        try:
            vis_height = os.get_terminal_size().lines - 6
        except (ValueError, OSError):
            vis_height = 30
        total_lines = len(P)
        scroll_info = ""
        if total_lines > vis_height:
            self.trace_scroll = max(0, min(self.trace_scroll, total_lines - vis_height))
            P = P[self.trace_scroll:self.trace_scroll + vis_height]
            scroll_info = f"  [{self.trace_scroll + 1}–{min(self.trace_scroll + vis_height, total_lines)}/{total_lines}]"
        else:
            self.trace_scroll = 0

        step_label = f"Step {step_idx + 1}/{len(steps)}  Tick {ticks}/{ticks}  t={total_t:.1f}s"
        return Panel(Text("\n").join(P),
                     title=f"[bold]Sim Debugger: {cid}[/bold]  │  {step_label}{scroll_info}",
                     subtitle=self._trace_nav_bar(), border_style="bright_magenta")

    def _trace_nav_bar(self):
        return Text.assemble(
            ("[←→]", "bold"), " Step  ",
            ("[\\[\\]]", "bold"), " Scenario  ",
            ("[↑↓]", "bold"), " Scroll  ",
            ("[Esc]", "bold"), " Back")

    def _render_block_boxes(self, P, block_states):
        """Render block-centric debugger boxes with wiring arrows."""
        for pos, bs in enumerate(block_states):
            # Arrow between blocks
            if pos > 0:
                P.append(Text("                          │", style="dim"))
                P.append(Text("                          ▼", style="dim"))

            # Block header
            room_str = f" [{bs.room}]" if bs.room else ""
            header = f" {bs.name} ({bs.block_type}){room_str} "
            box_w = max(len(header) + 2, 40)
            pad = " "

            # Check if any output is checked
            has_check = bool(bs.checked)
            check_label = ""
            if has_check:
                all_pass = all(ch.get("pass", False) for ch in bs.checked.values())
                check_label = " ✓ CHECK" if all_pass else " ✗ CHECK"

            P.append(Text(f"{pad}┌─{header}{'─' * max(0, box_w - len(header) - 2)}┐"))

            # Inputs
            if bs.inputs or bs.sources:
                all_keys = sorted(set(list(bs.inputs.keys()) + list(bs.sources.keys())))
                for k in all_keys:
                    val = bs.inputs.get(k, 0.0)
                    val_str = f"{val:.1f}" if isinstance(val, float) else str(val)
                    src = bs.sources.get(k, "")
                    if bs.injected and not src:
                        src_str = "(injected)"
                    elif src:
                        src_str = f"← {src}"
                    else:
                        src_str = ""
                    line = f"  in:  {k} = {val_str}  {src_str}"
                    line_padded = line.ljust(box_w)[:box_w]
                    P.append(Text(f"{pad}│{line_padded}│"))
            elif bs.injected:
                # Show injected values from outputs
                if bs.outputs:
                    for k, v in sorted(bs.outputs.items()):
                        val_str = f"{v:.1f}" if isinstance(v, float) else str(v)
                        line = f"  in:  {k} = {val_str}  (injected)".ljust(box_w)[:box_w]
                        P.append(Text(f"{pad}│{line}│", style="bold cyan"))
                else:
                    line = "  in:  (injected)".ljust(box_w)[:box_w]
                    P.append(Text(f"{pad}│{line}│"))

            # Outputs
            for k, v in sorted(bs.outputs.items()):
                val_str = f"{v:.1f}" if isinstance(v, float) else str(v)
                tgt = bs.targets.get(k, "")
                tgt_str = f"──→  {tgt}" if tgt else ""
                chk = ""
                if k in bs.checked:
                    ch = bs.checked[k]
                    chk = " ✓" if ch.get("pass", False) else " ✗"
                line = f"  out: {k} = {val_str}  {tgt_str}{chk}"
                line_padded = line.ljust(box_w)[:box_w]
                if k in bs.checked:
                    check_ann = check_label
                    P.append(Text.assemble(
                        (f"{pad}│{line_padded}│", ""),
                        (check_ann, "bold green" if "✓" in check_ann else "bold red")))
                else:
                    P.append(Text(f"{pad}│{line_padded}│"))

            # If no inputs and no outputs shown
            if not bs.inputs and not bs.sources and not bs.outputs and not bs.injected:
                line = "  (no signals)".ljust(box_w)[:box_w]
                P.append(Text(f"{pad}│{line}│", style="dim"))

            P.append(Text(f"{pad}└{'─' * box_w}┘"))

    def _render_flat_signals(self, P, signals, checks):
        """Fallback: flat signal list when dump is unavailable."""
        non_zero = {k: v for k, v in signals.items() if v != 0 and v != 0.0}
        if non_zero:
            P.append(Text("  Signals (non-zero):", style="dim"))
            check_outputs = {ch.get("output", ""): ch for ch in checks}
            max_key_len = max((len(k) for k in non_zero), default=0)
            for k, v in sorted(non_zero.items()):
                val_str = f"{v:>8.1f}" if isinstance(v, float) else f"{v!s:>8}"
                annotation = ""
                for co in check_outputs:
                    if co and co in k:
                        annotation = "  ← target"
                        break
                P.append(Text(f"    {k:<{max_key_len}}  {val_str}{annotation}",
                              style="bold" if annotation else ""))
        else:
            P.append(Text("  Signals: (all zero)", style="dim"))
        P.append(Text(""))

    def _get_dump(self, case_id):
        if case_id in self._dump_cache:
            return self._dump_cache[case_id]
        cfg = self.configs_dir / f"{case_id}.Loxone"
        if not cfg.exists():
            return None
        try:
            out = subprocess.run([str(LOX_BIN), "sim", "dump", str(cfg), "--json"],
                                capture_output=True, text=True, timeout=10)
            self._dump_cache[case_id] = data = json.loads(out.stdout)
            return data
        except Exception:
            return None

    def _run_sim(self, case_id):
        spec = self.selected.get("spec", {})
        sim_specs = spec.get("expected", {}).get("simulation", [])
        cfg = self.configs_dir / f"{case_id}.Loxone"
        if not cfg.exists():
            return f"Config not found: {cfg}"
        if not sim_specs:
            return "No simulation specs defined for this case."
        lines = []
        for sc in sim_specs:
            try:
                res = subprocess.run([str(LOX_BIN), "sim", "run", str(cfg), "--sim", json.dumps(sc)],
                                     capture_output=True, text=True, timeout=30)
                # Filter out harmless structural type warnings
                _NOISE = {"VirtualInCaption", "WeatherServer", "LightscenesC",
                          "LightsceneC", "TreeDevice", "LoxAIRDevice"}
                for w in [line for line in res.stderr.splitlines()
                          if (line.startswith("WARNING:") or line.startswith("warning:"))
                          and not any(n in line for n in _NOISE)][:3]:
                    lines.append(f"  ⚠ {w}")
                for ln in res.stdout.splitlines():
                    ln = ln.strip()
                    if not ln or not ln.startswith("{"):
                        continue
                    try:
                        d = json.loads(ln)
                        for s in d.get("scenarios", []):
                            lines.append(f"  Scenario: {s['name']}")
                            for ch in s.get("checks", []):
                                ci = "✓" if ch["pass"] else "✗"
                                lines.append(f"    {ci} {ch['output']}: {ch['actual']} "
                                             f"(expected {ch['comparator']} {ch['expected']})")
                    except json.JSONDecodeError:
                        lines.append(f"  {ln}")
                steps = sc.get("steps", [sc] if "inputs" in sc else [])
                for i, step in enumerate(steps):
                    inp = ", ".join(f"{k}={v}" for k, v in step.get("inputs", {}).items())
                    lines.append(f"  Step {i}: {inp}  ({step.get('ticks',10)} ticks × {step.get('dt',0.1)}s)")
                lines.append("")
            except subprocess.TimeoutExpired:
                lines.append(f"  ⚠ Timeout for '{sc.get('name', '?')}'")
            except Exception as e:
                lines.append(f"  ⚠ Error: {e}")
        return "\n".join(lines) or "No simulation output."

    def _jump_fail(self, d):
        n = len(self.rows)
        for i in range(1, n):
            idx = (self.cursor + i * d) % n
            if not self.rows[idx]["pass"]:
                self.cursor = idx
                return

    def _copy_to_clipboard(self, text):
        """Copy text to clipboard via OSC 52 escape sequence (works in most terminals)."""
        import base64
        encoded = base64.b64encode(text.encode()).decode()
        sys.stdout.write(f"\033]52;c;{encoded}\a")
        sys.stdout.flush()

    def _copy_case_summary(self):
        """Build a shareable summary of the current case and copy to clipboard."""
        r = self.selected
        if not r:
            return
        cid = r["case_id"]
        report_name = Path(self.report_path).name if self.report_path else "live-sim"
        run_id = ""
        if self.report:
            run_id = self.report.get("meta", {}).get("run_id", "")

        lines = [
            f"## Eval Case: {cid}",
            f"Run: {run_id or report_name}",
            f"Utterance: {r.get('utterance', '')}",
            f"Difficulty: {r.get('difficulty', '?')}",
            f"Result: {'PASS' if r['pass'] else 'FAIL'}  Sim: {r['sim_passed']}/{r['sim_total']}",
            f"Block F1: {r['block_f1']:.0%}  Wiring: {r['wiring_acc']:.0%}  Params: {r['param_acc']:.0%}",
        ]

        # Add sim check details
        scenarios = r.get("simulation_detail", {}).get("scenarios", [])
        if scenarios:
            lines.append("")
            for sc in scenarios:
                icon = "✓" if sc.get("pass") else "✗"
                lines.append(f"{icon} {sc.get('name', '?')}")
                for ch in sc.get("checks", []):
                    ci = "✓" if ch.get("pass") else "✗"
                    lines.append(f"  {ci} {ch.get('output','?')}: {ch.get('actual','?')} "
                                 f"(expected {ch.get('comparator','?')} {ch.get('expected','?')})")

        # Add hint if present
        hint = r.get("spec", {}).get("hint", "")
        if hint:
            lines.append(f"\nHint: {hint[:200]}")

        text = "\n".join(lines)
        self._copy_to_clipboard(text)
        self._status_msg = f"Copied {cid} to clipboard"

    def handle_key(self, key, live):
        # Help overlay: any key dismisses it
        if self.view == "help":
            self.view = self._help_from or "dashboard"
            return True
        # ? or h shows help from current view
        if key in ("?", "h") and self.view != "help":
            self._help_from = self.view
            self.view = "help"
            return True
        # q only quits from dashboard/report picker; elsewhere it goes back
        if key == "q":
            if self.view in ("dashboard", "report_picker"):
                return False
            elif self.view == "detail":
                self.view = "dashboard"
            elif self.view in ("sim_rerun", "sim_trace"):
                self.view = "detail"
            return True
        if self.view == "report_picker":
            if key == "up" and self.report_cursor > 0:
                self.report_cursor -= 1
            elif key == "down" and self.report_cursor < len(self.available_reports) - 1:
                self.report_cursor += 1
            elif key in ("\r", "\n") and self.available_reports:
                self._load_report(self.available_reports[self.report_cursor])
                self.view = "dashboard"
            elif key in ("\x1b", "esc"):
                if self.all_rows:
                    self.view = "dashboard"
            elif key == "G":
                self.report_cursor = max(0, len(self.available_reports) - 1)
            elif key == "g":
                self.report_cursor = 0
        elif self.view == "dashboard":
            if key == "up" and self.cursor > 0:
                self.cursor -= 1
            elif key == "down" and self.cursor < len(self.rows) - 1:
                self.cursor += 1
            elif key in ("\r", "\n") and self.rows:
                self.selected = self.rows[self.cursor]
                self.detail_tab = 1
                self.view = "detail"
            elif key == "s":
                self.sort_key = SORT_KEYS[(SORT_KEYS.index(self.sort_key) + 1) % len(SORT_KEYS)]
                self._apply_sort()
            elif key == "r":
                self.sort_rev = not self.sort_rev
                self._apply_sort()
            elif key == "R":
                self.available_reports = scan_reports(self.reports_dir)
                self.report_cursor = 0
                self.report_scroll = 0
                self.view = "report_picker"
            elif key == "f":
                c = ["all", "pass", "fail"]
                self.filt_status = c[(c.index(self.filt_status) + 1) % 3]
                self._apply_filters()
            elif key == "d":
                ds = [None, "easy", "medium", "hard", "expert"]
                if self.filt_diff in ds:
                    self.filt_diff = ds[(ds.index(self.filt_diff) + 1) % 5]
                else:
                    self.filt_diff = ds[1]
                self._apply_filters()
            elif key == "/":
                live.stop()
                try:
                    self.filt_pat = input("Filter pattern (empty=clear): ").strip()
                except (EOFError, KeyboardInterrupt):
                    self.filt_pat = ""
                live.start()
                self._apply_filters()
            elif key == "G":
                self.cursor = max(0, len(self.rows) - 1)
            elif key == "g":
                self.cursor = 0
        elif self.view == "detail":
            if key in ("\x1b", "esc"):
                self.view = "dashboard"
            elif key == "up":
                self.detail_scroll = max(0, self.detail_scroll - 1)
            elif key == "down":
                self.detail_scroll += 1
            elif key == "page_up":
                try:
                    page_size = os.get_terminal_size().lines - 6
                except (ValueError, OSError):
                    page_size = 24
                self.detail_scroll = max(0, self.detail_scroll - page_size)
            elif key == "page_down":
                try:
                    page_size = os.get_terminal_size().lines - 6
                except (ValueError, OSError):
                    page_size = 24
                self.detail_scroll += page_size
            elif key == "n" and self.rows:
                self._jump_fail(1)
                self.selected = self.rows[self.cursor]
                self.detail_scroll = 0
            elif key == "p" and self.rows:
                self._jump_fail(-1)
                self.selected = self.rows[self.cursor]
                self.detail_scroll = 0
            elif key == "r":
                self.view = "sim_rerun"
                self.sim_output = ""
            elif key == "t":
                self.trace_data = None
                self.trace_scenario_idx = 0
                self.trace_step_idx = 0
                self.trace_scroll = 0
                self.view = "sim_trace"
            elif key in ("1", "2", "3"):
                self.detail_tab = int(key)
                self.detail_scroll = 0
            elif key == "g":
                self.detail_scroll = 0
            elif key == "G":
                self.detail_scroll = 9999
            elif key == "c":
                self._copy_case_summary()
        elif self.view == "sim_rerun":
            if key in ("\x1b", "esc"):
                self.view = "detail"
        elif self.view == "sim_trace":
            if key in ("\x1b", "esc"):
                self.view = "detail"
            elif key == "up":
                self.trace_scroll = max(0, self.trace_scroll - 1)
            elif key == "down":
                self.trace_scroll += 1
            elif key == "page_up":
                try:
                    page_size = os.get_terminal_size().lines - 6
                except (ValueError, OSError):
                    page_size = 24
                self.trace_scroll = max(0, self.trace_scroll - page_size)
            elif key == "page_down":
                try:
                    page_size = os.get_terminal_size().lines - 6
                except (ValueError, OSError):
                    page_size = 24
                self.trace_scroll += page_size
            elif key == "left" and self.trace_data:
                # Step back within current scenario
                if self.trace_step_idx > 0:
                    self.trace_step_idx -= 1
                    self.trace_scroll = 0
            elif key == "right" and self.trace_data:
                # Step forward within current scenario
                sc_idx = max(0, min(self.trace_scenario_idx, len(self.trace_data) - 1))
                n_steps = len(self.trace_data[sc_idx].get("steps", []))
                if self.trace_step_idx < n_steps - 1:
                    self.trace_step_idx += 1
                    self.trace_scroll = 0
            elif key == "[" and self.trace_data:
                # Previous scenario
                if self.trace_scenario_idx > 0:
                    self.trace_scenario_idx -= 1
                    self.trace_step_idx = 0
                    self.trace_scroll = 0
            elif key == "]" and self.trace_data:
                # Next scenario
                if self.trace_scenario_idx < len(self.trace_data) - 1:
                    self.trace_scenario_idx += 1
                    self.trace_step_idx = 0
                    self.trace_scroll = 0
            elif key == "g":
                self.trace_scroll = 0
            elif key == "G":
                self.trace_scroll = 9999
        return True

    def run(self):
        if not self.rows:
            self.console.print("[red]No data found. Check --report and --cases paths.[/red]")
            return
        self.console.print(f"[dim]Loaded {len(self.all_rows)} cases. Starting TUI…[/dim]")
        fd = sys.stdin.fileno()
        old_term = termios.tcgetattr(fd)
        try:
            with Live(self.render_dashboard(), console=self.console, screen=True, auto_refresh=False) as live:
                while True:
                    try:
                        render = {"dashboard": self.render_dashboard, "detail": self.render_detail,
                                  "sim_rerun": self.render_sim, "report_picker": self.render_report_picker,
                                  "help": self.render_help, "sim_trace": self.render_trace}
                        if self.view == "sim_rerun" and not self.sim_output:
                            live.update(self.render_sim(), refresh=True)
                            self.sim_output = self._run_sim(self.selected["case_id"])
                        if self.view == "sim_trace" and self.trace_data is None:
                            live.update(self.render_trace(), refresh=True)
                            self.trace_data = self._run_trace(self.selected["case_id"]) or []
                        live.update(render[self.view](), refresh=True)
                        if not self.handle_key(readkey(), live):
                            break
                    except KeyboardInterrupt:
                        break
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_term)


def main():
    p = argparse.ArgumentParser(description="Loxone Eval TUI — inspect evaluation results")
    p.add_argument("--report", default=None, help="Path to report JSON (default: live sim on saved configs)")
    p.add_argument("--cases", default=str(DEFAULT_CASES), help="Path to eval cases directory")
    p.add_argument("--configs", default=str(DEFAULT_CONFIGS), help="Path to saved .Loxone configs")
    a = p.parse_args()
    EvalTUI(a.report, a.cases, a.configs).run()


if __name__ == "__main__":
    main()
