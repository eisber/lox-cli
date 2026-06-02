#!/usr/bin/env python3
"""
LLM-powered Loxone Eval Agent

Uses GitHub Models API (OpenAI-compatible) to translate natural-language
automation requests into `lox config` CLI commands, then evaluates the
result against expected outcomes.

Usage:
  python3 tests/eval/scripts/llm-agent.py --case s01-piano-protection --model gpt-4o
  python3 tests/eval/scripts/llm-agent.py --all --model gpt-4o --max-cases 20 --output report.json
  python3 tests/eval/scripts/llm-agent.py --report report.json
  python3 tests/eval/scripts/llm-agent.py --all --filter synthetic --model gpt-4o-mini
"""

import argparse
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

# ── Paths ────────────────────────────────────────────────────

SCRIPT_DIR = Path(__file__).parent
EVAL_DIR = SCRIPT_DIR.parent
REPO_ROOT = EVAL_DIR.parent.parent
FIXTURE = EVAL_DIR / "fixture.Loxone"
COMMANDS_MD = REPO_ROOT / "COMMANDS.md"
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

MAX_RETRIES = 3


def _find_lox_sim():
    """Find the lox sim binary (prefers main lox binary with sim subcommand)."""
    # Prefer the unified binary
    for candidate in ["lox", "./target/release/lox", str(EVAL_DIR.parent.parent / "target" / "release" / "lox")]:
        if shutil.which(candidate):
            return [candidate, "sim"]
        p = Path(candidate)
        if p.exists():
            return [str(p), "sim"]
    # Fall back to standalone lox-sim
    for candidate in ["lox-sim", "./target/release/lox-sim", str(EVAL_DIR.parent.parent / "target" / "release" / "lox-sim")]:
        if shutil.which(candidate):
            return [candidate]
        p = Path(candidate)
        if p.exists():
            return [str(p)]
    return ["lox", "sim"]  # hope lox is on PATH


def run_simulation(case_id: str, case: dict, config_path: str) -> dict:
    """Run simulation specs via Rust lox-sim binary."""
    sims = case.get("expected", {}).get("simulation", [])
    if not sims:
        return {"case_id": case_id, "pass": True, "passed_count": 0, "total_count": 0, "scenarios": []}

    sim_json = json.dumps(sims)
    lox_sim = _find_lox_sim()

    try:
        r = subprocess.run(
            lox_sim + ["run", config_path, "--sim", sim_json],
            capture_output=True, text=True, timeout=30
        )
        if r.stdout.strip():
            result = json.loads(r.stdout)
            return {
                "case_id": case_id,
                "pass": result.get("pass", False),
                "passed_count": result.get("passed", 0),
                "total_count": result.get("total", 0),
                "scenarios": result.get("scenarios", []),
            }
        else:
            return {"case_id": case_id, "pass": False, "passed_count": 0, "total_count": len(sims),
                    "error": r.stderr.strip()[:500], "scenarios": []}
    except Exception as e:
        return {"case_id": case_id, "pass": False, "passed_count": 0, "total_count": len(sims),
                "error": str(e), "scenarios": []}


# ── LLM Client ──────────────────────────────────────────────

def _create_client():
    import openai

    token = os.environ.get("GITHUB_TOKEN") or os.environ.get(
        "GITHUB_MODELS_TOKEN", ""
    )
    if not token:
        print(
            "error: set GITHUB_TOKEN or GITHUB_MODELS_TOKEN for GitHub Models API",
            file=sys.stderr,
        )
        sys.exit(1)

    return openai.OpenAI(
        base_url="https://models.inference.ai.azure.com",
        api_key=token,
    )


# ── Prompt Construction ─────────────────────────────────────

def _find_lox():
    if shutil.which("lox"):
        return ["lox"]
    return ["cargo", "run", "--quiet", "--"]


def _truncate_commands_md(max_chars: int = 4000) -> str:
    """Return the config-relevant portion of COMMANDS.md."""
    if not COMMANDS_MD.exists():
        return "(COMMANDS.md not found)"
    text = COMMANDS_MD.read_text()
    # Keep from '## Config' onward, or fall back to the whole file
    idx = text.find("## Config")
    if idx == -1:
        idx = text.find("## config")
    if idx != -1:
        text = text[idx:]
    if len(text) > max_chars:
        text = text[:max_chars] + "\n…(truncated)"
    return text


def _describe_fixture(config_path: str) -> str:
    """Run `lox config describe` + `lox config devices --ports`."""
    lox = _find_lox()
    parts = []
    for cmd_args in [
        lox + ["config", "describe", config_path],
        lox + ["config", "devices", "--ports", config_path],
    ]:
        try:
            r = subprocess.run(
                cmd_args, capture_output=True, text=True, timeout=30
            )
            parts.append(r.stdout.strip())
        except Exception as exc:
            parts.append(f"(error running {' '.join(cmd_args)}: {exc})")
    return "\n\n".join(parts)


def _load_skill(name: str) -> str:
    """Load a skill SKILL.md file, stripping YAML frontmatter."""
    path = SKILL_DIR / name / "SKILL.md"
    if not path.exists():
        return f"(skill {name} not found at {path})"
    text = path.read_text()
    # Strip YAML frontmatter (--- ... ---)
    if text.startswith("---"):
        end = text.find("---", 3)
        if end != -1:
            text = text[end + 3:].lstrip("\n")
    return text


def _build_instructions(case: dict, config_path: str) -> str:
    """Build workflow instructions, incorporating hints if available."""
    hint_block = ""
    if case.get("hint"):
        hint_block = f"""
## Recommended Approach (follow these steps closely)
{case['hint'].replace('FILE', config_path)}
"""

    return f"""\
{hint_block}
## Your Workflow (follow this EXACT order)

1. **SEARCH**: Find block types: `lox blocks search "keyword" -o json`
2. **ADD**: Create all needed blocks with `lox config add`
3. **⚠ WIRE (CRITICAL)**: Wire EVERY connection with `lox config wire-connector`.
   - Every logic block output MUST connect to the next block or actuator
   - Every logic block input MUST connect from a sensor or upstream block
   - A block with unwired outputs does NOTHING — the circuit is broken
   - Run `lox config check {config_path}` — if it shows "unwired input" errors, ADD MORE WIRES
4. **PARAMS**: Set all parameters: `lox config set-param`
5. **CHECK**: `lox config check {config_path}` — zero errors required
6. **DONE**: Output the word DONE

## Rules
- Output `lox` commands one per line
- You may output multiple rounds of commands — each will be executed
- After executing, you'll see the stdout/stderr of each command
- ⚠ NEVER skip wiring — blocks without wires are useless
- When satisfied, output the single word: DONE
- Do NOT explain — just output commands or DONE
"""


def build_llm_prompt(case, config_path: str) -> str:
    config_skill = _load_skill("loxone-config")
    fixture_desc = _describe_fixture(config_path)

    return f"""\
{config_skill}

## Current Config
File: {config_path}

{fixture_desc}

## Task
{case['utterance']}
{_build_instructions(case, config_path)}
"""


def build_retry_prompt(commands_run: list[str], check_output: str, config_path: str) -> str:
    return f"""\
The commands you ran produced validation errors.

## Commands Executed
{chr(10).join(commands_run)}

## Validation Output
{check_output}

## Instructions
Fix the issues by outputting corrected or additional `lox config` commands.
The config file is: {config_path}
End with: lox config validate {config_path}
Respond ONLY with the CLI commands, one per line.
"""


# ── Command Parsing & Execution ─────────────────────────────

_CMD_RE = re.compile(r"^\s*(lox\s+(?:config|sim)\s+.+)$", re.MULTILINE)


def parse_commands(llm_text: str) -> list[str]:
    """Extract `lox config …` and `lox sim …` commands from LLM output."""
    # Strip markdown fences and normalize escaped quotes
    clean = llm_text.replace('\\"', '"').replace("\\'", "'")
    return _CMD_RE.findall(clean)


def execute_commands(commands: list[str], tracker: CLITracker, cwd=None):
    """Run a list of shell commands via the tracker, returning outputs."""
    outputs = []
    for cmd_str in commands:
        try:
            args = shlex.split(cmd_str)
        except ValueError:
            args = cmd_str.split()
        start = time.monotonic()
        result = subprocess.run(
            args, capture_output=True, text=True, cwd=cwd, timeout=60
        )
        elapsed = time.monotonic() - start

        tracker.invocations.append({
            "command": cmd_str,
            "exit_code": result.returncode,
            "stdout_len": len(result.stdout),
            "stderr_len": len(result.stderr),
        })
        if "validate" in cmd_str:
            tracker.validation_runs += 1
            if result.returncode != 0 or "✗" in result.stdout:
                tracker.validation_errors += 1

        outputs.append({
            "command": cmd_str,
            "returncode": result.returncode,
            "stdout": result.stdout,
            "stderr": result.stderr,
        })
    return outputs


# ── Single-Case Execution ───────────────────────────────────

def run_case(case, work_dir: Path, client, model: str, verbose: bool = False):
    """
    Execute the LLM agent for one eval case with multi-turn feedback.

    Loop: build → check → sim test → fix → repeat (up to MAX_RETRIES)

    Returns: (result_path, tracker, token_usage)
    """
    lox_cmd = _find_lox()
    tracker = CLITracker()

    config_path = work_dir / f"{case['id']}.Loxone"
    shutil.copy2(str(FIXTURE), str(config_path))

    prompt = build_llm_prompt(case, str(config_path))

    messages = [{"role": "user", "content": prompt}]
    total_input_tokens = 0
    total_output_tokens = 0
    all_commands_run: list[str] = []

    for attempt in range(1 + MAX_RETRIES):
        if verbose:
            tag = "retry" if attempt > 0 else "initial"
            print(f"    [{tag}] calling {model}…", file=sys.stderr)

        response = client.chat.completions.create(
            model=model,
            messages=messages,
            temperature=0.2,
        )

        reply = response.choices[0].message.content or ""
        usage = response.usage
        if usage:
            total_input_tokens += usage.prompt_tokens or 0
            total_output_tokens += usage.completion_tokens or 0

        commands = parse_commands(reply)
        if verbose:
            print(f"    → {len(commands)} commands parsed", file=sys.stderr)

        # Execute all commands
        outputs = execute_commands(commands, tracker, cwd=str(work_dir))
        all_commands_run.extend(commands)

        # Run check (always)
        check_cmd = f"lox config check {config_path}"
        if not any("config check" in c for c in commands):
            check_out = execute_commands([check_cmd], tracker, cwd=str(work_dir))
            all_commands_run.append(check_cmd)
        else:
            check_out = [o for o in outputs if "config check" in o.get("command", "")]

        check_stdout = check_out[-1].get("stdout", "") if check_out else ""
        has_check_errors = "✗" in check_stdout

        # Auto-detect orphaned blocks: if agent added blocks but no wire commands
        has_add = any("config add" in c for c in commands)
        has_wire = any("wire-connector" in c or "config wire " in c for c in commands)
        if has_add and not has_wire and attempt < MAX_RETRIES:
            tracker.retries += 1
            wiring_prompt = (
                "⚠ CRITICAL: You created blocks but did NOT wire any of them!\n"
                "Blocks without wires are useless — the circuit is completely broken.\n\n"
                f"## Validation Output\n{check_stdout}\n\n"
                "## Instructions\n"
                "You MUST add `lox config wire-connector` commands to connect:\n"
                "- Each sensor output to the logic block input\n"
                "- Each logic block output to the next block or actuator\n"
                "- Use: `lox config wire-connector FILE \"Target.Input\" \"Source.Output\"`\n"
                f"- Run `lox sim dump {config_path}` to see all blocks and find connector names\n\n"
                f"The config file is: {config_path}\n"
                "Respond ONLY with `lox config wire-connector` commands, one per line."
            )
            messages.append({"role": "assistant", "content": reply})
            messages.append({"role": "user", "content": wiring_prompt})
            continue

        # If check has errors, retry with check feedback
        if has_check_errors and attempt < MAX_RETRIES:
            tracker.retries += 1
            retry_prompt = build_retry_prompt(
                all_commands_run, check_stdout, str(config_path)
            )
            messages.append({"role": "assistant", "content": reply})
            messages.append({"role": "user", "content": retry_prompt})
            continue

        # Run simulation test if case has sim specs
        sims = case.get("expected", {}).get("simulation", [])
        if sims and attempt < MAX_RETRIES:
            sim_result = run_simulation(case["id"], case, str(config_path))
            if not sim_result.get("pass", True):
                # Build sim feedback
                tracker.retries += 1
                sim_feedback = _build_sim_feedback(sim_result, all_commands_run, str(config_path))
                messages.append({"role": "assistant", "content": reply})
                messages.append({"role": "user", "content": sim_feedback})
                continue

        # All good (or out of retries)
        break

    token_usage = {
        "input_tokens_est": total_input_tokens,
        "output_tokens_est": total_output_tokens,
        "input_chars": total_input_tokens * 4,
        "output_chars": total_output_tokens * 4,
    }

    return config_path, tracker, token_usage


def _build_sim_feedback(sim_result: dict, commands_run: list[str], config_path: str) -> str:
    """Build a retry prompt from simulation test failures."""
    failures = []
    for scenario in sim_result.get("scenarios", []):
        if not scenario.get("pass"):
            for check in scenario.get("checks", []):
                if not check.get("pass"):
                    failures.append(
                        f"  {scenario['name']}: {check['output']} = {check['actual']}"
                        f" (expected {check['comparator']} {check['expected']})"
                    )

    # Run lox blocks search to suggest better block types based on the task
    block_suggestions = ""
    # Extract key intent words from the commands (block types used)
    used_types = set()
    for cmd in commands_run:
        if "--type" in cmd:
            parts = cmd.split("--type")
            if len(parts) > 1:
                type_word = parts[1].strip().split()[0].strip('"').strip("'")
                used_types.add(type_word)

    if used_types:
        lox = _find_lox()
        suggestions = []
        for btype in used_types:
            try:
                r = subprocess.run(
                    lox + ["blocks", "info", btype],
                    capture_output=True, text=True, timeout=10
                )
                if r.stdout and "Don't confuse with" in r.stdout:
                    # Extract the confusion warning
                    for line in r.stdout.split("\n"):
                        if "Don't confuse" in line or ("—" in line and any(t in line for t in ["OnPulseDelay", "StairwayLS", "OffDelay", "Monoflop", "Memory", "FlipFlop"])):
                            suggestions.append(line.strip())
            except Exception:
                pass

        if suggestions:
            block_suggestions = "\n## Block Type Warnings\n" + "\n".join(suggestions)

    # Also run a search for common timer patterns if OnPulseDelay/OffDelay was used
    timer_hint = ""
    if used_types & {"OnPulseDelay", "OffDelay", "OnDelay"}:
        lox = _find_lox()
        try:
            r = subprocess.run(
                lox + ["blocks", "search", "timed light"],
                capture_output=True, text=True, timeout=10
            )
            if r.stdout:
                timer_hint = f"\n## Better Block Types for Timed Switches\n{r.stdout[:500]}"
        except Exception:
            pass

    return f"""\
Your circuit was built but the simulation test FAILED.

## Simulation Failures
{chr(10).join(failures)}

## What This Means
The blocks exist but the signal doesn't flow correctly from sensor to actuator.
Common issues:
- Missing wire between a logic block output and the actuator
- Wrong block type: OnPulseDelay WAITS before pulsing (use StairwayLS for immediate timed switch)
- OffDelay keeps output on AFTER input drops (use StairwayLS for "on for N seconds on trigger")
- Sensor wired to wrong input connector
{block_suggestions}
{timer_hint}
## Commands Already Run
{chr(10).join(commands_run[-10:])}

## Instructions
1. Consider replacing OnPulseDelay/OffDelay with StairwayLS if the task is "turn on for X minutes"
2. Check if all outputs are wired to actuators
3. Run `lox sim dump {config_path}` to inspect wire values
4. Fix the wiring or block issues

The config file is: {config_path}
End with: lox config check {config_path}
Respond ONLY with the CLI commands, one per line.
"""


# ── Main ─────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="LLM-powered Loxone Eval Agent")
    parser.add_argument("--case", help="Run a single case by ID")
    parser.add_argument("--all", action="store_true", help="Run all cases")
    parser.add_argument("--filter", help="Filter cases by pattern/difficulty/keyword")
    parser.add_argument("--section", help="Only run a specific case section")
    parser.add_argument("--max-cases", type=int, help="Max cases to run")
    parser.add_argument(
        "--model", default="gpt-4o", help="Model name (default: gpt-4o)"
    )
    parser.add_argument(
        "--output",
        default="tests/eval/reports/llm-report.json",
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
    args = parser.parse_args()

    # ── Pretty-print mode ──
    if args.report:
        with open(args.report) as f:
            report = json.load(f)
        print_report(report)
        return

    if not args.case and not args.all:
        parser.print_help()
        sys.exit(2)

    # ── Load cases ──
    if args.case:
        cases = load_cases()
        cases = [c for c in cases if c["id"] == args.case]
        if not cases:
            print(f"Case '{args.case}' not found", file=sys.stderr)
            sys.exit(2)
    else:
        cases = load_cases(args.filter, args.section, args.max_cases)

    print(f"LLM Eval Agent — model={args.model}  cases={len(cases)}")
    print(f"Fixture: {FIXTURE}")
    print()

    # ── OpenAI client ──
    client = _create_client()

    # ── Work directory ──
    if args.work_dir:
        work_dir = Path(args.work_dir)
        work_dir.mkdir(parents=True, exist_ok=True)
    else:
        work_dir = Path(tempfile.mkdtemp(prefix="lox-llm-eval-"))

    results = []
    passed = 0
    failed = 0

    for i, case in enumerate(cases):
        case_id = case["id"]
        sys.stdout.write(f"  [{i + 1}/{len(cases)}] {case_id:42s} ")
        sys.stdout.flush()

        try:
            result_path, tracker, tokens = run_case(
                case, work_dir, client, args.model, verbose=args.verbose
            )

            eval_result = evaluate_correctness(FIXTURE, result_path, case)

            # Run simulation (behavioral test — the real metric)
            sim_result = run_simulation(case_id, case, str(result_path))
            eval_result["simulation"] = sim_result
            sim_pass = sim_result.get("pass", False)
            sim_total = sim_result.get("total_count", 0)
            sim_passed = sim_result.get("passed_count", 0)

            # Override pass/fail: simulation is the primary metric
            if sim_total > 0:
                eval_result["sim_pass"] = sim_pass
                eval_result["sim_score"] = sim_passed / sim_total if sim_total else 0
            else:
                eval_result["sim_pass"] = None
                eval_result["sim_score"] = None

            eval_result["case_id"] = case_id
            eval_result["difficulty"] = case.get("difficulty", "medium")
            eval_result["patterns"] = case.get("pattern", [])
            eval_result["utterance"] = case["utterance"]
            eval_result["cli_invocations"] = tracker.summary()["total_invocations"]
            eval_result["retries"] = tracker.summary()["retries"]
            eval_result["validation_runs"] = tracker.summary()["validation_runs"]
            eval_result["tokens"] = tokens
            eval_result["model"] = args.model

            results.append(eval_result)

            if eval_result.get("sim_pass"):
                passed += 1
                m = eval_result["metrics"]
                chk = "clean" if eval_result.get("validation_pass") else "dirty"
                print(
                    f"\033[32m✓ PASS\033[0m  sim={sim_passed}/{sim_total}  check={chk}  "
                    f"(struct: B={m['blocks']['f1']:.0%} P={m['params']['accuracy']:.0%})"
                )
            elif sim_total > 0:
                failed += 1
                m = eval_result["metrics"]
                chk = "clean" if eval_result.get("validation_pass") else "dirty"
                print(
                    f"\033[31m✗ FAIL\033[0m  sim={sim_passed}/{sim_total}  check={chk}  "
                    f"(struct: B={m['blocks']['f1']:.0%} P={m['params']['accuracy']:.0%})"
                )
            elif eval_result.get("validation_pass"):
                passed += 1
                m = eval_result["metrics"]
                print(
                    f"\033[32m✓ PASS\033[0m  check=clean  "
                    f"(struct: B={m['blocks']['f1']:.0%} P={m['params']['accuracy']:.0%})  (no sim spec)"
                )
            else:
                failed += 1
                m = eval_result["metrics"]
                print(
                    f"\033[31m✗ FAIL\033[0m  check=dirty  "
                    f"(struct: B={m['blocks']['f1']:.0%} P={m['params']['accuracy']:.0%})"
                )

        except Exception as e:
            failed += 1
            results.append(
                {
                    "case_id": case_id,
                    "pass": False,
                    "overall_score": 0,
                    "error": str(e),
                    "difficulty": case.get("difficulty", "medium"),
                    "patterns": case.get("pattern", []),
                    "utterance": case["utterance"],
                    "metrics": {
                        "blocks": {
                            "precision": 0,
                            "recall": 0,
                            "f1": 0,
                            "true_positives": 0,
                            "false_positives": 0,
                            "false_negatives": 0,
                        },
                        "wiring": {
                            "accuracy": 0,
                            "precision": 0,
                            "correct": 0,
                            "total": 0,
                            "extra": 0,
                        },
                        "params": {"accuracy": 0, "correct": 0, "total": 0},
                        "ux": {"score": 0, "issues": []},
                    },
                    "cli_invocations": 0,
                    "retries": 0,
                    "tokens": {"input_tokens_est": 0, "output_tokens_est": 0},
                    "model": args.model,
                }
            )
            print(f"\033[33m⚠ ERROR\033[0m  {e}")

    # ── Generate report ──
    report = generate_report(results)
    report["meta"] = {
        "model": args.model,
        "agent": "llm-agent",
        "max_retries": MAX_RETRIES,
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
