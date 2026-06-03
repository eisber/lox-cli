#!/usr/bin/env python3
"""Beautiful terminal UI for inspecting Loxone eval results."""
import argparse
import json
import shutil
import subprocess
import sys
import termios
import tty
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
LOX_BIN = REPO / "target/release/lox"
DIFF_COLORS = {"easy": "green", "medium": "yellow", "hard": "red", "expert": "magenta"}
SORT_KEYS = ["case_id", "difficulty", "sim", "block_f1", "wiring", "tokens"]
DIFF_ORDER = {"easy": 0, "medium": 1, "hard": 2, "expert": 3}


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
    lox_sim = None
    for candidate in ["./target/release/lox-sim", "lox-sim"]:
        if Path(candidate).exists() or shutil.which(candidate):
            lox_sim = candidate
            break

    for cid, spec in sorted(specs.items()):
        config = configs_path / f"{cid}.Loxone"
        rc = {"case_id": cid}
        if config.exists() and lox_sim:
            sims = spec.get("expected", {}).get("simulation", [])
            if sims:
                try:
                    r = subprocess.run(
                        [lox_sim, "run", str(config), "--sim", json.dumps(sims)],
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


SKIP_TYPES = {"VirtualInCaption", "WeatherServer", "LightscenesC", "LightsceneC"}


class EvalTUI:
    _dump_cache = {}

    def __init__(self, report_path, cases_dir, configs_dir):
        self.console = Console(highlight=False)
        self.specs = load_cases(cases_dir)
        self.report = load_report(report_path)
        self.configs_dir = Path(configs_dir) if configs_dir else DEFAULT_CONFIGS
        self.all_rows = merge_data(self.report, self.specs, configs_dir)
        self.rows = list(self.all_rows)
        self.view = "dashboard"
        self.cursor = self.scroll_offset = 0
        self.selected = None
        self.sort_key, self.sort_rev = "case_id", False
        self.filt_status, self.filt_diff, self.filt_pat = "all", None, ""
        self.sim_output = ""
        self._apply_sort()

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

        t = Table(show_header=True, header_style="bold cyan", expand=True, show_lines=False, padding=(0, 1))
        t.add_column("", width=2, justify="center")
        t.add_column("Case ID", ratio=3, no_wrap=True)
        t.add_column("Diff", width=7, justify="center")
        t.add_column("Sim", width=7, justify="center")
        for c in ("Block", "Wire", "Param"):
            t.add_column(c, width=6, justify="right")
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
            t.add_row(
                "▸" if sel else " ",
                Text(r["case_id"], style="bold" if sel else ""),
                Text(r["difficulty"], style=f"bold {dc}"),
                f"{r['sim_passed']}/{r['sim_total']}",
                f"{r['block_f1']:.0%}", f"{r['wiring_acc']:.0%}", f"{r['param_acc']:.0%}",
                f"{r['tokens_total']:,}" if r["tokens_total"] else "—",
                style=st)

        nav = Text.assemble(("  [↑↓]", "bold"), " Navigate  ", ("[Enter]", "bold"), " Detail  ",
                            ("[f]", "bold"), " Filter  ", ("[s]", "bold"), " Sort  ",
                            ("[r]", "bold"), " Reverse  ", ("[q]", "bold"), " Quit")
        hdr = f"Eval Dashboard — {passed}/{total} pass ({rate:.1f}%){filt}  │  {sort_s}"
        return Panel(Layout(t, name="t"), title=f"[bold]{hdr}[/bold]  showing {len(self.rows)}/{total}",
                     subtitle=nav, border_style="bright_blue")

    def render_detail(self):
        r = self.selected
        if not r:
            return Panel("No case selected", border_style="red")
        expected = r.get("spec", {}).get("expected", {})
        cid = r["case_id"]
        utt = (r["utterance"][:77] + "…") if len(r["utterance"]) > 80 else r["utterance"]
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
        # Agent-built
        dump = self._get_dump(cid)
        if dump:
            # Build cid→name map for readable wiring
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
                # Show unwired only for blocks that have SOME wiring (partial = interesting)
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
        nav = Text.assemble(("[Esc]", "bold"), " Back  ", ("[n]", "bold"), " Next fail  ",
                            ("[p]", "bold"), " Prev fail  ", ("[r]", "bold"), " Re-run sim  ",
                            ("[q]", "bold"), " Quit")
        return Panel(Text("\n").join(P), title=f"[bold]{cid}[/bold] — {utt}",
                     subtitle=nav, border_style="bright_blue")

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
                for w in [line for line in res.stderr.splitlines()
                          if line.startswith("WARNING:") or line.startswith("warning:")][:3]:
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

    def handle_key(self, key, live):
        if key == "q":
            return False
        if self.view == "dashboard":
            if key == "up" and self.cursor > 0:
                self.cursor -= 1
            elif key == "down" and self.cursor < len(self.rows) - 1:
                self.cursor += 1
            elif key in ("\r", "\n") and self.rows:
                self.selected = self.rows[self.cursor]
                self.view = "detail"
            elif key == "s":
                self.sort_key = SORT_KEYS[(SORT_KEYS.index(self.sort_key) + 1) % len(SORT_KEYS)]
                self._apply_sort()
            elif key == "r":
                self.sort_rev = not self.sort_rev
                self._apply_sort()
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
            with Live(self.render_dashboard(), console=self.console, screen=True, refresh_per_second=10) as live:
                while True:
                    try:
                        render = {"dashboard": self.render_dashboard, "detail": self.render_detail,
                                  "sim_rerun": self.render_sim}
                        if self.view == "sim_rerun" and not self.sim_output:
                            live.update(self.render_sim())
                            self.sim_output = self._run_sim(self.selected["case_id"])
                        live.update(render[self.view]())
                        if not self.handle_key(readkey(), live):
                            break
                    except KeyboardInterrupt:
                        break
        finally:
            termios.tcsetattr(fd, termios.TCSADRAIN, old_term)


def main():
    p = argparse.ArgumentParser(description="Loxone Eval TUI — inspect evaluation results")
    p.add_argument("--report", default=str(DEFAULT_REPORT), help="Path to llm-report.json")
    p.add_argument("--cases", default=str(DEFAULT_CASES), help="Path to eval cases directory")
    p.add_argument("--configs", default=str(DEFAULT_CONFIGS), help="Path to saved .Loxone configs")
    a = p.parse_args()
    EvalTUI(a.report, a.cases, a.configs).run()


if __name__ == "__main__":
    main()
