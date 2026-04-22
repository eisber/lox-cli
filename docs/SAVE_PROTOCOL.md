# Loxone Miniserver Config Push Protocol

## Overview

This document describes how to programmatically upload configuration
files to a Loxone Miniserver and trigger a fast SPS reload (~4s).

## Quick Reference

```
1. FTP STOR /prog/sps_new.zip         (upload config ZIP)
2. /wsx 0x3A pre-save → ACK           (open save window)
3. /wsx 0x05 post-save → SPS reload   (trigger fast reload)
```

Total cycle: ~4 seconds. No reboot required.

## Authentication

### Token Acquisition

```
GET /jdev/sys/getkey2/{user}  → { key, salt, hashAlg }
pwHash = SHA256("{password}:{salt}").toUpperCase()
sig    = HMAC-SHA256(hex_decode(key), "{user}:{pwHash}").toLowerCase()
GET /jdev/sys/getjwt/{sig}/{user}/8/{uuid}/{client}  → { token, key }
```

The `key` in both responses is double-hex encoded: the hex string decodes
to an ASCII hex string (e.g., `"3041"` → `"0A"`).

### autht Computation

For authenticated HTTP requests, obtain a fresh one-time key:
```
GET /jdev/sys/getkey  → one-time key (double-hex encoded)
key_ascii = hex_decode_to_ascii(getkey_value)
autht = HMAC-SHA256(key_ascii.bytes(), jwt_token.bytes()).toUpperCase()
```

Use as query parameter: `?autht={autht}&user={user}`

Each `getkey` value is single-use. Call `getkey` before each authenticated request.

## /wsx WebSocket Connection

### Upgrade

```
GET /wsx?autht={token}&user={user} HTTP/1.1
Host: {miniserver}:443
Upgrade: WebSocket
Connection: Upgrade
```

Server responds with `101 Web Socket Protocol Handshake` (hixie-76 style).

### Startup Sequence

After upgrade, send three **separate** TLS writes (do not bundle):

1. Text frame: `\x00dev/loxone/start\xff` (wait 100ms)
2. 64-byte binary handshake packet
3. Server responds with 0x01 (Capabilities)

### Handshake Packet (64 bytes)

```
Bytes  0-15:  Nonce (Miniserver uptime in ms, interleaved with constants)
Bytes 16-31:  Header (version=1, features=0x20, magic=0xEFBEEDFE)
Bytes 32-63:  RC6-encrypted credentials ("user\0password\0DEU\0", zero-padded to 32B)
```

**Nonce layout** (T = uptime in milliseconds):
```
[0x29, 0x00, 0x00, T[3], T[0], 0xBE, 0x18, 0x84,
 0x6C, T[1], 0xE1, 0x4A, 0xD6, 0x2C, T[2], 0xAE]
```

**RC6 key derivation** from timestamp T:
```
key_u32 = ((T & 0x1000 | T >> 16) >> 8) | ((T << 16 | T & 1) << 8)
```

The uptime can be read via:
```
GET /jdev/sps/io/20123f74-0222-3d2f-ffff234d69b98eb1/state
```

### Binary Message Format

All binary messages use a 16-byte header:
```
[cmd:1][flags:1][size:u16 LE][seq:u32 LE][reserved:u32 LE][magic:4]
```

- Client magic: `0xEFBEEDFE`
- Server magic: `0xFEAFEDFE`

Key commands:

| Cmd  | Name         | Direction | Description                    |
|------|--------------|-----------|--------------------------------|
| 0x01 | Capabilities | S→C       | Sent after handshake           |
| 0x05 | PostSave     | C→S       | Trigger SPS reload (fast)      |
| 0x14 | Keepalive    | Both      | Sent every ~3s                 |
| 0x29 | Handshake    | C→S       | 64-byte auth handshake         |
| 0x3A | PreSave      | C→S       | Open save window               |

### WebSocket Text Framing

Uses hixie-76 framing: `\x00{utf8_text}\xFF`

## Config Upload (Fast Push)

### Step 1: Upload via FTP

Upload the config ZIP to `/prog/sps_new.zip`:

```
FTP connect {miniserver}:21
USER {user}
PASS {password}
CWD /prog
STOR sps_new.zip {zip_data}
QUIT
```

The ZIP must contain:
- `sps0.LoxCC` — compiled SPS program
- `LoxAPP3.json` — app structure
- `Emergency.LoxCC` — emergency fallback
- `permissions.bin` — user permissions
- `Music.json` — music config

### Step 2: Trigger Fast Reload via /wsx

With an active /wsx connection:

```
Send: 0x3A PreSave   → Server responds with 0x3A ACK
Send: 0x05 PostSave  → Server reloads SPS from sps_new.zip
```

The Miniserver will:
1. Read `sps_new.zip` from `/prog/`
2. Load the config (sps0.LoxCC, LoxAPP3.json, etc.)
3. Create a backup as `sps_VERS_TIMESTAMP.zip`
4. Delete `sps_new.zip`
5. Restart SPS (~2-4 seconds)

### Alternative: Slow Reload

If /wsx is not available, use HTTP:
```
GET /dev/sps/restartclear?autht={autht}&user={user}
```

This triggers a full SPS restart (~30-60 seconds).

## Config Download

### Download via authenticated HTTP

```
GET /dev/fsget/lx/prog/sps.LoxCC?autht={autht}&user={user}
```

### Download via FTP

```
FTP GET /prog/sps_VERS_TIMESTAMP.zip
```

The latest `sps_VERS_*.zip` backup contains the full config ZIP.

## Encrypted Authentication (enc/)

For enhanced security, commands can be AES-256-CBC encrypted:

1. `GET /jdev/sys/getPublicKey` → RSA public key
2. Generate AES-256 key + IV, RSA-encrypt `"{key_hex}:{iv_hex}"`
3. AES-CBC encrypt: `salt/{uuid}/{command}` (zero-byte padded to 16B blocks)
4. `GET /jdev/sys/enc/{base64_ciphertext}?sk={base64_rsa_session_key}`
5. Subsequent requests omit `?sk=` (session persists on the connection)

Use `fenc/` instead of `enc/` for AES-encrypted responses.

## Timing

| Method              | Duration  | Mechanism              |
|---------------------|-----------|------------------------|
| FTP + 0x3A/0x05     | ~4s       | Fast SPS reload        |
| FTP + restartclear  | ~30-60s   | Full SPS restart       |
| FTP + reboot        | ~60-90s   | Full system reboot     |
