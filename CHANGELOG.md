# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.14.0] — 2026-05-08

### Added
- `lox config devices -o json` emits per-physical-device records with bus detection, channel grouping, and stable identity components.

## [0.13.0] — 2026-05-08

### Added
- `lox config wires -o json` emits the full wire graph including resolved source/target block + connector UUIDs.

### Fixed
- Windows `/wsx` fast reload now uses native TLS, so config inspection builds do not require OpenSSL while fast reload remains available.

## [0.12.0] — 2026-04-30

### Added
- **SPS Simulator** (`lox-sim`) — offline Miniserver circuit simulator
  - 195 block types: logic, math, lighting, HVAC, timers, blinds, I/O
  - Topological evaluation engine with cycle detection
  - Multi-step temporal specs: test heating cycles, timer delays, schedules
  - Time injection for DayTimer/AlarmClock schedule testing
  - Structured trace output (JSON) for signal auto-discovery
  - 367 unit tests, 6k lines of Rust
- **Eval harness** — 285 behavioral test cases across 10 categories
  - Utterance → agent builds circuit → simulator verifies signals
  - 94% raw LLM pass rate (5 sections at 100%)
  - Parallel execution, incremental reporting
- **JSON output** on all config commands (`-o json`)
  - `config add` returns UUID + connector map with UUIDs
  - `config check/validate` returns structured warnings/errors
  - `config describe` returns full block inventory with connectors
  - `config get-params` returns all connectors with wiring status
  - `config wire-connector` and `set-param` return structured confirmation
- **Skill references** (`.github/skills/`) for AI agent integration
  - loxone-config: CLI commands, block types, worked examples, common mistakes
  - loxone-patterns: 13 automation recipes
  - loxone-sim: simulator testing patterns
- **`lox config describe`** — human-readable config summary grouped by room
- **`lox config device-bind`** — wire a control's output to a physical device
- **`lox config template`** — room templates (bedroom, bathroom, kitchen, etc.)
- **`lox config layout`** — ELK-based auto-layout engine for block pages
- **190 block types** with full connector maps (2384 connectors with I/O/P types)
- **Levenshtein fuzzy matching** on all error messages
- **GitOps** — `config pull` downloads, diffs, and commits with semantic messages

### Fixed
- **TLS verify_ssl respected in ALL code paths** — previously hardcoded in ws.rs, token.rs, stream.rs
- **Token file permissions** — now chmod 0600 on Unix
- **Atomic config writes** — write-to-temp + rename prevents corruption
- JSON output on `config controls`, `rooms`, `stats` (was broken/missing)
- **Config patch panic** — empty replacement pattern now returns error instead of panicking
- **LoxCC allocation cap** — capped at 64MB to prevent OOM from malicious headers
- **Plaintext token fallback** — now warns when sending token unhashed
- All clippy warnings resolved (50 warnings → 0)
- Formatting violations fixed across entire codebase
- Failing test `test_require_one_no_match` updated for new error message format

### Changed
- License updated: Amy Cozowicz added as co-author and copyright holder
- License badge corrected: MIT → AGPL-3.0
- `ConnectorMap` and `DescribeEntry` type aliases reduce type complexity

### Tests
- **495 tests** (374 unit + 102 CLI integration + 19 config integration), all passing
- 32 in-process config_cmd tests for tarpaulin coverage (config_cmd.rs: 0% → 26.5%)
- httpmock-based tests for cache/token HTTP paths
- LoxCC compression pipeline round-trip tests
- Code coverage: 37% overall (core modules 72-100%)

- **Multi-Miniserver context management** (`lox ctx`) — `kubectl`-style context switching for multiple Miniservers
  - `lox ctx add <name> --host ... --user ... --pass ...` — add a named context
  - `lox ctx use <name>` / `lox ctx <name>` — switch active context
  - `lox ctx list` — list all contexts (`*` = active)
  - `lox ctx current` — show active context
  - `lox ctx remove <name>` / `lox ctx rename <old> <new>` — manage contexts
  - `lox ctx init` — create project-local `.lox/` directory (auto-discovered like `.git`)
  - `lox ctx migrate` — convert existing flat config to a `default` context
- `--ctx <name>` global flag — run any command against a specific context without switching
- Per-context data isolation — each context gets its own cache, token, and scenes directory under `~/.lox/contexts/<name>/`
- Project-local `.lox/` directory support — walks up from cwd, like `.git` resolution
- Backward-compatible config format: existing flat `~/.lox/config.yaml` files continue to work unchanged

## [0.8.0] — 2026-03-21

### Added
- **Windows support** — pre-built binaries for Windows x86_64 and aarch64 are now included in every release
- `lox config init <path>` — initialize a git repository for config version tracking (multi-Miniserver via serial subdirectories)
- `lox config pull [--quiet]` — download config via FTP, decompress LoxCC, generate semantic diff, and git-commit with meaningful change messages
- `lox config log [-n COUNT]` — show config change history from the git repository
- `lox config restore <commit> --force` — restore a previous config version from git history and upload to Miniserver
- `lox health` — device health dashboard showing battery, signal, offline status, and bus errors for Tree/Air devices (`--type tree|air`, `--problems`)
- `lox schema` — command schema introspection for AI agent discovery; lists commands with metadata, args, and valid actions
- `--dry-run` global flag — validates and resolves inputs without executing commands; returns structured JSON envelope with `-o json`
- `--non-interactive` global flag — fails instead of prompting for confirmation (implied by `-o json`)
- `--trace-id` global flag — correlation ID for tracking agent actions in logs
- `-v`/`--verbose` global flag — `-v` shows HTTP requests, `-vv` shows requests + response bodies
- `--all-in-room` flag on `lox on`/`lox off` — apply command to all controls in a room
- Structured JSON error envelopes when using `-o json` (categorized error codes: `control_not_found`, `ambiguous_control`, `unauthorized`, `connection_error`, etc.)

### Changed
- `lox extensions` now queries `/data/status` instead of `LoxApp3.json` — provides richer device information including Tree branch error counts, device parent relationships, and plugin versions
- CI: moved linting to a dedicated Ubuntu job; Windows builds no longer run redundant clippy/fmt checks

### Removed
- `lox daemon` — automation daemon (WebSocket/polling rule engine)
- `lox automation` — automation rule management
- `lox service` — systemd service management
- `timezone:` config field (was only used for automation time windows)

## [0.1.0] — 2024-01-01

### Added
- `lox ls` — list controls with optional `--type`, `--room`, `--values` filters
- `lox get <name>` — show full state of a control
- `lox on/off/pulse <name>` — send on/off/pulse commands
- `lox send <name> <cmd>` — send arbitrary raw command
- `lox blind <name> <action>` — control Jalousie blinds (up/down/stop/shade/pos)
- `lox mood <name> <action>` — control LightControllerV2 moods
- `lox set <name> <value>` — set analog/virtual input value
- `lox if <name> <op> <value>` — conditional state check (exit 0/1)
- `lox watch <name>` — poll state changes
- `lox status [--energy]` — Miniserver health; auto-discovers energy meters
- `lox rooms` — list all rooms
- `lox config set/show` — manage connection config
- `lox token fetch/info/clear` — RSA+AES token auth management
- `lox cache info/clear/refresh` — structure cache management
- `lox scene list/show/new` + `lox run <scene>` — multi-step scenes
- `--room` flag on all commands for disambiguation
- Bracket room qualifier: `"Name [Room]"` syntax
- Alias support in config (`aliases:` map)
- Structure cache (24h TTL) at `~/.lox/cache/structure.json`
- Token auth (acquired via `lox token fetch`) used for all HTTP requests
### Fixed
- `lox config set` no longer clobbers the alias list
- `lox set` percent-encodes values to avoid malformed URLs
- Token auth is now actually used for all requests (was always falling back to Basic Auth)
- `lox blind` and `lox mood` now support `--room` flag
- WebSocket nonce uses cryptographically random bytes
- Debug `eprintln!` removed from token RSA parsing

[Unreleased]: https://github.com/eisber/lox-cli/compare/v0.13.0...HEAD
[0.13.0]: https://github.com/eisber/lox-cli/compare/v0.12.0...v0.13.0
[0.12.0]: https://github.com/eisber/lox-cli/compare/v0.8.0...v0.12.0
[0.8.0]: https://github.com/eisber/lox-cli/compare/v0.1.0...v0.8.0
[0.1.0]: https://github.com/eisber/lox-cli/releases/tag/v0.1.0
