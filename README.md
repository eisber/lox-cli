# lox — AI Agent Tooling for Loxone Miniserver

[![CI](https://github.com/eisber/lox-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/eisber/lox-cli/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/lox-cli.svg)](https://crates.io/crates/lox-cli)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Eval](https://img.shields.io/badge/eval-94%25%20pass-brightgreen.svg)](#eval-results)

**Tell an AI agent what you want. It configures your Loxone Miniserver.**

*"Set up the kitchen so the light comes on when dark, blinds go up in strong wind, and they close after five minutes of sunshine if there's no wind."* → Agent searches block types, adds 5 logic blocks, wires 3 circuits, fixes a pulse duration error after sim feedback, and verifies all 7 test scenarios — in under 5 minutes.

<p align="center">
  <img src="docs/assets/demo.gif" alt="Demo: AI agent configures kitchen automation" width="662">
  <br><sub>▶ Click to play — Copilot CLI builds 3 kitchen circuits from a natural language prompt</sub>
</p>

## Why

[Loxone Config](https://www.loxone.com/enen/products/software/loxone-config/) is a 400MB Windows-only desktop app. You drag blocks onto a canvas, wire them by hand, and hope you didn't miss a connection. For a single room with lighting, blinds, and climate you're easily placing 15+ blocks and 30+ wires.

This CLI replaces that with shell commands an AI agent can call. The agent reads a [skill reference](.github/skills/loxone-config/SKILL.md) (block types, connectors, common mistakes), builds the circuit, then self-tests with an offline SPS simulator — catching wiring errors before anything touches your live system.

Works with [GitHub Copilot CLI](https://docs.github.com/en/copilot/github-copilot-in-the-cli), [Claude Code](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview), [OpenCode](https://opencode.ai), or any agent that can run shell commands.

## Install

**Homebrew** (macOS/Linux):
```bash
brew install eisber/tap/lox-cli
```

**Prebuilt binaries** (Linux, macOS, Windows) — see [Releases](https://github.com/eisber/lox-cli/releases):
```bash
curl -LO https://github.com/eisber/lox-cli/releases/latest/download/lox-cli-linux-x86_64
chmod +x lox-cli-linux-x86_64 && sudo mv lox-cli-linux-x86_64 /usr/local/bin/lox
```

**From crates.io:**
```bash
cargo install lox-cli
```

**From source:**
```bash
git clone https://github.com/eisber/lox-cli && cd lox-cli
cargo build --release    # Binary at target/release/lox (~14MB)
```

**Requirements:** Loxone Miniserver Gen 1 or 2 (firmware 12.0+), local network access.

## Quick Start

```bash
# Connect to your Miniserver
lox setup set --host https://192.168.1.100 --user admin --pass secret

# Download & inspect config
lox config download --extract
lox config describe config.Loxone

## How It Works

```
User: "Close the living room blinds when it's sunny and above 25°C"
  ↓
Agent reads skill reference (.github/skills/loxone-config/)
  ↓
Agent runs CLI commands:
  lox blocks search "threshold"           → GreaterEqual
  lox config add --type GreaterEqual ...  → adds block, returns connector UUIDs
  lox config wire-connector ...           → wires sensor → logic → actuator
  lox sim run config.Loxone --sim '...'   → ✅ signal propagates correctly
  ↓
Config ready to deploy: lox config push config.Loxone --reboot
```

The CLI handles LoxCC compression, CRC32 checksums, UUID generation, connector maps for 190+ block types, and wiring validation. All commands support `-o json` for agent consumption.

---

## Config-as-Code

```bash
# Add logic blocks & wire them
lox config add --type GreaterEqual --title "Temp über 25" config.Loxone -o json
# {"ok":true,"uuid":"...","type":"GreaterEqual","connectors":[{"key":"Input1","uuid":"...","direction":"I"},...]

lox config set-param config.Loxone "Temp über 25" Input2 25
lox config wire-connector config.Loxone "Temp über 25.Input1" "Außentemperatur.AQ"
lox config wire-connector config.Loxone "Jalousie 1 [Wohnzimmer].InputTriggerDown" "Temp über 25.Q"

# Validate & test
lox config check config.Loxone -o json     # Structured warnings/errors
lox sim run config.Loxone --sim '{"inputs":{"Außentemperatur":30},...}'

# Deploy
lox config push config.Loxone --reboot --force  # Upload + SPS reload (~4s)
```

Room templates for common setups:
```bash
lox config template config.Loxone bedroom --room "DG Schlafzimmer"
# Creates LightController2, JalousieUpDown2, Thermostat with sensible defaults
```

GitOps versioning:
```bash
lox config pull    # Download → diff → semantic commit message
lox config log     # Change history
```

## SPS Simulator

Offline Miniserver SPS simulator — test config changes without hardware:

- **195 block types** — logic, math, lighting, HVAC, timers, I/O
- **Topological engine** — automatic evaluation order, cycle detection
- **Temporal testing** — multi-step specs for heating cycles, timer delays, schedules
- **6k lines of Rust** in `lox-sim/` with 367 unit tests

```bash
lox sim run config.Loxone --sim '{
  "inputs": {"Außentemperatur": 30, "Sonnenschein": 1},
  "ticks": 10, "dt": 0.1,
  "expected_outputs": {"Jalousie 1 [Wohnzimmer].InputTriggerDown": {">": 0.5}}
}'
# {"pass":true,"passed":1,"scenarios":[{"pass":true,...}]}
```

---

## Eval Results

The eval harness tests whether an AI agent can correctly configure a Loxone Miniserver from natural language instructions. Each case: utterance → agent builds circuit via CLI → Rust simulator verifies signals propagate correctly.

**285 test cases across 10 categories.**

Raw LLM pass rate (agent builds circuit from scratch each run):

| Section | Score | Examples |
|---------|-------|---------|
| Synthetic | 20/20 (100%) | "Close blinds when sunny and above 25°C" |
| Patterns | 25/25 (100%) | Memory blocks, delayed triggers, schedules |
| Reference | 24/24 (100%) | Threshold, negation, fan-out, stairway light |
| Generated | 50/50 (100%) | Natural language variants, German/English |
| Rooms | 15/15 (100%) | Full room configurations (bedroom, bathroom, garden) |
| Extended | 14/15 (93%) | Pushbutton, AutoJalousie, sunrise triggers |
| Advanced | 28/29 (97%) | Multi-room goodnight, frost protection, solar pool |
| Use-Cases | 35/40 (88%) | Wallbox charging, NFC lockers, alarm systems |
| HVAC | 15/18 (83%) | Weather-compensated heating, dewpoint cooling |
| HVAC-Extracted | 26/32 (81%) | IRC zones, fancoil, HRV ventilation, heat pump |
| **Total** | **252/268 (94%)** | Core automation: 5 sections at 100% |

Non-deterministic: scores vary ±5% across runs. The remaining failures are complex HVAC chains and specialized hardware (wallbox billing, NFC lockers) where the agent times out or misses wiring.

### How Evals Work

Each eval case is a JSON spec with an utterance and simulation tests. The fixture config (`fixture.Loxone`) provides pre-configured rooms with sensors (temperature, motion, brightness) and actuators (blinds, lights, fans) — the agent adds logic blocks and wires them:

```json
{
  "id": "s01-piano-protection",
  "utterance": "Lower blinds in the living room when it's sunny and the temperature is over 25 degrees",
  "expected": {
    "simulation": [
      {
        "name": "blinds close when hot+sunny",
        "inputs": {"Außentemperatur": 30, "Sonnenschein": 1},
        "ticks": 10, "dt": 0.1,
        "expected_outputs": {"Jalousie 1 [Wohnzimmer].InputTriggerDown": {">": 0.5}}
      },
      {
        "name": "blinds stay when cold",
        "inputs": {"Außentemperatur": 15, "Sonnenschein": 0},
        "ticks": 10, "dt": 0.1,
        "expected_outputs": {"Jalousie 1 [Wohnzimmer].InputTriggerDown": {"<": 0.5}}
      }
    ]
  }
}
```

The pipeline:
1. **Agent** receives the utterance + fixture config (with sensors & actuators) + skill reference
2. **Agent** uses `lox config add`, `wire-connector`, `set-param` to add logic blocks and wire them
3. **Simulator** injects test values into sensors, ticks the engine, checks actuator outputs
4. **Pass/fail** based on whether the right actuator fires with the right inputs

### Sample Transcript (actual LLM run)

**Utterance:** *"When humidity in the bathroom goes above 70%, turn on the fan. Keep it running for 5 minutes after humidity drops back down."*

The agent reads the skill reference, searches for block types, then builds:

```bash
# 1. Search for the right blocks
lox blocks search "greater equal threshold"
# → GreaterEqual (Threshold comparator)
lox blocks search "off delay timer"
# → OffDelay (keeps output on after input drops)

# 2. Add blocks
lox config add --type GreaterEqual --title "Feuchte hoch" --room Bad config.Loxone
# ✓ Added GreaterEqual 'Feuchte hoch' on page 'Bad'
lox config add --type OffDelay --title "Lüfter Nachlauf" --room Bad config.Loxone
# ✓ Added OffDelay 'Lüfter Nachlauf' on page 'Bad'

# 3. Set parameters
lox config set-param config.Loxone "Feuchte hoch" Input2 70
# ✓ Set 'Input2' to 70 on 'Feuchte hoch'
lox config set-param config.Loxone "Lüfter Nachlauf" Time 300
# ✓ Set 'Time' to 300 on 'Lüfter Nachlauf'

# 4. Wire the signal path
lox config wire-connector config.Loxone "Feuchte hoch.Input1" "Raumfeuchtigkeit Bad.AQ"
# ✓ Wired Raumfeuchtigkeit Bad.AQ → Feuchte hoch.Input1
lox config wire-connector config.Loxone "Lüfter Nachlauf.InputTrigger" "Feuchte hoch.Q"
# ✓ Wired Feuchte hoch.Q → Lüfter Nachlauf.InputTrigger
lox config wire-connector config.Loxone "Lüfter Bad.I1" "Lüfter Nachlauf.Q"
# ✓ Wired Lüfter Nachlauf.Q → Lüfter Bad.I1

# 5. Validate
lox config check config.Loxone
# 1 ok, 0 errors

# 6. Test with simulator
lox sim run config.Loxone --sim '{"inputs":{"Raumfeuchtigkeit Bad":80},
  "ticks":10,"dt":0.1,"expected_outputs":{"Lüfter Bad.I1":{">":0.5}}}'
# {"pass":true,"passed":1,"scenarios":[{"pass":true, ...}]}

lox sim run config.Loxone --sim '{"inputs":{"Raumfeuchtigkeit Bad":50},
  "ticks":10,"dt":0.1,"expected_outputs":{"Lüfter Bad.I1":{"==":0}}}'
# {"pass":true,"passed":1,"scenarios":[{"pass":true, ...}]}
```

**Result:** Humidity 80% → fan ON ✅ · Humidity 50% → fan OFF ✅ · Total: 94 seconds, 6 premium requests.

### Running Evals

```bash
# Single case
python3 tests/eval/scripts/eval-agent.py --case s01-piano-protection --agent copilot

# Full suite (parallel)
python3 tests/eval/scripts/eval-agent.py --all --parallel 2 --agent copilot

# Re-eval saved configs (no LLM calls)
lox sim run saved-config.Loxone --sim '[...]'
```

---

## For AI Agents

Skill references in `.github/skills/` give agents everything they need:

- **loxone-config** — CLI commands, 190+ block types, worked examples, common mistakes
- **loxone-sim** — simulator testing commands and patterns
- **loxone-patterns** — 13 automation recipes (threshold, timer, schedule, HVAC)

All errors include fuzzy matching suggestions and available options:
```
Error: Block 'Threshold' not found.
  Did you mean: GreaterEqual, LessEqual, AnalogThresholdTrigger?
```

### Multiple Miniservers

```bash
lox ctx add home --host https://192.168.1.100 --user admin --pass secret
lox ctx add office --host https://10.0.0.50 --user admin --pass secret
lox ctx use home
```

---

## Architecture

Single static Rust binary (~14MB). No runtime dependencies. Works on Linux, macOS, and Windows.

See **[COMMANDS.md](COMMANDS.md)** for the full command reference, **[DESIGN.md](DESIGN.md)** for architecture details, and **[AGENTS.md](AGENTS.md)** for AI agent integration guidance.

## License

Dual-licensed: [AGPL-3.0](LICENSE-AGPL) for open use, [Commercial License](LICENSE-COMMERCIAL) for proprietary redistribution.

Copyright © 2025-2026 Markus Cozowicz
