#!/usr/bin/env python3
"""
Eval harness that delegates to external agent tools.

Supports:
  --agent opencode    # OpenCode (default)
  --agent copilot     # GitHub Copilot CLI
  --agent claude      # Claude Code
  --agent builtin     # Our built-in LLM loop (fallback)

Usage:
  python3 eval-agent.py --case s01-piano-protection --agent opencode
  python3 eval-agent.py --all --section synthetic --max-cases 10 --agent opencode
  python3 eval-agent.py --all --agent copilot --model gpt-4o
  python3 eval-agent.py --all --agent builtin --model gpt-4o
  python3 eval-agent.py --report report.json
"""

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

# ── Paths ────────────────────────────────────────────────────

SCRIPT_DIR = Path(__file__).parent
EVAL_DIR = SCRIPT_DIR.parent
REPO_ROOT = EVAL_DIR.parent.parent
FIXTURE = EVAL_DIR / "fixture.Loxone"
SKILL_DIR = REPO_ROOT / ".github" / "skills"

# Import helpers from sibling scripts
sys.path.insert(0, str(SCRIPT_DIR))
from importlib import import_module as _imp

_agent_runner = _imp("agent-runner")
load_cases = _agent_runner.load_cases
evaluate_correctness = _agent_runner.evaluate_correctness
generate_report = _agent_runner.generate_report
print_report = _agent_runner.print_report
CLITracker = _agent_runner.CLITracker

_llm_agent = _imp("llm-agent")
run_simulation = _llm_agent.run_simulation

from trace_eval import evaluate_by_trace

AGENT_TIMEOUT = 600  # 10 minutes per case


# ── Agent Backends ───────────────────────────────────────────

def _build_instructions(utterance: str, config_path: str, hint: str = "") -> str:
    """Build the instruction prompt sent to external agents."""
    skill_path = SKILL_DIR / "loxone-config" / "SKILL.md"
    skill_text = ""
    if skill_path.exists():
        text = skill_path.read_text()
        # Strip YAML frontmatter
        if text.startswith("---"):
            end = text.find("---", 3)
            if end != -1:
                text = text[end + 3:].lstrip("\n")
        skill_text = text

    hint_section = ""
    if hint:
        hint_section = f"""
Implementation hint (use this plan to work efficiently — batch commands with &&):
{hint.replace('FILE', config_path)}

"""

    return f"""\
You are configuring a Loxone Miniserver. The config file is: {config_path}

Read the skill reference at .github/skills/loxone-config/SKILL.md for CLI commands and block types.
{hint_section}
Follow this workflow:
1. Build: use lox config add, wire-connector, set-param commands (batch with && for speed)
2. Check: lox config check {config_path}
3. Fix any issues from check output and re-check

Task: {utterance}
"""


def run_opencode(utterance: str, config_path: str, work_dir: str, hint: str = "") -> int:
    """Run OpenCode agent with lox CLI tools available."""
    instructions = _build_instructions(utterance, config_path, hint)

    env = {
        **os.environ,
        "OPENAI_BASE_URL": os.environ.get(
            "OPENAI_BASE_URL", "https://models.inference.ai.azure.com"
        ),
        "OPENAI_API_KEY": os.environ.get(
            "GITHUB_TOKEN", os.environ.get("OPENAI_API_KEY", "")
        ),
    }

    result = subprocess.run(
        ["opencode", "-p", instructions, "--cwd", work_dir],
        capture_output=True, text=True, timeout=AGENT_TIMEOUT, env=env,
    )
    return result.returncode


def run_copilot(utterance: str, config_path: str, work_dir: str, hint: str = "") -> int:
    """Run GitHub Copilot CLI."""
    instructions = _build_instructions(utterance, config_path, hint)
    project_root = str(EVAL_DIR.parent.parent)

    result = subprocess.run(
        ["copilot", "-p", instructions,
         "--add-dir", work_dir, "--add-dir", project_root, "--allow-all"],
        capture_output=True, text=True, timeout=AGENT_TIMEOUT,
        cwd=project_root,
    )
    return result.returncode


def run_claude(utterance: str, config_path: str, work_dir: str, hint: str = "") -> int:
    """Run Claude Code."""
    instructions = _build_instructions(utterance, config_path, hint)

    result = subprocess.run(
        ["claude", "-p", instructions, "--allowedTools", "Bash"],
        capture_output=True, text=True, timeout=AGENT_TIMEOUT,
        cwd=work_dir,
    )
    return result.returncode


    client = _llm_agent._create_client()
    work_path = Path(work_dir)

    # Prevent run_case from re-copying (file already in place)
    orig_fixture = _llm_agent.FIXTURE
    _llm_agent.FIXTURE = Path(config_path).resolve()
    # Use a separate work subdir so run_case's copy is to a different path
    case_dir = work_path / "_builtin"
    case_dir.mkdir(exist_ok=True)
    shutil.copy2(config_path, case_dir / Path(config_path).name)
    try:
        result_path, _, _ = _llm_agent.run_case(case, case_dir, client, model, verbose=verbose)
        # Copy result back
        shutil.copy2(str(result_path), config_path)
    finally:
        _llm_agent.FIXTURE = orig_fixture

    return 0


AGENTS = {
    "opencode": run_opencode,
    "copilot": run_copilot,
    "claude": run_claude,
    
}


# ── Evaluation Flow ──────────────────────────────────────────

def _run_fixture_setup(case: dict, config_path: Path, verbose: bool = False):
    """Run fixture_setup commands to pre-build partial config before agent starts."""
    setup_cmds = case.get("fixture_setup", [])
    if not setup_cmds:
        return
    lox_bin = shutil.which("lox") or str(REPO_ROOT / "target" / "release" / "lox")
    for cmd_template in setup_cmds:
        cmd = cmd_template.replace("${FILE}", str(config_path))
        if verbose:
            print(f"  [setup] {cmd}", file=sys.stderr)
        # Split respecting shell quoting
        import shlex
        parts = shlex.split(cmd)
        # Replace 'lox' with actual binary path
        if parts and parts[0] == "lox":
            parts[0] = lox_bin
        result = subprocess.run(parts, capture_output=True, text=True, timeout=30)
        if result.returncode != 0:
            raise RuntimeError(
                f"fixture_setup failed: {cmd}\n{result.stderr.strip()}"
            )


def evaluate_case(case: dict, agent_name: str, work_dir: Path,
                  model: str = "gpt-4o", verbose: bool = False,
                  trace_eval: bool = False) -> dict:
    """Run one eval case: agent → sim → structural eval."""
    case_id = case["id"]
    config_path = work_dir / f"{case_id}.Loxone"
    shutil.copy2(str(FIXTURE), str(config_path))

    # Run fixture_setup commands to pre-build partial config
    try:
        _run_fixture_setup(case, config_path, verbose=verbose)
    except Exception as e:
        return _error_result(case, f"fixture_setup error: {e}", 0.0, model)

    # Run the agent
    start = time.time()
    try:
        if agent_name == "builtin":
            run_builtin(case["utterance"], str(config_path), str(work_dir),
                        model=model, verbose=verbose)
        else:
            agent_fn = AGENTS[agent_name]
            hint = case.get("hint", "")
            agent_fn(case["utterance"], str(config_path), str(work_dir), hint=hint)
        agent_error = None
    except subprocess.TimeoutExpired:
        agent_error = f"Agent timed out after {AGENT_TIMEOUT}s"
    except FileNotFoundError as e:
        agent_error = f"Agent binary not found: {e}"
    except Exception as e:
        agent_error = str(e)
    elapsed = time.time() - start

    if agent_error:
        return _error_result(case, agent_error, elapsed, model)

    # Structural evaluation
    eval_result = evaluate_correctness(FIXTURE, config_path, case)

    # Standard sim-spec evaluation (deterministic signal checks)
    sim_result = run_simulation(case_id, case, str(config_path))
    eval_result["simulation"] = sim_result
    sim_total = sim_result.get("total_count", 0)
    sim_passed = sim_result.get("passed_count", 0)

    if sim_total > 0:
        eval_result["sim_pass"] = sim_result.get("pass", False)
        eval_result["sim_score"] = sim_passed / sim_total if sim_total else 0
        sim_pass = sim_result.get("pass", False)
    else:
        eval_result["sim_pass"] = None
        eval_result["sim_score"] = None
        sim_pass = None

    if trace_eval:
        # Trace-based evaluation: probe circuit and judge behavior
        trace_result = evaluate_by_trace(
            str(config_path), case["utterance"],
            agent_backend=agent_name, verbose=verbose,
        )
        eval_result["trace_judge"] = trace_result
        trace_pass = trace_result["pass"]

        # Hybrid pass: sim specs (if present) AND trace judge must agree
        if sim_pass is not None:
            eval_result["pass"] = sim_pass and (trace_pass is True or trace_pass is None)
        else:
            eval_result["pass"] = trace_pass if trace_pass is not None else False
    else:
        # Sim-only mode
        if sim_pass is not None:
            eval_result["pass"] = sim_pass
        # else: eval_result["pass"] already set by evaluate_correctness

    eval_result["case_id"] = case_id
    eval_result["difficulty"] = case.get("difficulty", "medium")
    eval_result["patterns"] = case.get("pattern", [])
    eval_result["utterance"] = case["utterance"]
    eval_result["elapsed_seconds"] = round(elapsed, 1)
    eval_result["cli_invocations"] = 0  # external agents don't expose this
    eval_result["retries"] = 0
    eval_result["tokens"] = {"input_tokens_est": 0, "output_tokens_est": 0}
    eval_result["model"] = model

    return eval_result


def _error_result(case: dict, error: str, elapsed: float, model: str) -> dict:
    """Build a failed result dict for agent errors."""
    return {
        "case_id": case["id"],
        "pass": False,
        "sim_pass": False,
        "sim_score": 0,
        "overall_score": 0,
        "error": error,
        "difficulty": case.get("difficulty", "medium"),
        "patterns": case.get("pattern", []),
        "utterance": case["utterance"],
        "elapsed_seconds": round(elapsed, 1),
        "metrics": {
            "blocks": {"precision": 0, "recall": 0, "f1": 0,
                       "true_positives": 0, "false_positives": 0,
                       "false_negatives": 0},
            "wiring": {"accuracy": 0, "precision": 0, "correct": 0,
                       "total": 0, "extra": 0},
            "params": {"accuracy": 0, "correct": 0, "total": 0},
            "ux": {"score": 0, "issues": []},
        },
        "cli_invocations": 0,
        "retries": 0,
        "tokens": {"input_tokens_est": 0, "output_tokens_est": 0},
        "model": model,
    }


# ── Main ─────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(
        description="Eval harness with external agent tool support"
    )
    parser.add_argument("--case", help="Run a single case by ID")
    parser.add_argument("--all", action="store_true", help="Run all cases")
    parser.add_argument("--filter", help="Filter cases by pattern/difficulty/keyword")
    parser.add_argument("--section", help="Only run a specific case section")
    parser.add_argument("--max-cases", type=int, help="Max cases to run")
    parser.add_argument("--skip", type=int, default=0, help="Skip first N cases")
    parser.add_argument(
        "--agent", default="copilot",
        choices=list(AGENTS.keys()),
        help="Agent backend (default: opencode)",
    )
    parser.add_argument(
        "--model", default="gpt-4o",
        help="Model name for builtin agent (default: gpt-4o)",
    )
    parser.add_argument(
        "--output", default="tests/eval/reports/eval-agent-report.json",
        help="Output report file",
    )
    parser.add_argument("--report", help="Pretty-print an existing report")
    parser.add_argument("--work-dir", help="Working directory for result configs")
    parser.add_argument(
        "--keep", action="store_true", help="Keep working directory after run"
    )
    parser.add_argument(
        "--verbose", "-v", action="store_true", help="Verbose output"
    )
    parser.add_argument(
        "--parallel", type=int, default=1,
        help="Number of parallel agent instances (default: 1, max recommended: 2)"
    )
    parser.add_argument(
        "--exclude-section", action="append", default=[],
        help="Exclude a section (can be repeated)"
    )
    parser.add_argument(
        "--trace-eval", action="store_true",
        help="Use trace-based eval: probe circuit behavior instead of pre-written sim specs"
    )
    args = parser.parse_args()

    # Pretty-print mode
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
        cases = load_cases(args.filter, args.section, args.max_cases)
        if args.skip:
            cases = cases[args.skip:]

    # Exclude sections
    if args.exclude_section:
        excluded = set(args.exclude_section)
        cases = [c for c in cases if c.get("_section") not in excluded]

    mode = "trace-eval" if args.trace_eval else "sim-spec"
    print(f"Eval Agent — agent={args.agent}  model={args.model}  mode={mode}  cases={len(cases)}  parallel={args.parallel}")
    print(f"Fixture: {FIXTURE}")
    print()

    # Work directory
    if args.work_dir:
        work_dir = Path(args.work_dir)
        work_dir.mkdir(parents=True, exist_ok=True)
    else:
        work_dir = Path(tempfile.mkdtemp(prefix="lox-eval-agent-"))

    results_by_idx = {}
    passed = 0
    failed = 0
    print_lock = threading.Lock()
    counter = {"done": 0}

    def format_result(idx, case_id, result):
        sim_pass = result.get("sim_pass")
        sim_total = result.get("simulation", {}).get("total_count", 0)
        sim_passed_count = result.get("simulation", {}).get("passed_count", 0)
        m = result.get("metrics", {})
        chk = "clean" if result.get("validation_pass") else "dirty"
        err = result.get("error")
        elapsed = result.get("elapsed_seconds", 0)
        struct = (f"B={m.get('blocks', {}).get('f1', 0):.0%} "
                  f"P={m.get('params', {}).get('accuracy', 0):.0%}")

        if err:
            return False, f"  [{idx + 1}/{len(cases)}] {case_id:42s} \033[33m⚠ ERROR\033[0m  {err}"
        elif sim_pass:
            return True, (f"  [{idx + 1}/{len(cases)}] {case_id:42s} "
                         f"\033[32m✓ PASS\033[0m  sim={sim_passed_count}/{sim_total}  "
                         f"check={chk}  (struct: {struct})  [{elapsed:.0f}s]")
        elif sim_total > 0:
            return False, (f"  [{idx + 1}/{len(cases)}] {case_id:42s} "
                          f"\033[31m✗ FAIL\033[0m  sim={sim_passed_count}/{sim_total}  "
                          f"check={chk}  (struct: {struct})  [{elapsed:.0f}s]")
        elif result.get("validation_pass"):
            return True, (f"  [{idx + 1}/{len(cases)}] {case_id:42s} "
                         f"\033[32m✓ PASS\033[0m  check=clean  "
                         f"(struct: {struct})  (no sim spec)  [{elapsed:.0f}s]")
        else:
            return False, (f"  [{idx + 1}/{len(cases)}] {case_id:42s} "
                          f"\033[31m✗ FAIL\033[0m  check=dirty  "
                          f"(struct: {struct})  [{elapsed:.0f}s]")

    def run_one(idx_case):
        idx, case = idx_case
        result = evaluate_case(
            case, args.agent, work_dir, model=args.model, verbose=args.verbose,
            trace_eval=args.trace_eval,
        )
        is_pass, line = format_result(idx, case["id"], result)
        with print_lock:
            counter["done"] += 1
            results_by_idx[idx] = result
            # Incremental save every 5 cases
            if counter["done"] % 5 == 0 or counter["done"] == len(cases):
                partial = [results_by_idx[j] for j in sorted(results_by_idx)]
                partial_report = generate_report(partial)
                partial_report["meta"] = {
                    "agent": args.agent, "model": args.model,
                    "parallel": args.parallel, "progress": f"{counter['done']}/{len(cases)}",
                    "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                }
                out_path = Path(args.output)
                out_path.parent.mkdir(parents=True, exist_ok=True)
                with open(out_path, "w") as f:
                    json.dump(partial_report, f, indent=2, ensure_ascii=False)
            print(line, flush=True)
        return idx, result, is_pass

    if args.parallel > 1:
        with ThreadPoolExecutor(max_workers=args.parallel) as pool:
            futures = [pool.submit(run_one, (i, c)) for i, c in enumerate(cases)]
            for future in as_completed(futures):
                idx, result, is_pass = future.result()
                results_by_idx[idx] = result
                if is_pass:
                    passed += 1
                else:
                    failed += 1
    else:
        for i, case in enumerate(cases):
            idx, result, is_pass = run_one((i, case))
            results_by_idx[idx] = result
            if is_pass:
                passed += 1
            else:
                failed += 1

    # Reassemble results in order
    results = [results_by_idx[i] for i in range(len(cases))]

    # Generate report
    report = generate_report(results)
    report["meta"] = {
        "agent": args.agent,
        "model": args.model,
        "timeout_seconds": AGENT_TIMEOUT,
        "parallel": args.parallel,
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    }

    out_path = Path(args.output)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, "w") as f:
        json.dump(report, f, indent=2, ensure_ascii=False)

    print()
    print_report(report)
    print(f"\nReport saved to: {args.output}")

    if not args.keep and not args.work_dir:
        shutil.rmtree(work_dir, ignore_errors=True)

    sys.exit(0 if failed == 0 else 1)


if __name__ == "__main__":
    main()
