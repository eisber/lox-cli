# Design: `lox backup` — Miniserver Configuration Backup

> **Status: IMPLEMENTED** — All phases done. Renamed to `lox config` (see [design-backup-inspect.md](design-backup-inspect.md)).

> Download, inspect, and manage Loxone Miniserver configuration backups from the CLI.

## Motivation

A Loxone Miniserver configuration represents hours/days of programming work. Today, backups require either the Loxone Config desktop app or manual FTP access. `lox backup` brings this to the CLI — enabling automated backup scripts, config diffing, and disaster recovery without needing a Windows machine with Loxone Config.

---

## Commands

```
lox backup                              Download latest config backup
lox backup list                         List available backups on Miniserver
lox backup extract <file>               Decompress LoxCC → XML from a local backup ZIP
lox backup diff <file1> <file2>         Compare two backup ZIPs (summary)
lox backup restore <file> [--force]     Upload backup to Miniserver via FTP (dangerous)
```

### `lox backup`

Downloads the newest configuration backup ZIP from the Miniserver.

```bash
$ lox backup
Connecting to 192.168.1.77:21...
Found 3 backups in /prog/
Downloading sps_194_20260308182256.zip (247 KB)...
Saved to sps_194_20260308182256.zip

$ lox backup --output my-backup.zip
Saved to my-backup.zip

$ lox backup --extract
Downloading sps_194_20260308182256.zip (247 KB)...
Extracting sps0.LoxCC...
Decompressing LoxCC (247 KB → 1.8 MB)...
Saved to sps_194_20260308182256.Loxone
```

Flags:
- `--output <path>` — Custom output filename
- `--extract` — Also decompress the LoxCC to a `.Loxone` XML file
- `--json` — Machine-readable output (filename, size, timestamp)

### `lox backup list`

Lists all configuration backups stored on the Miniserver.

```bash
$ lox backup list
  #  Version  Date                 Size
  1  194      2026-03-08 18:22:56  247 KB  (current)
  2  193      2026-02-14 09:15:33  245 KB
  3  192      2026-01-03 20:41:07  243 KB

$ lox backup list --json
[{"filename":"sps_194_20260308182256.zip","version":194,"date":"2026-03-08T18:22:56","size":252416}]
```

### `lox backup extract <file>`

Decompresses a local backup ZIP's `sps0.LoxCC` into readable XML.

```bash
$ lox backup extract sps_194_20260308182256.zip
Extracting sps0.LoxCC...
Decompressing LoxCC (247 KB → 1.8 MB)...
Saved to sps_194_20260308182256.Loxone

$ lox backup extract sps_194_20260308182256.zip --output config.xml
Saved to config.xml
```

This is a **local-only** operation — no Miniserver connection needed.

### `lox backup diff <file1> <file2>`

Compares two backup ZIPs and shows a human-readable summary of changes.

```bash
$ lox backup diff old-backup.zip new-backup.zip
Config version: 193 → 194
Modified: 2026-02-14 → 2026-03-08

Controls:
  + Added: "Steckdose Garage" (Switch, Garage)
  ~ Changed: "Licht Küche" — room moved Küche → Wohnküche
  - Removed: "Unused VI 3" (VirtualInput)

Rooms:
  ~ Renamed: "Küche" → "Wohnküche"

Summary: 1 added, 1 changed, 1 removed
```

The diff parses the decompressed XML and compares structural elements (controls, rooms, categories, users). It does NOT diff the raw XML line-by-line.

### `lox backup restore <file> [--force]`

Uploads a backup ZIP to the Miniserver via FTP.

```bash
$ lox backup restore sps_194_20260308182256.zip
⚠  WARNING: Uploading a configuration backup will replace the current
   Miniserver programming. A bad configuration can require physical
   SD card access to recover.

   Current firmware: 14.5.12.3
   Backup file: sps_194_20260308182256.zip (247 KB)

   Use --force to proceed, or Ctrl-C to cancel.

$ lox backup restore sps_194_20260308182256.zip --force
Uploading to /prog/sps_194_20260308182256.zip (247 KB)...
Upload complete.
Reboot the Miniserver to apply: lox reboot
```

This is a **dangerous operation**. Safeguards:
- Requires `--force` flag (no interactive prompt — CLI is often non-interactive)
- Does NOT auto-reboot (user must explicitly run `lox reboot`)
- Prints firmware version to help catch version mismatches

---

## Architecture

### New Source File: `src/ftp.rs`

FTP client wrapper for Miniserver access.

```rust
pub struct LoxFtp {
    host: String,
    user: String,
    pass: String,
}

impl LoxFtp {
    /// Connect to Miniserver FTP (port 21)
    pub fn connect(cfg: &Config) -> Result<Self>;

    /// List files in a directory, returns Vec<FtpEntry>
    pub fn list(&self, path: &str) -> Result<Vec<FtpEntry>>;

    /// Download a file, returns bytes
    pub fn get(&self, path: &str) -> Result<Vec<u8>>;

    /// Upload a file
    pub fn put(&self, path: &str, data: &[u8]) -> Result<()>;
}

pub struct FtpEntry {
    pub name: String,
    pub size: u64,
    pub modified: Option<NaiveDateTime>,
}
```

### New Source File: `src/loxcc.rs`

LoxCC decompression (LZSS variant).

```rust
/// Decompress a sps0.LoxCC binary blob into XML bytes.
/// Returns error if magic header doesn't match.
pub fn decompress_loxcc(data: &[u8]) -> Result<Vec<u8>>;

/// Extract sps0.LoxCC from a backup ZIP, then decompress.
pub fn extract_and_decompress(zip_data: &[u8]) -> Result<Vec<u8>>;
```

### New Dependencies

```toml
# Cargo.toml additions
suppaftp = "6"    # FTP client (maintained, supports TLS)
zip = "2"         # ZIP archive extraction
```

Both are well-maintained, pure-Rust crates. `suppaftp` uses blocking I/O (matches our `reqwest::blocking` pattern). `zip` is the standard Rust ZIP library.

### Integration in `main.rs`

```rust
// New subcommand enum
#[derive(Subcommand)]
enum BackupCmd {
    /// Download latest configuration backup
    Download {
        #[arg(short, long)]
        output: Option<String>,
        #[arg(long)]
        extract: bool,
    },
    /// List available backups on Miniserver
    List,
    /// Decompress a local backup ZIP to XML
    Extract {
        file: String,
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Compare two backup ZIPs
    Diff {
        file1: String,
        file2: String,
    },
    /// Upload backup to Miniserver (dangerous)
    Restore {
        file: String,
        #[arg(long)]
        force: bool,
    },
}
```

The `Cmd::Backup` variant with no subcommand defaults to `Download`.

### Data Flow

```
lox backup
    │
    ├─ LoxFtp::connect(cfg)
    │      └─ TCP → miniserver:21 (admin credentials)
    │
    ├─ LoxFtp::list("/prog/")
    │      └─ parse filenames: sps_<ver>_<ts>.zip
    │      └─ sort by timestamp, pick newest
    │
    ├─ LoxFtp::get("/prog/sps_194_20260308182256.zip")
    │      └─ binary download
    │
    └─ write to disk
           └─ if --extract: zip::extract → loxcc::decompress → write .Loxone


lox backup extract <file>
    │
    ├─ read ZIP from disk
    ├─ zip: find sps0.LoxCC entry
    ├─ loxcc::decompress_loxcc(data)
    │      ├─ verify magic 0xaabbccee
    │      ├─ read header (compressed_size, uncompressed_hint)
    │      └─ LZSS decompress → XML bytes
    └─ write .Loxone file


lox backup diff <a> <b>
    │
    ├─ extract_and_decompress(a) → XML_a
    ├─ extract_and_decompress(b) → XML_b
    ├─ parse XML: extract controls, rooms, categories
    └─ compare maps, report additions/removals/changes
```

---

## LoxCC Decompression Algorithm

Reference: [sarnau/LoxCC Parser](https://gist.github.com/sarnau/e14ff9fe081611782a3f3cb2e2c2bacd)

```
Input:  compressed byte stream (after 16-byte header)
Output: decompressed XML bytes

1. Read header:
   magic     = u32_le  → must be 0xaabbccee
   comp_size = u32_le  → compressed payload size
   uncomp    = u32_le  → approximate uncompressed size (for pre-allocation)
   checksum  = u32_le  → validation (details unclear)

2. Decompress loop:
   while input remaining:
     a) Read 1 byte → split into high nibble (copy_count) and low nibble (ref_extra)
     b) Literal copy:
        - count = high nibble
        - if count == 15: keep reading bytes, add to count, until byte != 0xFF
        - copy `count` bytes verbatim from input to output
     c) Back-reference:
        - offset = u16_le from input (bytes_back into output buffer)
        - length = low nibble + 4
        - if low nibble == 15: keep reading bytes, add to length, until byte != 0xFF
        - copy `length` bytes from output[pos - offset] to output[pos]
        - (byte-by-byte copy — source and dest may overlap for run-length patterns)
```

This is a variant of LZ4/LZSS. The algorithm is ~50 lines of Rust.

---

## Backup ZIP Structure

A Miniserver backup ZIP (`sps_<ver>_<ts>.zip`) contains:

```
sps_194_20260308182256.zip
  └── sps0.LoxCC          ← the compressed config (the only file we need)
```

The filename encodes:
- `sps` — fixed prefix
- `194` — config version number (incremented on each save from Loxone Config)
- `20260308182256` — timestamp: 2026-03-08 18:22:56

---

## XML Config Structure (After Decompression)

The decompressed `.Loxone` XML contains the full Miniserver programming:

```xml
<LoxoneProject>
  <MiniServer>
    <Info SerialNr="..." Name="..." .../>
    <Program>
      <Page Name="Beleuchtung">
        <Control UUID="..." Name="Licht Wohnzimmer" Type="Switch" Room="...">
          <IoData> ... </IoData>
        </Control>
        <!-- function blocks, connections, logic -->
      </Page>
    </Program>
    <Rooms>
      <Room UUID="..." Name="Wohnzimmer"/>
      ...
    </Rooms>
    <Categories>...</Categories>
    <Users>...</Users>
  </MiniServer>
</LoxoneProject>
```

For `diff`, we compare:
- Controls: UUID → (name, type, room, category)
- Rooms: UUID → name
- Categories: UUID → name
- Users: list (without passwords)

We do NOT attempt to diff programming logic (function blocks, connections) — that's too complex and not useful from a CLI.

---

## FTP Connection Details

- **Port**: 21 (configurable in Loxone Config, but 21 is default)
- **Credentials**: Same admin user/password as HTTP API (from `~/.lox/config.yaml`)
- **Protocol**: Plain FTP (no FTPS) — the Miniserver doesn't support FTP over TLS
- **Paths**: Absolute from FTP root (e.g., `/prog/`, `/log/`, `/data/`)

### Security

FTP transmits credentials in plaintext. Mitigations:
1. Only connect over trusted local networks (same as current HTTP basic auth behavior)
2. Print a warning on first FTP use: `Note: FTP credentials are sent unencrypted. Use only on trusted networks.`
3. If `serial` is set in config, warn that FTP cannot use TLS (unlike HTTPS via DynDNS)

---

## Config Changes

No config changes needed. FTP uses the same `host`, `user`, `pass` from `~/.lox/config.yaml`. The FTP port (21) is hardcoded as the default; a `--port` flag can be added later if needed.

---

## Error Handling

| Scenario | Behavior |
|----------|----------|
| FTP connection refused | `Error: Could not connect to {host}:21 — is FTP enabled on the Miniserver?` |
| FTP auth failed | `Error: FTP login failed — check your admin credentials in lox config` |
| No backups in `/prog/` | `No configuration backups found on the Miniserver.` |
| Invalid LoxCC magic | `Error: File is not a valid LoxCC file (bad magic header)` |
| ZIP missing sps0.LoxCC | `Error: Backup ZIP does not contain sps0.LoxCC` |
| Restore without --force | Print warning, exit with code 1 |
| Network timeout | Standard anyhow error propagation |

---

## Implementation Phases

### Phase 1: Core (MVP)

- [ ] `src/ftp.rs` — FTP client wrapper using `suppaftp`
- [ ] `src/loxcc.rs` — LoxCC decompression algorithm
- [ ] `lox backup` — download latest backup
- [ ] `lox backup list` — list available backups
- [ ] `lox backup extract` — local decompression
- [ ] Tests for LoxCC decompression (use a known compressed/decompressed pair)
- [ ] Tests for backup filename parsing

### Phase 2: Diff

- [ ] `lox backup diff` — structural comparison of two configs
- [ ] XML parsing for controls, rooms, categories
- [ ] Colored diff output (controls added/removed/changed)

### Phase 3: Restore (Later)

- [ ] `lox backup restore` — FTP upload with safety checks
- [ ] Firmware version check before restore
- [ ] This is the riskiest feature — implement last, test carefully

---

## Testing Strategy

### Unit Tests

```rust
#[test]
fn test_loxcc_decompress_known_input() {
    // Small hand-crafted LoxCC blob → expected XML output
    let compressed = include_bytes!("../testdata/test.LoxCC");
    let xml = decompress_loxcc(compressed).unwrap();
    assert!(xml.starts_with(b"<?xml"));
}

#[test]
fn test_parse_backup_filename() {
    let entry = parse_backup_name("sps_194_20260308182256.zip").unwrap();
    assert_eq!(entry.version, 194);
    assert_eq!(entry.year, 2026);
}

#[test]
fn test_loxcc_bad_magic() {
    let bad = vec![0x00; 100];
    assert!(decompress_loxcc(&bad).is_err());
}
```

### Integration Tests

FTP tests require a live Miniserver. These should be gated behind an environment variable:

```rust
#[test]
#[ignore] // run with: cargo test -- --ignored
fn test_ftp_list_prog() {
    let cfg = Config::load().unwrap();
    let ftp = LoxFtp::connect(&cfg).unwrap();
    let entries = ftp.list("/prog/").unwrap();
    assert!(!entries.is_empty());
}
```

### Test Data

Commit a small synthetic `testdata/test.LoxCC` file (hand-crafted, not from a real Miniserver) for decompression tests. Real Miniserver configs should NOT be committed (may contain sensitive data).

---

## Open Questions

1. **FTP port configuration** — Should we add `ftp_port` to `config.yaml`, or is port 21 always sufficient?
2. **Backup retention** — Should `lox backup` auto-rotate old local backups? Or keep it simple (one file per invocation)?
3. **Scheduled backups** — Out of scope for the CLI itself, but document how to use `cron` + `lox backup` for automated backups.
4. **Miniserver Gen2** — Does the Gen2 Miniserver use the same FTP/LoxCC format? Needs testing.
