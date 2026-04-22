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
FIXTURE = EVAL_DIR / "fixture.Loxone"
COMMANDS_MD = EVAL_DIR.parent.parent / "COMMANDS.md"

# Import helpers from sibling agent-runner.py
sys.path.insert(0, str(SCRIPT_DIR))
from importlib import import_module as _imp

_agent_runner = _imp("agent-runner")
load_cases = _agent_runner.load_cases
evaluate_correctness = _agent_runner.evaluate_correctness
generate_report = _agent_runner.generate_report
print_report = _agent_runner.print_report
CLITracker = _agent_runner.CLITracker

MAX_RETRIES = 3


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


def build_llm_prompt(case, config_path: str) -> str:
    fixture_desc = _describe_fixture(config_path)

    return f"""\
You are a Loxone home automation expert. You configure Miniservers using the `lox` CLI.

## Commands

### Add a block
lox config add --type TYPE --title TITLE --room ROOM --page PAGE FILE

### Set a parameter on a block
lox config set-param FILE "BlockTitle" ParamName Value

### Wire two connectors together
lox config wire-connector FILE "TargetBlock.InputConnector" "SourceBlock.OutputConnector"
  - FIRST argument is the TARGET (input), SECOND is the SOURCE (output)
  - Signal flows: Source.Output → Target.Input

### Verify your work
lox config check FILE

## Block Type Reference (connectors)

| Type | Inputs | Outputs | Params |
|------|--------|---------|--------|
| GreaterEqual | Input1 (analog) | Q (1 if Input1 ≥ Input2) | Input2=threshold |
| Less | Input1 (analog) | Q (1 if Input1 < Input2) | Input2=threshold |
| Greater | Input1 (analog) | Q (1 if Input1 > Input2) | Input2=threshold |
| And | I1, I2 (digital) | Q (1 if all inputs >0.5) | |
| Or | I1, I2 (digital) | Q (1 if any input >0.5) | |
| Not | I (digital) | Q (inverted) | |
| Mult | Input1, Input2 | AQ (product) | Input2=factor |
| Add | Input1, Input2 | AQ (sum) | |
| Sub | Input1, Input2 | AQ (difference) | |
| Memory | S (set), R (reset) | Q (stored value) | |
| FlipFlop | InputS (set), InputR (reset) | Q (toggle) | |
| Monoflop | InputTrigger | Q (pulse for Time seconds) | Time |
| OnPulseDelay | InputTrigger | Q (delayed pulse) | Time |
| StairwayLS | InputTrigger, On | Q (timed switch) | TimeHigh |
| OffDelay | InputTrigger | Q (stays on for Time after off) | Time |
| DayTimer | InputTrigger | AQ, Qon, Qoff | (schedule in UI) |
| PulseGen | InputEnable | Q (periodic pulse) | Period |
| State | I1..I20 | AQ, TQ (selected state) | |

## Fixture Sensors (wire FROM these .AQ/.Q outputs)

| Sensor | Output | Signal |
|--------|--------|--------|
| Außentemperatur | .AQ | Temperature °C |
| Sonnenschein | .AQ | Sunshine 0/1 |
| Windgeschwindigkeit | .AQ | Wind speed km/h |
| Regen | .AQ | Rain 0/1 |
| Luftfeuchtigkeit | .AQ | Humidity % |
| Helligkeit | .AQ | Brightness lux |
| CO2 Sensor | .AQ | CO2 ppm |
| Bewegungsmelder | .OutputPresence | Motion 0/1 |
| Türkontakt Eingang | .Q | Door contact 0/1 |
| Türklingel | .Q | Doorbell pulse |
| Pool Temperatur | .AQ | Pool temp °C |
| Raumtemperatur Wohnzimmer | .AQ | Room temp °C |
| Schalter 1 | .Q | Switch 0/1 |

## Fixture Actuators (wire TO these inputs)

| Actuator | Room | Key Inputs |
|----------|------|------------|
| Jalousie 1, Jalousie 2 | each room | .InputTriggerDown, .InputTriggerUp, .InputDisable |
| Lichtsteuerung | each room | .I1 (on/off), .Presence, .Brightness |
| Klimaanlage | Wohnzimmer | .toggle |
| Poolpumpe | Garten | .I1 |
| Lüfter Bad | Bad | .I1 |
| Raumregler | Wohnzimmer | .Temp |

## Worked Example

Task: "Close blinds when it's sunny and above 25°C"

```
lox config add --type GreaterEqual --title "Temp über 25" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config set-param config.Loxone "Temp über 25" Input2 25
lox config add --type And --title "Sonne und Warm" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config wire-connector config.Loxone "Temp über 25.Input1" "Außentemperatur.AQ"
lox config wire-connector config.Loxone "Sonne und Warm.I1" "Sonnenschein.AQ"
lox config wire-connector config.Loxone "Sonne und Warm.I2" "Temp über 25.Q"
lox config wire-connector config.Loxone "Jalousie 1 [Wohnzimmer].InputTriggerDown" "Sonne und Warm.Q"
lox config check config.Loxone
```

## Current Config
File: {config_path}

{fixture_desc}

## Task
{case['utterance']}

Respond ONLY with `lox config` commands, one per line. No explanations.
End with: lox config check {config_path}
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

_CMD_RE = re.compile(r"^\s*(lox\s+config\s+.+)$", re.MULTILINE)


def parse_commands(llm_text: str) -> list[str]:
    """Extract `lox config …` commands from LLM output."""
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
    Execute the LLM agent for one eval case.

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

        # Separate validate from mutating commands
        mutating = [c for c in commands if "validate" not in c]
        validates = [c for c in commands if "validate" in c]

        # Execute mutating commands first
        outputs = execute_commands(mutating, tracker, cwd=str(work_dir))
        all_commands_run.extend(mutating)

        # Run validation (always, even if LLM forgot)
        validate_cmd = f"lox config validate {config_path}"
        val_outputs = execute_commands(
            validates if validates else [validate_cmd], tracker, cwd=str(work_dir)
        )
        if not validates:
            all_commands_run.append(validate_cmd)
        else:
            all_commands_run.extend(validates)

        # Check for errors
        val_out = val_outputs[-1] if val_outputs else {}
        val_stdout = val_out.get("stdout", "")
        val_rc = val_out.get("returncode", 1)
        has_errors = val_rc != 0 or "✗" in val_stdout

        if not has_errors or attempt >= MAX_RETRIES:
            break

        # Retry: feed errors back to LLM
        tracker.retries += 1
        retry_prompt = build_retry_prompt(
            all_commands_run, val_stdout + val_out.get("stderr", ""), str(config_path)
        )
        messages.append({"role": "assistant", "content": reply})
        messages.append({"role": "user", "content": retry_prompt})

    token_usage = {
        "input_tokens_est": total_input_tokens,
        "output_tokens_est": total_output_tokens,
        "input_chars": total_input_tokens * 4,
        "output_chars": total_output_tokens * 4,
    }

    return config_path, tracker, token_usage


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

            if eval_result["pass"]:
                passed += 1
                print(
                    f"\033[32m✓ PASS\033[0m  {eval_result['overall_score']:.0%}"
                )
            else:
                failed += 1
                m = eval_result["metrics"]
                print(
                    f"\033[31m✗ FAIL\033[0m  {eval_result['overall_score']:.0%}  "
                    f"B={m['blocks']['f1']:.0%} W={m['wiring']['accuracy']:.0%} P={m['params']['accuracy']:.0%}"
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
