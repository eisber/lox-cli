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
