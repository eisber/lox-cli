# Command Reference

Loxone Config-as-Code CLI. All commands operate on local `.Loxone` XML config files — no live Miniserver connection required for editing, simulation, or validation.

Global flags: `--json` (shorthand for `--output json`), `-o`/`--output json|csv|table`, `-q`/`--quiet`, `--no-color` (also respects `NO_COLOR` env var), `--no-header`, `-v`/`--verbose` (request logging, `-vv` for request+response bodies), `--dry-run`, `--non-interactive` (fail instead of prompting; implied by `--json`), `--trace-id <ID>` (correlation ID for agent tracing), `--ctx <name>` (use a specific context)

`--dry-run` never writes local files or live state, uploads configurations, reloads,
or reboots, even with `--force`, `--write`, or `--save-as`. Existing safe planners
are available for `config energy` edits and `stats download`; audited read-only
commands retain their normal output and may make read-only network requests.
Other commands fail before dispatch with “dry-run not supported” (exit 1;
JSON: `{"ok":false,"error":"dry_run_not_supported","message":"..."}`), not a
successful plan. Their inputs are **not** fully validated. This includes all
configuration deployment commands, other local editing commands, and `live get`
(name resolution can write the structure cache). Dry runs also skip telemetry,
including its local first-run/session files. Remove `--dry-run` only to execute
an unsupported operation.

---

## Setup

```bash
lox setup set --host https://192.168.1.100 --user admin --pass secret
lox setup set --serial YOUR_SERIAL        # enables correct TLS hostname (DynDNS)
lox setup set --verify-ssl                # enable cert verification
lox setup set --no-verify-ssl             # disable (default, self-signed Miniserver certs)
lox setup show                            # show config (password redacted)
lox setup show -o json                    # JSON output
```

All config fields also support env vars: `LOX_HOST`, `LOX_USER`, `LOX_PASS`, `LOX_SERIAL`.

---

## Config Inspection

```bash
lox config describe file.Loxone                   # human-readable config summary by room
lox config describe file.Loxone --room "Küche"    # describe a single room only
lox config describe file.Loxone -o json           # JSON output
lox config wires file.Loxone -o json              # resolved wire graph for block connectors
lox config wires --file file.Loxone -o json       # same, with explicit file flag
lox config wires file.Loxone --room "Küche" -o json # wires targeting blocks in one room
lox config devices file.Loxone -o json            # physical devices with bus identity + connectors
lox config devices file.Loxone --room "Küche" -o json # devices detected in one room
lox config devices file.Loxone --summary -o json  # print device count summary to stderr

lox config rooms file.Loxone                      # list rooms with item counts
lox config controls file.Loxone                   # list controls with type/room/category
lox config controls file.Loxone -t WeatherData    # filter by type
lox config controls file.Loxone -r "Zentral"      # filter by room
lox config controls file.Loxone --limit 10        # limit number of results
lox config users file.Loxone                      # list user accounts (with permission count; -o json lists permitted objects)
lox config devices file.Loxone                    # list hardware devices (Tree/Air/Network)
lox config devices file.Loxone --ports            # show I/O ports with used/free status

lox config get-params file.Loxone "Kitchen Light" # show all params (alias: lox config get)
lox config stats file.Loxone                      # comprehensive config statistics
lox config autopilot-list file.Loxone             # list automation/autopilot rules
lox config mode-list file.Loxone                  # list operating modes
```

---

## Config Editing

Every editing command that changes the file also bumps the Document save stamp (`Date` / `DateS` on `<C Type="Document">`) to the current time, or one second past its previous value if that is later. This keeps rapid successive edits distinct even within one second or after a clock adjustment. Loxone Config compares this stamp, not the content, to decide whether the opened file is the program already on the Miniserver — without the bump it would treat the edited file as identical and load the Miniserver's old program instead. Commands that end up changing nothing leave the stamp as is.

### Add blocks

```bash
lox config add --type light --title "Kitchen Light" --room "Kitchen" file.Loxone
lox config add --type switch --title "Main Switch" file.Loxone
lox config add --type timer --title "Night Timer" --page "Timers" file.Loxone
lox config add --type mqtt-sub --title "Temp Feed" --parent "MQTT Gateway" --topic "home/temp" file.Loxone
```

Available `--type` values: `light`, `switch`, `presence`, `alarm-clock`, `memory`, `timer`, `mqtt-sub`, `mqtt-pub`, `calendar`, `autopilot`.

Options: `--room <ROOM>`, `--category <CATEGORY>`, `--page <PAGE>`, `--parent <PARENT>`, `--topic <TOPIC>`, `--save-as <FILE>`.

### Set parameters

```bash
lox config set-param file.Loxone "Kitchen Light" FadingTime 2.0    # alias: lox config set
lox config set-param file.Loxone "Night Timer" Duration 300
```

Arguments: `<FILE> <SELECTOR> <PARAM> <VALUE>`. Selector is a block title or `"uuid:..."`.

### EnergyManager2 verified storage pilot

```bash
lox config energy icons --output json
lox config energy inspect file.Loxone "Energy" --output json
lox config energy set file.Loxone "Energy" O -1.25 --save-as edited.Loxone
lox config energy set file.Loxone "Energy" MinSoc 100 --dry-run
lox config energy load set file.Loxone "Energy" <LOAD_ID> --rated-power 2.5 --switch-on-power 0
lox config energy load set file.Loxone "Energy" <LOAD_ID> --preparation-seconds 30 --min-runtime-minutes -1 --daily-minutes 1440 --deadline dusk
lox config energy load move file.Loxone "Energy" <LOAD_ID> 2 --save-as edited.Loxone
lox config energy load set file.Loxone "Energy" <LOAD_ID> --icon electric-fan.svg
lox config energy load set file.Loxone "Energy" <LOAD_ID> --no-icon
lox config energy load set file.Loxone "Energy" <LOAD_ID> --no-name
```

`icons` lists built-ins without a project file or block selector. Other targets
are local files and exactly one **EnergyManager2**, selected by title,
`uuid:…`, or `"Name [Room]"`; `--room/-r` also disambiguates. All commands inherit
`--output json|table|csv`, `--quiet`, and `--no-header`. Edits support `--save-as`;
otherwise they overwrite the input. `--dry-run` runs the same validation and
reports the proposed state without writing the source or destination. Any
validation failure leaves both files and the in-memory editor unchanged.

`inspect` separates exact `stored` spellings/omissions from `effective` values and
known `default` values. Missing/ambiguous bindings have diagnostics, not guessed
values. JSON includes all twelve slots, output associations, GUI-observed deadline
labels, unknown attributes/child names, and limitations. It is an inspection
format, **not an export/reapply patch**: reapplying defaults can change omitted
storage. Table output summarizes scalars and loads; CSV exposes scalar values and
stored load attributes.

Per-load `inheritance.mode_refresh` and `inheritance.name_refresh` report native
customization gates, not derived values. `Anc=true` blocks both refreshes;
`Ncu=true` independently blocks name refresh. Open gates report
`source_unresolved`, not a guessed connected name/type or proof of a connection.
Invalid markers remain explicitly unknown unless another known marker blocks
the gate. Table/CSV expose the same statuses. No intentional restore-inherited
caller has been established, so inspection does not clear markers or refresh
stored values.

The source contract additionally records connection insertion ordering, one-hop
`Ref` binding and an OutputRef cached-mode fallback. These are not a per-document
resolver: raw XML order or an arbitrary connector's `Analog` flag cannot establish
the selected inherited values. See `contracts/energy-manager2.json`.

Scalar `set` accepts only `O` (signed kW, default 0), `MinSoc` (0..100%, default 0),
and `MaxSpwr` (kW >=0, default 0). `O` is a **parameter**, not an output.
Values are finite invariant decimals; decimal commas, scientific/nonfinite input,
and GUI prefix filtering are intentionally unsupported CLI policies. Explicit
nonzero decimals that underflow to floating-point zero are rejected before
default omission or power substitution; this applies to scalar and load power
edits, for either sign. Explicit
scalar zero removes `Def`, retaining its default semantics; untouched omissions
remain omitted. The runtime meaning of `MaxSpwr=0` is unverified.

`load set` configures an **existing persistent Id UUID**, not its `U` flag, name,
priority, or output index. Options:

| Flag | Meaning |
|------|---------|
| `--name NAME` | Set `Na` and native name-customization marker `Ncu` |
| `--no-name` | Store an empty `Na` and set `Ncu=true`; mutually exclusive with `--name`, not inherited naming |
| `--mode digital\|analog` | Set `An` and native mode-customization marker `Anc` |
| `--preparation-seconds S` | `RT`, integer seconds 30..86400; default 60 |
| `--min-runtime-minutes M` | `MinRt`, integer minutes converted to signed 32-bit seconds; negatives are permitted |
| `--daily-minutes M` | `MDay`, integer minutes/day 0..1440 converted to seconds |
| `--deadline CHOICE` | `tpR`: `sunset`, `dusk`, `sunrise`, `dawn`, or hourly `01:00`..`24:00`; all 28 choices GUI-save observed |
| `--rated-power KW` | `MaxPwr`, finite kW >=0 |
| `--switch-on-power KW` | `SwPwr`, finite kW >=0; zero substitutes rated power |
| `--icon BUILTIN` | Exact case-sensitive canonical filename or native/RFC ID from `energy icons`; optional `builtin:` prefix |
| `--no-icon` | Remove only the matched Load's `ico`; mutually exclusive with `--icon` |

Name and mode edits set their customization markers even for unchanged values.
Clearing a name leaves it explicitly customized; selecting Digital retains
`Anc=true`. Neither restores inheritance. Icon clearing leaves both markers
untouched. Reset-to-inherited naming/mode is unsupported; these setter/storage
contracts are source-backed, not new GUI save/reopen observations.

Minutes use checked multiplication by 60, not fractional GUI rounding or unknown
overflow semantics. Zero runtime/daily/deadline/rated fields use omission for their
known zero defaults. Rated power is applied before an explicit switch-on value;
changing rated power copies it into switch-on power only when the previous
switch-on value was zero. **Switch-on power greater than rated power is allowed.**
Rated zero does not clear `U=true`; unused feedback is `!U || MaxPwr<=0`.

Load edits/reordering require exactly twelve direct slots, unique UUID identities,
and explicit unique output indices 0..11. Unsupported sparse/duplicate/malformed
shapes fail rather than being repaired. `load move` sets priority **1..12 by XML
record order**, moves the complete record, and never renumbers `OIdx` (association
with `L(OIdx+1)`), changes Id, or rewrites connector/wiring/icon references. Unknown
attributes/children, unused slots, BOM/CRLF and native `12hTF` attributes are
preserved by ConfigEditor. No add/delete or editing of U/Id/OIdx is provided.
Load identities accept strict hexadecimal native `8-4-4-16` and RFC `8-4-4-4-12`
spellings. Native/RFC aliases and letter-case variations resolve the same identity
for selection/duplicate detection, while the original stored spelling is retained.
Misplaced dashes are rejected.

#### Built-in icons and manual catalog refresh

`energy icons` lists **623 loaded filled records** from the packaged
`docs\schemas\loxone-icons.json`: canonical filenames, native IDs and independent
archive provenance. JSON returns `source`, `catalog.version`,
`catalog.archive_sha256`, `count`, `icons` and limitations. CSV includes provenance
on each row; table output includes it after the count. `--no-header` suppresses
table/CSV headers, and `--quiet` suppresses notices, not list data. There are no
invented translated display names, fuzzy matches or GUID-prefix/range guesses.

Selection accepts a strict native `8-4-4-16` or RFC `8-4-4-4-12` alias,
case-insensitively by 128-bit identity, and stores the catalog's canonical native
ID. For example `electric-fan.svg` maps to
`00000094-00ff-0000-0000000000000000`. Disabled `awning-close.svg`
(`0000001d-00ff-0000-0000000000000000`) and absent
`000000a0-00ff-0000-0000000000000000` are not selectable. Empty, malformed,
unknown and project/custom selections fail atomically, including compound patches.
An all-zero ID is rejected: use explicit `--no-icon`. Clearing removes only
the matched Load's `ico`; definitions, other references, Id/OIdx, children and
wires remain intact. There is no orphan cleanup.

Inspection retains the raw `icon` and adds `icon_resolution` (`known_builtin`,
`none`, `unresolved`, or `malformed_id`) plus `builtin_icon_catalog` provenance.
An unresolved project reference is **not** claimed valid or decoded; unrelated
edits preserve its exact raw spelling. No project/custom selection, arbitrary GUID
assignment, creation, import, replacement, deletion or SVG/image authoring is
supported. Exact catalog membership does not establish Qt SVG renderability.
The explicit-clear GUI save/reopen oracle remains missing; CLI/unit proof is
not GUI parity.

The packaged catalog is archive version **20261009**, SHA256
`11656e55212ffbf0153f9075ee21da28da04f8c23d5d5e6efe945f4782391df4`.
This is **independent of Config executable version**. Native Config chooses the
user Documents archive only when its integer `version` is strictly greater than
the installed ProgramData archive's; ties use the installed archive. A Config
17.2.10.21 executable therefore does not imply these exact catalog bytes.

Refresh manually from an authorized, locally supplied `IconLibrary.zip`:

```powershell
python scripts\inventory-loxone-icons.py "PATH\IconLibrary.zip" --output docs\schemas\loxone-icons.json
python -m unittest discover -s scripts -p test_inventory_loxone_icons.py
cargo test --bin lox --test config_integration energy_
```

Choose/freeze the applicable archive explicitly (compare both archives' `version`
members when needed), review the resulting metadata/provenance diff, then rebuild
the CLI. The standalone deterministic stdlib generator checks fallback/localized
identity agreement, unique IDs/filenames, and nonempty loaded asset members.
It emits **only factual filenames/IDs and archive version/hash**, never absolute
source paths, localized tags, artwork, image payloads or archives. Keep all raw
vendor resources ignored. Catalog-only refresh needs neither a decompiler rerun
nor the bulk reverse-engineering pipeline; there are no scheduled refresh jobs.

This is a version-bound storage subset recovered for Config **17.2.10.21**, not
complete official GUI or Miniserver runtime parity. Deadline choices and name/mode
customization-marker transitions have version-bound GUI observations. Custom icons,
structural CRUD, runtime scheduling/storage behavior, and complete GUI validation
remain outside this CLI subset. Native malformed-input observations are not proof
of valid or supported CLI edits. Synthetic Rust/CLI tests are not GUI oracle
evidence; the original GUI observations remain separate.

To reproduce the offline proof, run
`python tests/ux/energy-manager2/run-cli-cases.py` with Cargo available (optional
`--lox PATH`). It executes the named core Rust/CLI integration tests and one
simulator-parser regression for O parameter direction/incoming wire,
then actual CLI assertions, generating `CLI-case-definitions.json` and a new
ignored `.reverse-engineering/ux/energy-manager2/cli/<run>/` directory containing
stdout, commands, digests and a CLI-only runner record. It never executes Config
GUI or a server and does not update the original contract or GUI records.

### Lighting stored inspection and mood-count repair

```bash
lox config lighting inspect file.Loxone "Lighting" --output json
lox config lighting recount-moods file.Loxone "Lighting" --save-as edited.Loxone
lox config lighting recount-moods file.Loxone "Lighting" --room "Study" --output json
lox config lighting set-mood-level file.Loxone "Lighting" --mood "Evening" --output-index 2 --level 37 --save-as edited.Loxone
lox config lighting set-mood-cid file.Loxone "Lighting" --mood "Evening" --cid 13 --save-as rebound.Loxone
lox config lighting rename-mood file.Loxone "Lighting" --mood "Evening" --name "Late Evening" --save-as renamed.Loxone
lox config lighting set-mood-mix file.Loxone "Lighting" --mood "Unbound scene" --no-mix --save-as unmixed.Loxone
```

Targets are local files and exactly one **LightController2**, selected by title,
`uuid:...` or a room-qualified name. `--room/-r` disambiguates names; do not
combine it with a bracket room qualifier. These commands support JSON/CSV/table,
`--quiet`, and `--no-header`; none contacts a Miniserver.

`inspect` preserves stored spellings and omissions, lists mood `Qn` values and
output references without inferred channel roles, and reports
`inconsistent_mood_count` when `LightscenesC/@Num` differs from the direct
`LightsceneC` record count.

Each stored output also has an `editor` object in JSON: `kind` is `numeric`,
`color`, `other` or `unresolved`, with the stored `type_tag`, needed
`capability` (`stored`/`effective`/`default`) and any `issue`. Table output adds
an `EDITOR` column; CSV appends editor, type-tag, capability and issue columns
after its existing five fields. This is the recovered **selected-mood editor**
classification, not a physical channel role, unit or graph-refresh prediction.
Unneeded capability is null; missing needed `CapAI` reports raw omission and
effective default 0. Unsupported explicit capability is unresolved, never
defaulted to 0.

Choose `config lighting set-mood-level` for a supported numeric kind; the edit
still validates its target and storage. `color` identifies the native color
control branch, **not** a mode-aware CLI color editor: existing
`config set-mood-color` remains a raw packed-value write. It does not validate
the selected mode or reproduce native color, dynamic-white or sequence handling.
`other` is outside the supported numeric/color branches. Opaque/out-of-range
Qn, unsupported widths/indices, nested lookalikes and ambiguous containers remain
visible as `unresolved` with reasons. Inspection changes no bytes, values,
flags, widths, IDs or HCL fields.
Noncanonical keys such as `Q0`/`Q01` remain in raw mood attributes, not classified
output records.

Inspection also reports version-bound native reconciliation risks:
`native_empty_mood_collection_reinitialized` for an explicitly empty `Num=0`
collection, and `native_zero_mood_sid_reconciled` for a supported loaded scene
with explicit `SID=0`. Config 17.2.10.21 can initialize default scenes or replace
zero scene IDs after nested serialization; this is not a prediction that the
current save already contains those changes. The inspector does not generate
IDs, normalize output values or simulate saving. Diagnostics remain visible in
JSON or on stderr for table/CSV, including with `--quiet`.

`recount-moods` repairs only that `Num`, including missing, invalid, negative or
stale counts. It requires exactly one direct collection and ignores nested
lookalike records. It does not normalize either `Outputs` field, fill missing
`Qn`, change identities, reorder records or repair references. An equivalent
signed-32-bit spelling such as `0002` or `+2` is preserved, and an in-place no-op
does not rewrite the file. Otherwise the command overwrites the input unless
`--save-as` is supplied; the existing writer preserves BOM/line endings and may
update the document save stamp after a content change. JSON returns `before`,
`after`, `scenes` and `changed`. Invalid target/container shapes fail before
either file is written.

`set-mood-level` is a numeric edit, **not RGB packing**. It preserves the
selected Qn's high type nibble and replaces its low payload with an integer
in `0..100`; values outside that range are rejected, never clamped.
It requires one direct mood matching exact stored SID spelling or Name,
an existing signed-32-bit Qn within the stored reader width, and a supported
numeric editor kind. Missing/zero scene `Outputs` means 12 on that reader;
the recovered editor path supports indices 1..18, not a universal maximum.
Missing Qn, ambiguous matches and unsupported widths fail before writing.

The Config 17.2.10.21 per-mood renderer uses tags 1/2/3/7 directly as numeric.
Tag 6 is capability-dependent: for Qn it reads `CapAI<n-1>`. With bit `0x40`
clear and `(CapAI & 7) != 7`, it selects a numeric control; otherwise it selects
color controls. Missing capability has the recovered reader default 0,
reported separately from its raw omission in JSON. Needed explicit capabilities
must be nonnegative 16-bit decimals; unsupported values are not truncated.
Color, switch and unresolved modes are rejected by this command.
These are stored editor-kind checks, not inferred actuator roles or a simulation
of graph-dependent pre-modal capability/type refresh.

One accepted native numeric edit of Q2 to 37 saved `1610612773`
(`0x60000000 | 37`) and reopened as 37; the new CLI command produces that same
selected encoding. The native workflow also changed Q1 to a numeric 100 payload
and normalized HCL fields. Those differences are retained in
`tests/ux/light-controller2/numeric-edit-save-reopen-20261111.json`, not ignored.
The bounded trigger trace identifies a programmatic numeric-refresh notification
path and HCL setup writes during dialog initialization; it does not uniquely
attribute every historical callback or reproduce those side effects. See
`tests/ux/light-controller2/incidental-normalization-triggers-source.json`.
The CLI intentionally changes only the requested Qn: it does not normalize other
outputs, HCL, IDs, widths, counts, sequences or unknown data. An equivalent
existing value retains its spelling and an in-place no-op retains original bytes.
Otherwise the existing writer's formatting and Document stamps still apply.
JSON includes `before`, `after`, `level`, `type_tag`, needed `capability` and
`changed`. Existing `set-mood-color` remains a packed/raw storage operation;
do not use `hsv(...)` as numeric dimmer brightness.

Existing `config add-mood` also sets `Num` from the resulting direct scene count,
instead of incrementing a possibly stale stored count. Its default output
initialization now follows the recovered Config 17.2.10.21 append path:

```bash
lox config add-mood file.Loxone "Lighting" --name "Evening" --color "hsv(30,80,20)" --save-as edited.Loxone
```

New moods have `Outputs=18` and `Q1..Q18`; this is the observed construction
width, **not a universal channel maximum**. `--color` sets `Q1`. Other values
inherit only the first direct mood's encoded type nibble (`value & 0xf0000000`),
with brightness/value bits cleared. The first mood's stored `Outputs` bounds
those source values; missing/zero means 12 on the recovered reader path.
Missing or out-of-bound source values initialize to zero. An empty collection
has no inherited types. Neither collection `Outputs` nor any existing scene's
width, sparse values, sequence data, identities or unknown fields are rewritten.

This intentionally replaces the previous container-width/zero-fill defaults.
Exactly one direct collection is required. Source widths and needed type values
must be supported signed-32-bit decimals; malformed, opaque or out-of-range
values fail before either file is written rather than being guessed. Opaque
`Q1` is untouched because the new scene's color is explicit; values outside the
source bound or new construction width are preserved without interpretation.
Without `--sid` or `--cid`, IDs now follow the recovered native automatic path:
the smallest unused `SID` in 1..99, filling gaps (including SID 1). For SID 1..8,
try only `CID=SID+11`; if that CID is already occupied, use 0. SID 9..99 also uses
CID 0. Unlike native append's zero-SID exhaustion sentinel, the CLI fails
atomically when all 99 automatic SIDs are occupied.

Automatic allocation considers the actual direct stored records, not native
import's `Num`-limited vector. Absent IDs reserve no stored candidate; present
needed IDs must be signed-32-bit decimals or the operation fails without writing.
CID values are not interpreted when the new SID is above 8.

Supplying **either** `--sid` or `--cid` retains the legacy explicit-override path:
the omitted partner uses max-custom-plus-one (base SID 2 / CID 9), with
SID 1/776/777/778 excluded from those maxima. Duplicate explicit SID remains
an error; duplicate explicit CID remains accepted, not swapped. An overflowing
partner successor is an atomic `mood_identity_exhausted` error. Existing scene
IDs are never reassigned by `add-mood`.

`lighting set-mood-cid` edits an **existing** mood binding, separately from
append overrides. It follows the recovered handler's collision swap and SID
role transaction: an occupied nonzero CID swaps the other record to the selected
record's former CID; CID 0 does not swap. CID 22 maps to SID 777 and CID 24 to
SID 776. Other affected special SIDs at least 776 use a free SID in 1..99.
The first existing CID0 record can also be role-normalized, even without a swap.
Affected identity records and ordered `LSConfig/ScMv` remaps are reported.
The native early SID allocation and subsequent reference remaps retain their
exact order, including cascades; they are not a simultaneous UUID-based rename.

This command requires one direct collection, matching `Num`, one exact
unambiguous Name/SID target, distinct positive signed-32-bit SIDs and unique
nonzero CIDs. Absent CID has the recovered effective value 0; its omission is
preserved when unchanged. Present malformed/negative/out-of-range identities,
needed unsupported selection lists, exhausted allocation and final role-ID
collisions are atomic errors. No native zero-SID exhaustion sentinel is written.
Needed selection lists support comma-separated signed-32-bit decimals;
unvisited `ScMv` tail tokens, `OpM`, `ScAc` and unknown data stay intact.
Missing/empty selection vectors cause no reference writes.

Only affected IDs and required `ScMv` tokens change, plus the existing writer's
normal formatting/save stamps. Qn, flags, widths/counts, HCL, record order,
sequence data and unrelated references are not normalized. JSON includes
`identities`, ordered `selection_remaps`, optional `selection_refs` and `changed`;
CSV includes mood and consulted-selection rows. Native validation is separately
queued and currently blocked on desktop admission, not claimed by local tests.
See `tests/ux/light-controller2/mood-cid-edit-source.json`.

`lighting rename-mood` changes only an existing direct mood's `Name`. Supported
new names contain ASCII letters/digits and ordinary internal spaces, with a
letter/digit at each end. Empty, edge-space, punctuation/control and Unicode
inputs are rejected rather than approximating native normalization. Duplicate
new names are allowed, matching the native row-bound setter; if that makes a
later Name selector ambiguous, use its exact stored SID spelling. Missing `Name`
can be populated without generating IDs. `SName` is a different field and stays
unchanged. An in-place identical-name edit preserves original bytes.

`lighting set-mood-mix` requests exactly one of `--mix` / `--no-mix` and changes
only an eligible existing `MixMod` flag. This is not a raw boolean/cohort editor:
it requires selected SID 1..31, explicit nonnegative CID **outside 12..19**,
explicit stored `MixMod="true"`/`"false"` and a unique selected numeric SID among
direct records. All direct sibling SIDs must be supported signed-32-bit
decimals. CID 12..19 can force/disable the native checkbox through graph/role
state, so those rows are rejected even on a requested no-op. Special/non-checkbox
rows and missing/opaque identity or flag values are also rejected; no defaults
or role-state guesses are applied.

Both metadata commands require exactly one direct collection and one
unambiguous direct mood selected by exact stored SID or case-sensitive Name.
They preserve all unselected fields/records, IDs, Qn, flags, widths/counts,
selection/sequence data, HCL, capabilities and unknown storage. Errors precede
original/save-as writes; no-op originals are not rewritten. Existing writer
formatting/stamps still apply to actual saves. JSON reports exact stored
identities and `before`/`after`/`changed`; mix also reports `enabled` and
`scope="selected_record_role_independent"`. These are selected-field
transactions, not full dialog permissions, Unicode normalization, graph refresh
or accepted-dialog/runtime parity. `set-mood-color` remains raw packed storage.
See `mood-name-source.json` and `mood-mix-source.json` under
`tests/ux/light-controller2`.

One bounded Config 17.2.10.21 observation completed CLI mood creation, Q2 edit,
native Save As to a new file, normal exit and fresh reopen. The new SID 1 /
CID 12, 18-output fields, Q type nibbles and all unrelated bytes survived
exactly; the reopened editor recognized the new mood. This is storage evidence,
not color semantics: this fixture's dimmer controls displayed both RGB-packed
values as 100. Neither dirty native-edit serialization nor family/runtime parity
is established. See
`tests/ux/light-controller2/create-edit-saveas-reopen-20261111.json`.

Count repair can expose previously under-counted records on native import.
Neither command reproduces default-scene injection, sequence normalization,
whole-document save/cancel or runtime behavior. See
`contracts/light-controller2.json` for the recovered paths and exact dependencies.

Recovered native pre-modal preparation can conditionally change the original
model's channel names/capabilities, `COName` and scene `Qn`, including an existing
SID 777 payload, before copying values into the dialog. Cancel skips accepted-only
copy-back but does not explicitly roll back those preparation writes. This is
not immediate disk persistence or a document-specific CLI risk prediction;
native GUI validation remains unrun.

### Wire connectors

```bash
lox config wire-connector file.Loxone "Light.I1" <source-uuid>                    # alias: lox config wire
lox config wire-connector file.Loxone "Both Sensors.I2" <sensor-q-uuid> --save-as out.Loxone
```

Arguments: `<FILE> <TARGET> <SOURCE_UUID>`. Target format: `"BlockTitle.ConnectorKey"`.

### Device binding

```bash
lox config device-bind file.Loxone "Kitchen Light" AQ1 --device "RGBW Dimmer"     # alias: lox config bind
```

Arguments: `<FILE> <CONTROL> <OUTPUT_CONN>`. Required: `--device <DEVICE>`.

### Room templates

```bash
lox config template file.Loxone bedroom --room "DG Schlafzimmer"
lox config template file.Loxone bathroom --room "OG Bad" --save-as out.Loxone
```

Templates: `standard`, `bathroom`, `hallway`, `bedroom`, `kitchen`, `outdoor`.

### Timer schedules

```bash
lox config timer-schedule file.Loxone "Night Timer" "20:00-24:00"
lox config timer-schedule file.Loxone "Night Timer" "06:00-08:00" --value 0.5
```

Arguments: `<FILE> <SELECTOR> <RANGE>`. Range format: `HH:MM-HH:MM`. `--value` sets the output during active period (default: 1).

### Virtual inputs

```bash
lox config add-virtual-in file.Loxone "Sensor Feed"                               # alias: lox config add-vi
lox config add-virtual-in file.Loxone "Analog Sensor" --analog
lox config add-virtual-in file.Loxone "Custom Input" --parent "VirtualInCaption"
lox config add-virtual-in file.Loxone "UniFi Webhook" --grant webhook_api         # also grant an API user access
lox config add file.Loxone --type virtual-input --title "UniFi Webhook" --grant webhook_api
```

A user calling `/dev/sps/io/<input>/...` needs permission for that input, otherwise the
Miniserver answers HTTP 403 (a wrong password gives 401). Use `--grant` or `lox config user-grant`
to add a direct permission entry; inherited denies still apply.

### Room, control, and MQTT management

```bash
lox config room <SUBCOMMAND> ...           # manage rooms (add/rename/move)
lox config control <SUBCOMMAND> ...        # manage controls
lox config mqtt <SUBCOMMAND> ...           # manage MQTT plugin configuration
```

### User management

```bash
lox config user-add file.Loxone ...        # add a user account
lox config user-remove file.Loxone ...     # remove a user account

# Object permissions (the user's <PH>/<PRM> list in the .Loxone file)
lox config user-perms file.Loxone webhook_api                          # direct permission entries
lox config user-grant file.Loxone webhook_api "UniFi Doorbell"         # grant control access (ACT=3)
lox config user-grant file.Loxone webhook_api uuid:16e2a4e7-...        # exact object by UUID
lox config user-grant file.Loxone Webhooks "UniFi Doorbell" --group  # grant a user group instead
lox config user-revoke file.Loxone webhook_api "UniFi Doorbell"        # remove the permission
```

Syntax: `lox config user-grant <FILE> <USER> <OBJECT> [--group] [--act <N>] [--save-as <PATH>]`,
`lox config user-revoke <FILE> <USER> <OBJECT> [--group] [--save-as <PATH>]`,
`lox config user-perms <FILE> <USER> [--group]`.

- `<USER>` is the exact user (or `--group` user group) title, case-insensitive, or `uuid:<uuid>`.
  No fuzzy matching, so the wrong account is never edited.
- `<OBJECT>` uses the element selector (fuzzy title, `"Title [Room]"`, `uuid:<uuid>`, `Type:<type>`)
  and must match exactly one element. `user-revoke` also accepts `uuid:<uuid>` of an object that no
  longer exists in the file.
- Granting an existing permission, or revoking a missing one, is a no-op and leaves the file untouched.
  `--act` changes the access value of an existing entry (default `3` = control access; Loxone
  Config also writes `1` for some Door/Notification entries).
- These commands edit/list **direct `<PH>` entries**, not effective access. They do not evaluate
  group membership, room/category grants, or explicit denies. Existing raw `ACT` values (including
  restrictions) and `TIM` schedules are preserved unless `--act` is supplied.
- Revoke **removes** the direct entry; it does not create a deny or remove inherited access.
  Removing an explicit deny can therefore expose access allowed by a group. Use Loxone Config's
  "Show All Permissions" to verify effective access.
- Permission edits change the `<PH>` block and the Document save stamp described above:
  grant appends `<PRM REF="<uuid>" ACT="3"/>` and updates `NUM`
  (creating `<PH>` if needed); revoke removes the entry and drops an emptied `<PH>`.
  Password hashes (`<HP>`) are never touched. The editor preserves native Loxone formatting,
  BOM, and line endings; a pre-existing empty `<PH>` is not recreated after revoke.
- `user-perms` supports `-o json` and `-o csv`; `--no-header` suppresses table/CSV headers.
- `-o json` on `user-grant`/`user-revoke` returns `{principal, object, change, changed, act, previous_act}`
  where `change` is `added`, `updated`, `unchanged`, `removed` or `absent`.

```
$ lox config user-grant sps.Loxone webhook_api "UniFi Doorbell"
✓ Saved to sps.Loxone
✓ Added permission entry for user 'webhook_api' on VirtualIn 'UniFi Doorbell' (uuid:vi-1-ffff504f94a010d0) [ACT=3]
$ lox config user-grant sps.Loxone webhook_api "UniFi Doorbell"
✓ User 'webhook_api' already has a direct permission entry on VirtualIn 'UniFi Doorbell' (uuid:vi-1-ffff504f94a010d0) [ACT=3] — no change
$ lox config user-perms sps.Loxone webhook_api
  TITLE                            TYPE                   ACT  UUID
  ────────────────────────────────────────────────────────────────────────────────────────────────
  UniFi Gate                       VirtualIn              3    vi-2-ffff504f94a010d0
  UniFi Doorbell                   VirtualIn              3    vi-1-ffff504f94a010d0

2 direct permission(s) for user 'webhook_api'
```

### Autopilot and calendar

```bash
lox config autopilot-add file.Loxone ...   # add an autopilot rule
lox config calendar-add file.Loxone ...    # add a calendar/schedule element
```

### Auto-layout

```bash
lox config layout file.Loxone              # ELK-based auto-layout for all pages
lox config layout file.Loxone --page "Lighting"  # layout a specific page
```

### Low-level XML editing

```bash
lox config xml <SUBCOMMAND> ...            # power-user XML operations
```

---

## Config Validation

```bash
lox config validate file.Loxone            # check UUIDs, wiring, orphans
lox config check file.Loxone               # check blocks for unset params, dead-end wiring
lox config check file.Loxone --selector "Kitchen Light"  # check a specific block only
lox config scan file.Loxone                # scan for PII, secrets, and credentials
lox config scan file.Loxone --strict       # exit with error code if PII found (for CI)
lox config report file.Loxone              # anonymised diagnostic report (scan + stats + check)
```

`validate` also flags wiring that Loxone Config accepts but that silently misbehaves:

- a Lighting Controller with motion/presence wired whose motion table has neither an "All other times" row nor a row for an operating mode most other motion-wired controllers list (motion does nothing while that mode is active), or no rows at all;
- a fixed pulse (EdgeDetection `PulseTime`, Monoflop `Time`) into a mood input `Sel1..SelN` that reaches the controller's `SceneMixTime` (Mmd, default 1 s) — a long-click can mix the assigned mood in instead of selecting it. Disabled mixing (`SceneMixTime=0`) and dynamically wired durations are not diagnosed from static defaults;
- a wire taken from the `Q` of a reference to a device or virtual input (only `AQ` is used there; `Q` is valid on Memory flag references).

Both `validate` and `check` warn when a PicoC `Code1`/`Code4`/`Code8`/`Code16`
block's `Code` attribute has no line breaks and begins with `//` (ignoring leading
whitespace). The entire source is then a comment, possibly from collapsed line
breaks. This is a review warning, not a C syntax error; ordinary one-line programs
are not rejected. XML edits preserve multiline `Code` attributes, including CRLF
and `&#xD;&#xA;` input.

PicoC scripts are **not executed** by the offline simulator. These four block
types are unsupported: neither simulation engine copies inputs to outputs.
Outputs remain at their initial or explicitly injected values, not script results.
`lox sim check FILE --strict` rejects them. Non-strict inspection/runs remain
available with unreliable-result diagnostics. `sim check` JSON reports their
count in `unsupported` and the compatibility field `passthrough_unreliable`.
Interpreter support remains blocked on an authentic public Code-block XML fixture
with connector layout and an authoritative Loxone PicoC runtime API contract.

---

## Config Management

```bash
lox config download                        # download latest config ZIP via FTP
lox config download --extract              # download + decompress to .Loxone XML
lox config download --save-as config.zip   # custom output filename
lox config ls                              # list all configs on the Miniserver
lox config extract config.zip              # decompress LoxCC → .Loxone XML
lox config extract config.zip --save-as out.Loxone
lox config extract config.LoxCC --save-as out.Loxone
lox config extract tdc_DEV.LxRes --save-as blocks.xml  # installed resource metadata
lox config surfaces file.Loxone -o json     # field names and nested storage, without values
lox config compress file.Loxone            # compress .Loxone XML → LoxCC (with CRC32)
lox config upload config.zip --force       # upload to Miniserver (dangerous)
lox config diff old.Loxone new.Loxone      # compare two configs (accepts .zip or .Loxone)
lox config patch ...                       # download, patch, recompress, and upload in one step
lox config push file.Loxone --force        # recompress + upload edited .Loxone XML
lox config push file.Loxone --reboot --force  # ...and reboot Miniserver after upload
lox config push-http ...                   # upload config ZIP via HTTP POST (fsput)
lox config snapshot file.Loxone            # save a snapshot for evaluation (before/after)
```

`config diff` matches controls by UUID, not title or document order, so controls
with duplicate titles remain distinct. Room assignments come from each control's
own `IoData Pr` attribute; nested references and unrelated elements do not change
the control's room.

`download` (including its ZIP) and `pull` incorporate pending App/Web UI controls
from `/sys/custom/*.LoxCC`. An unchanged, overlay-free ZIP is saved byte-for-byte.
Failed listings, corrupt fragments, UUID/type mismatches, and new controls with
no evidenced placement fail explicitly rather than silently dropping changes.
New `AutopilotRule` controls require exactly one `Page` with `SpNo="1"`.

Downloads and pulls save an adjacent `*.lox-base.json` acquisition baseline,
bound to the configured host and serial. Keep it with the edited file; offline
`extract` copies an existing baseline to the extracted XML. It contains the full
configuration, so protect it like the `.Loxone` file.

Deployment (`push`, `patch`, `upload`, `push-http`, and forced `restore`) reads
the current effective configuration immediately before preparing the upload.
With a baseline, a UUID-based three-way merge preserves independent local and
live changes; conflicting edits/deletions or ambiguous additions are refused.
Concurrent changes within the same control payload may require manual
reconciliation even when they affect different attributes. Pending overlays
without an original baseline are refused: download again and reapply edits,
or use Loxone Config's **Load from Miniserver** workflow. `--force` does not pick
an overwrite policy. Legacy files without a baseline retain their existing
deployment behavior only when no pending runtime overlays are observed.

The saved config and runtime overlays are read twice during acquisition and
checked again before upload. This detects observed changes, **not an atomic
transaction**: App edits after the last check or before a deferred reboot can
still race with deployment. Avoid App edits throughout upload/reload and use
the supported Loxone Config workflow when that coordination is not possible.
The existing archive sidecar capability checks still apply to merged changes.

`push` and `patch` preserve the native archive's sidecars, not just its XML.
They reject edits requiring a new app catalogue or compiled permissions: for
example, adding/removing controls, changing visible control metadata or native
state identities, and changing room/category rights or user permissions.
`--force` does not bypass this guard. Save and deploy those edits with Loxone
Config instead. XML extraction/compression alone is **not** a complete deployment.

Existing sidecars and unknown native versions are preserved for sidecar-neutral
edits (save stamps, layout, and parameter/wiring edits on existing unvisualized
logic/math blocks). This is conservative impact detection, not a sidecar compiler
or an audit/repair of previously stale native records.
Raw ZIP uploads must also retain matching native catalogue, permission and
display sidecars from the trusted backup; identical XML does not make missing or
replacement sidecars safe. Raw XML/LoxCC is not a complete deployment archive.

Touch Pure Display (`TPDC`, Tree subtype `32811`) modifications are not supported
for deployment. Their managed API loop, `TPDISP/<UUID>.json` payloads (including
the separate `/prog/TPDISP` files), and signed `Displayconf` checksums must be
generated together by Loxone Config. Untouched display artifacts remain opaque
and are preserved; their presence does not imply validation of screen rendering.

---

## GitOps (Config Version Tracking)

```bash
lox config init ~/loxone-config            # initialize a git repo for config tracking
lox config pull                            # download, decompress, diff & git-commit
lox config pull --quiet                    # cron-friendly (no output unless error)
lox config log                             # show config change history
lox config log -c 5                        # last 5 entries
lox config restore abc123 --force          # restore config from git history & upload
```

The `pull` workflow: FTP download → LoxCC decompress → semantic diff → git commit with meaningful message. Multi-Miniserver: each serial gets its own subdirectory in the repo.

---

## Simulation

```bash
lox sim check file.Loxone                  # parse config and report block/connector counts

lox sim run file.Loxone --sim '{"inputs":{"Sensor.Q":1},"expected_outputs":{"Light.AQ":{"==":1}}}'
lox sim run file.Loxone --sim-file test.json  # run sim spec(s) from a JSON file

lox sim step file.Loxone --sim '{"inputs":{"Sensor.Q":1}}'
lox sim step file.Loxone --sim-file test.json # step-by-step showing signal changes per tick

lox sim dump file.Loxone                   # dump full graph (blocks, connectors, wires)
lox sim dump file.Loxone --sim '...'       # apply sim spec before dumping
lox sim dump file.Loxone -o json           # JSON output
```

`--sim` accepts inline JSON (single object or array). `--sim-file` loads from a file.
Unknown scenario, step, or clock keys are rejected. An input that cannot be resolved fails its scenario.
`sim run` requires at least one scenario, and each scenario needs an output assertion
in `expected_outputs` or a step's `expected_outputs`. An empty comparator map or
trace-only scenario does not count as verification and fails. Use `sim step` or
`sim dump` to explore a circuit without assertions.

---

## Statistics

Reads the statistics the Miniserver logs to its SD card. These commands need a Miniserver connection (`lox setup`). All read-only.

```bash
lox stats ls                               # every statistic file: name, group, month, uuid, label
lox stats ls --filter kitchen              # case-insensitive substring on the label
lox stats ls --month 2026-10               # one month only
lox stats ls --latest                      # one row per statistic and group, newest month

lox stats get "Heat Pump" --group 2        # values of one statistic, all available months
lox stats get Kitchen --from 2026-09 --to 2026-10 -o csv   # month range (YYYY-MM, inclusive)
lox stats get <uuid> -o json               # {name, group, outputs, rows:[{t, values}]}

lox stats download --dir ./stats           # mirror the monthly XML files, incrementally
lox stats download --dir ./stats --from 2026-01 --filter energy
lox stats download --dir ./stats --force   # refetch files that already exist
lox stats download --dir ./stats --dry-run # list what would be fetched
```

`ls` also accepts `list` as an alias. `get` resolves the name case-insensitively (exact match first, then substring) or takes the uuid, including historical months saved under an earlier name. A name shared by several statistics is reported as ambiguous with each uuid; a grouped statistic (e.g. a meter) needs `--group`, and the error lists the groups and their outputs. Timestamps are printed as the Miniserver writes them, without timezone conversion; the row a month file shares with the next month is printed once. Duplicate output labels cannot be represented by the JSON values object; use CSV or table output to preserve all columns in that case. Malformed XML and non-finite values are errors rather than partial data or JSON nulls.

`download` always refetches the current month, which is still growing. An existing file of an earlier month is kept only when it was saved at least one day after that month ended; an older copy may be incomplete and is refetched. With `-o csv`, it reports `FILE,STATUS` rows (`downloaded`, `skipped`, or `would_download` for a dry run); `--no-header` suppresses the header.

---

## Block Library

```bash
lox blocks search "timed light"            # search by intent or keyword
lox blocks search "blind wind" -o json     # JSON output

lox blocks info StairwayLS                 # full details: inputs, outputs, parameters
lox blocks info And -o json

lox blocks list                            # list all block types
lox blocks list --category logic           # filter by category
```

`blocks info` combines the embedded block metadata with the connector map.
Its `inputs`, `outputs`, and `parameters` JSON arrays include connectors supplied
by either source, including inputs absent from the primary block metadata.

Categories: `logic`, `math`, `compare`, `timer`, `schedule`, `state`, `lighting`, `shading`, `hvac`, `security`, `energy`, `io`, `button`, `misc`.

---

## Multi-Context Management

Manage multiple Miniserver connections, similar to `kubectl config use-context`:

```bash
lox ctx add home --host https://192.168.1.100 --user admin --pass secret
lox ctx add office --host https://10.0.0.50 --user admin --pass secret --serial SERIAL
lox ctx use home                           # switch active context
lox ctx list                               # list contexts (* = active)
lox ctx current                            # show active context
lox ctx remove office                      # remove a context
lox ctx rename home house                  # rename a context
```

### One-off context override

```bash
lox --ctx office config download           # run against 'office' without switching
```

### Project-local config

```bash
lox ctx init                               # create .lox/ in current directory
lox ctx init --host ... --user ... --pass ...  # with connection details
```

Project-local `.lox/config.yaml` is auto-discovered by walking up from cwd (like `.git`). Secrets are excluded via `.lox/.gitignore`.

### Migration

```bash
lox ctx migrate                            # convert flat config to 'default' context
```

### Config resolution order

1. `LOX_CONFIG` env var (absolute priority)
2. Project-local `.lox/config.yaml` (walk up from cwd)
3. Global `~/.lox/config.yaml` (flat or multi-context)
4. `--ctx` flag overrides context selection within global config

---

## Cache

```bash
lox cache info                             # show cache age and path
lox cache check                            # check if cache is current (without full download)
lox cache refresh                          # force re-fetch
lox cache clear                            # delete local cache
```

---

## Token Auth

More secure than Basic Auth. Token is valid ~20 days.

```bash
lox token fetch                            # fetch & save token (RSA/AES key exchange)
lox token info                             # show token status
lox token check                            # verify token on Miniserver
lox token refresh                          # extend validity
lox token revoke                           # revoke on Miniserver
lox token clear                            # delete local token file
```

---

## Telemetry

```bash
lox telemetry status                       # show current telemetry setting
lox telemetry enable                       # enable anonymous usage analytics (default)
lox telemetry disable                      # opt out
```

Usage telemetry sends anonymous command names and aggregate statistics to
PostHog's EU endpoint. See [TELEMETRY.md](TELEMETRY.md) for collection details
and environment overrides.

---

## Issue Reporting

```bash
lox report --title "Simulator issue" --body "Sanitized reproduction steps" --json
lox report --title "Simulator issue" --body-file report.txt --json
lox report --title "Simulator issue" --body-file - --json  # read stdin
lox report --title "Simulator issue" --body "Details" --dry-run --json
```

Non-interactive; no Miniserver configuration or connection is required. Attempts
to create a **public** issue in `eisber/lox-cli` using an authenticated `gh` CLI.
If GitHub is unavailable, sends an `issue_report` event through the existing
PostHog telemetry setup, **including the explicitly supplied title and body**.
Telemetry fallback respects `lox telemetry disable`, `LOX_TELEMETRY` and
`DO_NOT_TRACK`; it never enables telemetry automatically.

Only supply sanitized issue details: reproduction steps, expected behavior, and
actual behavior. Do not include credentials, private configuration, personal
data, or raw logs containing those details. No files or logs are attached
automatically. Version, OS, architecture, and a random report ID are added.

JSON receipts include `ok`, `channel` (`github` or `telemetry`), `report_id`,
and `url` (null for telemetry). A telemetry receipt also includes `github_error`
explaining the fallback. Telemetry reporting waits up to two seconds for an
HTTP acknowledgement; failures return a nonzero exit code and the usual JSON
error envelope rather than claiming success. `--dry-run` previews the report
without submitting it; `--quiet` suppresses confirmations, not JSON receipts.

### Reporting from an agent

`lox --help`, `lox report --help`, and `lox schema` expose reporting guidance.
Both runtime errors and argument-parsing errors return a `report_hint` in JSON
mode and a hint on stderr in human mode. Argument errors retain exit code 2;
runtime errors retain exit code 1. `--help` and `--version` still exit normally.
Rust panics retain their original stderr diagnostics and reporting guidance;
panics on the main thread also emit a JSON error when JSON output was requested.

Correct parameter, input, authentication, and connection mistakes first. If
documented or schema-generated usage still fails, or the CLI panics, use
`lox report` with a minimal sanitized reproduction and expected/actual behavior.
No issue reports are submitted automatically on errors or panics. Check the
delivery receipt; do not repeatedly report the same problem or recursively
report a failed submission.

`lox config report FILE` is different: it generates configuration diagnostics.
Inspect and sanitize any diagnostic content before including it in an issue.

For receipt-based feedback lookup, dashboard access, and native exception
investigation, see the [maintainer access and triage guide](TELEMETRY.md#maintainer-guide-access-learn-and-fix).
Automatic sanitized `$exception` events are separate from explicit feedback;
they do not automatically create issue reports.

---

## Shell Completions

```bash
lox completions bash                       # generate bash completions
lox completions zsh                        # generate zsh completions
lox completions fish                       # generate fish completions
lox completions --install                  # install to standard location for your shell
```

---

## Command Schema (AI Agents)

```bash
lox schema                                 # list all commands with metadata
lox schema config                          # schema for a specific command group
lox schema -o json                         # JSON output for programmatic use
```

Returns command structure, arguments, subcommands, and valid parameters. Designed for AI agent discovery.
