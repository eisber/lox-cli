# lox — AI Agent Tooling for Loxone Miniserver

[![CI](https://github.com/eisber/lox-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/eisber/lox-cli/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.91%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Eval](https://img.shields.io/badge/eval-94%25%20pass-brightgreen.svg)](#eval-results)

**Tell an AI agent what you want. It configures your Loxone Miniserver.**

*"When humidity in the bathroom goes above 70%, turn on the fan and keep it running for 5 minutes after humidity drops."* → Agent searches block types, adds a threshold comparator + off-delay timer, wires them to the fan, and verifies the circuit with a built-in SPS simulator — all in 90 seconds.

Built for [GitHub Copilot CLI](https://docs.github.com/en/copilot/github-copilot-in-the-cli), [Claude Code](https://docs.anthropic.com/en/docs/agents-and-tools/claude-code/overview), [OpenCode](https://opencode.ai), and any agent that can run shell commands.

### How It Works

```
User: "Close the living room blinds when it's sunny and above 25°C"
  ↓
Agent reads skill reference (.github/skills/loxone-config/)
  ↓
Agent runs CLI commands:
  lox blocks search "threshold"           → GreaterEqual
  lox config add --type GreaterEqual ...  → adds block
  lox config wire-connector ...           → wires sensor → logic → actuator  
  lox sim run config.Loxone --sim '...'   → ✅ signal propagates correctly
  ↓
Config ready to deploy: lox config push config.Loxone --reboot
```

**94% pass rate** on 285 behavioral eval cases (5 sections at 100%). [See eval results →](#eval-results)

---

## Config-as-Code

The CLI treats your Miniserver configuration as code — download, edit with semantic commands, validate, push back. Works in scripts, CI/CD, or AI agent workflows.

```bash
# Download & inspect
lox config download --extract          # Download config XML from Miniserver
lox config describe config.Loxone      # Human-readable summary by room

# Add logic blocks & wire them
lox config add --type GreaterEqual --title "Temp über 25" config.Loxone
lox config set-param config.Loxone "Temp über 25" Input2 25
lox config wire-connector config.Loxone "Temp über 25.Input1" "Außentemperatur.AQ"
lox config wire-connector config.Loxone "Jalousie 1 [Wohnzimmer].InputTriggerDown" "Temp über 25.Q"

# Validate & test
lox config check config.Loxone         # Check wiring completeness
lox sim run config.Loxone --sim '{"inputs":{"Außentemperatur":30},...}'  # Simulate

# Deploy
lox config push config.Loxone --reboot --force  # Upload + SPS reload (~4s)
```

**No desktop app needed.** Handles LoxCC compression, CRC32 checksums, UUID generation, connector maps for 190+ block types, and wiring validation.

---

## Room Templates

Standard presets for common room types. Each creates appropriate controls with sensible parameter defaults:

```bash
lox config template config.Loxone standard --room "EG Wohnzimmer"
lox config template config.Loxone bedroom --room "DG Schlafzimmer"
lox config template config.Loxone bathroom --room "EG Bad"
```

Templates: `standard`, `bedroom`, `bathroom`, `hallway`, `kitchen`, `outdoor`, `office`.
German aliases work too: `schlafzimmer`, `badezimmer`, `flur`, `küche`, `büro`.

---

## Config Versioning (GitOps)

Track configuration changes in git with semantic commit messages:

```bash
lox config init ~/loxone-config        # Initialize git repo
lox config pull                        # Download, diff, commit with meaningful message
lox config log                         # Show change history
lox config restore abc123 --force      # Restore from git history
```

Each pull generates commits like:
```
[504F94AABBCC] Config backup 2026-04-19 (v267)
+ Added control: "Küche Licht" (LightController2)
~ Renamed: "Licht EG" → "Licht Erdgeschoss"
- Removed user: "guest"
```

---

## SPS Simulator (lox-sim)

Offline Miniserver SPS simulator for testing config changes without hardware:

- **195 block types** — logic, math, lighting, HVAC, timers, I/O
- **JIT-compiled engine** — topological sort, batch evaluation, cycle detection
- **Multi-step temporal specs** — test heating cycles, timer delays, schedule transitions
- **Time injection** — set `minutes_since_midnight` on all DayTimers for schedule testing
- **Structured trace** — JSON output of all signal values for auto-discovery
- **6k lines of Rust** in `lox-sim/` with 367 unit tests

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

Designed for LLM agent integration with skill references in `.github/skills/`:

- **loxone-config** — CLI commands, block types, worked examples, common mistakes
- **loxone-sim** — simulator testing commands
- **loxone-patterns** — 13 automation recipes (threshold, timer, schedule, HVAC)

The agent reads the skill, searches block types with `lox blocks search`, builds circuits, and self-tests with `lox sim run`:

```bash
# Agent workflow
lox blocks search "timer"              # Find the right block type
lox config add --type StairwayLS --title "Treppenlicht" config.Loxone
lox config set-param config.Loxone "Treppenlicht" TimeHigh 300
lox config wire-connector config.Loxone "Treppenlicht.InputTrigger" "Bewegungsmelder.OutputPresence"
lox config wire-connector config.Loxone "Lichtsteuerung [Flur].I1" "Treppenlicht.Q"
lox sim run config.Loxone --sim '{"inputs":{"Bewegungsmelder.OutputPresence":1},...}'
```

All errors include structured suggestions, fuzzy matching, and available options.

---

## Install

**Build from source (all platforms):**
```bash
git clone https://github.com/eisber/lox-cli
cd lox-cli
cargo build --release
# Binary at target/release/lox (~14MB)
```

**Requirements:** Rust 1.91+, Loxone Miniserver Gen 1/2 (firmware 12.0+), local network access.

## Setup

```bash
lox setup set --host https://192.168.1.100 --user USER --pass PASS
```

### Multiple Miniservers

```bash
lox ctx add home --host https://192.168.1.100 --user admin --pass secret
lox ctx add office --host https://10.0.0.50 --user admin --pass secret
lox ctx use home
```

---

## Architecture

Single static Rust binary (~14MB). No runtime dependencies. Works on Windows, macOS, and Linux.

```
~/.lox/
  config.yaml          # Host, credentials, aliases
  cache/structure.json # LoxApp3.json (24h TTL)
  contexts/            # Per-Miniserver data
```

See **[COMMANDS.md](COMMANDS.md)** for the full command reference, **[DESIGN.md](DESIGN.md)** for architecture details, and **[AGENTS.md](AGENTS.md)** for AI agent integration guidance.

---

## Status

This project is an experiment. Expect rough edges. **Use at your own risk** — commands that modify your Miniserver can affect your live system. Always have a backup.

## License

Dual-licensed: [AGPL-3.0](LICENSE-AGPL) for open use, [Commercial License](LICENSE-COMMERCIAL) for proprietary redistribution.

Copyright © 2025-2026 Markus Cozowicz
