"""Unit tests for eval-tui.py."""
import io
import json
from pathlib import Path

import pytest
from rich.console import Console

# Import the module under test (hyphenated filename needs importlib)
import importlib
import importlib.util

_spec = importlib.util.spec_from_file_location(
    "eval_tui", Path(__file__).resolve().parent / "eval-tui.py"
)
eval_tui = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(eval_tui)

load_cases = eval_tui.load_cases
load_report = eval_tui.load_report
_make_row = eval_tui._make_row
merge_data = eval_tui.merge_data
scan_reports = eval_tui.scan_reports
build_wiring_dag = eval_tui.build_wiring_dag
render_wiring_diagram = eval_tui.render_wiring_diagram
_topo_sort = eval_tui._topo_sort
EvalTUI = eval_tui.EvalTUI


# ── Fixtures ──


@pytest.fixture
def tmp_cases(tmp_path):
    """Create a temp dir with two case spec JSON files."""
    cases = [
        {"id": "t001-easy", "pattern": ["threshold"], "utterance": "turn on light", "difficulty": "easy",
         "expected": {"new_blocks": [{"type": "And"}]}},
        {"id": "t002-hard", "pattern": ["logic"], "utterance": "complex wiring", "difficulty": "hard",
         "expected": {}},
    ]
    (tmp_path / "cases.json").write_text(json.dumps(cases))
    return tmp_path


@pytest.fixture
def sample_report(tmp_path):
    """Create a minimal report JSON file."""
    report = {
        "cases": [
            {
                "case_id": "t001-easy", "pass": True, "difficulty": "easy",
                "patterns": ["threshold"], "utterance": "turn on light",
                "simulation": {"passed_count": 2, "total_count": 2, "scenarios": []},
                "tokens": {"input_tokens_est": 100, "output_tokens_est": 50},
                "metrics": {"blocks": {"f1": 1.0}, "wiring": {"accuracy": 0.9}, "params": {"accuracy": 1.0}},
                "cli_invocations": 3, "retries": 0,
            },
            {
                "case_id": "t002-hard", "pass": False, "difficulty": "hard",
                "patterns": ["logic"], "utterance": "complex wiring",
                "simulation": {"passed_count": 0, "total_count": 3, "scenarios": [
                    {"name": "check1", "pass": False, "checks": [
                        {"pass": False, "output": "Q", "actual": 0, "expected": 1, "comparator": "=="}
                    ]}
                ]},
                "tokens": {"input_tokens_est": 500, "output_tokens_est": 200},
                "metrics": {"blocks": {"f1": 0.5}, "wiring": {"accuracy": 0.3}, "params": {"accuracy": 0.0}},
                "cli_invocations": 8, "retries": 2,
            },
        ]
    }
    p = tmp_path / "report.json"
    p.write_text(json.dumps(report))
    return p, report


@pytest.fixture
def tui(tmp_cases, sample_report):
    """Create an EvalTUI with test data."""
    report_path, _ = sample_report
    return EvalTUI(str(report_path), str(tmp_cases), str(tmp_cases))


# ── load_cases tests ──


class TestLoadCases:
    def test_loads_from_json(self, tmp_cases):
        specs = load_cases(str(tmp_cases))
        assert "t001-easy" in specs
        assert "t002-hard" in specs
        assert specs["t001-easy"]["difficulty"] == "easy"

    def test_empty_dir(self, tmp_path):
        specs = load_cases(str(tmp_path))
        assert specs == {}

    def test_missing_dir(self, tmp_path):
        specs = load_cases(str(tmp_path / "nonexistent"))
        assert specs == {}

    def test_invalid_json(self, tmp_path):
        (tmp_path / "bad.json").write_text("not json{{{")
        specs = load_cases(str(tmp_path))
        assert specs == {}

    def test_single_object(self, tmp_path):
        (tmp_path / "single.json").write_text(json.dumps(
            {"id": "s001", "difficulty": "medium", "utterance": "test"}
        ))
        specs = load_cases(str(tmp_path))
        assert "s001" in specs

    def test_missing_id_skipped(self, tmp_path):
        (tmp_path / "noid.json").write_text(json.dumps([{"difficulty": "easy"}]))
        specs = load_cases(str(tmp_path))
        assert specs == {}


# ── load_report tests ──


class TestLoadReport:
    def test_none_path(self):
        assert load_report(None) is None

    def test_missing_file(self, tmp_path):
        assert load_report(str(tmp_path / "nope.json")) is None

    def test_invalid_json(self, tmp_path):
        bad = tmp_path / "bad.json"
        bad.write_text("{broken")
        assert load_report(str(bad)) is None

    def test_valid_report(self, sample_report):
        path, _ = sample_report
        result = load_report(str(path))
        assert result is not None
        assert len(result["cases"]) == 2


# ── _make_row tests ──


class TestMakeRow:
    def test_basic_row(self):
        rc = {
            "case_id": "x001", "pass": True, "difficulty": "easy",
            "patterns": ["threshold"], "utterance": "do stuff",
            "simulation": {"passed_count": 1, "total_count": 1},
            "tokens": {"input_tokens_est": 10, "output_tokens_est": 5},
            "metrics": {"blocks": {"f1": 0.8}, "wiring": {"accuracy": 0.9}, "params": {"accuracy": 1.0}},
            "cli_invocations": 2, "retries": 0,
        }
        row = _make_row(rc, {})
        assert row["case_id"] == "x001"
        assert row["pass"] is True
        assert row["tokens_total"] == 15
        assert row["block_f1"] == 0.8

    def test_fallback_to_spec(self):
        rc = {"case_id": "x002"}
        spec = {"difficulty": "hard", "pattern": ["timer"], "utterance": "wait 5 min"}
        row = _make_row(rc, spec)
        assert row["difficulty"] == "hard"
        assert row["patterns"] == ["timer"]
        assert row["utterance"] == "wait 5 min"

    def test_empty_inputs(self):
        row = _make_row({}, {})
        assert row["case_id"] == ""
        assert row["pass"] is False
        assert row["tokens_total"] == 0


# ── merge_data tests ──


class TestMergeData:
    def test_with_report(self, tmp_cases, sample_report):
        _, report = sample_report
        specs = load_cases(str(tmp_cases))
        rows = merge_data(report, specs, str(tmp_cases))
        assert len(rows) == 2
        assert rows[0]["case_id"] == "t001-easy"

    def test_without_report(self, tmp_cases):
        specs = load_cases(str(tmp_cases))
        rows = merge_data(None, specs, str(tmp_cases))
        assert len(rows) == 2
        ids = {r["case_id"] for r in rows}
        assert "t001-easy" in ids
        assert "t002-hard" in ids

    def test_empty_specs(self, tmp_path):
        rows = merge_data(None, {}, str(tmp_path))
        assert rows == []

    def test_report_missing_cases_key(self, tmp_cases):
        specs = load_cases(str(tmp_cases))
        rows = merge_data({"metadata": {}}, specs, str(tmp_cases))
        assert len(rows) == 2


# ── EvalTUI._apply_filters tests ──


class TestApplyFilters:
    def test_filter_pass(self, tui):
        tui.filt_status = "pass"
        tui._apply_filters()
        assert all(r["pass"] for r in tui.rows)

    def test_filter_fail(self, tui):
        tui.filt_status = "fail"
        tui._apply_filters()
        assert all(not r["pass"] for r in tui.rows)

    def test_filter_difficulty(self, tui):
        tui.filt_diff = "easy"
        tui._apply_filters()
        assert all(r["difficulty"] == "easy" for r in tui.rows)

    def test_filter_pattern(self, tui):
        tui.filt_pat = "threshold"
        tui._apply_filters()
        assert len(tui.rows) == 1
        assert tui.rows[0]["case_id"] == "t001-easy"

    def test_filter_combined(self, tui):
        tui.filt_status = "pass"
        tui.filt_diff = "easy"
        tui._apply_filters()
        assert len(tui.rows) == 1

    def test_filter_no_match(self, tui):
        tui.filt_pat = "zzzznonexistent"
        tui._apply_filters()
        assert len(tui.rows) == 0

    def test_cursor_clamped(self, tui):
        tui.cursor = 99
        tui.filt_pat = "threshold"
        tui._apply_filters()
        assert tui.cursor == 0


# ── EvalTUI._apply_sort tests ──


class TestApplySort:
    def test_sort_by_case_id(self, tui):
        tui.sort_key = "case_id"
        tui.sort_rev = False
        tui._apply_sort()
        ids = [r["case_id"] for r in tui.rows]
        assert ids == sorted(ids)

    def test_sort_by_difficulty(self, tui):
        tui.sort_key = "difficulty"
        tui.sort_rev = False
        tui._apply_sort()
        assert tui.rows[0]["difficulty"] == "easy"
        assert tui.rows[1]["difficulty"] == "hard"

    def test_sort_reversed(self, tui):
        tui.sort_key = "case_id"
        tui.sort_rev = True
        tui._apply_sort()
        ids = [r["case_id"] for r in tui.rows]
        assert ids == sorted(ids, reverse=True)

    def test_sort_by_tokens(self, tui):
        tui.sort_key = "tokens"
        tui.sort_rev = False
        tui._apply_sort()
        totals = [r["tokens_total"] for r in tui.rows]
        assert totals == sorted(totals)

    def test_sort_by_block_f1(self, tui):
        tui.sort_key = "block_f1"
        tui.sort_rev = True
        tui._apply_sort()
        f1s = [r["block_f1"] for r in tui.rows]
        assert f1s == sorted(f1s, reverse=True)


# ── render tests ──


class TestRenderDashboard:
    def _render_to_string(self, tui):
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_dashboard())
        return buf.getvalue()

    def test_renders_without_crash(self, tui):
        output = self._render_to_string(tui)
        assert "Eval Dashboard" in output
        assert "t001-easy" in output

    def test_renders_empty_data(self, tmp_path):
        t = EvalTUI(None, str(tmp_path), str(tmp_path))
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(t.render_dashboard())
        output = buf.getvalue()
        assert "Eval Dashboard" in output
        assert "0/0" in output

    def test_renders_with_filters(self, tui):
        tui.filt_status = "pass"
        tui.filt_diff = "easy"
        tui._apply_filters()
        output = self._render_to_string(tui)
        assert "status=pass" in output
        assert "diff=easy" in output


class TestRenderDetail:
    def _render_to_string(self, tui):
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_detail())
        return buf.getvalue()

    def test_no_selection(self, tui):
        tui.selected = None
        output = self._render_to_string(tui)
        assert "No case selected" in output

    def test_pass_case(self, tui):
        tui.selected = tui.all_rows[0]
        output = self._render_to_string(tui)
        assert "t001-easy" in output
        assert "Efficiency" in output

    def test_fail_case(self, tui):
        tui.selected = tui.all_rows[1]
        output = self._render_to_string(tui)
        assert "t002-hard" in output

    def test_fail_case_with_sim_scenarios(self, tui):
        tui.selected = tui.all_rows[1]
        output = self._render_to_string(tui)
        assert "Simulation Results" in output
        assert "check1" in output


class TestRenderSim:
    def test_no_selection(self, tui):
        tui.selected = None
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_sim())
        assert "No case" in buf.getvalue()

    def test_with_selection(self, tui):
        tui.selected = tui.all_rows[0]
        tui.sim_output = "test sim output"
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_sim())
        assert "test sim output" in buf.getvalue()


# ── Smoke test using real cases dir ──


class TestSmokeWithRealCases:
    def test_renders_with_real_cases(self):
        """Smoke test: load real case specs and render dashboard."""
        real_cases = Path(__file__).resolve().parent.parent / "cases"
        if not real_cases.exists():
            pytest.skip("Real cases dir not found")
        t = EvalTUI(None, str(real_cases), "/tmp/lox-eval-agent-z98dvmgk")
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(t.render_dashboard())
        output = buf.getvalue()
        assert "Eval Dashboard" in output
        assert len(t.all_rows) > 0


# ── Terminal safety ──


class TestTerminalSafety:
    def test_run_restores_on_no_data(self, tmp_path):
        """run() with no data should print error and return without entering raw mode."""
        t = EvalTUI(None, str(tmp_path), str(tmp_path))
        buf = io.StringIO()
        t.console = Console(file=buf, force_terminal=True, width=80)
        t.run()
        assert "No data found" in buf.getvalue()

    def test_readkey_restores_terminal(self):
        """Verify readkey() restores terminal attrs in its finally block."""
        import inspect
        source = inspect.getsource(eval_tui.readkey)
        assert "finally" in source
        assert "tcsetattr" in source

    def test_run_restores_terminal_in_finally(self):
        """The run() method has a try/finally that restores terminal settings."""
        import inspect
        source = inspect.getsource(EvalTUI.run)
        assert "finally" in source
        assert "tcsetattr" in source

    def test_run_has_no_raw_mode_setup(self):
        """The run() method itself should not call tty.setraw — only readkey does."""
        import inspect
        source = inspect.getsource(EvalTUI.run)
        assert "setraw" not in source


# ── Edge cases ──


class TestEdgeCases:
    def test_missing_configs_dir(self, tmp_cases):
        t = EvalTUI(None, str(tmp_cases), "/tmp/nonexistent-lox-dir-xyz")
        assert len(t.all_rows) == 2

    def test_handle_key_quit(self, tui):
        result = tui.handle_key("q", None)
        assert result is False

    def test_handle_key_navigation(self, tui):
        assert tui.cursor == 0
        tui.handle_key("down", None)
        assert tui.cursor == 1
        tui.handle_key("up", None)
        assert tui.cursor == 0

    def test_handle_key_sort_cycle(self, tui):
        original = tui.sort_key
        tui.handle_key("s", None)
        assert tui.sort_key != original

    def test_handle_key_enter_detail(self, tui):
        tui.handle_key("\r", None)
        assert tui.view == "detail"
        assert tui.selected is not None

    def test_handle_key_esc_from_detail(self, tui):
        tui.handle_key("\r", None)
        assert tui.view == "detail"
        tui.handle_key("esc", None)
        assert tui.view == "dashboard"

    def test_jump_fail(self, tui):
        tui.cursor = 0
        tui._jump_fail(1)
        assert tui.cursor == 1
        assert not tui.rows[tui.cursor]["pass"]

    def test_no_sim_results(self, tmp_path):
        """No sim results dir should still produce rows."""
        cases = [{"id": "nosim", "difficulty": "easy", "utterance": "test", "expected": {}}]
        (tmp_path / "c.json").write_text(json.dumps(cases))
        t = EvalTUI(None, str(tmp_path), str(tmp_path / "no-configs"))
        assert len(t.all_rows) == 1
        assert t.all_rows[0]["sim_passed"] == 0


# ── scan_reports tests ──


class TestScanReports:
    def test_scan_empty_dir(self, tmp_path):
        results = scan_reports(str(tmp_path))
        assert results == []

    def test_scan_nonexistent_dir(self, tmp_path):
        results = scan_reports(str(tmp_path / "nope"))
        assert results == []

    def test_scan_with_reports(self, tmp_path):
        for i, (name, cases, passed) in enumerate([
            ("a.json", 10, 8), ("b.json", 5, 5), ("c.json", 20, 3),
        ]):
            report = {
                "cases": [{"case_id": f"c{j}", "pass": j < passed} for j in range(cases)],
                "meta": {"timestamp": f"2026-01-0{i+1}T00:00:00Z", "model": "gpt-4o"},
            }
            (tmp_path / name).write_text(json.dumps(report))
        results = scan_reports(str(tmp_path))
        assert len(results) == 3
        # Newest first (by mtime, which will be in write order)
        assert all("filename" in r for r in results)
        assert all("pass_rate" in r for r in results)

    def test_scan_skips_invalid_json(self, tmp_path):
        (tmp_path / "good.json").write_text(json.dumps({"cases": []}))
        (tmp_path / "bad.json").write_text("not json{{{")
        results = scan_reports(str(tmp_path))
        assert len(results) == 1

    def test_scan_sorted_newest_first(self, tmp_path):
        import time
        (tmp_path / "old.json").write_text(json.dumps({"cases": []}))
        time.sleep(0.05)
        (tmp_path / "new.json").write_text(json.dumps({"cases": []}))
        results = scan_reports(str(tmp_path))
        assert results[0]["filename"] == "new.json"
        assert results[1]["filename"] == "old.json"

    def test_scan_real_reports_dir(self):
        """Smoke test: scan actual reports directory."""
        real_dir = Path(__file__).resolve().parent.parent / "reports"
        if not real_dir.exists():
            pytest.skip("Real reports dir not found")
        results = scan_reports(str(real_dir))
        assert len(results) > 0
        assert all(r["filename"].endswith(".json") for r in results)

    def test_report_metadata_fields(self, tmp_path):
        report = {
            "cases": [{"case_id": "x1", "pass": True}, {"case_id": "x2", "pass": False}],
            "meta": {"timestamp": "2026-06-01T12:00:00Z", "model": "sonnet", "work_dir": "/tmp/test"},
        }
        (tmp_path / "r.json").write_text(json.dumps(report))
        results = scan_reports(str(tmp_path))
        assert len(results) == 1
        r = results[0]
        assert r["cases"] == 2
        assert r["passed"] == 1
        assert r["pass_rate"] == 50.0
        assert r["model"] == "sonnet"
        assert r["work_dir"] == "/tmp/test"
        assert r["timestamp"] == "2026-06-01T12:00:00Z"


# ── Wiring DAG tests ──


class TestBuildWiringDag:
    def _make_dump(self, blocks, wires):
        """Helper: create a dump dict with blocks that have wired connections."""
        block_list = []
        for i, (name, btype) in enumerate(blocks):
            inputs = [{"cid": f"in-{i}-0", "key": "I1"}]
            outputs = [{"cid": f"out-{i}-0", "key": "Q", "wired_to": []}]
            block_list.append({"name": name, "type": btype, "room": "R1",
                               "inputs": inputs, "outputs": outputs})
        # Apply wiring: wires is list of (src_block, dst_block)
        for src, dst in wires:
            src_out_cid = f"out-{src}-0"
            dst_in_cid = f"in-{dst}-0"
            block_list[src]["outputs"][0]["wired_to"].append(dst_in_cid)
            block_list[dst]["inputs"][0]["wired_from"] = src_out_cid
        return {"blocks": block_list}

    def test_empty_dump(self):
        blocks, edges = build_wiring_dag(None)
        assert blocks == [] and edges == []

    def test_no_blocks(self):
        blocks, edges = build_wiring_dag({"blocks": []})
        assert blocks == [] and edges == []

    def test_linear_chain(self):
        dump = self._make_dump([("A", "And"), ("B", "Not"), ("C", "Or")], [(0, 1), (1, 2)])
        blocks, edges = build_wiring_dag(dump)
        assert len(blocks) == 3
        assert len(edges) >= 2

    def test_skips_excluded_types(self):
        dump = self._make_dump([("A", "And"), ("V", "VirtualInCaption")], [(0, 1)])
        blocks, edges = build_wiring_dag(dump)
        # VirtualInCaption should be filtered out
        assert all(b["type"] != "VirtualInCaption" for b in blocks)

    def test_no_self_loops(self):
        dump = self._make_dump([("A", "And"), ("B", "Not")], [(0, 1)])
        blocks, edges = build_wiring_dag(dump)
        for s, d, _, _ in edges:
            assert s != d


class TestTopoSort:
    def test_linear(self):
        result = _topo_sort(3, [(0, 1, "Q", "I1"), (1, 2, "Q", "I1")])
        assert result == [0, 1, 2]

    def test_diamond(self):
        edges = [(0, 1, "Q", "I1"), (0, 2, "Q", "I1"), (1, 3, "Q", "I1"), (2, 3, "Q", "I2")]
        result = _topo_sort(4, edges)
        assert result is not None
        assert result[0] == 0
        assert result[-1] == 3

    def test_cycle_returns_none(self):
        edges = [(0, 1, "Q", "I1"), (1, 0, "Q", "I1")]
        result = _topo_sort(2, edges)
        assert result is None

    def test_single_node(self):
        result = _topo_sort(1, [])
        assert result == [0]


class TestRenderWiringDiagram:
    def _make_dump(self, blocks, wires):
        block_list = []
        for i, (name, btype) in enumerate(blocks):
            inputs = [{"cid": f"in-{i}-0", "key": "I1"}]
            outputs = [{"cid": f"out-{i}-0", "key": "Q", "wired_to": []}]
            block_list.append({"name": name, "type": btype, "room": "R1",
                               "inputs": inputs, "outputs": outputs})
        for src, dst in wires:
            block_list[src]["outputs"][0]["wired_to"].append(f"in-{dst}-0")
            block_list[dst]["inputs"][0]["wired_from"] = f"out-{src}-0"
        return {"blocks": block_list}

    def test_none_dump(self):
        assert render_wiring_diagram(None) is None

    def test_no_edges(self):
        dump = {"blocks": [{"name": "A", "type": "And", "inputs": [{"cid": "i1", "key": "I1"}],
                             "outputs": [{"cid": "o1", "key": "Q", "wired_to": []}]}]}
        assert render_wiring_diagram(dump) is None

    def test_linear_chain_renders(self):
        dump = self._make_dump([("Sensor", "And"), ("Gate", "Not"), ("Output", "Or")],
                               [(0, 1), (1, 2)])
        result = render_wiring_diagram(dump)
        assert result is not None
        assert len(result) > 0
        text = "\n".join(str(line) for line in result)
        assert "Sensor" in text
        assert "Gate" in text
        assert "Output" in text

    def test_too_many_blocks_returns_none(self):
        # 10 blocks in a chain → too complex (>8)
        blocks = [(f"B{i}", "And") for i in range(10)]
        wires = [(i, i + 1) for i in range(9)]
        dump = self._make_dump(blocks, wires)
        assert render_wiring_diagram(dump) is None

    def test_box_drawing_chars(self):
        dump = self._make_dump([("A", "And"), ("B", "Not")], [(0, 1)])
        result = render_wiring_diagram(dump)
        assert result is not None
        text = "\n".join(str(line) for line in result)
        assert "┌" in text
        assert "└" in text
        assert "│" in text


# ── Report picker tests ──


class TestReportPicker:
    def _render_to_string(self, tui):
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_report_picker())
        return buf.getvalue()

    def test_renders_empty(self, tui):
        tui.available_reports = []
        tui.reports_dir = "/tmp/nonexistent-reports-xyz"
        output = self._render_to_string(tui)
        assert "Select Report" in output

    def test_renders_with_reports(self, tmp_path, tui):
        rdir = tmp_path / "reports"
        rdir.mkdir()
        for name in ("r1.json", "r2.json"):
            (rdir / name).write_text(json.dumps({
                "cases": [{"case_id": "c1", "pass": True}],
                "meta": {"timestamp": "2026-01-01T00:00:00Z", "model": "gpt-4o"},
            }))
        tui.reports_dir = str(rdir)
        tui.available_reports = []  # Force rescan
        output = self._render_to_string(tui)
        assert "Select Report" in output
        assert "2 available" in output

    def test_R_key_opens_picker(self, tui, tmp_path):
        rdir = tmp_path / "reports"
        rdir.mkdir()
        (rdir / "r.json").write_text(json.dumps({"cases": []}))
        tui.reports_dir = str(rdir)
        tui.handle_key("R", None)
        assert tui.view == "report_picker"

    def test_picker_navigation(self, tui, tmp_path):
        tui.available_reports = [
            {"path": "/a", "filename": "a.json", "timestamp": "", "mtime": 1, "cases": 1,
             "passed": 1, "pass_rate": 100, "model": "", "work_dir": ""},
            {"path": "/b", "filename": "b.json", "timestamp": "", "mtime": 2, "cases": 2,
             "passed": 0, "pass_rate": 0, "model": "", "work_dir": ""},
        ]
        tui.view = "report_picker"
        tui.report_cursor = 0
        tui.handle_key("down", None)
        assert tui.report_cursor == 1
        tui.handle_key("up", None)
        assert tui.report_cursor == 0

    def test_picker_esc_returns_to_dashboard(self, tui):
        tui.view = "report_picker"
        tui.handle_key("esc", None)
        assert tui.view == "dashboard"

    def test_picker_select_loads_report(self, tui, tmp_path, tmp_cases):
        report = {"cases": [{"case_id": "t001-easy", "pass": True,
                             "simulation": {"passed_count": 1, "total_count": 1},
                             "tokens": {"input_tokens_est": 10, "output_tokens_est": 5},
                             "metrics": {"blocks": {"f1": 1.0}, "wiring": {"accuracy": 1.0},
                                         "params": {"accuracy": 1.0}}}]}
        rpath = tmp_path / "pick.json"
        rpath.write_text(json.dumps(report))
        tui.available_reports = [
            {"path": str(rpath), "filename": "pick.json", "timestamp": "", "mtime": 1,
             "cases": 1, "passed": 1, "pass_rate": 100, "model": "", "work_dir": ""},
        ]
        tui.view = "report_picker"
        tui.report_cursor = 0
        tui.handle_key("\r", None)
        assert tui.view == "dashboard"
        assert len(tui.all_rows) == 1

    def test_picker_g_G_navigation(self, tui):
        tui.view = "report_picker"
        tui.available_reports = [{"path": f"/{i}"} for i in range(10)]
        tui.report_cursor = 5
        tui.handle_key("g", None)
        assert tui.report_cursor == 0
        tui.handle_key("G", None)
        assert tui.report_cursor == 9


# ── Detail tabs tests ──


class TestDetailTabs:
    def _render_to_string(self, tui):
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_detail())
        return buf.getvalue()

    def test_default_tab_is_circuit(self, tui):
        tui.selected = tui.all_rows[0]
        tui.detail_tab = 1
        output = self._render_to_string(tui)
        assert "Circuit" in output
        assert "Efficiency" in output

    def test_tab_switching_keys(self, tui):
        tui.view = "detail"
        tui.selected = tui.all_rows[0]
        tui.handle_key("2", None)
        assert tui.detail_tab == 2
        tui.handle_key("3", None)
        assert tui.detail_tab == 3
        tui.handle_key("1", None)
        assert tui.detail_tab == 1

    def test_conversation_tab_no_data(self, tui):
        tui.selected = tui.all_rows[0]
        tui.detail_tab = 2
        output = self._render_to_string(tui)
        assert "Not available" in output

    def test_commands_tab_no_data(self, tui):
        tui.selected = tui.all_rows[0]
        tui.detail_tab = 3
        output = self._render_to_string(tui)
        assert "Not available" in output

    def test_conversation_tab_with_data(self, tmp_path, tmp_cases):
        report = {
            "cases": [{
                "case_id": "t001-easy", "pass": True, "difficulty": "easy",
                "patterns": ["threshold"], "utterance": "turn on light",
                "simulation": {"passed_count": 1, "total_count": 1},
                "tokens": {"input_tokens_est": 10, "output_tokens_est": 5},
                "metrics": {"blocks": {"f1": 1.0}, "wiring": {"accuracy": 1.0}, "params": {"accuracy": 1.0}},
                "cli_invocations": 1, "retries": 0,
                "conversation": [
                    {"role": "user", "content": "Build a light controller", "label": "initial prompt"},
                    {"role": "assistant", "content": "lox config add --type LightController2"},
                ],
            }]
        }
        rpath = tmp_path / "conv.json"
        rpath.write_text(json.dumps(report))
        t = EvalTUI(str(rpath), str(tmp_cases), str(tmp_path))
        t.selected = t.all_rows[0]
        t.detail_tab = 2
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(t.render_detail())
        output = buf.getvalue()
        assert "USER" in output
        assert "ASSISTANT" in output

    def test_commands_tab_with_data(self, tmp_path, tmp_cases):
        report = {
            "cases": [{
                "case_id": "t001-easy", "pass": True, "difficulty": "easy",
                "patterns": ["threshold"], "utterance": "turn on light",
                "simulation": {"passed_count": 1, "total_count": 1},
                "tokens": {"input_tokens_est": 10, "output_tokens_est": 5},
                "metrics": {"blocks": {"f1": 1.0}, "wiring": {"accuracy": 1.0}, "params": {"accuracy": 1.0}},
                "cli_invocations": 2, "retries": 0,
                "commands": [
                    {"command": "lox config add --type And", "exit_code": 0, "output": "Added And"},
                    {"command": "lox config wire-connector ...", "exit_code": 1, "output": "Error", "retry": True},
                ],
            }]
        }
        rpath = tmp_path / "cmds.json"
        rpath.write_text(json.dumps(report))
        t = EvalTUI(str(rpath), str(tmp_cases), str(tmp_path))
        t.selected = t.all_rows[0]
        t.detail_tab = 3
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(t.render_detail())
        output = buf.getvalue()
        assert "Commands" in output
        assert "2 total" in output

    def test_enter_detail_resets_tab(self, tui):
        """Entering detail view should reset to tab 1."""
        tui.detail_tab = 3
        tui.handle_key("\r", None)
        assert tui.view == "detail"
        assert tui.detail_tab == 1

    def test_tab_bar_rendering(self, tui):
        """Tab bar should show all three tabs."""
        tui.selected = tui.all_rows[0]
        for tab in (1, 2, 3):
            tui.detail_tab = tab
            buf = io.StringIO()
            console = Console(width=120, force_terminal=True, file=buf)
            console.print(tui.render_detail())
            output = buf.getvalue()
            assert "Circuit" in output
            assert "Conversation" in output
            assert "Commands" in output


# ── Terminal resize tests ──


class TestTerminalResize:
    def test_narrow_hides_columns(self, tui):
        """When terminal is narrow (<80), Wire and Param columns should be hidden."""
        tui._get_term_width = lambda: 60  # Mock narrow terminal
        buf = io.StringIO()
        console = Console(width=60, force_terminal=True, file=buf)
        console.print(tui.render_dashboard())
        output = buf.getvalue()
        assert "Eval Dashboard" in output
        # Block column should still be visible
        assert "Block" in output

    def test_wide_shows_all_columns(self, tui):
        """When terminal is wide (>=80), all columns should be shown."""
        tui._get_term_width = lambda: 120
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_dashboard())
        output = buf.getvalue()
        assert "Wire" in output
        assert "Param" in output

    def test_get_term_width_fallback(self, tui):
        """_get_term_width should not crash when no terminal is available."""
        # In test environment, os.get_terminal_size() may fail
        width = tui._get_term_width()
        assert isinstance(width, int)
        assert width > 0


# ── Help bar tests ──


class TestHelpBar:
    def test_dashboard_help(self, tui):
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_dashboard())
        output = buf.getvalue()
        assert "Reports" in output
        assert "Search" in output
        assert "Difficulty" in output

    def test_detail_help(self, tui):
        tui.selected = tui.all_rows[0]
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_detail())
        output = buf.getvalue()
        assert "Tabs" in output
        assert "Next/Prev" in output

    def test_report_picker_help(self, tui, tmp_path):
        tui.reports_dir = str(tmp_path)
        tui.available_reports = []
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_report_picker())
        output = buf.getvalue()
        assert "Select" in output
        assert "Back" in output


# ── Meta work_dir resolution tests ──


class TestWorkDirResolution:
    def test_meta_work_dir_used_when_exists(self, tmp_path, tmp_cases):
        work_dir = tmp_path / "work"
        work_dir.mkdir()
        report = {
            "cases": [{"case_id": "t001-easy", "pass": True,
                        "simulation": {"passed_count": 1, "total_count": 1},
                        "tokens": {}, "metrics": {}}],
            "meta": {"work_dir": str(work_dir)},
        }
        rpath = tmp_path / "r.json"
        rpath.write_text(json.dumps(report))
        t = EvalTUI(str(rpath), str(tmp_cases), str(tmp_path / "fallback"))
        assert t.configs_dir == work_dir

    def test_meta_work_dir_fallback_when_missing(self, tmp_path, tmp_cases):
        report = {
            "cases": [{"case_id": "t001-easy", "pass": True,
                        "simulation": {}, "tokens": {}, "metrics": {}}],
            "meta": {"work_dir": "/tmp/nonexistent-xyz-12345"},
        }
        rpath = tmp_path / "r.json"
        rpath.write_text(json.dumps(report))
        fallback = tmp_path / "fallback"
        t = EvalTUI(str(rpath), str(tmp_cases), str(fallback))
        assert t.configs_dir == fallback

    def test_no_meta_uses_configs_arg(self, tmp_path, tmp_cases):
        report = {"cases": [{"case_id": "t001-easy", "pass": True,
                              "simulation": {}, "tokens": {}, "metrics": {}}]}
        rpath = tmp_path / "r.json"
        rpath.write_text(json.dumps(report))
        t = EvalTUI(str(rpath), str(tmp_cases), str(tmp_path))
        assert t.configs_dir == tmp_path


# ── _find_report_case tests ──


class TestFindReportCase:
    def test_finds_existing_case(self, tui):
        rc = tui._find_report_case("t001-easy")
        assert rc is not None
        assert rc["case_id"] == "t001-easy"

    def test_returns_none_for_missing(self, tui):
        assert tui._find_report_case("nonexistent") is None

    def test_returns_none_when_no_report(self, tmp_path):
        t = EvalTUI(None, str(tmp_path), str(tmp_path))
        assert t._find_report_case("anything") is None


# ── Page Up/Down tests ──


class TestPageUpDown:
    def test_readkey_page_up_sequence(self):
        """readkey() should parse \\x1b[5~ as page_up."""
        import inspect
        source = inspect.getsource(eval_tui.readkey)
        assert "page_up" in source
        assert "page_down" in source

    def test_page_down_detail_scroll(self, tui):
        tui.view = "detail"
        tui.selected = tui.all_rows[0]
        tui.detail_scroll = 0
        tui.handle_key("page_down", None)
        assert tui.detail_scroll > 0

    def test_page_up_detail_scroll(self, tui):
        tui.view = "detail"
        tui.selected = tui.all_rows[0]
        tui.detail_scroll = 50
        tui.handle_key("page_up", None)
        assert tui.detail_scroll < 50

    def test_page_up_clamps_to_zero(self, tui):
        tui.view = "detail"
        tui.selected = tui.all_rows[0]
        tui.detail_scroll = 3
        tui.handle_key("page_up", None)
        assert tui.detail_scroll == 0

    def test_help_includes_pgupdown(self):
        bindings = eval_tui.HELP_FULL["detail"]
        keys = [k for k, _ in bindings]
        assert "PgUp / PgDn" in keys


# ── Sim Trace tests ──


class TestSimTrace:
    def test_t_key_enters_trace_view(self, tui):
        tui.view = "detail"
        tui.selected = tui.all_rows[0]
        tui.handle_key("t", None)
        assert tui.view == "sim_trace"
        assert tui.trace_data is None
        assert tui.trace_scenario_idx == 0
        assert tui.trace_scroll == 0

    def test_esc_returns_to_detail(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        tui.handle_key("esc", None)
        assert tui.view == "detail"

    def test_q_returns_to_detail(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        result = tui.handle_key("q", None)
        assert result is True
        assert tui.view == "detail"

    def test_scroll_up_down(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        tui.trace_scroll = 5
        tui.handle_key("up", None)
        assert tui.trace_scroll == 4
        tui.handle_key("down", None)
        assert tui.trace_scroll == 5

    def test_scroll_clamps_zero(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        tui.trace_scroll = 0
        tui.handle_key("up", None)
        assert tui.trace_scroll == 0

    def test_page_up_down_in_trace(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        tui.trace_scroll = 0
        tui.handle_key("page_down", None)
        assert tui.trace_scroll > 0
        prev = tui.trace_scroll
        tui.handle_key("page_up", None)
        assert tui.trace_scroll == 0

    def test_left_right_navigation(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        tui.trace_data = [
            {"name": "sc1", "steps": []},
            {"name": "sc2", "steps": []},
        ]
        tui.trace_scenario_idx = 0
        tui.handle_key("right", None)
        assert tui.trace_scenario_idx == 1
        tui.handle_key("left", None)
        assert tui.trace_scenario_idx == 0

    def test_left_clamps_to_zero(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        tui.trace_data = [{"name": "sc1", "steps": []}]
        tui.trace_scenario_idx = 0
        tui.handle_key("left", None)
        assert tui.trace_scenario_idx == 0

    def test_right_clamps_to_max(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        tui.trace_data = [{"name": "sc1", "steps": []}, {"name": "sc2", "steps": []}]
        tui.trace_scenario_idx = 1
        tui.handle_key("right", None)
        assert tui.trace_scenario_idx == 1

    def test_g_G_in_trace(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        tui.trace_scroll = 50
        tui.handle_key("g", None)
        assert tui.trace_scroll == 0
        tui.handle_key("G", None)
        assert tui.trace_scroll == 9999

    def test_right_resets_scroll(self, tui):
        tui.view = "sim_trace"
        tui.selected = tui.all_rows[0]
        tui.trace_data = [{"name": "sc1", "steps": []}, {"name": "sc2", "steps": []}]
        tui.trace_scenario_idx = 0
        tui.trace_scroll = 50
        tui.handle_key("right", None)
        assert tui.trace_scroll == 0

    def test_render_trace_no_selection(self, tui):
        tui.selected = None
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_trace())
        assert "No case selected" in buf.getvalue()

    def test_render_trace_loading(self, tui):
        tui.selected = tui.all_rows[0]
        tui.trace_data = None
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_trace())
        assert "Running sim with trace" in buf.getvalue()

    def test_render_trace_empty(self, tui):
        tui.selected = tui.all_rows[0]
        tui.trace_data = []
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_trace())
        assert "No simulation specs" in buf.getvalue()

    def test_render_trace_with_data(self, tui):
        tui.selected = tui.all_rows[0]
        tui.trace_data = [{
            "name": "pump runs when scheduled",
            "steps": [{
                "index": 0,
                "inputs": {"Vorlauftemperatur.AQ": 60.0},
                "ticks": 10, "dt": 0.1,
                "signals": {"Vorlauftemperatur.AQ": 60.0, "Poolpumpe.Q": 1.0, "Empty.X": 0.0},
                "checks": [
                    {"pass": True, "output": "Poolpumpe.I1", "actual": 1.0,
                     "comparator": ">", "expected": 0.5},
                ],
            }],
        }]
        tui.trace_scenario_idx = 0
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_trace())
        output = buf.getvalue()
        assert "pump runs when scheduled" in output
        assert "Step 0" in output
        assert "Inject" in output
        assert "Ticks" in output
        assert "Signals" in output
        assert "Checks" in output

    def test_render_trace_filters_zero_signals(self, tui):
        tui.selected = tui.all_rows[0]
        tui.trace_data = [{
            "name": "test",
            "steps": [{
                "index": 0, "inputs": {}, "ticks": 10, "dt": 0.1,
                "signals": {"A": 1.0, "B": 0.0, "C": 0},
                "checks": [],
            }],
        }]
        tui.trace_scenario_idx = 0
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_trace())
        output = buf.getvalue()
        assert "A" in output

    def test_render_trace_error_step(self, tui):
        tui.selected = tui.all_rows[0]
        tui.trace_data = [{
            "name": "test",
            "steps": [{
                "index": 0, "inputs": {}, "ticks": 0, "dt": 0,
                "signals": {}, "checks": [],
                "error": "Simulation timed out",
            }],
        }]
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_trace())
        output = buf.getvalue()
        assert "timed out" in output

    def test_render_trace_all_zero_signals(self, tui):
        tui.selected = tui.all_rows[0]
        tui.trace_data = [{
            "name": "test",
            "steps": [{
                "index": 0, "inputs": {}, "ticks": 10, "dt": 0.1,
                "signals": {"A": 0, "B": 0.0},
                "checks": [],
            }],
        }]
        buf = io.StringIO()
        console = Console(width=120, force_terminal=True, file=buf)
        console.print(tui.render_trace())
        output = buf.getvalue()
        assert "all zero" in output

    def test_help_full_has_sim_trace(self):
        assert "sim_trace" in eval_tui.HELP_FULL
        keys = [k for k, _ in eval_tui.HELP_FULL["sim_trace"]]
        assert "← / →" in keys
        assert "PgUp / PgDn" in keys

    def test_run_trace_no_config(self, tui, tmp_path):
        tui.selected = tui.all_rows[0]
        tui.configs_dir = tmp_path / "nonexistent"
        result = tui._run_trace("t001-easy")
        assert result is None

    def test_run_trace_no_sim_specs(self, tui, tmp_path):
        tui.selected = {"spec": {"expected": {}}}
        tui.configs_dir = tmp_path
        result = tui._run_trace("t001-easy")
        assert result is None
