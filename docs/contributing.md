---
title: Contributing
layout: default
nav_order: 6
---

# Contributing
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Development setup

```bash
git clone https://github.com/eisber/lox-cli
cd lox-cli
cargo build
cargo test
```

No live Miniserver is needed for development. All unit tests use pure functions or mocked data.

## Build commands

```bash
cargo build              # debug build
cargo build --release    # production binary (~4MB)
cargo test               # run all tests
cargo clippy -- -D warnings  # lint (warnings are errors)
cargo fmt --check        # check formatting
```

## Pre-push checklist

All four checks must pass before pushing (mirrors CI):

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo build --release
cargo test
```

## Manual testing

To test against a real Miniserver:

1. Loxone Miniserver (Gen 1 or Gen 2) on your local network
2. Configure credentials: `lox setup set --host ... --user ... --pass ...`
3. Verify: `lox status`

## Pull request process

- One feature or fix per PR
- Tests expected for new logic
- Run `cargo fmt` and `cargo clippy` before submitting
- Update `CHANGELOG.md` under `[Unreleased]`

## Project status

This is an experimental project. Contributions welcome — especially for:

- Testing with different Miniserver configurations
- New control type support
- Bug reports and fixes
- Documentation improvements
