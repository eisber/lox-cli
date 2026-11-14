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
  This is a storage observation, not proof that arbitrary inputs expose the
  same editor or that Remanence can be enabled by a general inversion operation.
- **Ordinary And input inversion (17.2.10.21, source-only):** the programming
  view hit-test `0x14014a670:252–269` returns action 5 and a connector index only
  after the owner's `+0xc20` capability and `+0x5d0` eligibility checks.
  The click branch `0x1401e6630:719–735` retrieves that same connector, rechecks
  eligibility, and toggles only bit `0x20` in connector state `+0x80`, then calls
  `0x140469a90`. This is not an analog sign, output or rising-edge operation.
  Type 331 is allocated as `CControlObjectLogic` by `0x140432790:2446–2459`;
  its original PE vtable `0x140f5c790` resolves `+0x5a8` to false,
  `+0x5d0` to `0x140468540`, `+0xc20` to `0x140463f70`, and descriptor factory
  `+0xf18` to `0x1407e7280`. Factory lines 41–66 declare digital `I1`, `I2`
  and output `Q`; those exact literals were read from the frozen original PE.
  The `I1`/`I2` descriptors have neither capability exclusion `0x40000000`
  nor special eligibility bit `0x400000`, so the native gate admits both
  connected and unconnected ordinary inputs. `0x140467520:87–92` and
  `0x140463f70` close the digital capability branch; zero constructor flags
  alone are not used as an effective-load proof.
  Modern connector loading `0x14047fb20:175–281` matches descriptor key, loads
  UUID, Def, Inv and references; `0x14043d790` copies that accepted storage
  into the existing connector. The writer at `0x14047fb20:84–148` emits
  `Inv=true` iff bit `0x20` is set and omits it otherwise, leaving Def and
  reference traversal independent. Initial unconnected digital state derives
  from that bit in `0x1404416c0:37–50`; setting Def=1 is not inversion.
  `0x140469a90` can set persisted WF `0x20000000` and propagate through
  references in template-instance documents. `0x140525ac0:116` initializes
  both template fields to zero, `0x1405288f0:485–505` loads Template/IfT,
  and `0x14050be70` reads accepted `+0x1120`. Therefore this CLI branch
  refuses every Template/IfT/uuidParent/DocumentTemplate occurrence rather
  than assuming a plain owning Document excludes derived state.
  `set-input-inversion` initially supports only unconnected existing I1/I2
  with neutral stored Def, complete three-connector owners and exact unique
  native35 identities. It refuses wired inputs instead of claiming reference
  graph parity, as well as namespaces, unsupported ancestor flags,
  device bindings and shared-loader rewrites (lox-n37w), even on no-op.
  The ordinary ancestry admits Page directly in the unique Document or through
  exactly one ordinary Program, matching Config-saved projects. The already
  recovered hit-test enumerates current Page children (`0x14014a670:85–104`);
  the closed capability/eligibility and selected-connector transaction do not
  predicate on the outer XML depth.
  On these guarded Document/Program/Page/And objects, absent WF, literal zero
  and literal `16384` (`0x4000`) are accepted and preserved. This correction is
  justified by the already-closed operation, not by golden frequency or a
  constructor default: `0x140463f70` and `0x140468540` inspect connector
  descriptors/signal kind rather than C.WF, and the only recovered persisted
  marker propagation in `0x140469a90` requires accepted IfT and concerns
  `0x20000000`, not `0x4000`. The no-Template guard keeps that gate false.
  Other flags/combinations remain refused; no universal meaning for `0x4000`
  or whole-native-resave equivalence is inferred. A regression adds an And
  with the public CLI to the unchanged Guest Room Door golden Page and edits
  its input while all four original/generated WF values remain `16384`.
  The local raw reader follows the existing Autopilot pattern: preserve
  unrelated raw-value-identical duplicate attributes byte-for-byte, but
  refuse conflicting duplicates, any duplicate on guarded objects and
  duplicate global identity fields before no-op/stamp/save. Parser failures
  are contextualized as input-inversion refusals. No shared-parser refactor
  or additional native tracing was needed for these review corrections.
  No native UI execution, Miniserver work or new native qualification was used.
- **Ordinary Or input inversion (17.2.10.21, source-only owner extension):**
  this is a separately bound owner branch of the existing operation, not
  inferred And/Or symmetry. Registry construction `0x1404ba670:78–110` allocates
  type 333 (`0x14d`), invokes its actual descriptor virtual `+0xf18`, and indexes
  that descriptor by type and XML name. The actual factory
  `0x140432790:2446–2459,4182–4185` assigns `CControlObjectLogic` and stores
  type 333. Its registered primary vtable `0x140f5c790` has the same concrete
  `+0x5a8=false`, `+0x5d0=0x140468540`, `+0xc20=0x140463f70` and
  `+0xf18=0x1407e7280` bindings used by the closed ordinary-input transaction.
  The **Or-specific** descriptor case `0x1407e7280:68–93` registers XML `Or`
  and exactly digital `I1`, `I2`, `Q`, with input descriptor flags
  `type & 0xfff` and `(type & 0xfff) | 0x1000`, output flags
  `(type & 0xfff) | 0x4002000`, and neutral zero default values.
  These literals and slots were checked against the frozen original PE.
  XML loading `0x140487c90:54–130` resolves `C.Type` through that name registry,
  instantiates its actual type, establishes parent/document ownership and
  invokes its loader. The registered `+8=0x14047e260` serializer reads modern
  `Nio` and matches the existing connectors by accepted descriptor key
  (`415–475`); its writer traverses the actual connector count (`327–351`).
  Thus explicit `Nio=3` and the complete unique I1/I2/Q layout bind the original
  XML, rather than relying on constructor-zero flags. The already-closed modern
  connector load/copy, eligibility, click and Inv writer above apply through
  those actual bindings, preserve Def independently, and do not read ordinary
  owner WF. The same guarded no-Template context prevents the persisted
  `0x20000000` marker/propagation branch. Other owners, outputs, connected inputs
  and special contexts remain unsupported.
  Admission now also verifies the entire raw XML has one balanced root before
  no-op/stamping/writing, refusing trailing hidden identities (lox-fd9l).
  Valid BOM, EOL, comments, processing instructions and unrelated raw storage
  remain byte-preserved. The shared-loader lox-n37w refusal remains local.
  The local raw reader validates PI targets with XML 1.0 NameStartChar/NameChar,
  refuses the case-insensitive reserved target `xml`, and checks PI XML
  characters before no-op/stamping/writing. Shared DOM acceptance is not used
  as strict PI proof; legitimate Unicode targets and unescaped PI data remain
  raw-identical. This is a local admission correction, not new native evidence.
- **Ordinary Xor/Not input inversion (17.2.10.21, source-only):** these are
  separately proven owner bindings of the existing input operation, not
  assumed And/Or symmetry. Actual factory cases `0x14e` (334) and `0x14a`
  (330) in `0x140432790:2446–2459` construct `CControlObjectLogic`; registration
  `0x1404ba670:78–110` calls its actual descriptor slot `+0xf18`.
  `0x1407e7280:95–121` registers exact XML `Xor`, digital inputs `I1`/`I2`
  and output `Q`; lines 22–40 register exact XML `Not`, digital input `I`
  and output `Q`. The literals were read from the frozen original PE.
  Thus modern explicit `Nio="3"` is required for Xor, but `Nio="2"` for
  Not: neither Not.I1 nor an inferred three-port Not layout is accepted.
  Both actual input descriptor cases supply neutral default zero and have
  neither analog/special/exclusion flags. Their registered vtable
  `0x140f5c790` concretely binds the closed capability `0x140463f70`,
  eligibility `0x140468540`, false `+0x5a8`, and loader `0x14047e260`.
  The actual connector constructor `0x140423420` installs PE table
  `0x140ec3a60`: `+0x78=0x140467520` returns digital for both actual types,
  `+8=0x14047fb20` loads/writes Inv, and `+0x18=0x14043d790` accepts the
  selected connector copy. This closes the same hit/action-5/click/bit-0x20
  transaction above for each new owner. All ordinary-context, neutral Def,
  raw preservation, no-op, identity and template exclusions remain unchanged.
  This adds owner branches, not a new base API or native qualification.
- **Ordinary latch input inversion (17.2.10.21, source-only owner branches):**
  actual factory cases `0x14f`/`0x150`/`0x151` in
  `0x140432790:2446–2459,4182–4185` construct `CControlObjectLogic` with native
  types 335/336/337, not catalog-name guesses. Registry
  `0x1404ba670:78–110` calls each actual owner's descriptor slot `+0xf18`.
  The separately registered cases in `0x1407e7280:122–234` declare exact XML
  `FlipFlop`, `RSFlipFlop`, `SRFlipFlop`. FlipFlop has four native connectors:
  `InputS`, `InputR`, parameter `Remanence`, output `Q`; RSFlipFlop and
  SRFlipFlop have five: `InputS`, `InputTrigger`, `InputR`, `Remanence`, `Q`.
  `0x14043dff0:39–91` creates every descriptor in native order, including
  parameters, with its independent numeric default (zero for these descriptors).
  Hence existing explicit `Nio="4"`/`"5"` and complete exact connector sets
  are required; neither `I1`/`I2` nor a three-port copy is valid.
  Their actual primary table `0x140f5c790` binds the same concrete capability,
  eligibility, false `+0x5a8`, accepted loader and connector factory as above.
  Each actual `InputS`/`InputR` descriptor has digital input role, neither
  analog `0x200000`, exclusion `0x40000000`, nor special `0x400000`.
  The concrete connector table `0x140ec3a60` binds signal kind `0x140467520`
  (digital for all three actual types), accepted copy `0x14043d790` and
  serializer `0x14047fb20`. Thus the native action-5/click/bit-`0x20`/`Co.Inv`
  transaction is independently closed for each owner's `InputS` and `InputR`.
  `InputTrigger` has special descriptor bit `0x400000`; the unconnected,
  noninverted branch returns false in `0x140468540:52–63`, so this CLI refuses
  it rather than extrapolating ordinary-input capability. `Remanence` is a
  parameter, not another editable input. The template propagation gate
  `0x140469a90` still requires accepted owning-Document IfT via
  `0x14050be70`; the existing global Template/IfT/uuidParent exclusions close
  it for these actual new owners as well. Ordinary absent/0/16384 WF remains
  preserved operation-specifically, not inferred from latch constructors.
  Only the selected input needs neutral Def and no wires: other inputs may
  stay wired and inverted, Remanence may keep its stored value, and output
  references, defaults, rights, positions and unknown bytes remain untouched.
  No latch runtime/clock/state persistence, Trigger/wired-input inversion,
  output/parameter operation, new base API or native qualification is claimed.
- **Ordinary Counter/HourCounter input inversion (17.2.10.21, source-only):**
  these are two new owner/input branches of `set-input-inversion`, not new
  APIs or inferred counter/timer symmetry. Factory `0x140432790:2921–2946`
  binds type 382 to `CControlObjectCounter`, primary PE table `0x140efae60`,
  and type 380 to `CControlObjectHourCounter`, table `0x140f42198`.
  Registry `0x1404ba670:78–110` calls each actual `+0xf18`: respectively
  `0x1407abf40` and `0x14073fcc0`. They register XML `Counter` with seven
  connectors and exact input `Trigger`, and XML `HourCounter` with ten
  connectors and exact input `InputEnable`. Each selected descriptor has
  flags `type & 0xfff` and neutral default zero. Other native keys are
  `Reset`/`Remanence`/`EndValue`/`Mode`/`Q`/`AQ` for Counter, and
  `Reset`/`InputResetAll`/`Unit`/`Maintenance`/`Q`/`AQ`/`AQl`/`AQr`/`AQs`
  for HourCounter. Parameter and output names are not alternative inputs.
  HourCounter's `0x1404bea20(...,1)` changes descriptor metadata `+0x104`,
  not the selected connector flags.
  Both actual tables bind `+0x4b0=0x140446520`, false `+0x5a8`,
  `+0x5d0=0x140468540`, `+0xc20=0x140463f70` and
  `+8=0x14047e260`. The accepted connector constructor `0x140423420`
  installs table `0x140ec3a60`; `+0x78=0x140467520:87–92` returns digital
  for these actual types and selected flags (neither bit 21 nor bit 31).
  There is no selected special/exclusion bit `0x400000`/`0x40000000`.
  Thus the concrete capability and hit/action-5/click path above admits
  these two ports; neither the word Trigger nor the analog AQ output is
  used to infer eligibility.
  Modern name-registry loading `0x140487c90:110–130` sets the actual
  parent/document, then invokes each bound loader. `0x14047e260:415–475`
  accepts the complete explicit `Nio=7`/`Nio=10` layout by descriptor key;
  connector `+0x18=0x14043d790` copies UUID, Def and Inv independently.
  Connector `+8=0x14047fb20:84–103,175–281` loads/writes owned Inv,
  independently of unselected defaults and reference traversal;
  `0x1404416c0:37–50` derives unconnected digital state from bit `0x20`.
  For each new owner, the actual click calls `0x140469a90`, but its
  persisted WF29/reference propagation gate requires the accepted owning
  Document's IfT (`0x14050be70`, `0x140525ab0`). Reused constructor/loader
  `0x140525ac0:116` and `0x1405288f0:485–505`, with the existing global
  Template/IfT/uuidParent/DocumentTemplate refusal, keep that gate false.
  The concrete capability/eligibility functions do not inspect ordinary
  C.WF, so the existing absent/0/16384 ancestry restriction is also proved
  for these two branches, without assuming counter flag symmetry.
  Only the selected unconnected neutral-Def input is admitted; other wires,
  inversion/defaults, remanence, maintenance, references, rights, positions,
  UUIDs, order and raw storage are preserved, not reset or normalized.
  Reset inputs, wired/default inversion, output inversion, runtime behavior,
  native execution and new qualification credit are not claimed.
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
