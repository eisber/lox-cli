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
- **Autodiff** — automatic differentiation for parameter sensitivity analysis
- **6k lines of Rust** in `lox-sim/` with benchmarks

---

## Eval Harness

Behavioral test suite for config editing commands (`tests/eval/`):

- Structured JSON test cases covering add, wire, template, validate workflows
- Fixture-based: each test runs against `fixture.Loxone`
- Automated pass/fail with structured assertions

---

## For AI Agents

Designed for LLM agent integration. Give an agent a shell tool:

```json
{
  "name": "lox",
  "description": "Configure Loxone smart home. Use -o json for structured output.",
  "parameters": {
    "command": { "type": "string" }
  }
}
```

The agent can:
1. `lox config describe config.Loxone` — understand what's configured
2. `lox config devices config.Loxone --ports` — see available hardware
3. `lox config template config.Loxone bedroom --room "Schlafzimmer"` — create controls
4. `lox config device-bind config.Loxone "Licht" AQ1 --device "Dimmer Tree"` — wire hardware
5. `lox config validate config.Loxone` — check for errors
6. `lox config push config.Loxone --reboot --force` — deploy

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
