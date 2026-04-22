# Agent Instructions

This file provides guidance to Claude Code and other AI agents working with this repository.
See [API_DESIGN_GUIDELINES.md](API_DESIGN_GUIDELINES.md) for CLI command naming, flag conventions, and output format rules.

## Commands

```bash
# Build
cargo build           # debug
cargo build --release # production binary (~4MB)

# Run
cargo run -- <args>   # e.g. cargo run -- ls -o json
./target/release/lox <args>

# Check / Lint
cargo check
cargo clippy

# Test
cargo test
cargo test <test_name>  # run a single test
```

## Pre-push CI Checklist

**ALWAYS run these four checks before committing/pushing.** They mirror `.github/workflows/ci.yml` (ubuntu + macos):

```bash
cargo fmt --check            # formatting
cargo clippy -- -D warnings  # lints (warnings are errors)
cargo build --release        # release build
cargo test                   # all tests
```

Do not push until all four pass. Fix issues and re-run before committing.

## Architecture

Single Rust binary. CLI commands use reqwest blocking; token auth uses tokio + WebSocket for the RSA/AES key exchange.

### Source files

| File | Purpose |
|------|---------|
| `src/main.rs` | All CLI commands (60+), clap argument parsing, helper functions (RGB→HSV, weather/stats binary parsing) |
| `src/client.rs` | `LoxClient` (HTTP) — control resolution, structure cache, categories, global states, operating modes |
| `src/config.rs` | `Config` + `GlobalConfig` — loads/saves config (flat or multi-context), context resolution, project-local `.lox/` discovery |
| `src/config_edit.rs` | `ConfigEditor` — DOM-based XML editing engine: properties, attributes, room moves, wiring, element add/remove, BOM-aware write-back |
| `src/errors.rs` | Rich error types with Levenshtein fuzzy matching, "did you mean?" suggestions, doc links |
| `src/loxcc.rs` | LoxCC compress/decompress — LZ4-style algorithm with CRC32 checksums for Miniserver config archives |
| `src/loxone_xml.rs` | XML parsing: rooms, controls, users, devices, config summary, diff |
| `src/commands/config_cmd.rs` | Config editing commands: rooms, controls, MQTT, wiring, validate, push, add, user CRUD |
| `src/commands/ctx.rs` | `lox ctx` commands — add/use/list/remove/rename/init/migrate contexts |
| `src/gitops.rs` | Git-based config versioning — init, pull (FTP→LoxCC→diff→commit), log, restore workflows |
| `src/stream.rs` | WebSocket binary protocol: state events, weather, keepalive, AES-256-CBC encrypted commands |
| `src/scene.rs` | Scene loading/listing from `~/.lox/scenes/*.yaml` |
| `src/ws.rs` | `LoxWsClient` — async WebSocket connection used by token auth (RSA+AES key exchange handshake) |
| `src/token.rs` | Token auth flow: RSA key exchange, AES-encrypted credential exchange, token storage, HMAC token hashing |

### Key design points

**Control resolution** (`LoxClient::resolve_with_room`): Names are matched against the structure cache using fuzzy substring matching. Resolution order: alias → exact UUID → bracket room qualifier (`"Name [Room]"`) → `--room` flag → fuzzy substring. Ambiguous matches are an error.

**Multi-context config**: `~/.lox/config.yaml` supports both flat (single-Miniserver, backward compatible) and multi-context format. `Config::load()` resolution: `LOX_CONFIG` env → project-local `.lox/` (walk up from cwd) → global config → `--ctx` flag override. Each context gets isolated data under `~/.lox/contexts/<name>/`.

**Structure cache**: `LoxApp3.json` (~150KB) is cached per-context (e.g. `~/.lox/contexts/<name>/cache/structure.json`) with a 24h TTL. All commands that need control UUIDs load this cache first; `lox cache refresh` forces a re-fetch.

**Mixed sync/async**: The CLI commands use `reqwest::blocking`. `main.rs` uses `#[tokio::main]` because `lox token fetch` needs async (WebSocket for the key exchange). The blocking reqwest client spawns its own thread pool so both modes coexist.

**Token auth**: RSA public key fetched from Miniserver → encrypt AES session key → send encrypted credentials via WebSocket → receive token. Token stored per-context (e.g. `~/.lox/contexts/<name>/token.json`), valid ~20 days.

**Config editing** (`ConfigEditor`): DOM-based XML editing via `xmltree` crate. Element selector syntax: `"Title"` (fuzzy), `"Type:WeatherData"`, `"uuid:abc-123"`, `"gid:Mqtt"`. Write-back preserves UTF-8 BOM and line endings via post-processing. LoxCC compression uses literals-only LZ4 with CRC32 checksums (required for the Miniserver to trust password fields).

**LoxCC format**: Binary config archive with 16-byte header (magic `0xAABBCCEE`, compressed size, uncompressed size, CRC32). The CRC32 at offset 12 is critical — zero CRC causes the Miniserver to ignore encrypted `t="15"` password fields. Plaintext `t="11"` passwords are accepted by the Miniserver directly.

### Pipeline scripts

| Script | Purpose | When to run |
|--------|---------|-------------|
| `scripts/scrape-docs.py` | Fetch loxone.com/enen/kb/ articles → markdown in `docs/kb/` | New Loxone Config version |
| `scripts/extract-strings.py` | Parse `LoxoneConfigres_*.dll` → `i18n/<locale>.json` | New Loxone Config version |
| `scripts/build-templates.py` | Generate type schemas from `.Loxone` XML → `docs/templates.json` | After config changes |

### User data layout

```
~/.lox/
  config.yaml          # flat (single-Miniserver) or multi-context format
  contexts/            # per-context data (multi-context mode)
    <name>/
      cache/
        structure.json # LoxApp3.json cache (24h TTL)
      token.json       # optional token auth
      scenes/*.yaml    # multi-step scene definitions
  # Legacy flat mode (backward compatible):
  cache/
    structure.json     # LoxApp3.json cache (24h TTL)
  token.json           # optional token auth
  scenes/*.yaml        # multi-step scene definitions
```

Project-local config (auto-discovered by walking up from cwd):
```
project/
  .lox/
    config.yaml        # connection settings
    .gitignore         # excludes secrets and cache
    cache/
    scenes/
```

### Loxone HTTP API used

```
GET /data/LoxApp3.json              → full structure (controls, rooms, categories, globalStates)
GET /jdev/sps/io/{uuid}/{cmd}       → send command → JSON response
GET /dev/sps/io/{uuid}/all          → XML: all state outputs for a control
GET /dev/sps/io/{name}/state        → input state by name
GET /dev/sys/heap, /dev/sps/state, /dev/cfg/version, /data/status  → status info
GET /jdev/sys/lastcpu, numtasks, contextswitches, sdtest           → diagnostics
GET /jdev/sys/ints, comints, contextswitchesi                      → additional diagnostics
GET /jdev/cfg/ip, mac, mask, gateway, dns1, dns2, dhcp, ntp       → network config
GET /jdev/bus/packetssent, packetsreceived, ..., parityerrors      → CAN bus stats
GET /jdev/lan/txp, txe, txc, txu, rxp, rxo, eof, exh, nob        → LAN stats
GET /jdev/sys/date, /jdev/sys/time                                 → system clock
GET /jdev/sps/LoxAPPversion3        → structure file version check
GET /binstatisticdata/{uuid}/{period} → binary statistics (u32 ts + f64[] values)
GET /data/weatheru.bin              → binary weather data (108-byte entries)
GET /dev/fsget/{path}               → filesystem access
GET /jdev/sys/checktoken, refreshtoken, killtoken                  → token management
GET /jdev/sys/reboot                → reboot Miniserver
GET /jdev/sys/updatetolatestrelease → firmware update
WSS /ws/rfc6455                     → WebSocket for token auth key exchange
UDP :7070                           → Miniserver discovery (broadcast)
HTTP :7091/zone/{n}/{cmd}           → Music server API (unofficial)
```

TLS: `danger_accept_invalid_certs(true)` is used throughout (Miniserver uses self-signed certs). When `serial` is set in config, `Config::tls_host()` generates the DynDNS hostname for valid cert matching.

## Config Editing Scenarios

### Scenario: Configure a new room from scratch

```bash
# 1. Download and extract current config
lox config download --extract
# → sps_0267_20260409.Loxone

# 2. Understand what's already configured
lox config describe sps_0267_20260409.Loxone

# 3. See available hardware
lox config devices sps_0267_20260409.Loxone --ports

# 4. Apply a room template (creates LightController2, Blinds, Thermostat)
lox config template sps_0267_20260409.Loxone bedroom --room "DG Schlafzimmer"

# 5. Customize parameters
lox config set-param sps_0267_20260409.Loxone "DG Schlafzimmer Licht" FadingTime 2.0
lox config set-param sps_0267_20260409.Loxone "DG Schlafzimmer Licht" MoveTimeout 300

# 6. Wire control output to physical actor
lox config device-bind sps_0267_20260409.Loxone "DG Schlafzimmer Licht" AQ1 \
  --device "RGBW 24V Dimmer Tree"

# 7. Wire sensor input to control
lox config wire-connector sps_0267_20260409.Loxone "DG Schlafzimmer Licht.I1" \
  <sensor-connector-uuid>

# 8. Auto-layout and validate
lox config layout sps_0267_20260409.Loxone
lox config validate sps_0267_20260409.Loxone

# 9. Deploy
lox config push sps_0267_20260409.Loxone --reboot --force
```

### Scenario: Add logic blocks

```bash
# Create logic gate with auto-placement on Page
lox config add --type And --title "Both Sensors" sps.Loxone

# Set parameters
lox config set-param sps.Loxone "Both Sensors" I1 1

# Wire: Sensor1.Q → And.I1, Sensor2.Q → And.I2
lox config wire-connector sps.Loxone "Both Sensors.I1" <sensor1-q-uuid>
lox config wire-connector sps.Loxone "Both Sensors.I2" <sensor2-q-uuid>
```

### Available block types (190 total)

Logic: And, Or, Not, Xor, FlipFlop, AMemory, Counter, State, Monoflop
Math: Add, Sub, Mult, Div, Mod, Int, Avg, Minmax
Control: LightController2, JalousieUpDown2, Thermostat, AlarmClock, Ventilation2
Timing: OnPulseDelay, PulseGen, PWM, EdgeDetection
I/O: VirtualIn, InputRef, OutputRef, StateV

Use `lox config add --type <name>` — fuzzy matching suggests corrections for typos.

### Key files for config editing

| File | Purpose |
|------|---------|
| `src/config_edit.rs` | Core XML editing engine (4100 lines) |
| `src/commands/config_cmd.rs` | CLI handlers for all config subcommands |
| `docs/schemas/connector-map.json` | 190 block types with all connectors (25KB) |
| `docs/schemas/loxone-block-types-full.json` | 221 types from TechDoc with descriptions |
| `docs/schemas/loxone-config.xsd` | XML schema (309 types, 4892 lines) |

## Agent Workflow

### Non-Interactive Shell Commands

**ALWAYS use non-interactive flags** with file operations to avoid hanging on confirmation prompts.

Shell commands like `cp`, `mv`, and `rm` may be aliased to include `-i` (interactive) mode on some systems, causing the agent to hang indefinitely waiting for y/n input.

```bash
cp -f source dest    # NOT: cp source dest
mv -f source dest    # NOT: mv source dest
rm -f file           # NOT: rm file
rm -rf directory     # NOT: rm -r directory
```

Other commands that may prompt: `scp`/`ssh` — use `-o BatchMode=yes`; `apt-get` — use `-y`; `brew` — set `HOMEBREW_NO_AUTO_UPDATE=1`.

### Session Completion

**Work is NOT complete until `git push` succeeds.**

1. File issues for remaining work (`bd create`)
2. Run quality gates — `cargo test`, `cargo clippy`
3. Update issue status — close finished work
4. Push:
   ```bash
   git pull --rebase
   bd backup        # export issues to .beads/backup/ and commit
   git push
   git status       # must show "up to date with origin"
   ```
5. Verify all changes committed and pushed before ending the session

<!-- BEGIN BEADS INTEGRATION -->
## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Version-controlled: Built on Dolt with cell-level merge
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update <id> --claim --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task atomically**: `bd update <id> --claim`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Sync

Issues are committed to git as JSONL in `.beads/backup/`. `bd backup` exports and commits; runs automatically on `git push` via the pre-push hook.

### Important Rules

- Use bd for ALL task tracking
- Always use `--json` flag for programmatic use
- Link discovered work with `discovered-from` dependencies
- Check `bd ready` before asking "what should I work on?"
- Do NOT create markdown TODO lists
- Do NOT use external issue trackers
- Do NOT duplicate tracking systems

<!-- END BEADS INTEGRATION -->
