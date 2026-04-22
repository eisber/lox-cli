# Design: `lox config` — Loxone Config Inspection

> **Status: IMPLEMENTED** — All commands implemented: `download`, `list`, `extract`, `upload`, `users`, `devices`, `diff`.

## Command Rename

"Loxone Config" is the well-known desktop application for programming Miniservers.
The `lox config` namespace should map to that concept — operating on Loxone Config
project files — rather than managing CLI connection settings.

### Rename plan

| Before | After | Rationale |
|--------|-------|-----------|
| `lox config set/show` | **`lox setup set/show`** | CLI connection settings → `setup` (run-once onboarding feel) |
| `lox backup download` | **`lox config download`** | Downloading a Loxone Config from the Miniserver |
| `lox backup list` | **`lox config list`** | Listing Loxone Configs on the Miniserver |
| `lox backup extract` | **`lox config extract`** | Decompressing LoxCC → `.Loxone` XML |
| `lox backup restore` | **`lox config upload`** | FTP upload; `upload` says what happens without implying automatic activation (still needs `lox reboot`) |
| *(new)* | **`lox config users`** | List user accounts from `.Loxone` file |
| *(new)* | **`lox config devices`** | List hardware inventory from `.Loxone` file |

### Full command tree after rename

```
lox setup set --host ... --user ... --pass ... --serial ...
lox setup show

lox config download [--output <file>] [--extract]
lox config list
lox config extract <file.zip> [--output <file>]
lox config upload <file.zip> --force
lox config users <file.Loxone>
lox config devices <file.Loxone>
```

## Background

The `lox backup` PR (#27) adds `download`, `list`, `extract`, and `restore` commands
for Miniserver configuration backups. The `extract` command decompresses the custom
LoxCC format to produce a `.Loxone` XML file — the full configuration as authored in
the Loxone Config desktop application.

This XML contains significantly more detail than the runtime `LoxApp3.json` structure
file. Two categories of data are **completely absent from the REST API and LoxApp3**:

1. **User accounts** — who has access, NFC authentication status, account descriptions
2. **Physical hardware inventory** — every Tree, LoxAIR, and network device with serial numbers and type codes

These are read-only inspection commands with zero manipulation risk.

## Motivation

**Users:** The Miniserver has no REST endpoint for user management. The only way to
see user accounts is through the Loxone Config Windows app or by reading the `.Loxone`
file. For a headless Miniserver admin (the target audience of `lox`), there is currently
no way to answer: "Which users have NFC access?", "Is the HomeAssistant integration
account still configured?", or "How many accounts exist?"

**Devices:** The REST API provides system-level diagnostics (`/dev/sys/heap`,
`/dev/cfg/version`) but no device inventory. You cannot enumerate Tree sensors, LoxAIR
actuators, or their serial numbers via HTTP. For maintenance ("what's the serial of
the broken room sensor?") or inventory ("how many smoke detectors do I have?"), the
only option is the Windows app or the `.Loxone` file.

## Data Source

Both commands parse the `.Loxone` XML file produced by `lox config extract`. The XML
uses `<C Type="...">` elements throughout:

### User entries

```xml
<C Type="User" V="174" U="15ea0aa5-024c-3a55-ffffed57184a04d2"
   Title="christoph" NFCArr="F1A2B3..." Desc="">
  <HP H="612097D6..." Sc="1" S="31663831..." Alg="1"/>
</C>
```

Key attributes:
- `Title` — username
- `NFCArr` — present (non-empty) when user has NFC authentication configured
- `Desc` — optional description (e.g. "Für_Matter_Integration_(Alexa)")
- `Admin` — admin flag (rare; most use permission groups)
- `<HP>` — password hash (hashed, never plaintext) — we do NOT display this

### Device entries

```xml
<!-- Tree wired device -->
<C Type="TreeDevice" V="174" U="..." Title="Raumklima Schlafzimmer"
   Serial="B03C7ED6" SubType="32784" HwVersion="1">
  <C Type="TreeAsensor" IName="AI1" Title="Temperatur" .../>
  <C Type="TreeAsensor" IName="AI2" Title="Feuchtigkeit" .../>
</C>

<!-- LoxAIR wireless device -->
<C Type="LoxAIRDevice" V="174" U="..." Title="Rauchmelder Keller"
   SubType="19">
  <C Type="LoxAIRAsensor" IName="Alarm" Title="Alarm" .../>
</C>

<!-- Network device -->
<C Type="NetworkDevice" V="174" U="..." Title="Intercom"
   SubType="2" Addr="ice0b354:7091" MAC="504F94E0F182"/>
```

Key attributes:
- `Title` — human name
- `Serial` — hardware serial number (Tree devices have this; LoxAIR devices may not)
- `SubType` — device type code (see mapping table below)
- `HwVersion` — hardware revision
- `MAC`, `Addr` — network details (NetworkDevice only)

### Known SubType codes (from config analysis)

| SubType | Device |
|---------|--------|
| 7 | Nano IO Air (relay) |
| 19 | Smoke detector |
| 32 | Water sensor |
| 37 | Smartaktor (RGBW/dimmer) |
| 48 | Room climate sensor (LoxAIR) |
| 32780 | Dimmer (Tree) |
| 32784 | Room climate sensor (Tree) |
| 32794 | Presence sensor (Tree) |
| 32796 | Code Touch keypad (Tree) |
| 32797 | LoxAIR Bridge |
| 32799 | Flex connector (Tree) |

## New Commands

### `lox config users <file>`

List user accounts from an extracted `.Loxone` config file.

```
$ lox config users sps_0259_20260301111229.Loxone

  Name                      NFC    Description
  admin                     -
  homeassistant             -
  loxberry                  -
  bastian                   yes
  christoph                 yes
  conny                     -
  matter                    -      Für Matter Integration (Alexa)
  homeassistant-pyloxone    -
  airwin                    -

9 users (2 with NFC)
```

With `--json`:
```json
[
  { "name": "admin", "nfc": false, "description": "" },
  { "name": "christoph", "nfc": true, "description": "" },
  ...
]
```

### `lox config devices <file>`

List physical hardware devices from an extracted `.Loxone` config file.

```
$ lox config devices sps_0259_20260301111229.Loxone

  Tree devices (8)
  Name                          Serial      Type
  Raumklima Schlafzimmer        B03C7ED6    Room climate sensor
  Raumklima Bastian             B03C7ED2    Room climate sensor
  Raumklima Sophie              B03C7ED0    Room climate sensor
  Code Touch Haustür            B04A9AFB    Code Touch keypad
  Flex Durchgang                B05DF106    Flex connector
  Dimmer Tisch                  C0172338    Dimmer
  Dimmer Insel                  C017305E    Dimmer
  Präsenz Küche                 B02E4EB5    Presence sensor

  LoxAIR devices (37)
  Name                          Type
  Raumklima Wirtschaftsraum     Room climate sensor
  Rauchmelder Keller            Smoke detector
  Wassersensor Küche            Water sensor
  Relais Jalousie Fenster       Nano IO Air (relay)
  ...

  Network devices (1)
  Name                          Address           MAC
  Intercom                      ice0b354:7091     504F94E0F182

46 devices total
```

With `--json`: flat array with `"bus"` field (`"tree"`, `"air"`, `"network"`).

## Implementation

### File input

Both commands accept a `.Loxone` file path. This is the XML produced by `lox config
extract`, not the raw `.zip`. If the argument ends in `.zip`, print a hint:

```
Error: Expected a .Loxone XML file. Run `lox config extract <file>.zip` first.
```

### Parsing

Add a new `src/loxone_xml.rs` module (or extend `src/loxcc.rs`) with:

```rust
pub struct LoxUser {
    pub name: String,
    pub nfc: bool,
    pub description: String,
}

pub struct LoxDevice {
    pub name: String,
    pub bus: DeviceBus, // Tree, Air, Network
    pub serial: Option<String>,
    pub sub_type: Option<u32>,
    pub type_label: String, // human-readable from SubType map
    pub mac: Option<String>,
    pub address: Option<String>,
}

pub enum DeviceBus { Tree, Air, Network }

pub fn parse_users(xml: &[u8]) -> Result<Vec<LoxUser>>;
pub fn parse_devices(xml: &[u8]) -> Result<Vec<LoxDevice>>;
```

Use `quick-xml` or `xml-rs` for parsing. Since we only need to iterate `<C>` elements
and read attributes, a streaming/event-based parser is sufficient — no need for a full
DOM. However, the file is only ~1 MB, so even a full DOM parse is fine.

**Dependency choice:** Prefer `quick-xml` (already widely used in the Rust ecosystem,
small, fast, serde-optional). Alternatively, `xml-rs` works but is slower. Avoid
pulling in a heavy dependency — this is simple attribute reading.

### CLI integration (src/main.rs)

Rename existing `ConfigCmd` → `SetupCmd`, then add new `ConfigCmd` enum:

```rust
/// Loxone Config operations — download, inspect, upload project files
#[derive(Subcommand)]
enum ConfigCmd {
    /// Download the latest config from the Miniserver via FTP
    Download { ... },
    /// List configs on the Miniserver
    List,
    /// Decompress a backup ZIP to .Loxone XML
    Extract { ... },
    /// Upload a config to the Miniserver via FTP (dangerous — requires --force)
    Upload { ... },
    /// List user accounts from a .Loxone config file
    Users { file: String },
    /// List hardware devices from a .Loxone config file
    Devices { file: String },
}

/// CLI connection settings
#[derive(Subcommand)]
enum SetupCmd {
    Set { ... },
    Show,
}
```

Both `users` and `devices` support the existing global `--json` flag.

### SubType label mapping

A `fn subtype_label(sub_type: u32) -> &'static str` function maps known codes to
human-readable names. Unknown subtypes display as `"Unknown (<code>)"`. The table
can be extended as more device types are encountered.

## What We Explicitly Skip

| Feature | Reason |
|---------|--------|
| Wiring graph / dependency map | 770 connections link connector-pin UUIDs (`<Co>`), not control UUIDs. Requires two-level traversal and produces deeply Loxone-internal output. High complexity, niche audience. |
| Display unit enrichment | LoxApp3.json already has unit/format info in control `details`. Marginal gain. |
| Operating mode schedules | Complex condition trees. Already partially exposed via REST `operatingModes`. |
| Config modification | High risk. Bad configs can brick the Miniserver (requires physical SD card recovery). |
| Password / hash display | Security-sensitive. We confirm NFC presence but never expose hashes or salts. |

## Test Plan

- Unit tests for XML parsing with synthetic XML fragments (users, devices, mixed)
- Unit tests for SubType label mapping (known codes + unknown fallback)
- Unit test for `.zip` extension detection (hint message)
- Manual test against real `sps_0259_20260301111229.Loxone` file
- `cargo clippy` clean
