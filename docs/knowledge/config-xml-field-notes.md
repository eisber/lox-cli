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

### Global operating mode creation (17.2.10.21 static recovery)

`config add --type operating-mode` is a dedicated branch of the existing add
command. Generic XML add is not equivalent: a real global mode has a numeric
`Mode` ID in addition to its UUID and belongs under `ModeCaption`, not a Page.

Frozen AMD64 executable SHA-256:
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
The concrete native chain is UI command `0x2997`
(`0x140131b10:2058–2065`) → `0x1401456a0` → `0x140145520` →
`0x1404eecc0` → `0x1404e8420`. Factory `0x140432790:155–160`
allocates `CControlObjectMode` through `0x1406d4fc0`.
Its primary vtable is `0x141063e08`; slot `+0xf18` points to
`0x1406d53e0`, which registers XML `Mode` and ordered I/Q connectors.
The base registration `0x14045ccb0:49–53` maps CT20 to `ModeCaption`.

The CT21 creation-switch byte at `0x1404ee0a9` is `0x0e`. That branch
(`0x1404e8420:870–910`) scans the collection's stored numeric IDs and chooses
the first unused negative integer, starting at -1 and decrementing. The CLI
requires the existing -1..-6 built-ins so this allocation actually produces a
custom ID; it does not invent a separate custom-ID range or Alarm bit mask.
`0x1406d5890` retains a fresh UUID for negative IDs; nonnegative IDs instead
use a deterministic builtin identity. `0x14042a510` appends the object and sets
WF `0x4000`, without changing existing sibling IDs/references.

Serializer `0x1406d5c30` persists IoData, Display and the signed `Mode`
attribute. It then resets accepted Display state to `StateOnly=1` and an empty
unit. The CLI encodes that accepted normalized state, not a byte-for-byte
first-native-save snapshot. Native creation's post-add default-values path was
checked rather than assumed: CT21 maps to false case 2 in `0x140461500`
(switch byte at `0x140461748` is `0x02`), so the registry default-values branch
in `0x14011fcf0:164–181` is not reached for this type.

`0x1405288f0:442–461` obtains `NumO` from `0x1404e6160`; one new ordinary
Mode contributes one C object, not three objects for its two Co records.
This operation increments the existing positive count by one, without
reproducing unrelated whole-project native save normalization.
Missing default IDs 0..9 or -1..-6 are refused rather than silently invoking
the native builtin-repair path. Only the newly added C fragment is serialized;
existing XML bytes remain intact except the Document count and save stamp.

Scope is deliberately an existing single, plain Document/ModeCaption
collection. Native accepted-add code `0x140145750:43–56` can set instance
WF `0x20000000`; the Document loader's direct `Template` child updates fields
`0x111c/0x1120` (`0x1405288f0:485–505`). Templates, IfT/uuidParent instance
state, ambiguous identities, numeric XML references in identity/name metadata,
and affected digit-like text (`lox-n37w`) are
therefore refused before any destination write. No native experiment or
qualification was performed for this recovery.

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
  `config central-light remove-link` removes one existing member by native UUID.
  Generated UI `0x1409803e0` creates `m_btnDelete` at dialog `+0x78`; constructor
  `0x14097ede0` binds it to `0x14097ff10`. Enable `0x1409802b0` only requires a
  selected object. The callback compares both UUID halves, removes exactly one
  entry via `0x14051fa40(index, 1)`, refreshes and restores selection; it contains
  no confirmation, minimum-count guard or replacement/default-link insertion.
  Accepted generic copies still use `0x1401b4270` and always run `0x140519890`:
  clear reference bit on every control, then mark every generic linked target.
  Removal therefore clears the target marker only after its last incoming
  generic reference disappears, unlike membership-preserving movement.
  Native `0x14047e260` reads per-control `V`; its `V >= 90` branch loads WF from
  a zero temporary and explicitly handles bit `0x10000000`. WF uses integer
  type 5 (`%d`, omit zero); an empty generic list is not emitted at all.
  The removal CLI conservatively requires explicit modern versions throughout
  the Document and a nonnegative signed32 changed WF, computes every delta
  before mutation, preserves unrelated bits and does not migrate legacy data.
  This is a minimal raw-preserving edit, not native full-resave normalization
  or byte/whole-model equivalence: the native writer clears `0x03000000`
  (bits 24/25) and may drop bits not loaded, while this CLI changes only the
  reference marker and preserves all other raw WF bits. Its native zero-omission
  comparison covers only WF made zero by marker removal, not whole-word/model
  equivalence.
  `config central-light move-link` edits this list for standalone CentralLight
  owners, **not** the central block's separate controlled-light list. The native
  `CControlLinkingSortingDlg` also serves tablet ambient lists through different
  callers/setters, so its title does not identify a universal persisted family.
  Generic caller `0x1401b4270` accepts into owner member `+0xdb8`;
  `0x14097ffd0`/`0x140980140` identify members by the selected control's full UUID,
  remove/reinsert one position, and leave the first/last boundary unchanged.
  Serializer `0x14047e260` persists ordered `linkC` through UUID-list type `0x16`
  in `0x1404c52f0`. Accepted changes invoke `0x140469a90` template propagation and
  `0x140519890` document-wide linked-marker recomputation. The CLI refuses the
  actual Document `Template` coupling and inconsistent reference graphs; it
  neither silently prunes unresolved links like native refresh nor rejects every
  unrelated `iObjS` field. Direct owner eligibility uses explicit `IoData Visu`,
  not an assumed row count. No native edit/save/reopen qualification is claimed.
  **Visibility is separate from raw membership:** refresh `0x14097f8d0` uses
  `0x140441250` to find the target or a qualifying ancestor through virtual
  `+0xb78`, then requires a nonempty display label. Resolved but hidden UUIDs
  remain in raw `linkC`. Button eligibility uses visible position, while movement
  callbacks use raw position. The CLI therefore admits movement only when all
  selected-owner members are direct `ModbusServer` targets with nonempty Titles:
  factory CT `0x30`, explicit XML registration `0x1407f37a0`, primary vtable
  `0x140f62ba8`, `+0xb78 -> 0x140460b20` (CT `0x30` returns true), and
  `+0x1a8 -> 0x140452310` (copies nonempty Title). Other native-visible families
  and ancestor paths are unproven CLI branches, not native limitations.
  Inspection still reports raw persisted order. Native UI-history/undo is not
  qualified or emulated.
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
