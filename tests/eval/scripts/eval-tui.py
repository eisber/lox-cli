#!/usr/bin/env python3
"""Beautiful terminal UI for inspecting Loxone eval results."""
import argparse
import json
import os
import shutil
import subprocess
import sys
import termios
import tty
from collections import defaultdict
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
    ("[1-3]", "bold"), " Tabs  ", ("[n/p]", "bold"), " Next/Prev fail  ",
    ("[c]", "bold"), " Copy  ", ("[r]", "bold"), " Re-run sim  ",
    ("[Esc]", "bold"), " Back  ", ("[q]", "bold"), " Quit")

HELP_REPORTS = Text.assemble(
    ("[↑↓]", "bold"), " Navigate  ", ("[Enter]", "bold"), " Select  ",
    ("[Esc]", "bold"), " Back  ", ("[q]", "bold"), " Quit")


def readkey():
    fd = sys.stdin.fileno()
    old = termios.tcgetattr(fd)
    try:
        tty.setraw(fd)
        ch = sys.stdin.read(1)
        if ch == "\x1b":
            ch2 = sys.stdin.read(1)
            if ch2 == "[":
                return {"A": "up", "B": "down", "C": "right", "D": "left"}.get(sys.stdin.read(1), "")
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
        subtitle = HELP_DETAIL
        if self._status_msg:
            subtitle = Text.assemble(("✓ ", "green"), (self._status_msg, "green bold"), ("  ", ""), *HELP_DETAIL._spans) if hasattr(HELP_DETAIL, '_spans') else HELP_DETAIL
            subtitle = Text.assemble(("✓ ", "green"), (self._status_msg, "green bold"), ("  │  ", "dim")) + HELP_DETAIL
            self._status_msg = ""
        return Panel(Text("\n").join(P), title=f"[bold]{cid}[/bold] — {utt}",
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
            P.append(Text("Not available — re-run with latest agent to capture.", style="dim italic"))
            P.append(Text(""))
            P.append(Text("The conversation log requires the eval agent to record", style="dim"))
            P.append(Text("messages in the report JSON under each case's", style="dim"))
            P.append(Text("'conversation' or 'messages' key.", style="dim"))
            return P

        for msg in messages:
            role = msg.get("role", "unknown")
            content = msg.get("content", "")
            label_text = msg.get("label", "")

            if role in ("user", "human"):
                icon, label, style = "🧑", f" USER{f' ({label_text})' if label_text else ''}", "bold cyan"
            elif role in ("assistant", "ai"):
                icon, label, style = "🤖", " ASSISTANT", "bold green"
            else:
                icon, label, style = "📎", f" {role.upper()}", "bold"

            P.append(Text(f"{icon}{label}", style=style))
            lines = content.split("\n") if isinstance(content, str) else [str(content)]
            box_w = min(max((len(ln) for ln in lines), default=40) + 2, 60)
            P.append(Text(f"  ┌{'─' * box_w}┐"))
            for line in lines[:20]:
                padded = line[:box_w].ljust(box_w)
                P.append(Text(f"  │{padded}│", style="dim"))
            if len(lines) > 20:
                trunc = f"… {len(lines) - 20} more lines"
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
        retries = sum(1 for c in commands if c.get("retry", False))
        P.append(Text(f"Commands ({total} total, {retries} retries)", style="bold"))
        P.append(Text(""))
        for cmd in commands:
            cmd_str = cmd.get("command", cmd.get("cmd", ""))
            exit_code = cmd.get("exit_code", cmd.get("returncode", "?"))
            output = cmd.get("output", cmd.get("stdout", ""))
            is_retry = cmd.get("retry", False)
            prefix = "  ↻ $" if is_retry else "  $"
            P.append(Text(f"{prefix} {cmd_str}", style="bold" if not is_retry else "yellow"))
            if exit_code == 0:
                summary = output.split("\n")[0][:60] if output else ""
                P.append(Text(f"    → exit 0: ✓ {summary}", style="green"))
            else:
                summary = output.split("\n")[0][:60] if output else ""
                P.append(Text(f"    → exit {exit_code}: ✗ {summary}", style="red"))
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
        if key == "q":
            return False
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
            elif key == "n" and self.rows:
                self._jump_fail(1)
                self.selected = self.rows[self.cursor]
            elif key == "p" and self.rows:
                self._jump_fail(-1)
                self.selected = self.rows[self.cursor]
            elif key == "r":
                self.view = "sim_rerun"
                self.sim_output = ""
            elif key in ("1", "2", "3"):
                self.detail_tab = int(key)
            elif key == "c":
                self._copy_case_summary()
        elif self.view == "sim_rerun":
            if key in ("\x1b", "esc"):
                self.view = "detail"
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
                                  "sim_rerun": self.render_sim, "report_picker": self.render_report_picker}
                        if self.view == "sim_rerun" and not self.sim_output:
                            live.update(self.render_sim(), refresh=True)
                            self.sim_output = self._run_sim(self.selected["case_id"])
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
