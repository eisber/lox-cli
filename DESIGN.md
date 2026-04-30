# lox — Design Document

> Config-as-Code CLI for Loxone Miniserver

## Status

Working. Config editing, SPS simulation, block library, and GitOps workflows functional and tested.

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│  lox CLI (single binary)                                         │
│                                                                  │
│  ┌──────────────┐  ┌───────────┐  ┌──────────┐  ┌───────────┐  │
│  │ lox config   │  │ lox sim   │  │ lox blocks│  │ lox ctx   │  │
│  │ download     │  │ check     │  │ search    │  │ add/use   │  │
│  │ edit/push    │  │ run/step  │  │ info/list │  │ list      │  │
│  │ validate     │  │ dump      │  │           │  │           │  │
│  └──────┬───────┘  └─────┬─────┘  └──────────┘  └───────────┘  │
│         │                │                                       │
│  ┌──────▼────────────────▼──────────────────┐                    │
│  │  ConfigEditor (DOM-based XML editing)    │                    │
│  │  LoxCC compress/decompress               │                    │
│  │  SPS Simulator engine (lox-sim)          │                    │
│  └──────┬───────────────────────────────────┘                    │
│         │                                                        │
│  ┌──────▼──────────────────────┐                                 │
│  │  HTTP/FTP Client + Token   │                                  │
│  │  reqwest + WS (RSA/AES)    │                                  │
│  └──────────┬─────────────────┘                                  │
└─────────────┼────────────────────────────────────────────────────┘
              │ HTTPS / FTP / WSS
    ┌─────────▼──────────┐
    │  Loxone Miniserver │
    │   /jdev/sps/io/    │
    │   /dev/fsget/      │
    │   /data/LoxApp3    │
    └────────────────────┘
```

---

## What Works Today

| Feature | Status | Notes |
|---------|--------|-------|
| `lox config download/extract` | ✅ | FTP download + LoxCC decompression |
| `lox config describe/controls/rooms` | ✅ | Inspect config structure |
| `lox config add/template` | ✅ | Add blocks, apply room templates |
| `lox config wire-connector/device-bind` | ✅ | Wire controls to inputs/devices |
| `lox config set-param/get-params` | ✅ | Read/write block parameters |
| `lox config validate/check/scan` | ✅ | Validation, dead-end detection, PII scan |
| `lox config layout` | ✅ | Auto-arrange blocks via ELK engine |
| `lox config push/upload` | ✅ | Recompress + upload to Miniserver |
| `lox config init/pull/log/restore` | ✅ | Git-based config versioning |
| `lox config diff` | ✅ | Compare two config files |
| `lox config mqtt` | ✅ | MQTT plugin configuration |
| `lox config user-add/user-remove` | ✅ | User CRUD in config |
| `lox sim run/step/check/dump` | ✅ | SPS simulator for offline testing |
| `lox blocks search/info/list` | ✅ | Block type library (190+ types) |
| `lox ctx add/use/list/remove` | ✅ | Multi-Miniserver contexts |
| `lox token fetch/refresh/kill` | ✅ | Token auth (RSA+AES key exchange) |
| `--output json/csv/table` | ✅ | All commands |

---

## Data Model

```
Config (~/.lox/config.yaml)
  host, user, pass, serial
  Supports flat (single-Miniserver) or multi-context format

Contexts (~/.lox/contexts/<name>/)
  cache/structure.json    # LoxApp3.json (24h TTL)
  token.json              # token auth credentials

Structure (cached from LoxApp3.json)
  controls: {uuid → {name, type, room, states}}
  rooms:    {uuid → name}

Project-local (.lox/ discovered by walking up from cwd)
  config.yaml, cache/, .gitignore
```

---

## Known Limitations

| Issue | Impact | Workaround |
|-------|--------|------------|
| Self-signed Miniserver TLS certs | `danger_accept_invalid_certs` used | Set `serial` in config for DynDNS hostname matching |
| LoxCC CRC32 must be non-zero | Zero CRC causes Miniserver to ignore `t="15"` password fields | `lox config compress` computes correct CRC32 |

---

## Source Files

### CLI (`src/`)

| File | Purpose |
|------|---------|
| `src/main.rs` | CLI entry point, clap argument parsing, top-level command dispatch |
| `src/client.rs` | `LoxClient` — HTTP client for Miniserver (structure cache, control resolution) |
| `src/config.rs` | `Config` + `GlobalConfig` — config loading, context resolution, project-local `.lox/` discovery |
| `src/config_edit/mod.rs` | `ConfigEditor` — DOM-based XML editing engine (element CRUD, wiring, properties) |
| `src/config_edit/blocks.rs` | Block creation with type-aware connector defaults |
| `src/config_edit/describe.rs` | Human-readable config description generator |
| `src/config_edit/layout.rs` | Auto-layout blocks on Pages via ELK engine |
| `src/config_edit/rooms.rs` | Room management (create, move, rename) |
| `src/config_edit/selector.rs` | Element selector syntax: `"Title"`, `"Type:And"`, `"uuid:abc-123"`, `"gid:Mqtt"` |
| `src/config_edit/template.rs` | Room templates (bedroom, bathroom, etc.) |
| `src/config_edit/validation.rs` | Config validation and automation checking |
| `src/config_edit/wiring.rs` | Wire-connector and device-bind operations |
| `src/config_edit/write.rs` | XML write-back (BOM-aware, preserves line endings) |
| `src/config_edit/properties.rs` | Property and parameter get/set |
| `src/errors.rs` | Rich error types with Levenshtein fuzzy matching, "did you mean?" suggestions |
| `src/loxcc.rs` | LoxCC compress/decompress — LZ4-style with CRC32 checksums |
| `src/loxone_xml.rs` | XML parsing: rooms, controls, users, devices, config summary, diff |
| `src/gitops.rs` | Git-based config versioning — init, pull, log, restore workflows |
| `src/ftp.rs` | FTP client for config download/upload to Miniserver |
| `src/ws.rs` | `LoxWsClient` — async WebSocket connection for token auth (RSA+AES key exchange) |
| `src/token.rs` | Token auth flow: RSA key exchange, AES-encrypted credentials, HMAC hashing |
| `src/telemetry.rs` | Anonymous usage analytics |
| `src/rc6.rs` | RC6 cipher support |
| `src/commands/config_cmd.rs` | CLI handlers for all `lox config` subcommands |
| `src/commands/ctx.rs` | `lox ctx` — add/use/list/remove/rename contexts |
| `src/commands/sim_cmd.rs` | `lox sim` — SPS simulator CLI entry points |
| `src/commands/blocks_cmd.rs` | `lox blocks` — block type search/info/list |

### SPS Simulator (`lox-sim/src/`)

| File | Purpose |
|------|---------|
| `lox-sim/src/lib.rs` | Library entry point |
| `lox-sim/src/engine.rs` | Simulation engine — tick loop, signal propagation |
| `lox-sim/src/compiler.rs` | Compiles `.Loxone` XML into simulation graph |
| `lox-sim/src/graph.rs` | Directed graph of blocks and wires |
| `lox-sim/src/parser.rs` | YAML spec parser for simulation test cases |
| `lox-sim/src/state.rs` | Block state management |
| `lox-sim/src/types.rs` | Core type definitions |
| `lox-sim/src/clock.rs` | Virtual clock for time-dependent blocks |
| `lox-sim/src/trace.rs` | Signal trace recording |
| `lox-sim/src/batch.rs` | Batch simulation runner |
| `lox-sim/src/io.rs` | I/O handling for simulation |
| `lox-sim/src/profiler.rs` | Performance profiling |
| `lox-sim/src/autodiff.rs` | Automatic differentiation support |
| `lox-sim/src/blocks/*.rs` | Block implementations: logic, math, timers, controllers, I/O, state, schedule, security, compare, energy, misc |

---

## API Reference (Loxone HTTP)

```
GET /data/LoxApp3.json                     → structure (controls, rooms)
GET /jdev/sps/io/{uuid}/{cmd}              → send command, returns JSON
GET /dev/sps/io/{uuid}/all                 → all outputs as XML
GET /dev/sps/io/{name}/state               → input state (works by name)
GET /dev/sps/io/{name}/astate              → output state
GET /dev/sys/cpu                           → CPU load (admin only)
GET /dev/sys/heap                          → memory usage
GET /dev/sps/state                         → PLC state (0-8)
GET /dev/cfg/version                       → firmware version
GET /data/status                           → full status XML
GET /dev/fsget/log/def.log                 → system log (admin)
WSS /ws/rfc6455                            → WebSocket API
  → jdev/sps/enablestatusupdate            → subscribe to state push
  → keepalive                              → keepalive ping
```

---

## Loxone WebSocket Protocol

```
Connection: WSS /ws/rfc6455
Auth: Basic Auth in HTTP Upgrade header

Binary message format:
  Header (8 bytes):
    [0] = 0x03 (magic)
    [1] = message type
          0x00 = text
          0x02 = ValueEventTable  ← state updates
          0x06 = keepalive
    [2] = flags (bit0 = estimated value)
    [3] = reserved
    [4-7] = uint32_le payload length

  ValueEventTable payload:
    repeated 24-byte records:
      [0-15]  = UUID (uint32_le + uint16_le + uint16_le + 8 bytes)
      [16-23] = double (float64_le) = current value

UUID binary → string:
  bytes[0..4]  → uint32_le → 8 hex chars  (part 1)
  bytes[4..6]  → uint16_le → 4 hex chars  (part 2)
  bytes[6..8]  → uint16_le → 4 hex chars  (part 3)
  bytes[8..16] → raw       → 16 hex chars (part 4)
  → "{p1}-{p2}-{p3}-{p4}"
```

---

## LoxCC Binary Format

The Miniserver stores its configuration in `sps0.LoxCC` files inside backup ZIPs.
This is a custom LZ4-style compression format:

```
Header (16 bytes):
  [0..4]   u32_le  magic = 0xAABBCCEE
  [4..8]   u32_le  compressed payload size
  [8..12]  u32_le  uncompressed size
  [12..16] u32_le  CRC32 of uncompressed data

Payload: LZ4-style token-based compression
  Token byte: high nibble = literal count, low nibble = match_length - 4
  If nibble == 15: read additional bytes (0-255 each, stop at <255)
  After literals: 2-byte LE back-reference offset
  Last block: no back-reference needed (pure literals valid)
```

### CRC32 field

The CRC32 at offset 12 is `zlib::crc32()` of the uncompressed XML data.
It is **required** for the Miniserver to trust encrypted config fields.
A zero CRC32 causes the Miniserver to load structure but ignore `t="15"`
password hashes.

### Password fields

Config XML uses `t="15"` for encrypted passwords and `t="11"` for plaintext
strings. The Miniserver accepts `t="11"` for password fields (e.g.
`mqtt_auth_pwd`), using the value directly — no firmware encryption key needed.

### Config load priority

On boot, the Miniserver loads the first valid config found:

1. `/prog/Emergency.LoxCC` (crash recovery only)
2. `/prog/sps_new.zip` or `.LoxCC`
3. `/prog/sps_<vers>_<timestamp>.zip` or `.LoxCC` (latest wins)
4. `/prog/sps.zip`, `/prog/sps_old.zip`
5. `/prog/Default.Loxone` or `/prog/DefaultGo.Loxone`

---

## References

Community projects that document the Loxone Miniserver internals:

| Project | What it provides |
|---------|------------------|
| [sarnau/Inside-The-Loxone-Miniserver](https://github.com/sarnau/Inside-The-Loxone-Miniserver) | LoxCC format, config load priority, AES key extraction, weather codes, networking protocol |
| [JoDehli/PyLoxone](https://github.com/JoDehli/PyLoxone) | WebSocket auth protocol (RSA+AES+HMAC), token management, Home Assistant integration |
| [alladdin/node-lox-ws-api](https://github.com/alladdin/node-lox-ws-api) | Binary event table parsing, 3 auth methods (Token/AES/Hash), salt rotation |
| [codmpm/node-red-contrib-loxone](https://github.com/codmpm/node-red-contrib-loxone) | Node-RED integration, auth method selection by firmware version |

---

## Authentication Protocol

The Miniserver supports three authentication methods, selected by firmware version.
Based on reverse engineering by PyLoxone and node-lox-ws-api projects.

### Method 1: Token-Enc (v9+, recommended)

```
1. HTTP:  GET /jdev/sys/getPublicKey        → RSA public key (PEM)
2. WS:    jdev/sys/keyexchange/<b64(RSA_encrypt(aes_key:iv))>
          → Exchange AES-256-CBC session key
3. WS:    jdev/sys/getkey2/<username>        → server_key + salt
          pw_hash = SHA1(password:salt).toUpperCase()
          hash = HMAC-SHA1(username:pw_hash, server_key)
4. WS:    jdev/sys/gettoken/<hash>/<user>/2/<app_uuid>/<client_id>
          → { token, validUntil }  (epoch from 2009-01-01)
5. Commands encrypted: jdev/sys/enc/<b64(AES_CBC(salt/<salt>/<cmd>))>
```

Salt: 16 random bytes, regenerated every 20 uses or 30 seconds.
Tokens refresh at 50% lifetime via `jdev/sys/refreshtoken/<hash>/<user>`.

### Method 2: AES-256-CBC (v8)

Same key exchange as Token-Enc, but uses session encryption for all commands
instead of token-based auth. No persistent token.

### Method 3: Hash (v7 and earlier, legacy)

```
1. WS:    jdev/sys/getkey                   → server_key
          hash = HMAC-SHA1(username:password, server_key)
2. WS:    authenticate/<hash>
```

No encryption of subsequent commands.

### WebSocket Binary Messages

```
Header (8 bytes):
  Byte 0:   0x03 (identifier)
  Byte 1:   Message type
            0 = Text, 1 = Binary, 2 = ValueStates,
            3 = TextStates, 4 = Daytimer,
            5 = OutOfService, 6 = Keepalive, 7 = Weather
  Byte 2:   Flags (bit 7 = estimated/continuation)
  Bytes 3-7: Payload length (u32_le)
```

ValueStates: 24-byte records (16-byte UUID + 8-byte f64 value).
Keepalive: client sends "keepalive" text every 30s, server responds with type 6.
