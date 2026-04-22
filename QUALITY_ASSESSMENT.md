# Project Quality Assessment

_Generated 2026-03-16_

## Overview

| Metric | Value |
|--------|-------|
| Language | Rust 2021 edition |
| Total source lines | **11,400** across 11 files |
| Dependencies | 30 direct crates |
| Test count | **157** (all passing) |
| Clippy | Clean (zero warnings) |
| Unsafe blocks | **None** |

## 1. Architecture — Grade: B-

**Strengths:**
- Clean module separation for distinct concerns: `client.rs` (HTTP), `token.rs` (auth), `ws.rs` (WebSocket), `config.rs`, `scene.rs`, `gitops.rs`
- Good use of `anyhow::Result` for error propagation throughout
- Proper async/sync coexistence (tokio for WS, reqwest::blocking for CLI)

**Key problem — `main.rs` is a 5,729-line monolith:**

The `run()` function spans from line 1198 to ~line 4700 — a single `match` statement with **50+ arms** handling every CLI command inline. This is the project's biggest structural issue.

| File | Lines | % of total |
|------|-------|-----------|
| `main.rs` | 5,729 | **50%** |
| `otel.rs` | 1,211 | 11% |
| `client.rs` | 1,199 | 11% |
| `stream.rs` | 1,076 | 9% |
| All others | 2,185 | 19% |

Each command handler (e.g., `Cmd::Thermostat`, `Cmd::Blind`, `Cmd::Light`) contains 50-150 lines of formatting, HTTP calls, and output logic — all inlined in one giant function. This makes navigation, testing, and refactoring difficult.

**Recommendation:** Extract command handlers into separate modules (e.g., `commands/blind.rs`, `commands/thermostat.rs`). Each handler becomes an independently testable function.

## 2. Code Quality — Grade: B+

**Strengths:**
- Zero clippy warnings — the code follows Rust idioms well
- Consistent patterns: `Config::load()? → LoxClient::new() → resolve → send`
- Good use of clap derive macros for type-safe CLI parsing
- Proper error categorization for JSON output mode (`categorize_error()`)
- Custom `HttpStatusError` for structured HTTP error matching

**Concerns:**
- **177 `unwrap()` calls in production code** — While many are safe (e.g., `serde_json::to_string_pretty`), several are risky:
  - `client.rs:158` — `Client::builder().build().unwrap()` (HTTP client creation)
  - `client.rs:282` — `self.structure.as_ref().unwrap()` (assumes structure loaded)
  - `config.rs:54` — `path.parent().unwrap()` (could panic on root path)
- **Magic numbers scattered throughout:** `86400` (cache TTL), `10` (HTTP timeout), `200`/`400`/`800` (retry delays) — should be named constants
- Helper functions like `xml_attr()`, `bar()`, `kb_fmt()` sit in main.rs rather than in reusable utility modules

## 3. Test Coverage — Grade: C+

**157 tests total, but coverage is uneven:**

| File | Tests | Lines | Coverage Focus |
|------|-------|-------|---------------|
| `main.rs` | 63 | 5,729 | Mostly helper functions (XML parsing, stats parsing, formatting). **Zero tests for any command handler.** |
| `client.rs` | 29 | 1,199 | Control resolution, fuzzy matching — well tested |
| `otel.rs` | 17 | 1,211 | Binary parsing, state mapping |
| `stream.rs` | 16 | 1,076 | Binary protocol parsing |
| `loxone_xml.rs` | 7 | 619 | XML extraction |
| `gitops.rs` | 6 | 512 | Path/diff logic |
| `loxcc.rs` | 6 | 271 | Compression/decompression |
| `ftp.rs` | 5 | 186 | FTP parsing |
| `token.rs` | 5 | 337 | HMAC hashing, token parsing |
| `ws.rs` | 2 | 145 | URL construction |
| `config.rs` | 0 | 60 | **No tests** |
| `scene.rs` | 0 | 55 | **No tests** |
| Integration (`cli_smoke.rs`) | 65 | — | Only tests `--help` exits 0 |

**Critical gaps:**
- **No integration tests** that actually exercise command logic (all smoke tests just check `--help`)
- **No mocking of HTTP calls** for command handler tests — `httpmock` is a dev-dependency but used only in `client.rs` unit tests
- **No property-based or fuzz testing** for binary parsers (weather data, statistics, stream protocol)
- **Config and scene modules have zero tests**

## 4. Robustness — Grade: B

**Strengths:**
- HTTP retry logic with exponential backoff (3 retries, 200/400/800ms) — correctly skips 4xx errors
- Proper timeouts on all network operations (HTTP: 10s, WS: 5s)
- Config file permissions set to `0o600` on Unix
- Graceful error output in JSON mode with error categorization
- No `unsafe` code anywhere

**Concerns:**
- **TLS certificate verification disabled by default** — `danger_accept_invalid_certs(true)`. While documented and necessary for Loxone's self-signed certs, it's a MITM risk. Should at minimum log a warning when connecting over untrusted networks.
- **No retry logic for FTP or WebSocket operations** — only HTTP has retries
- **Credentials stored in plaintext** in `~/.lox/config.yaml` (though file perms are restricted)
- **No file locking** on config/cache writes — concurrent CLI invocations could corrupt files

## 5. Recommended Improvements (Priority Order)

### High Impact

1. **Break up `main.rs`** — Extract each `Cmd::*` handler into a `commands/` module. This is the single highest-impact refactoring. It would:
   - Make each command independently testable
   - Reduce cognitive load (50 match arms → 50 focused functions)
   - Enable parallel development on different commands

2. **Add integration tests with HTTP mocking** — Use the existing `httpmock` dependency to test actual command flows (e.g., `lox ls`, `lox on`, `lox status`). Currently the 65 integration tests only verify `--help` exits 0.

3. **Replace critical `unwrap()` calls with proper error handling** — At minimum, audit the ~20 `unwrap()` calls in `client.rs` and `config.rs` production paths. Use `.context("...")` or `?` instead.

### Medium Impact

4. **Extract magic numbers into constants** — Define `CACHE_TTL`, `HTTP_TIMEOUT`, `RETRY_DELAYS`, etc. at the module level.

5. **Add tests for config.rs and scene.rs** — These are small modules with zero test coverage.

6. **Add fuzz testing for binary parsers** — `stream.rs` and `main.rs` parse binary weather/statistics data. These are prime candidates for `cargo fuzz` or property-based testing with `proptest`.

7. **Add retry logic to FTP and WebSocket** — Currently only HTTP has retries. FTP and WS operations (especially token auth) are equally network-dependent.

### Lower Impact

8. **Add a `--warn-insecure` flag** or log a warning when TLS verification is disabled and connecting to a non-local address.

9. **Consider using `thiserror` for a domain error enum** — While `anyhow` works well for CLIs, a proper error enum would enable better programmatic error matching.

10. **Add CI coverage reporting** — The CI runs `cargo test` but doesn't track coverage. Tools like `cargo-tarpaulin` or `cargo-llvm-cov` would make coverage gaps visible.

## Summary

This is a **well-engineered CLI tool** with clean Rust idioms, solid dependency choices, and thoughtful error handling patterns. The codebase is production-quality for a single-developer project. The main risk factors are:

- The monolithic `main.rs` making it hard to test command logic
- Test coverage that is broad but shallow (lots of unit tests for parsers, zero for command handlers)
- 177 `unwrap()` calls that could be reduced

The highest-ROI improvements would be **splitting main.rs into command modules** and **adding HTTP-mocked integration tests**. Together, these two changes would dramatically reduce the risk of regressions as the project grows.
