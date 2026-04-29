# lox — Config-as-Code for Loxone Miniserver

[![CI](https://github.com/eisber/lox-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/eisber/lox-cli/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/Rust-1.91%2B-orange.svg?logo=rust)](https://www.rust-lang.org/)

**Configure your Loxone Miniserver entirely from the command line.**
Rooms, controls, wiring, parameters, device binding, templates — no desktop app required.
Download config → edit → validate → push. All headless. All scriptable. All version-controlled.

---

## Config-as-Code

The core idea: treat your Miniserver configuration like source code. Download it, edit it with semantic commands, validate it, push it back — in scripts, CI/CD pipelines, or AI agent workflows.

```bash
# Download & inspect
lox config download --extract          # Download config XML from Miniserver
lox config describe config.Loxone      # Human-readable summary by room
lox config devices config.Loxone --ports  # Hardware I/O with used/free status

# Create rooms & controls
lox config room add config.Loxone "DG Schlafzimmer"
lox config template config.Loxone bedroom --room "DG Schlafzimmer"
#   ✓ DG Schlafzimmer Licht (LightController2)
#   ✓ DG Schlafzimmer Beschattung (JalousieUpDown2)
#   ✓ DG Schlafzimmer Wecker (AlarmClock)

# Wire controls to hardware
lox config device-bind config.Loxone "DG Schlafzimmer Licht" AQ1 \
  --device "RGBW 24V Dimmer Tree"

# Configure parameters
lox config set-param config.Loxone "DG Schlafzimmer Licht" FadingTime 2.0

# Add logic blocks & wire them
lox config add --type And --title "Both Sensors" config.Loxone
lox config wire-connector config.Loxone "Both Sensors.I1" <sensor-uuid>

# Validate & deploy
lox config validate config.Loxone      # Check UUIDs, wiring, orphans
lox config layout config.Loxone        # Auto-arrange blocks (UX Ausrichten grid)
lox config push config.Loxone --reboot --force  # Upload + fast SPS reload (~4s)
```

**No desktop app needed.** The CLI handles LoxCC compression with correct CRC32 checksums, UUID generation, connector maps for 190+ block types, and wiring validation.

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

## Eval Results: 285/285 (100%)

The eval harness tests whether an AI agent can correctly configure a Loxone Miniserver from natural language instructions. Each case: utterance → agent builds circuit via CLI → Rust simulator verifies signals propagate correctly.

**285 test cases across 10 categories — all passing:**

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

### How Evals Work

Each eval case is a JSON spec with an utterance and simulation tests:

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
1. **Agent** receives the utterance + fixture config + skill reference
2. **Agent** uses `lox config add`, `wire-connector`, `set-param` to build the circuit
3. **Simulator** runs `lox sim run config.Loxone --sim '...'` to verify signal propagation
4. **Pass/fail** based on whether outputs match expected values

### Sample Transcript

**Utterance:** *"Set up the kitchen so the light comes on when it gets dark, both blinds go up in strong wind, and they only go down after five minutes of strong sunshine if there is no wind."*

**Agent builds:**
```
Helligkeit.AQ → Less "Dunkel" (< 100 lux) → Lichtsteuerung [Küche].I1
Windgeschwindigkeit.AQ → GreaterEqual "Starker Wind" (≥ 40 km/h) → Jalousie 1 + 2 [Küche].InputTriggerUp
Sonnenschein.AQ + (Wind < 5 km/h) → And "Sonne ohne Wind" → OnPulseDelay 300s → Jalousie 1 + 2 [Küche].InputTriggerDown
```

**Simulation verifies:**
- Wind 45 km/h → blinds go UP ✅
- Wind 5 km/h → blinds stay ✅
- 5 min sunshine + no wind → blinds go DOWN ✅

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
