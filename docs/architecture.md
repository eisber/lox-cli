---
title: Architecture
layout: default
nav_order: 5
---

# Architecture
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Overview

`lox` is a single static Rust binary (~4MB). TLS is handled by rustls (no OpenSSL dependency). Self-signed certificates from Miniservers are accepted by default.

## Source structure

| File | Purpose |
|:-----|:--------|
| `src/main.rs` | CLI entry point, clap argument definitions, command dispatch |
| `src/commands/control.rs` | Control commands (on, off, blind, light, gate, thermostat, etc.) |
| `src/commands/inspect.rs` | Inspection commands (ls, get, info, watch, stream, etc.) |
| `src/commands/system.rs` | System commands (status, log, health, extensions, etc.) |
| `src/commands/config_cmd.rs` | Config management commands (download, diff, git versioning) |
| `src/commands/ctx.rs` | Context management (add, use, list, remove, rename, init, migrate) |
| `src/client.rs` | `LoxClient` — HTTP client, control resolution, structure cache |
| `src/config.rs` | `Config` / `GlobalConfig` — loads/saves config, multi-context support |
| `src/stream.rs` | Real-time WebSocket state streaming |
| `src/otel.rs` | OpenTelemetry metrics, logs, and traces export |
| `src/gitops.rs` | Git-based config versioning (init, pull, log, restore) |
| `src/scene.rs` | Scene loading/listing from YAML |
| `src/ws.rs` | `LoxWsClient` — WebSocket for token auth key exchange |
| `src/token.rs` | Token auth flow (RSA + AES encryption, HMAC hashing) |
| `src/ftp.rs` | FTP client for config download |
| `src/loxcc.rs` | LoxCC format decompression (proprietary config format) |
| `src/loxone_xml.rs` | Loxone XML parsing |

## Control resolution

The `LoxClient::resolve_with_room` function resolves human-readable names to control UUIDs using fuzzy substring matching against the structure cache.

**Resolution order:**
1. Alias lookup (from config)
2. Exact UUID match
3. Bracket room qualifier (`"Name [Room]"`)
4. `--room` flag filtering
5. Fuzzy substring match

Ambiguous matches produce an error with a list of candidates.

## Structure cache

The Miniserver's `LoxApp3.json` (~150KB) is cached at `~/.lox/cache/structure.json` with a 24-hour TTL. All commands that need control UUIDs load this cache first.

```bash
lox cache info      # check cache status
lox cache refresh   # force re-fetch
```

## Mixed sync/async runtime

CLI commands use `reqwest::blocking` for synchronous HTTP. The `#[tokio::main]` attribute on `main()` exists because `lox token fetch` needs async WebSocket support for the RSA/AES key exchange handshake. The blocking reqwest client spawns its own thread pool, so both modes coexist.

## Token authentication

The token auth flow:

1. Fetch RSA public key from Miniserver
2. Generate random AES session key
3. Encrypt AES key with RSA public key
4. Send encrypted credentials via WebSocket
5. Receive encrypted token, decrypt with AES key
6. Store token at `~/.lox/token.json` (valid ~20 days)
7. Token is used for all subsequent HTTP requests via HMAC hashing

## User data layout

Single-context (flat config):
```
~/.lox/
  config.yaml          # Host, credentials, serial, aliases
  token.json           # Token auth (optional, valid ~20 days)
  cache/
    structure.json     # LoxApp3.json cache (24h TTL, ~150KB)
  scenes/*.yaml        # Multi-step scene definitions
```

Multi-context:
```
~/.lox/
  config.yaml          # active_context + contexts map
  contexts/
    home/
      cache/
        structure.json # Per-context structure cache
      token.json       # Per-context token
    office/
      cache/
        structure.json
      token.json
  scenes/*.yaml        # Shared scene definitions
```

Project-local (auto-discovered by walking up from cwd):
```
./project/.lox/
  config.yaml          # Connection settings
  .gitignore           # Excludes secrets and cache
  cache/
    structure.json
  scenes/*.yaml
```

## TLS

Self-signed certificates are accepted (`danger_accept_invalid_certs(true)`) since Miniservers use self-signed certs. When `serial` is set in config, `Config::tls_host()` generates the DynDNS hostname for valid certificate matching.

## Loxone HTTP API

The CLI communicates with the Miniserver via its HTTP API:

| Endpoint | Purpose |
|:---------|:--------|
| `GET /data/LoxApp3.json` | Full structure (controls, rooms, categories) |
| `GET /jdev/sps/io/{uuid}/{cmd}` | Send command to control |
| `GET /dev/sps/io/{uuid}/all` | All state outputs for a control (XML) |
| `GET /dev/sps/io/{name}/state` | Input state by name |
| `GET /dev/sys/heap` | System status |
| `GET /jdev/sys/lastcpu` | Diagnostics |
| `GET /jdev/cfg/ip` | Network configuration |
| `GET /binstatisticdata/{uuid}/{period}` | Binary statistics data |
| `GET /data/weatheru.bin` | Binary weather data |
| `GET /dev/fsget/{path}` | Filesystem access |
| `WSS /ws/rfc6455` | WebSocket (token auth) |
| `UDP :7070` | Miniserver discovery |
| `HTTP :7091/zone/{n}/{cmd}` | Music server (unofficial) |
