#!/usr/bin/env python3
"""
Loxone Eval — LLM Agent Prompt Builder

Generates the prompt for an LLM agent to implement a given eval case.
Used by the orchestrator (run from Copilot CLI) to build sub-agent prompts.

Usage:
  python3 tests/eval/build_prompt.py <case-id> <config-path>
"""

import json
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
EVAL_DIR = SCRIPT_DIR.parent
CASES_INDEX = EVAL_DIR / "cases-index.json"
SKILL_FILE = EVAL_DIR.parent.parent / "skills" / "loxone-automation" / "SKILL.md"
PATTERNS_FILE = EVAL_DIR.parent.parent / "skills" / "loxone-automation" / "references" / "PATTERNS.md"


def load_case(case_id):
    with open(CASES_INDEX) as f:
        index = json.load(f)
    
    for category, meta in index['categories'].items():
        case_file = EVAL_DIR / meta['file']
        with open(case_file) as f:
            cases = json.load(f)
            for c in cases:
                if c["id"] == case_id:
                    return c
    return None


def build_prompt(case, config_path):
    skill = SKILL_FILE.read_text() if SKILL_FILE.exists() else ""
    patterns = PATTERNS_FILE.read_text() if PATTERNS_FILE.exists() else ""

    return f"""You are a Loxone home automation agent. Your job is to modify a .Loxone config file using ONLY the `lox` CLI tool. Never edit XML directly.

## Task
Apply this automation request to the config file at `{config_path}`:

"{case['utterance']}"

## Workflow
1. Understand: `cargo run --quiet -- config describe {config_path}`
2. Find controls: `cargo run --quiet -- config controls {config_path}` (add `-r <room>` to filter)
3. Get connector UUIDs: `cargo run --quiet -- config control describe {config_path} "<block>"`
4. Create blocks: `cargo run --quiet -- config add --type <type> --title <name> --room <room> --page <page> {config_path}`
5. Set parameters: `cargo run --quiet -- config set-param {config_path} <selector> <param> <value>`
6. Wire: `cargo run --quiet -- config wire-connector {config_path} "<target>.<connector>" <source-uuid>`
7. Check: `cargo run --quiet -- config check {config_path}` — fix any ✗ errors (missing params, unwired blocks)
8. Validate: `cargo run --quiet -- config validate {config_path}`

## Rules
- Always specify --page and --room when adding blocks
- Use bracket syntax for ambiguous names: "Lichtsteuerung [Wohnzimmer]"
- Get connector UUIDs via `config control describe` BEFORE wiring
- The source UUID goes as the LAST argument to wire-connector
- Validate after changes

## Skill Reference
{skill}

## Automation Patterns
{patterns}

The working directory is /home/amy/src/lox. Run all commands from there.
Do NOT ask questions. Just implement the automation and validate.
When done, say DONE."""


def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <case-id> <config-path>", file=sys.stderr)
        sys.exit(2)

    case_id = sys.argv[1]
    config_path = sys.argv[2]

    case = load_case(case_id)
    if not case:
        print(f"Case '{case_id}' not found", file=sys.stderr)
        sys.exit(1)

    print(build_prompt(case, config_path))


if __name__ == "__main__":
    main()
