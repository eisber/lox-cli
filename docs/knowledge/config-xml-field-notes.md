# Config XML Field Notes

Practical notes for editing `.Loxone` project files by hand or with `lox config`.
They come from several weeks of edits to a real house project (Loxone Config
17.2, element version `V="178"`), each one uploaded and checked on a Miniserver.
Where a note contradicts the official docs in `docs/kb/`, the observed behaviour
is described and the difference is called out.

---

## 1. Saving a file Config will accept

### Document save stamp
The `<C Type="Document">` element carries `Date="YYYY-MM-DD HH:MM:SS"` (local
time) and `DateS` (whole seconds since 2009-01-01 00:00:00 UTC, the Loxone
epoch). Example: `Date="2026-10-12 07:55:33"` in Europe/London (UTC+1) is
`DateS="561020133"`.

Loxone Config decides whether an opened file is the same program as the one on
the Miniserver **by this stamp, not by content**. If an edited file keeps the old
stamp, *Connect* reports the program as identical and Config loads the
Miniserver's copy, so the edits are silently lost. Bump both values whenever you
write an edited file.

### Formatting
- Loxone's format uses a UTF-8 BOM and CRLF line endings, while attribute values
  (note text, AutoPilot JSON) can contain bare LF or CRLF. Preserve those
  attribute newlines independently of structural line endings. The write-back
  code is in `src/config_edit/write.rs`; versions that convert every LF to CRLF
  also change bare LF inside attributes, so check mixed-newline round-trips.
- Config 17.2 shows **"INVALID FORMULA"** in the block's tooltip on formulas that
  work, including long-running ones. Judge a formula by its Liveview output, not
  by that label. Keeping constants decimal-free (`875/100` rather than `8.75`)
  avoids decimal-comma questions in localised Configs.
- Config replaces `%` in titles with `_` on save.

### Generated identities and native counters

`lox config` intentionally generates random 64-bit UUID prefixes in the
`8-4-4-16` hex layout, reusing an existing `ffff` suffix for blocks **and**
connectors. The legacy helper name `find_miniserver_serial` does not prove that
the suffix is a hardware serial: native projects can contain identifiers from
multiple Config installations. With no existing suffix, generation uses
`ffff000000000000`. Random identifiers make collisions unlikely, not impossible;
they are not a proven disjoint namespace from native identifiers.

The CLI preserves `ControlList`'s `NextObj` and `NextMem`; it does not attempt to
emulate the desktop editor's unverified allocation algorithm. These fields must
not be reset to element counts or compared with an assumed UUID segment index.
For example, the public native fixtures under `tests/eval/golden/configs/` have:

| Fixture | `NextObj` | `NextMem` | `C` elements | `Co` elements | Largest UUID second group (hex decoded) |
|---------|-----------|-----------|--------------|---------------|----------------------------------------|
| `Cleaning Mode Activated by NFC.Loxone` | 29 | 1 | 429 | 410 | 854 |
| `Network Monitoring.Loxone` | 1 | 1 | 327 | 198 | 690 |

Neither these snapshots nor the permissive project XSD establish a native
counter-to-UUID derivation. `lox config validate` checks UUID shape and reference
integrity, not a speculative counter lower bound. Synthetic edit/save/reload
tests cover generated block/connector uniqueness and suffix preservation; this
is **not** evidence of a native-editor or Miniserver round-trip. Matching native
allocation remains deferred until the encoding and round-trip are verified.

### Virtual input registration

`add-virtual-in` (also `add --type virtual-input`) emits `WF="16400"`, `Idx="-1"`
and `IoData` room/category references. Each nonempty parent `Pr`/`Cr` is
preserved independently; a missing or empty reference falls back to the first
corresponding `Place`/`Category`, when available. Create rooms/categories first
if the project has none. When a programming `Page` exists, the `InputRef`
converter is placed there, never inside the `VirtualIn`. Page-free projects
retain their existing behavior: the virtual input is added without a converter.

---

## 2. Wires

| Pattern | Meaning |
|---------|---------|
| `<In Input="<connector U>"/>` | Wire from that output connector (same page) |
| `<In Input="..." FLG="2"/>` | Wire from a block on **another page** (drawn as an off-page label) |
| `<In Input="..." FLG="1"/>` | Link managed by Config itself (presence-sensor `DeviceActivate`, AutoPilot inputs). Not user wiring; don't add, remove or "fix" these |

- A cross-page wire without `FLG="2"` is not drawn correctly.
- Function block inputs accept **several wires**; digital inputs are OR-ed. To
  trigger an existing behaviour from a new source, add a wire to the existing
  input instead of adding a block or a mood.
- **InputRef outputs:** wire references to devices and virtual inputs from the
  ref's `AQ` connector. A wire from `Q` on such a ref is neither drawn nor used
  by Config. `Q` is only used on references to `Memory` flags.

---

## 3. Users and permissions

Object permissions live inside the user (or user group):

```xml
<C Type="User" Title="api-user" ...>
    <HP .../>                       <!-- password hashes: never touch -->
    <PH NUM="2">                    <!-- NUM = number of PRM children -->
        <PRM REF="<object U>" ACT="3"/>
        <PRM REF="<object U>" ACT="3"/>
    </PH>
</C>
```

An HTTP call such as `/dev/sps/io/<virtual input>/pulse` by a non-admin user
needs effective permission for that input. If no inherited group/room/category
permission grants access, a direct `PRM` entry is one way to grant it; explicit
denies still take precedence. Without effective access the Miniserver answers
`Code="403"` (forbidden); a wrong password gives `401`. A webhook user that
otherwise lacks access therefore needs permission for a new virtual input.

---

## 4. Lighting Controller (`LightController2`)

XML connector names vs. the abbreviations in `docs/kb/lighting-controller.md`:

| XML `K` | Docs | Notes |
|---------|------|-------|
| `Move` | Mo | Motion |
| `Presence` | P | Presence: mood stays on while 1; after manual operation, off after Met |
| `EnMove` | DisP | Disable motion/presence when 1 (despite the name) |
| `Reset` | Off | Pulse switches off. Does **not** start the Pto pause |
| `Sel1`…`Sel8` | T5/1…8 | Select the mood assigned to that input, not necessarily mood ID x |
| `Brightness` / `BrightnessLimit` | Br / Brt | Lux input / threshold |
| `MoveOn` | Moet | Motion extend time. For motion, the shorter of Met and Moet applies |
| `MoveTimeout` | Met | Manual operation extend time |
| `MoveIgnore` | Pto | Motion pause after a manual switch-off |

- **Mood pulses should be short.** In the observed project, pulses longer than
  about 0.2 s into `Sel<x>` caused long-click behaviour and **mixed** the assigned
  mood in (logged as e.g. `"Dim All,Dim Wall"`) instead of switching to it.
  `PulseTime` ≤ 0.2 was a safe choice on edge-detection blocks feeding these
  inputs, not a universal mixing threshold: `SceneMixTime` (Mmd, documented
  default 1 s) and the input's mood/presence settings control the action.
  Further pulses within 30 s step to the next mood; that is normal T5 behaviour.
- The `MoveOn` / `MoveTimeout` mapping above follows
  `docs/schemas/loxone-block-types-full.json`; the similar XML names are easy to
  confuse. `ScMv` mood IDs below are stored XML IDs, not the public API's
  built-in mood numbers (0 = Off, 99 = All on).
- **Motion mood per operating mode** is the child
  `<LSConfig OpM="-7,-10,-8,778" ScMv="3,3,3,777" ScAc="0,0,0,0"/>`: `OpM` lists
  operating-mode IDs (negative IDs are the project's own modes), `ScMv` the mood
  ID used for motion in that mode (`0` = none), `ScAc` the action. `OpM` `778` is
  the "all other times" row. **A controller without a row for the active
  mode and without a 778 row does nothing on motion**, which shows up as "motion
  stops working at night".
- Mood IDs: `777` is the built-in all-on mood, `778` is Off.
- A presence-sensor pulse into `Presence` plus a manual mood change makes the
  lights switch off after `MoveTimeout` (Met). That's a common cause of "lights go
  off by themselves".

### Native mood-output storage observation

One Windows Loxone Config **17.2.10.21** case preserved a CLI-authored
`LightsceneC Q2="1610622736"` for the same controller, mood UUID, SID and CID
through ordinary **Save As, full process close, and fresh-process reopen**.
The saved and reopened files were byte-identical; the original input remained
unchanged. This qualifies storage preservation for that case, not native mood
editing, every `Qn`, the `Qn`/`AQn` mapping, or actuator/runtime colour behavior.

Save As itself was **not byte-preserving**: the complete retained diff included
94 `Title` changes from German labels to English labels and two `WF` changes.
The cause and generality of those rewrites remain unqualified. Do not copy
them into the CLI as normalization rules or ignore them in native comparisons.

Private evidence: `vm122-saveas-pauseaware-20261110`,
strict raw manifest SHA-256
`533bc7454b2a921c3309cc13a10c03c69b08bccc5e7c620315f1e33777efdf1f`.
No raw vendor project is published with this observation.

---

## 5. Automatic Shading (`AutoJalousie`)

| XML `K` | Docs | Notes |
|---------|------|-------|
| `AutoShade` | Sps | Sun position automatic start |
| `EnAutoShade` | DisSp | **Disables** the sun automatic when 1 (despite the name) |
| `ReactAutoShade` | Spr | Restart the sun automatic |
| `EndUp` / `EndDown` | Co / Cc | Complete open / close. Counts as manual: cancels the sun automatic for the day |
| `AutoShadeEnd` | Spe | End action. Only runs at the computed "automatic shading until" time, not when the sun goes in |
| `DirTol` / `DirTol2` | Dts / Dte | Direction tolerance entering / leaving. If Dte is narrower than Dts, shading can stop straight after it starts |
| `Back` | Rd | Venetian: slat return time. Roller blinds (`Type` 1): position 0.1 open to 1.0 closed |
| `OutputAutoShade` | Sp | Sun automatic active |
| `OutputPos` | Pos | 0 = open, 1 = closed |

Roller blinds are positioned by travel time, so starting the sun automatic while
the blind is still moving leaves its position off. A common guard is a formula on
`EnAutoShade` that disables the automatic unless the blind is fully open or
already shading.

---

## 6. Other blocks

- **Remanence:** an unconnected inverted input means "on":
  `<Co K="Remanence" Inv="true" .../>` = remanent (survives restarts and uploads).
- **Combined window contact** (`JoinWindowSensor`): its `AQ` connector is the
  docs' status output (S): 1 = closed, 2 = tilted, 3 = open, 4 = closed and not
  locked, 5 = closed and locked, 0 = a sensor is offline. Test for "open" with
  `Equal(AQ, 3)`, not `AQ > 0`.
- **Presence block** (`PresenceDetector`): sensors are listed in `DEVS="dev1,dev2"`
  and linked with Config-managed `FLG="1"` `DeviceActivate` wires. Tree presence
  sensors in `DEVS` adopt the block's extend time (`ParamTOn`).
- **Linked functions** are stored as `linkC="uuid1,uuid2"` on the block, and each
  linked item needs `WF |= 0x10000000`; without that flag the app shows an empty
  *Linked Objects* list.
- **App favourites:** `IoData UseInFav="true"`. `IoData Rating` is the user's own
  ordering in the app; leave it alone.

---

## 7. Event log (Tracker)

A Tracker shows time-stamped events in the app. It lives under *Messages*
(`LoggerOutCaption`), not on a page:

```xml
<C Type="Tracker" V="178" U="<tracker U>" Title="Front Door - Log" WF="16384" Entries="100" Rem="true">
    <IoData Visu="true" Cr="<category U>" Pr="<room U>"/>
</C>
```

Blocks write to it through `OutputRefLM` references placed on a page:

```xml
<C Type="OutputRefLM" V="178" U="..." Title="Front Door - Log" Px=".." Py=".." Px2=".." Py2=".."
   Cl="0,0,0" Nio="2" Ref="<tracker U>" RefL="<tracker U>">
    <Co K="AI" Nc="1" U="..."><In Input="<source connector U>"/></Co>
    <Co K="AQ" U="..."/>
    <LoggerMailer RefTracker="<tracker U>" On="Webhook received" MinimumTime="5000"/>
</C>
```

`On` / `Off` are the texts written on the rising / falling edge (`<v>` inserts
the value); leave out `Off` to log only the rising edge. `Entries` is 2–100,
`Rem="true"` keeps entries across restarts. The same pattern with
`RefLogger` / `RefNotification` writes to a logger or a notification. Placing a
log reference after each step of a chain shows where a signal stops. See also
`tests/eval/golden/configs/UseCase-flood-warning.loxone`.
