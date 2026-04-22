# Loxone Config Interactions — Research

> **Status: REFERENCE** — Research document. Core features (backup download/list/extract/upload) implemented as `lox config`. See [design-backup.md](design-backup.md) and [design-backup-inspect.md](design-backup-inspect.md) for design details.

## Background

"Loxone Config" is the official desktop application for programming Loxone Miniservers. It creates `.Loxone` project files and uploads compiled configurations to the Miniserver. This document researches what programmatic interactions are possible from the CLI.

---

## How Configuration Works on the Miniserver

### File Chain

```
Loxone Config (.Loxone project)
    │  compile + upload
    ▼
Miniserver SD Card
    /prog/sps_<version>_<timestamp>.zip    ← backup ZIPs
        └── sps0.LoxCC                     ← compiled binary config
    /data/LoxApp3.json                     ← simplified JSON view for apps/APIs
```

- **`.Loxone`** — The project file used by Loxone Config (XML-based, editable in Config GUI)
- **`sps0.LoxCC`** — Compiled binary configuration on the Miniserver (custom LZSS-compressed format)
- **`LoxApp3.json`** — Simplified JSON extract: controls, rooms, categories, states (what we already cache)

### The LoxCC Format

The `sps0.LoxCC` file uses a custom compression:
- **Magic header**: `0xaabbccee` (4 bytes)
- **Compressed size**: 4 bytes
- **Header3/Header4**: Approximate uncompressed length + possible checksum
- **Payload**: LZSS-style variable-length encoding

When decompressed, the result is an XML file that can be opened in Loxone Config as a `.Loxone` project. Sarnau's Python script `loadMiniserverConfigurationFile.py` implements the full download + decompress flow.

---

## Access Methods

### 1. FTP (Port 21)

The Miniserver runs an FTP server. With admin credentials:

| Path | Content |
|------|---------|
| `/prog/sps_*.zip` | Configuration backup ZIPs (contain `sps0.LoxCC`) |
| `/data/LoxApp3.json` | Simplified config (JSON) |
| `/data/weatheru.bin` | Weather cache |
| `/log/` | System logs |
| `/stats/` | Statistics data |
| `/web/` | Web interface files |

**Backup naming**: `/prog/sps_<version>_<YYYYMMDDHHMMSS>.zip` — the newest ZIP is the current config.

FTP supports both read and write (with appropriate permissions). However, FTP transfers credentials in plaintext — only safe on local networks.

### 2. HTTP API (`/dev/fsget/`)

Read-only filesystem access already partially implemented in `lox`:

```
GET /dev/fsget/<path>     → download file (unencrypted only, not supported under command encryption)
```

**Not implemented** in the Miniserver HTTP API:
- No `fslist` endpoint for directory listing (despite some documentation hints)
- No `fsput` endpoint for uploads via HTTP

### 3. Loxone Config Software (Proprietary Protocol)

Loxone Config uses its own protocol for uploading compiled configurations. This is not publicly documented and involves:
- Compiling the `.Loxone` project into binary format
- Uploading via a proprietary mechanism (likely over HTTP/WebSocket)
- Triggering an automatic Miniserver restart

---

## What We Can Implement

### Tier 1: Config Backup/Download (High Value, Medium Effort)

**`lox backup`** — Download the current configuration from the Miniserver.

**Approach A: FTP-based** (most complete)
```
lox backup                          # download latest config backup
lox backup --output config.zip      # save to specific file
lox backup list                     # list available backup ZIPs
lox backup --extract                # download + decompress LoxCC → .Loxone XML
```

Implementation:
- Connect to Miniserver FTP (port 21, admin credentials)
- List `/prog/` directory to find backup ZIPs
- Download the latest `sps_*.zip`
- Optionally extract and decompress `sps0.LoxCC` to readable XML

Rust crates: `suppaftp` (FTP client) or `ftp` crate.

**Approach B: HTTP-based** (simpler, read-only)
```
lox backup                          # download via /dev/fsget/prog/...
```

Limitation: Requires knowing the exact filename, no directory listing via HTTP.

**Approach C: Hybrid**
- Use HTTP `/dev/fsget/` for known paths (logs, LoxApp3.json)
- Use FTP for directory listing and full backup downloads

### Tier 2: Config Diff / Inspection (High Value, Medium Effort)

**`lox config diff`** — Compare configurations.

```
lox config diff                     # diff current vs last backup
lox config diff backup1.zip backup2.zip   # diff two backups
lox config show                     # show decompressed config summary
lox config controls                 # list all controls from full config (more detail than LoxApp3)
```

Implementation:
- Download + decompress LoxCC files
- Parse the resulting XML
- Extract meaningful differences (controls added/removed/changed, room changes, etc.)
- The full XML config contains much more detail than LoxApp3.json (programming logic, virtual inputs, function blocks, connections)

### Tier 3: Config Validation / Audit (Medium Value, Medium Effort)

**`lox config audit`** — Analyze configuration for issues.

```
lox config audit                    # check for common issues
lox config audit --security         # security-focused checks
```

Possible checks:
- Controls without rooms assigned
- Unused virtual inputs/outputs
- Security settings (which controls are secured)
- Default passwords in use
- FTP/HTTP exposure settings
- Extension firmware versions

### Tier 4: Structure Export / Documentation (Medium Value, Low Effort)

**`lox export`** — Generate documentation from configuration.

```
lox export --format dot             # Graphviz control dependency graph
lox export --format csv             # spreadsheet of all controls
lox export --format html            # browsable documentation
```

Uses LoxApp3.json (already cached) + optionally the full LoxCC XML for deeper detail.

### Tier 5: Config Upload / Restore (High Value, High Risk)

**`lox config restore <backup.zip>`** — Upload a configuration backup.

```
lox config restore backup.zip       # upload via FTP
lox config restore backup.zip --reboot  # upload + trigger restart
```

Implementation:
- Upload ZIP to `/prog/` via FTP
- Optionally trigger reboot via `GET /jdev/sys/reboot`

**Risks:**
- Uploading a bad config can brick the Miniserver (requires SD card reflash)
- Version mismatches between config and firmware
- This is a **destructive operation** — must require explicit confirmation
- Should probably require `--force` flag and print warnings

### NOT Feasible: Live Config Modification

Modifying the Loxone programming (function blocks, logic connections, virtual inputs) from the CLI is **not feasible** because:
1. The compilation step (`.Loxone` → `sps0.LoxCC`) is proprietary and undocumented
2. The binary format has checksums/validation
3. Even modifying the decompressed XML and recompressing won't work — Loxone Config does additional processing
4. Risk of corrupting the configuration is very high

The only safe modification path is through Loxone Config software.

---

## Recommended Implementation Priority

| Priority | Command | Effort | Risk | Value |
|----------|---------|--------|------|-------|
| 1 | `lox backup` (FTP download) | Medium | Low | High — disaster recovery |
| 2 | `lox backup list` | Low | None | Medium — see history |
| 3 | `lox backup --extract` (decompress LoxCC) | Medium | None | High — inspect config |
| 4 | `lox config diff` | Medium | None | High — track changes |
| 5 | `lox export` (CSV/docs from LoxApp3) | Low | None | Medium — documentation |
| 6 | `lox config audit` | Medium | None | Medium — proactive checks |
| 7 | `lox config restore` (FTP upload) | Medium | **High** | Medium — requires caution |

---

## Technical Notes

### FTP Implementation

The Miniserver's FTP server is standard. Rust options:
- [`suppaftp`](https://crates.io/crates/suppaftp) — modern, maintained, supports TLS
- Direct TCP with raw FTP commands (minimal dependency)

FTP credentials = same admin user/password as HTTP.

### LoxCC Decompression

Reference implementation: [sarnau's Python script](https://gist.github.com/sarnau/e14ff9fe081611782a3f3cb2e2c2bacd)

Algorithm (LZSS variant):
1. Check magic header `0xaabbccee`
2. Read compressed size, uncompressed size hint
3. Decompress using nibble-based copy/literal encoding:
   - Upper nibble = literal bytes to copy
   - If nibble == 15, read extension bytes
   - Read 2-byte back-reference offset
   - Lower nibble + 4 = bytes to copy from history
   - If nibble == 15, read extension bytes

### Security Considerations

- FTP transmits credentials in plaintext — only use on trusted local networks
- The `serial` config option enables DynDNS TLS hostname — but FTP itself is unencrypted
- Config backups may contain sensitive data (user passwords, API keys, network config)
- Downloaded backups should be stored with restricted permissions

---

## Sources

- [sarnau/Inside-The-Loxone-Miniserver](https://github.com/sarnau/Inside-The-Loxone-Miniserver) — Reverse-engineered Miniserver internals
- [How to decompress sps0.LoxCC](https://gist.github.com/sarnau/e14ff9fe081611782a3f3cb2e2c2bacd) — Python decompression script
- [Loxone Official API Documentation (2025)](https://www.loxone.com/wp-content/uploads/datasheets/CommunicatingWithMiniserver.pdf)
- [LoxBerry Miniserver Backup NG](https://wiki.loxberry.de/plugins/miniserverbackup-ng/start) — Automated backup plugin
- [LoxWiki — Loxone Config Software](https://loxwiki.atlassian.net/wiki/spaces/LOX/pages/1516635013/Loxone+Config+Software)
