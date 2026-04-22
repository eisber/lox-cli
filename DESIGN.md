# lox — Design Document

> CLI for Loxone Miniserver

## Status

Working. Core commands functional and tested end-to-end.

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  lox CLI (single binary)                                 │
│                                                          │
│  ┌──────────┐  ┌──────────┐                             │
│  │ Commands │  │  Scenes  │                             │
│  │ on/off   │  │ YAML     │                             │
│  │ blind    │  │ run      │                             │
│  │ if/watch │  │ new      │                             │
│  └────┬─────┘  └─────┬────┘                             │
│       │              │                                   │
│  ┌────▼──────────────▼────┐                              │
│  │      HTTP Client       │                              │
│  │  reqwest + Basic Auth  │                              │
│  └──────────┬─────────────┘                              │
└─────────────┼───────────────────────────────────────────┘
              │ HTTPS
    ┌─────────▼──────────┐
    │  Loxone Miniserver │
    │   /jdev/sps/io/    │
    │   /dev/sps/io/all  │
    │   /data/LoxApp3    │
    └────────────────────┘
```

---

## What Works Today

| Feature | Status | Notes |
|---------|--------|-------|
| `lox ls / rooms` | ✅ | Structure from LoxApp3.json |
| `lox on/off/pulse/send` | ✅ | Via `/jdev/sps/io/{uuid}/{cmd}` |
| `lox get` | ✅ | Via `/dev/sps/io/{uuid}/all` XML |
| `lox blind` | ✅ | PulseUp/Down/FullUp/FullDown/AutomaticDown |
| `lox status` | ✅ | Firmware, PLC state, memory |
| `lox if` | ✅ | Exit codes for shell scripting |
| `lox watch` | ✅ | HTTP polling loop |
| `lox run <scene>` | ✅ | Multi-step YAML scenes |
| `lox log` | ⚠️ | Needs admin user |
| `lox config init/pull/log/restore` | ✅ | Git-based config versioning |
| `--json` output | ✅ | All commands |

---

## Next Steps

### 1. `lox watch` via WebSocket (Medium Priority)

Currently polling. The WS infrastructure exists in `ws.rs` (`connect_raw`). Once wired up:

```bash
lox watch "Lichtsteuerung"     # real-time, <100ms latency
lox watch --all                # stream all state changes
```

Requires Monitor rights enabled in Loxone Config → User Management.

---

### 2. `lox set` — Analog/Virtual Inputs (Medium Priority)

Send analog values to virtual inputs:

```bash
lox set "Solltemperatur" 21.5     # send analog value
lox set "Modus" "Urlaub"          # send text to virtual text input
```

API: `/jdev/sps/io/{uuid}/{value}` already supports this, just needs a dedicated command + type detection.

---

### 3. `lox mood` — LightControllerV2 Moods (Low Priority)

Loxone light controllers have named moods (Stimmungen):

```bash
lox mood "Wohnzimmer" list           # list available moods + IDs
lox mood "Wohnzimmer" set "Abend"   # activate mood by name
lox mood "Wohnzimmer" set 778       # activate mood by ID
```

The `/all` output for LightControllerV2 includes `moodList` — needs parsing.

---

### 4. `lox rooms` — Room-scoped Commands (Low Priority)

```bash
lox room "Wohnzimmer" off    # turn off everything in room
lox room "Wohnzimmer" ls     # list all controls in room
```

Already have room data in structure, just needs a Room command that iterates.

---

### 5. TLS Improvements (Low Priority)

Currently using `danger_accept_invalid_certs`. Options:

**Option A:** Use dyndns hostname (serial-based):
```
https://192-168-20-24.{SERIAL}.dyndns.loxonecloud.com
```
Already implemented in `Config::tls_host()`, just needs to be used everywhere.

**Option B:** Token auth (newer Loxone firmware)  
Loxone supports JWT tokens via `/jdev/sys/gettoken` — longer-lived, more secure than Basic Auth per-request.

---

### 6. `lox backup` — Structure/Config Backup (Low Priority)

```bash
lox backup          # save LoxApp3.json + current state snapshot
lox backup restore  # restore from backup (readonly — shows diff)
```

Also: `/dev/fslist/` and `/dev/fsget/` allow SD card file access (needs admin).

---

## Data Model

```
Config (~/.lox/config.yaml)
  host, user, pass, serial

Structure (cached from LoxApp3.json)
  controls: {uuid → {name, type, room, states}}
  rooms:    {uuid → name}

Scenes (~/.lox/scenes/*.yaml)
  name, description, steps[]
    step: {control, cmd, delay_ms}

```

---

## Known Limitations

| Issue | Impact | Fix |
|-------|--------|-----|
| `enablestatusupdate` needs Monitor rights | `lox watch` can't use WS | Enable Monitor right in Loxone Config |
| `CentralLightController` has no numeric `/all` value | Can't watch central lights via polling | Use LightControllerV2 UUIDs directly |
| State values only via WS (not HTTP) for most types | `lox get` shows limited info | WS with Monitor rights |
| `lox log` needs admin | Can't read Miniserver logs | Use admin user |
| No structured error codes | Rule engine uses string matching | Add error enum |

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
