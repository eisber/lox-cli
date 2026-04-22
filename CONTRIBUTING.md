# Contributing to lox

## Development Setup

```bash
# Clone
git clone https://github.com/eisber/lox-cli
cd lox-cli

# Build (debug)
cargo build

# Build (release)
cargo build --release

# Run tests (no Miniserver required)
cargo test

# Lint
cargo clippy -- -D warnings
cargo fmt --check
```

No live Miniserver is needed for development. All unit tests use pure functions
or mocked data. Integration tests (if added) use `wiremock`/`httpmock`.

## Issue Tracking (beads)

This project uses [beads](https://github.com/steveyegge/beads) (`bd`) for issue tracking.
Issue data is stored as JSONL in `.beads/backup/` and committed to git — no separate account needed.

```bash
# Install beads (macOS/Linux)
brew install beads

# After cloning: restore issues from git into your local Dolt DB
bd backup restore

# Set up Claude Code hooks (AI contributors only)
bd setup claude

# Find available work
bd ready
```

When finishing a session, beads auto-commits a backup on `git push`. You can also trigger it manually:

```bash
bd backup        # exports JSONL + commits
git push
```

## Manual Testing

To test against a real Miniserver you need:

- A Loxone Miniserver (Gen 1 or Gen 2) on your local network
- Host, username, and password in `~/.lox/config.yaml` (run `lox config set`)

```bash
lox ls                   # list all controls
lox status               # Miniserver health
lox token fetch          # acquire auth token
```

## PR Process

- One feature or fix per PR
- Tests expected for new logic (pure functions are easy to unit-test)
- Run `cargo fmt` and `cargo clippy` before submitting
- Update `CHANGELOG.md` under `[Unreleased]`

## Architecture

See [`CLAUDE.md`](CLAUDE.md) for a full architectural overview of the source files.
