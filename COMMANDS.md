# Command Reference

Loxone Config-as-Code CLI. All commands operate on local `.Loxone` XML config files — no live Miniserver connection required for editing, simulation, or validation.

Global flags: `--json` (shorthand for `--output json`), `-o`/`--output json|csv|table`, `-q`/`--quiet`, `--no-color` (also respects `NO_COLOR` env var), `--no-header`, `-v`/`--verbose` (request logging, `-vv` for request+response bodies), `--dry-run`, `--non-interactive` (fail instead of prompting; implied by `--json`), `--trace-id <ID>` (correlation ID for agent tracing), `--ctx <name>` (use a specific context)

`--dry-run` never writes local files or live state, uploads configurations, reloads,
or reboots, even with `--force`, `--write`, or `--save-as`. Existing safe planners
are available for `config energy` edits, supported `config lighting` mood edits
(including `set-mood-level`), the existing raw `config set-mood-color` edit,
existing `config add-mood` creation, `config splice-actor` reference replacement,
`config room rename` Title edits, `config control move` room-reference edits, and
`stats download`; audited read-only
commands retain their normal output and may make read-only network requests.
Other commands fail before dispatch with “dry-run not supported” (exit 1;
JSON: `{"ok":false,"error":"dry_run_not_supported","message":"..."}`), not a
successful plan. Their inputs are **not** fully validated. This includes all
configuration deployment commands, other local editing commands, and `live get`
(name resolution can write the structure cache). Dry runs also skip telemetry,
including its local first-run/session files. Remove `--dry-run` only to execute
an unsupported operation.

---

### Preview an existing actor splice

```text
lox config splice-actor file.Loxone "uuid:actor-or-outputref" --source "uuid:new-source" --dry-run --json
```

Only this already supported splice operation gains dry-run admission; `config wire`
and `config wire-connector` remain unsupported under global `--dry-run`.
The shared immutable plan reports the actual selected block, actor (when selected),
resolved target (historically called OutputRef), normalized stored source UUID,
ordered direct `In` reference attributes before/after, connector attributes
(including raw `Nc`), and any existing sparse-connector creation/`Nio` change.
Absent attributes and empty strings remain distinct; table/CSV retain the same
data as JSON using JSON-valued cells. `--quiet` suppresses the summary, not the
preview; `--no-header` suppresses table/CSV headers. JSON supports `--trace-id`.

The existing selector rules are unchanged. A non-OutputRef selector follows the
first stored source on its direct `I`/`AI` connectors and then the first `C`
owning that exact `AQ` UUID. Despite the historical name, the implementation
does not enforce an actor class or OutputRef owner type; this preview neither
adds nor narrows those checks. `--source` is a connector UUID, not a block/title
selector: one leading `uuid:` is stripped **before** trimming whitespace.
Input and unknown sources retain the existing warnings, not new rejections.

The first direct target `Co K="AI"` has all direct `In` children replaced, not
appended, by one reference. Raw old `Input`/`FLG` spellings, missing attributes,
extra attributes, and reference child positions are exposed; `discarded_children`
counts any nested payload removed with an old `In` (not a full XML diff).
Non-`In` siblings and later duplicate AI connectors remain untouched.
Cross-page flags use the existing page helpers: the new reference gets `FLG="2"`
when the source page is unknown or both known pages differ, otherwise no FLG.
`Nc` becomes `"1"`. Existing schema-driven AI creation remains supported,
including defaults and a generated connector UUID marked ephemeral across calls.

Preview returns before any DOM mutation, XML serialization, default insertion,
Document stamp, save, destination check, or filesystem/network/telemetry write,
even for in-place, existing, or missing-parent `--save-as` destinations.
Normal execution still replaces references, reports the original untrimmed
source spelling after prefix removal, and saves even when reference values
already match. Document stamping still depends on the existing normalized
content-change comparison. Its original overwrite and save-failure behavior is
unchanged. This is source-only CLI planning, not native UI, device-role,
physical-effect, whole-file, or SPS validation.
Source basis: `tests/ux/splice-actor-preview-source.json`.

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

### Load Manager configure-list order and consumer names

```bash
lox config load-manager inspect file.Loxone "Load Manager" -o json
lox config load-manager move file.Loxone "Load Manager" 4 up --save-as edited.Loxone -o json
lox config load-manager move file.Loxone "uuid:..." 4 down
lox config load-manager set-name file.Loxone "Load Manager" 4 "Heat pump" --save-as edited.Loxone -o json
lox config load-manager set-power file.Loxone "Load Manager" 4 2.125 --save-as edited.Loxone -o json
lox config load-manager set-max-off file.Loxone "Load Manager" 4 600 --save-as edited.Loxone -o json
lox config load-manager set-min-on file.Loxone "Load Manager" 4 600 --save-as edited.Loxone -o json
```

`move <FILE> <SELECTOR> <IO_INDEX> <up|down>` moves an existing **LoadShed**
load one position in Configure loads/XML order, matching the native arrow action.
`IO_INDEX` is the resolved zero-based `Load/@ioIdx` (0..11), **not** its displayed
1-based list position (`priority` in JSON/table output). Native XML omits `ioIdx`
for zero: an absent attribute resolves to **0**, while its absence remains visible
in JSON's raw `attributes`. The complete record moves with its connector association,
name, power, timing, unknown attributes and children. Connectors, UUIDs, other
siblings and block settings are untouched; no priority attribute is invented.
The usual save helper updates the Document save stamp on a successful change
and preserves BOM/line-ending convention. Without `--save-as`, the input is edited.

All these commands require exactly twelve direct `Load` records with **resolved**
unique `ioIdx` values 0..11. Exactly one omitted zero is supported; an omitted
attribute plus explicit zero, or two omissions, is a duplicate identity error.
Omission is preserved when the record moves: no `ioIdx="0"` is inserted, and
no name, power or timing defaults are filled. Explicit identities must contain
only decimal digits resolving to 0..11. Explicit empty, signed, whitespace or
nonnumeric values are conservatively rejected, even where the native parser
would tolerate them; stored values are never silently normalized.
Sparse/extended lists, wrong block types, ambiguous selectors and moves past
either boundary fail without writing. `--room`/`-r` disambiguates
the block; do not combine it with a bracket room qualifier. JSON includes every
stored load attribute; table/CSV shows priority, io-index and name.

This is a Config 17.2.10.21 source-recovered operation, not EnergyManager2's
separate load model. Moving does not add/delete loads or edit power/timing settings,
or claim native runtime/save-reopen qualification. Actual runtime shedding
priority/effects remain unproven; the recovered scope is configure-list/XML
order and configured-name storage. `--dry-run` remains
unsupported and fails closed before dispatch.
Source chain and a fresh native verification recipe:
`tests/ux/load-manager/move-source.json`.

`set-name <FILE> <SELECTOR> <IO_INDEX> <NAME>` edits just the configured consumer's
Name cell (`Load/@nm`), retaining its priority, stable connector identity and all
other fields. This is **new nested-load behavior**, not a block title rename or
`set-param` (which cannot address this cell). It follows the native single-cell
paste/clear callback, editable-column gate, string setter and accepted-copy path.
Unicode, duplicate names, leading/trailing spaces and whitespace-only names are
preserved exactly; there is no trimming, normalization or uniqueness requirement.
An empty name clears `nm` using the native serializer's omission rule. A no-op
preserves absent versus explicit-empty `nm` and existing spelling. In-place no-ops
do not write or stamp the file; `--save-as` still produces a file without changing
the Document stamp. JSON additionally reports `changed`.
For a name beginning with `-`, put `--` before the name so it is not parsed as
an option: `lox config load-manager set-name file.Loxone "Load Manager" 4 -- "-spare"`.

The CLI conservatively rejects control characters (including tab/newline/NUL),
U+2028/U+2029, U+FFFE/U+FFFF and names longer than **32767 UTF-16 code units**.
That is a declared CLI subset, **not a recovered native maximum**. Tab/newline
paste follows a different native multi-cell path, which this operation does not
emulate. Rejections are atomic. Clearing a name does not emulate the dialog's
subsequent initialization: on reopening, Config may populate an empty name from
a connected sensor. No connector-derived defaults are fabricated by the CLI.
Source proof, conservative limits and an independent fresh-native recipe:
`tests/ux/load-manager/name-source.json`. No native/runtime qualification is claimed,
and `--dry-run` remains unsupported for this new setter.

`set-power <FILE> <SELECTOR> <IO_INDEX> <POWER_KW>` edits only the configured
consumer's **Power [kW]** cell (`Load/@pwr`), not EnergyManager2 rated power or a
block connector parameter. It reuses the twelve-row identity/selection guards,
including omitted `ioIdx` meaning zero. Order, name, timing settings, all other
records, unknown fields, connectors, IDs and flags remain unchanged.

The recovered native numeric editor uses a minimum of zero, maximum finite
double and three decimal places; the setter stores a double and the XML writer
formats three decimal places and trims trailing fractional zeroes. The CLI
supports a conservative, locale-independent subset: nonnegative dot decimals
with digits before the dot and **at most three fractional digits**, provided the
requested decimal survives double/three-decimal conversion unchanged. For example,
`2.125` means 2.125 kW and `0.001` means 1 W. Signs, exponent notation, unit
suffixes, grouping, nonfinite values, extra precision and double precision loss
(such as `9007199254740993`) are rejected, never clamped or silently rounded.
Binary values exactly halfway between native three-decimal outputs are also
rejected rather than assuming a native CRT tie-breaking mode.
No exact native saved-byte/locale parity is claimed.
Exact native CRT formatting of very large accepted integers is also unqualified.

The selected record must already have an explicit, parseable, finite nonnegative
`pwr` decimal; only zero exponents on decimal mantissas (such as `2.500e0`) are
supported in existing storage. Other exponent forms, missing or malformed target
storage are rejected rather than interpreted using the permissive native reader.
No missing settings on other records are filled. A numerically equal request
preserves the original `pwr` spelling (including exponent/trailing zeroes).
In-place no-ops do not write or change the Document stamp; `--save-as` still writes
with the unchanged stamp. Actual edits use ordinary save/stamp and BOM/EOL behavior.
JSON reports `changed`; global `--dry-run` remains fail-closed and unsupported.
Source chain, limits and a fresh independent native verification recipe:
`tests/ux/load-manager/power-source.json`.

`set-max-off <FILE> <SELECTOR> <IO_INDEX> <MAX_OFF_SECONDS>` edits only the
configured consumer's **Maximum off time [s]** cell (`Load/@maxOff`). For example,
`600` means ten minutes: both the native heading and direct integer setter
establish **seconds**, with no scaling. This is nested-load behavior, not
`set-param`; `minOn` is never changed.

The CLI deliberately supports only **whole seconds in 10..2147483647**, supplied
as unsigned decimal digits (leading zeroes are allowed). Native column 5 uses the
numeric delegate with zero decimal places, but its setter converts to a signed
32-bit integer, clamps 1..9 to 10 and negatives to zero. The CLI rejects these
clamping cases, fractions, signs, exponent notation, whitespace, units, nonfinite
values and overflow rather than silently rounding or changing the request.
**Zero is unsupported: its UI meaning remains unqualified.** This positive-only
subset is not a claim about the full native input domain or runtime shedding.

An absent `maxOff` reads as zero in the recovered native integer reader; setting
a positive value is supported without filling any other omitted settings.
Explicit target storage must be a nonnegative signed-32-bit decimal integer;
an existing leading plus/zeroes is supported. Empty, negative, fractional,
exponent, whitespace, overflowing or malformed storage is rejected atomically.
Numeric no-ops preserve its exact old spelling, the file bytes and Document stamp
in place; `--save-as` uses the ordinary writer with the unchanged stamp.
Actual edits use normal Document stamping and BOM/EOL preservation.
When `maxOff` was absent, the new attribute is appended; native attribute-position
parity is unqualified. Existing attributes retain their relative order.

The existing twelve-row/unique resolved `ioIdx` guards apply, including omitted
`ioIdx` resolving to zero. Moving a row never changes which consumer is targeted.
Order, other records, power, name, `minOn`, unknown fields, IDs, flags and wiring
are preserved. JSON reports `changed`; global `--dry-run` still fails closed
before file access. This operation is source-recovered, not native save/reopen
or runtime qualified. Evidence and a fresh independent native recipe:
`tests/ux/load-manager/max-off-source.json`. This is source-only; typed editing,
native save/reopen and runtime effects remain unqualified.

`set-min-on <FILE> <SELECTOR> <IO_INDEX> <MIN_ON_SECONDS>` edits only the
configured consumer's **Minimum on time [s]** cell (`Load/@minOn`), not a direct
connector parameter. `600` means ten minutes: the column-6 heading, integer
getter/setter and serializer establish seconds without scaling, independently
of `maxOff`.

Supported requests are unsigned decimal digits representing **0 or whole seconds
in 10..2147483647** (leading zeroes allowed). The native column-6 setter accepts
zero, clamps 1..9 to 10 and negatives to zero. Its numeric editor has minimum zero,
zero decimal places and a double maximum, but conversion/storage uses signed
32-bit integers. The CLI rejects 1..9, signs, fractions, exponents, whitespace,
units, nonfinite values and overflow rather than rounding, clamping or guessing
conversion behavior.

**Zero is a supported storage edit, not a recovered SPS/runtime promise.** The
native row initializer defaults `minOn` to zero, its integer reader maps absence
to zero, and its writer omits zero. An actual positive-to-zero edit therefore
removes only `minOn`; a numeric no-op preserves existing spelling or omission.
Explicit target storage must be nonnegative signed-32-bit decimal digits, with
an optional leading plus. Empty, negative, fractional, exponent, whitespace,
overflowing or malformed storage is rejected atomically; the input and any
existing destination remain untouched.

The shared complete-twelve-load/unique resolved `ioIdx` guards apply. Omitted
`ioIdx` remains native zero, and identity stays stable after a move. No other
settings are filled or normalized: `maxOff`, power, name, order, other rows,
unknown fields, IDs, flags and wiring are preserved. In-place no-ops preserve
file bytes and Document stamp; `--save-as` uses the ordinary writer with the
unchanged stamp. Actual edits use ordinary Document stamping and BOM/EOL
preservation. Newly added attributes append; native attribute-position parity
is unqualified. JSON reports `changed`; global `--dry-run` remains unsupported
and fails before file access. Typed Qt conversion edge cases, native save/reopen
and runtime shedding are unqualified. Source evidence and fresh native recipe:
`tests/ux/load-manager/min-on-source.json`.

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

An Energy empty-name VM121 run saved/reopened `Na=""` with existing
`Ncu=true`/`Anc=true`; all twelve load records match its immutable CLI result,
but the full file differs. **It is not a qualified pass:** two reopened
navigation inputs were dispatched after finite resource admission expired.
The complete findings and invalidation are retained in
`tests/ux/energy-manager2/empty-name-persistence-invalid-20261112.json`;
no GUI-case count, inheritance reset or whole-file/runtime parity is promoted.
A separate VM121 rated-power operation is qualified within its selected-field
scope: **37 kW** persisted through save, normal exit and fresh-process native
readback, with SwPwr2.6 and all twelve load records matching the immutable CLI
expectation. All 27 input-admission audits passed. Whole-file parity remains
false; every unrelated difference is retained. See
`tests/ux/energy-manager2/rated-power-qualified-20261112.json`. This does not
repair the earlier invalid name-clear run or qualify family/runtime behavior.

### Lighting stored inspection and mood-count repair

```bash
lox config lighting inspect file.Loxone "Lighting" --output json
lox config lighting inspect-mood-options file.Loxone "Lighting" --mood "Evening" --output json
lox config lighting inspect-mood-references file.Loxone "Lighting" --mood "Evening" --output json
lox config lighting recount-moods file.Loxone "Lighting" --save-as edited.Loxone
lox config lighting recount-moods file.Loxone "Lighting" --room "Study" --output json
lox config lighting set-mood-level file.Loxone "Lighting" --mood "Evening" --output-index 2 --level 37 --save-as edited.Loxone
lox config lighting set-mood-level file.Loxone "Lighting" --mood "Evening" --output-index 2 --level 37 --dry-run --output json
lox config lighting set-mood-cid file.Loxone "Lighting" --mood "Evening" --cid 13 --save-as rebound.Loxone
lox config lighting set-mood-cid file.Loxone "Lighting" --mood "Evening" --cid 22 --dry-run --output json
lox config lighting set-mood-sid file.Loxone "Lighting" --mood "Evening" --sid 2 --swap --save-as exchanged.Loxone
lox config lighting set-mood-sid file.Loxone "Lighting" --mood "Evening" --sid 32 --allow-cid-clear
lox config lighting set-mood-sid file.Loxone "Lighting" --mood "Evening" --sid 32 --allow-cid-clear --dry-run -o json
lox config lighting rename-mood file.Loxone "Lighting" --mood "Evening" --name "Late Evening" --save-as renamed.Loxone
lox config lighting set-mood-mix file.Loxone "Lighting" --mood "Unbound scene" --no-mix --save-as unmixed.Loxone
lox config lighting set-mood-skip file.Loxone "Lighting" --mood "Evening" --skip --save-as skipped.Loxone
lox config lighting set-mood-skip file.Loxone "Lighting" --mood "Evening" --skip --dry-run -o json
lox config lighting set-mood-display-name file.Loxone "Lighting" --mood "Evening" --name "EVENING"
lox config lighting set-mood-display-name file.Loxone "Lighting" --mood "Evening" --name "Late Evening" --normalize-ascii
lox config lighting set-mood-display-name file.Loxone "Lighting" --mood "Evening" --clear
lox config lighting set-mood-display-name file.Loxone "Lighting" --mood "Evening" --name "Late Evening" --normalize-ascii --dry-run
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
control branch, **not** universal RGB support. The bounded `set-mood-rgb`
operation below validates one ordinary RGB branch; existing
`config set-mood-color` remains a raw packed-value write. The raw operation does not validate
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

With global `--dry-run`, the same count-repair plan is returned without mutation,
serialization, save stamps or destination handling, including `--save-as`.
JSON retains `before`, `after`, `scenes`, `changed` and adds `dry_run`, actual
controller UUID/title, controller-relative collection path, scope and
`before_effective`/`after_effective` signed-32 counts. Missing or invalid stored
counts have null `before_effective`; negative counts remain negative, not clamped.
Raw absence/spelling and the actual planned spelling (including no-op `+2`/`0002`)
remain visible in JSON, table and CSV; CSV includes presence columns.
This previews an existing CLI repair only, not a native UI operation or a
prediction that saving to the requested destination will succeed.

`set-mood-level` is a numeric edit, **not RGB packing**. It preserves the
selected Qn's high type nibble and replaces its low payload with an integer
in `0..100`; values outside that range are rejected, never clamped.
It requires one direct mood matching exact stored SID spelling or Name,
an existing signed-32-bit Qn within the stored reader width, and a supported
numeric editor kind. Missing/zero scene `Outputs` means 12 on that reader;
the recovered editor path supports indices 1..18, not a universal maximum.
Missing Qn, ambiguous matches and unsupported widths fail before writing.

One independently verified Config 17.2.10.21 VM122 operation qualifies only the
original tag6 numeric-capability **Q2=37**, mood SID1/CID12: native acceptance,
official save, normal exit, fresh-process native readback and final exit retained
identical saved/reopened/final bytes. **Native collateral remains; this is not
selected-only mutation or whole-file, lighting-family, SPS/runtime parity.**
The complete raw audit retains Q1, HCL, 94 Title, two WF and Date/DateS changes
(99 start tags/104 attributes from input; 103 attributes from immutable CLI).
All 17 qualifying input gates were independently verified; physical delivery
timing was not instrumented. See
`tests/ux/light-controller2/numeric-level37-qualified-20261112.json`.
Historical observations, failed phases and other cases receive no new credit.

`set-mood-rgb` supports **only an existing ordinary tag-6 RGB branch with
explicit `CapAI<n-1>=7`**, not every output classified `color`:

```bash
lox config lighting set-mood-rgb file.Loxone "Lighting" --mood "Evening" --output-index 2 --rgb 64,128,255 --save-as edited.Loxone
```

Inputs are three integer **bytes 0..255**, not percentages, HSV or raw packed
values. Each becomes `round(byte / 2.55)` percent, then the selected Qn is
`old_type_tag | (Rpercent + Gpercent*1000 + Bpercent*1000000)`.
For `64,128,255`, percentages are `25,50,100` and stored tag-6 Qn is
`1710662761`. No channel is clamped. An all-zero rounded result is rejected:
tag-6 zero reopens as a dynamic-white default, not an ordinary RGB-mode proof.

The selected existing Qn must already have a nonzero, valid ordinary RGB
percent payload, not a dynamic-white payload (`>=200000000` below the tag).
Require one direct collection/mood, explicit `Num` equal to the direct mood
count, parsed sibling SIDs and a unique selected ordinary SID in `1..90`.
Selection uses exact stored SID spelling or case-sensitive Name; no first-row
mode assumption. Indices are `1..18`, within the scene's native reader width
(absent/zero `Outputs` means 12). Selected `Locked`, `ReadOnly`, `Sequencing`
must be absent/false, with no `SeqConf`. Numeric, other tags/capabilities,
white/default/sequence state, malformed storage and ambiguous identities fail
before any save. These are conservative supported-subset guards, not universal
native restrictions or physical actuator roles.

Only the selected Qn changes: all other Qn, IDs/order, widths/Num, flags,
LSConfig, HCL and unknown data are retained. Equivalent numeric spelling is
preserved on no-op; BOM/EOL and ordinary DOM writer/save-stamp behavior remain.
JSON includes actual selected identity, raw before/after, input bytes,
rounded percentages, capability and `changed`; table/CSV, `--quiet`,
`--no-header`, `--room` and `--save-as` are supported.
`--dry-run` remains explicitly unsupported and fails closed.
This is a source-backed final-color encoding operation, not arbitrary native
picker text/HSV/brightness-gesture equivalence, a new native roundtrip result
or graph-dependent capability refresh. The existing raw `config set-mood-color`
parser, accepted inputs and writer are unchanged.
Native validation uses canonical decimal identities and capabilities; native
loading of plus-prefixed or zero-padded numeric spellings remains unverified.
Evidence and a bounded native fixture recipe:
`tests/ux/light-controller2/ordinary-rgb-source.json`.

The Config 17.2.10.21 per-mood renderer uses tags 1/2/3/7 directly as numeric.
Tag 6 is capability-dependent: for Qn it reads `CapAI<n-1>`. With bit `0x40`
clear and `(CapAI & 7) != 7`, it selects a numeric control; otherwise it selects
color controls. Missing capability has the recovered reader default 0,
reported separately from its raw omission in JSON. Needed explicit capabilities
must be nonnegative 16-bit decimals; unsupported values are not truncated.
Color, switch and unresolved modes are rejected by this command.
These are stored editor-kind checks, not inferred actuator roles or a simulation
of graph-dependent pre-modal capability/type refresh.

Global `--dry-run` previews **only this already supported numeric edit** with
the same ordered validation and one immutable plan shared with the existing
writer. JSON retains the normal writer fields (`mood`, `output`, raw `before`/
`after`, `level`, `type_tag`, needed `capability`, `changed`) and adds
`selected_mood` (actual raw UUID/SID/CID), `output_index`, `editor_kind`,
`before_effective`, `after_effective`, `dry_run: true` and optional `trace_id`.
The effective values are the selected stored **low-28-bit scalar payloads**,
not native displayed/clamped levels, RGB decoding or physical brightness.
Even an existing payload above 100 is reported without clamping; only the
requested replacement must be in `0..100`. Equivalent no-op decimal spellings
remain visible and unchanged.

Dedicated table/CSV show the selected raw identity, Qn/index, numeric
classification, raw and effective before/after, preserved tag and consulted
capability. CSV includes presence/consulted fields to distinguish omission from
unneeded capability. Quiet suppresses only the table summary; no-header
suppresses only headers. Normal writer output is unchanged.

Preview returns before mutation or save, including no-op, failures, and
`--save-as` pointing to the input, an existing output or a new missing directory.
It performs no source/destination/default/stamp/directory/cache/telemetry writes,
destination-writability checks or serialization/write-success predictions.
It adds no numeric/tag/capability/width/identity admission, repair or defaults.
`set-mood-color` stays a separate raw packed edit with its own preview below;
other unplanned commands still fail closed. This source-only
preview adds no native, SPS/runtime, family or whole-file parity credit.

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

### Raw packed mood-color preview

```bash
lox config set-mood-color file.Loxone "Lighting" --mood "Evening" --output-index 2 --color "hsv(120,100,10)" --dry-run --output json
```

This previews **only the already supported RAW packed Qn edit**, not a
mode-aware/native color-picker operation or a channel-appropriate color claim.
It uses the unchanged `hsv(...)`/`rgb(...)`/integer helper: actor composites
are converted to percent-channel mood packing, while accepted packed integers
pass through. Existing parsing, rounding and range checks are unchanged; no
new signed-32/u32 bound, normalization or correction is applied.

The shared immutable plan retains the raw writer's ordering: read/load XML,
parse color, resolve one controller, require `LightController2`, reject index
zero, then take the **first recursive** exact stored SID-or-Name match.
Unlike the numeric/metadata commands, duplicate mood matches are not rejected,
missing SID/Name still match an empty mood reference, and direct-container,
stored Outputs/Num, Qn type/tag, capability and identity validity are not
classified or newly enforced. Missing nonzero Qn can still be inserted, and
there is no new Q18 limit. Inspect the selected identity/path before execution.

JSON exposes `scope: "raw_packed_edit"`, actual raw `controller_uuid`/
`controller_title`, requested `mood`, `selected_mood` UUID/SID/CID,
`selected_mood_name`, `selected_mood_path` (zero-based XML child-node indices
relative to the controller, including non-element nodes), `output`,
`output_index`, raw `before` (null if absent), decimal `after`, parsed `value`,
`changed`, `dry_run: true` and optional `trace_id`. No decoded/effective/native
color or editor-kind fields are invented. `changed` compares stored strings:
equivalent plus/zero-padded decimal spellings are replaced by the existing
writer's decimal spelling, while an exact raw match reports false.

Dedicated table/CSV retain identities, selected path/Qn and raw before/after;
CSV has presence fields and quoted strings. Quiet hides only the table summary;
no-header hides only headers. Normal writer output, unconditional attribute
insertion/save, overwrite behavior, serialization and Document stamp rules
are unchanged, including exact no-op execution.

Preview returns before mutation/save/default/stamp/destination/directory/
cache/telemetry writes, even for no-op, failures, existing output, input
`--save-as` or a missing output directory. It does not check destination
writability or predict successful saving. Only this existing raw operation is
newly admitted under global dry-run; unrelated mutations remain fail-closed.
Source-only evidence: `tests/ux/light-controller2/raw-mood-color-preview-source.json`.
No new native, family, whole-file or SPS/runtime parity is claimed.

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

Preview the **already supported** creation operation without changing the file:

```bash
lox config add-mood file.Loxone "Lighting" --name "Evening" --color "hsv(30,80,20)" --dry-run --output json
```

Preview and execution share one immutable creation plan, retaining the exact
existing ordered errors, automatic/explicit identity allocation, duplicate-name
and SID rules, output initialization and direct-record recount. No new role,
Unicode, tag, capability or mode admission is imposed. `--color` remains the
existing raw packing parser, not a native color-picker operation.

JSON reports `scope: "stored_mood_creation"`, stored controller UUID/title,
`collection_path` (zero-based controller-relative XML child-node indices),
`container_created`, raw optional `before_num`, decimal `after_num`, and
`proposed_mood`: the actual proposed `LightsceneC` attributes, including Name,
UUID, SID, CID, Outputs and every Qn. The existing path requires exactly one
direct collection, so `container_created` is false; preview does not create one
for unsupported input. No decoded, effective, UI or physical-channel values
are invented. Table/CSV retain the same attributes; CSV has presence columns
and escaped strings. `--quiet` hides only the table summary, `--no-header` only
headers, and JSON supports `--trace-id`.

The planned UUID is **ephemeral**. Separate preview/execution invocations may
generate fresh UUIDs; execution does not reuse or accept a preview UUID.
Preview returns before mutation, saving or stamping, including with existing,
input or missing-parent `--save-as`. It writes no files, directories, defaults,
cache or telemetry and does not check destination writability or predict
successful saving. Normal execution keeps its existing API/report, overwrite,
save and Document-stamp behavior. Only AddMood gains this additional dry-run
path; unrelated unplanned mutations still fail closed.
Source-only evidence: `tests/ux/light-controller2/mood-creation-preview-source.json`.
No new native, family, whole-file or SPS/runtime credit is claimed.

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

Add `--dry-run` to preview **only this existing CID transaction** using the same
planner and admission guards as the writer. It never mutates the editor or saves
the input or `--save-as` destination, even for no-ops; no defaults, stamps or
destination directories are created. `--save-as` is ignored for preview and
destination writability is not tested. Unsupported/ambiguous inputs remain
errors, not successful empty plans. Unneeded opaque selection data and unvisited
opaque tails retain the writer's existing admission/preservation rules.

Preview JSON adds `dry_run: true`, `selected` (record/UUID and raw SID/CID
before/after, including null for absent CID), and effective `current_cid` to the
normal edit report; `requested_cid` is the proposed effective CID.
`identities`, ordered `selection_remaps`, optional `selection_refs` and `changed`
are exactly the writer's plan. Table/CSV show selected stored/effective identity,
affected identity records, ordered remaps and consulted `ScMv`; CSV presence
columns distinguish omission from empty text. `--quiet` hides only the table
summary, and `--no-header` hides headers without hiding preview data.
This source-only local preview does not predict GUI Cancel, dialog refresh,
final native save, file-write success, current graph state or runtime parity.
See `tests/ux/light-controller2/mood-cid-preview-source.json`.

`lighting set-mood-sid` reassigns an existing ordinary SID to **1..90**.
Both selected and requested SID must be in that range; special/display IDs
are not simulated. An occupied different SID
requires explicit `--swap`, matching native exchange confirmation intent.
All direct SIDs must be explicit, positive and numerically unique signed-32-bit
decimals; `Num` must match their count. An existing SID99 blocks occupied
exchange because native uses temporary99. No IDs are silently allocated.
Changing to a **free SID32..90** with positive CID additionally requires
`--allow-cid-clear`, accepting the native confirmation that sets selected CID0.
Missing/zero CID needs no clearing and its original omission/spelling remains.
Needed explicit CID must be a supported nonnegative signed32 decimal.
The flag permits this branch only: lower-ID edits, occupied exchange and
numeric no-ops do not clear CID; it does not substitute for `--swap`.

Free-ID edits remap selected-old to requested. Exchanges remap
requested-old to99, selected-old to requested, then99 to selected-old **in
order**, so an already visited ScMv99 also changes in the final step. The
shared CID/SID planner visits only the available `ScMv` prefix bounded by
`OpM` length, preserves whitespace/unvisited tails, and rejects needed opaque
lists before writing. Apart from explicitly accepted selected CID clearing,
CIDs, UUIDs, names, flags, outputs, counts, HCL and
unrelated storage stay unchanged. Numeric no-ops preserve original bytes.
JSON includes ordered remaps and affected identities; CSV uses the existing
identity/selection transaction rows. This is not a raw role-ID setter,
import/refresh prediction or native/runtime parity.
See `tests/ux/light-controller2/mood-sid-edit-source.json`.
Its supported-domain extension is `mood-higher-sid-source.json` in the same directory.

Add `--dry-run` to preview **only the existing ordinary SID transaction** with
the writer's identical guards and explicit `--swap`/`--allow-cid-clear`
acceptance. Missing confirmation flags still produce the existing error, not
a successful empty plan. No special/reserved-ID conversion or allocation is added.
One immutable plan serves both preview and writer. Preview returns before
mutation or any save, even with `--save-as` pointing to the input, an existing
file or a nonexistent directory; no defaults, stamps, directories or telemetry
files are written. Destination writability and writer serialization are not tested.

Preview JSON adds `dry_run: true`, `selected` with raw SID/CID before/after,
and effective `current_sid`; `requested_sid` is the proposed effective SID.
The existing `swapped`, `cid_cleared`, affected `identities`, ordered
`selection_remaps`, optional `selection_refs` and `changed` are the writer's
exact plan. Numeric no-ops show original SID spelling and CID omission/opaque
storage without normalizing either or parsing unneeded `OpM`/`ScMv`.
Table/CSV share CID preview rows for selected raw/effective identity,
affected identities, ordered remaps and consulted `ScMv`; CSV presence columns
distinguish absent from empty. `--quiet` hides only the table summary;
`--no-header` hides headers, not plan data. JSON retains `--trace-id`.
This is source-only stored transaction planning, not GUI Cancel/dialog refresh,
native final-save/file-write-success prediction, SPS/runtime/family or whole-file
parity. See `tests/ux/light-controller2/mood-sid-preview-source.json`.

`lighting rename-mood` changes only an existing direct mood's `Name`. Supported
new names contain ASCII letters/digits and ordinary internal spaces, with a
letter/digit at each end. Empty, edge-space, punctuation/control and Unicode
inputs are rejected rather than approximating native normalization. Duplicate
new names are allowed, matching the native row-bound setter; if that makes a
later Name selector ambiguous, use its exact stored SID spelling. Missing `Name`
can be populated without generating IDs. `SName` is a different field and stays
unchanged. An in-place identical-name edit preserves original bytes.

Add `--dry-run` to preview **only this existing ASCII Name edit** without
changing XML or saving, including with `--save-as` pointing to the input, an
existing destination or a new file under a missing directory. The new-name
validation runs before direct mood selection, exactly as in the writer, even
on no-op. No Mix/Skip numeric identity or role restrictions are imported:
unneeded opaque/missing IDs, flags and prior names retain existing admission.
One immutable typed plan is shared with the writer; no clone/diff simulation
or writer-then-discard is used. There are no source/destination/default/stamp/
directory/cache/telemetry writes or destination-writability/serialization
predictions. JSON adds `dry_run=true` and optional `trace_id` to the unchanged
writer report. Dedicated table/CSV always show selected raw UUID/SID/CID and
before/after names, including no-op; CSV presence columns distinguish absent
from empty identities/prior names. `--quiet` hides only the table summary and
`--no-header` only headers. Normal writer API/output/save/no-op semantics and
all other unsupported dry-run commands remain unchanged. This source-only
preview adds no Unicode normalization, native/GUI/permission/graph, SPS/runtime,
family or whole-file qualification.
See `tests/ux/light-controller2/mood-name-preview-source.json`.

`lighting set-mood-mix` requests exactly one of `--mix` / `--no-mix` and changes
only an eligible existing `MixMod` flag. This is not a raw boolean/cohort editor:
it requires selected SID 1..31, nonnegative CID **outside 12..19** or omission
(recovered effective0),
ASCII case variants of `MixMod="true"`/`"false"` or omission (effective/default false), and a unique selected numeric SID among
direct records. All direct sibling SIDs must be supported signed-32-bit
decimals. CID 12..19 can force/disable the native checkbox through graph/role
state, so those rows are rejected even on a requested no-op. Special/non-checkbox
rows, missing/opaque SIDs, unsupported explicit CIDs and explicit flag values are also
rejected. The omitted flag's false default is source-backed, not a role-state guess.
A `--no-mix` no-op on omitted MixMod preserves omission and exact in-place bytes;
JSON reports raw `before`/`after` null plus `before_effective=false` and
`default=false`. Enabling it writes only selected `MixMod=true`.
See `tests/ux/light-controller2/mood-mix-default-source.json`.
Omitted CID is independently source-backed0 and is never inserted on a mix
edit or no-op; JSON retains raw `mood.cid=null` with `cid_effective=0`.
Present malformed/negative/overflowing CID is still rejected, even on no-op.
See `tests/ux/light-controller2/mood-mix-cid-default-source.json`.

Add `--dry-run` to preview **only this existing guarded MixMod edit**. One
private immutable plan serves preview and writer; the exact selection, SID,
CID/role and stored-boolean guards are unchanged, including on no-op.
Preview returns before mutation or saving, even with `--save-as` pointing to
the input, an existing file or a missing directory. No defaults, stamps,
directories, cache or telemetry files are written; destination writability
and serialization are not predicted.

Preview JSON adds `dry_run: true` to the unchanged edit report (and retains
`--trace-id`). `mood` preserves raw UUID/SID/CID; `cid_effective` remains
separate. Raw `before`/`after` distinguish omitted MixMod (`null`) and original
ASCII case spelling from `before_effective` and proposed effective `enabled`.
A no-op never inserts missing CID/MixMod or normalizes identity/flag spelling.
Table/CSV expose the same selected raw/effective values even on no-op;
CSV has presence columns for CID and before/after flags. `--quiet` hides only
the table summary; `--no-header` hides headers, not preview rows.
Unsupported/ambiguous data still produces the writer's existing error.
Other unplanned lighting mutations remain fail-closed under `--dry-run`.
This is source-only stored-field planning, not a new role/visibility editor,
GUI Cancel/dialog refresh, native final-save/write-success prediction,
graph inference, SPS/runtime/family or whole-file parity.
See `tests/ux/light-controller2/mood-mix-preview-source.json`.

`lighting inspect-mood-options` reads one exact direct mood without saving.
It reports raw/effective/default values for SkipNext, MixMod, ReadOnly,
Sequencing, Move and the separate SName display text. Omitted flags remain
stored null with recovered false; omitted SName reads empty, not inherited.
Unsupported explicit flags remain unresolved, not false. Cycling projects
the inverse of SkipNext. Mix projection is direct only in its recovered
ordinary branch: SID>31 has no checkbox, CID12..19 needs current graph/role
state, and unsupported identities remain unresolved. These are stored
projections, **not** GUI permissions, import/row visibility or write admission.
JSON includes diagnostics; table/CSV retain limitation diagnostics even with
quiet/no-header. CSV includes `STORED_PRESENT` to distinguish absent from empty
text. See `tests/ux/light-controller2/mood-option-inspection-source.json`.
The additional `IID` row is a **numeric icon reference**, not an input binding.
It reports stored/effective/default0 and explicit unresolved catalog mapping;
unsupported explicit IDs are retained, not defaulted. The integer is not a
built-in icon GUID. No icon assignment/clear or input mutation is implied.
See `tests/ux/light-controller2/mood-icon-id-source.json`.

`lighting inspect-mood-references FILE BLOCK --mood SID_OR_NAME [--room ROOM]`
is read-only local XML inspection. It selects one exact stored SID spelling or
case-sensitive Name in one direct LightscenesC; all direct SIDs must be explicit
positive signed32 decimals, and the selected numeric SID must be unique.
Ambiguous mood/LSConfig containers or identity are explicit errors, not native
first-match guesses. CID is not interpreted.

The shared CID/SID reference codec visits only available ScMv entries below the
supported OpM vector count. Needed comma-separated signed32 tokens allow
surrounding ASCII whitespace and plus/zero spelling. JSON reports raw
`mood.sid`, numeric `selected_sid`, raw `opm`/`scmv`, `opm_count`, and zero-based
`visited` entries with `index`, `raw`, numeric `value` and
`matches_selected_sid`. A supported `unvisited_tail` reports `start_index`,
lexical `token_count` and raw suffix; opaque/empty tail tokens are not parsed.
Absent LSConfig or missing/empty vectors yield `status=no_visited_references`
and `visited=[]`, without parsing unneeded opposite values or inventing a count
or tail boundary. Unsupported needed values yield `status=unresolved`,
`visited=null` and an explicit diagnostic, never an empty resolved result.

Raw `stored_num` and actual `direct_moods` are reported separately; missing,
unsupported or mismatched Num warns without hiding records or repairing it.
JSON includes all diagnostics. Table/CSV retain status, raw vectors and
identity, visited matches and supported tail boundary; CSV `RAW_PRESENT`
distinguishes absence from empty. `--no-header` suppresses headers, and `--quiet`
suppresses only the table count, never data or stderr diagnostics. No saves,
stamps, normalization, Num-limited import, whole-transaction admission, native
visibility/writeability or GUI/runtime parity is inferred.
See `tests/ux/light-controller2/mood-reference-inspection-source.json`.

The rename/mix transactions require exactly one direct collection and one
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

`lighting set-mood-skip` requires exactly `--skip` / `--no-skip` and changes
only the selected `SkipNext`. The native cycling checkbox is **inverted**:
checked means included (`SkipNext=false`). JSON/CSV report stored/effective
before values, the resulting skip value and `checkbox_checked`. Missing
`SkipNext` has recovered effective/default false; a `--no-skip` no-op keeps
it absent. Explicit values must be ASCII case variants of `true`/`false`; malformed or
numeric flags are rejected, not treated as false. The selected positive
signed-32-bit SID must be numerically unique among supported direct sibling
SIDs. Unlike MixMod, this checkbox is not restricted to SID1..31 or by
CID12..19. No other flags, IDs, outputs, references or HCL are normalized.
This is a source-backed selected-field edit, not native/runtime cycling
parity. See `tests/ux/light-controller2/mood-skip-source.json`.

Add global `--dry-run` to preview this same guarded SkipNext edit without
changing XML or saving anything, including with `--save-as` pointing at the
input, an existing file or a missing directory. The immutable plan is shared
with the unchanged writer: exact selection, positive signed32 selected SID,
supported sibling SIDs, numeric uniqueness and supported boolean guards/errors
stay identical, even for no-op requests. CID is reported raw, not interpreted;
MixMod's SID1..31/CID12..19 restrictions do not apply here.
JSON adds `dry_run=true` and optional `trace_id` to the existing edit report;
`before.stored`/`after` retain raw omission or ASCII case spelling,
`before.effective`/`skip` show current/proposed booleans, and `checkbox_checked`
is the proposed inverse. Table/CSV always retain those raw/effective values,
raw UUID/SID/CID, default and changed, including no-op under `--quiet`.
CSV presence columns distinguish omission; `--no-header` hides only headers,
and quiet hides only the table summary. No source/destination/default/stamp,
directory/cache/telemetry writes, destination-writability checks or serialization
prediction occur. This is source-only planning of one already supported stored
field, not native dialog/save/reopen, GUI permissions/visibility/graph state,
physical cycling, SPS/runtime/family or whole-file qualification.
Other unsupported mutations still fail closed under global `--dry-run`.
See `tests/ux/light-controller2/mood-skip-preview-source.json`.

Boolean inspection and the guarded mix/skip edits accept native ASCII case
variants such as `TRUE`/`FaLsE`, retaining original spelling on no-op. A real
change writes only the requested flag. Numeric, space-padded, opaque and
non-ASCII lookalike values remain unresolved/rejected, not coerced to false.
See `tests/ux/light-controller2/mood-boolean-case-source.json`.

`lighting set-mood-display-name` edits **SName**, not `Name`, with exactly one
of `--name NAME` / `--clear`. New names use uppercase ASCII letters/digits and
ordinary internal spaces with alphanumeric edges. Lowercase, punctuation,
edge-space and Unicode inputs fail rather than being silently normalized.
Clear writes an explicit empty SName only when previously nonempty; an
absent/already-empty value is an exact-byte in-place no-op. Native optional
serialization can omit empty SName, so this is not byte-identical native-save
or physical display parity. No inherited-label semantics are inferred.
See `tests/ux/light-controller2/mood-display-name-source.json`.
Explicit `--normalize-ascii` permits lowercase ASCII and applies recovered
native uppercase conversion before the same validation. JSON `normalized_from`
shows supplied text when converted; `after` is actual stored uppercase.
Default validation stays strict. This does not trim, transliterate, replace
punctuation or accept arbitrary Unicode, and cannot be combined with `--clear`.
Equivalent normalized storage is a byte-preserving no-op.
See `tests/ux/light-controller2/mood-display-name-ascii-source.json`.

Global `--dry-run` previews only these already supported SName set/clear and
explicit `--normalize-ascii` branches, with the same ordered guards as the
writer. It never implicitly normalizes: unsupported input remains an error,
even on a requested no-op. JSON adds `dry_run: true`, optional `trace_id`,
`before_effective` and raw `after_stored` to the unchanged edit report.
`before`/`after_stored` distinguish omission (`null`) from explicit empty
(`""`); `after` is the actual planned effective text and `normalized_from`
retains converted input even on a storage no-op. Clear stores explicit empty
only when previously nonempty; an omitted clear stays omitted.
Table/CSV retain selected raw identity, raw/effective before/after text and
normalization input even with `--quiet`/`--no-header`. Quiet hides only the table
summary. Preview returns before mutation/save, including no-op, error and
input/existing/new missing-directory `--save-as`: no file, directory, default,
stamp, cache or telemetry writes, destination-writability checks or serialization
predictions. Normal writer API/output/save behavior is unchanged.
This is source-only stored-text planning, not native dialog/save/write-success,
physical display, live permissions/graph, SPS/runtime/family or whole-file parity.
See `tests/ux/light-controller2/mood-display-name-preview-source.json`.

Inspection emits `duplicate_mood_sid` / `duplicate_mood_cid` for numerically
equivalent positive signed-32-bit SIDs / nonzero signed-32-bit CIDs in one
direct collection, including zero-padded/plus spellings. Repeated CID0 is
allowed; nested or separate collections are never joined. Raw unsupported
identities remain untouched. Diagnostics appear in JSON or on stderr for
table/CSV even with `--quiet` / `--no-header`. No duplicates does **not** mean
valid native import or editability. See `identity-ambiguity-source.json`.

An independent repaired-SDK VM122 run actually opened the prescribed SID1/Q1
Color Picker for the tag6/CapAI0=7 input variant. This agrees with stored CLI
classification but is **not** saved/reopened or complete-case evidence:
capacity closed before persistence, and native pre-modal state was not
assumed unchanged. See `color-widget-observation-20261111.json` under
`tests/ux/light-controller2`; aggregate parity flags remain false.
A later, separately recorded phase saved this inspection, exited and reopened
in a fresh process with selected color-dropdown readback. Native bytes stayed
identical across exit/reopen, while all 98 changed nodes/100 attribute deltas
against input remain retained. **Full guarded qualification is withheld**:
earlier deadline evidence is missing and two post-closure read-only calls are
disclosed. See `color-widget-persistence-20261112.json` in the same directory.
Neither phase establishes an accepted color-value edit or whole-file parity.

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

Delete one existing native schedule entry without replacing its neighbors:

```bash
lox config timer-entry-remove file.Loxone "Night Timer" --mode-uuid 11111111-1111-1111-1111-111111111111 --from 20:00 --to 24:00 --save-as edited.Loxone -o json
```

Identity is the referenced **operating mode UUID plus exact From/To interval**,
not XML order, an entry UUID, weekday mask, or operating mode number. `--room`
disambiguates the timer. Times require exact `HH:MM`; split overnight intervals
are separate records. The old `timer-schedule` replacement command is unchanged.

This bounded operation requires ordinary digital `M=0` storage (`Analog` absent
or `false`), consistent native `N`,
valid unique `Modes` UUID references, in-range `Ix`, and non-touching,
non-overlapping intervals within each mode. Analog schedules are not yet qualified
and are rejected, not treated as unsupported by the native application.
Stored Modes and referenced Mode UUIDs must use the non-nil 36-character hyphenated form;
Modes is comma-separated without whitespace. Other UUID representations are
rejected rather than assumed to have identical native parsing.
Unsupported/ambiguous storage is rejected, not repaired. `From`/`Ix` omission
means native zero; it does not mean all days. Legacy To-only replacement
storage without native counts/references is not accepted.

Only the selected Entry and its native `N` count change, plus the normal Document
save stamp. The last entry may be deleted (`N=0` is omitted); the default value,
Modes order/references, surviving entry order/attributes, unknown fields, and
connector wires remain unchanged. The digital native default remains zero.
No normalization, renumbering, default
insertion, or mode deletion. Domain errors leave input and existing `--save-as`
files untouched. JSON/CSV/quiet output are supported; `--dry-run` remains
unsupported and fails before file I/O. See
[native source notes](docs/knowledge/daytimer-entry-delete-native.md).

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
lox config room <SUBCOMMAND> ...           # manage rooms (add/rename)
lox config control <SUBCOMMAND> ...        # manage controls
lox config mqtt <SUBCOMMAND> ...           # manage MQTT plugin configuration
```

#### Preview an existing room rename

```bash
lox config room rename file.Loxone "uuid:room" "Küche" --dry-run --json
lox config room rename file.Loxone "Old name" "New name" --dry-run -o csv --save-as output.Loxone
```

The already supported `room rename` operation has global dry-run admission.
One immutable plan is shared with the writer. It preserves the existing general
element selector, not a new room-only policy: fuzzy titles, `uuid:`, `gid:`,
`Type:`, bracket-room filtering, UUID mirror de-duplication, ambiguity and the
subsequent UUID lookup all retain their existing behavior. After the initial
selector is admitted, its selected element supplies the UUID, including when a
bracket-room qualifier disambiguates same-title elements. That UUID is still
resolved again, so an earlier canonical UUID mirror can remain the final target.
Preview exposes
`selected`, `uuid_source` and actual `target`, each with parsed child-index `path`
and raw stored attributes. `uuid_source` agrees with `selected`; `target` can
differ because of the final UUID lookup.

`before_title` preserves absence (`null`), empty values and stored spelling;
`after_title` is the exact requested name. Empty, whitespace, duplicate and
Unicode names remain admitted as before; no ASCII restriction, trimming or new
type/name guard is added. `changed` compares attribute presence/value only and
does not suppress the normal writer's save. Only the final target's `Title` is
edited; references, UUIDs, other attributes, mirrors and `IoData` are not renamed.

JSON includes `dry_run: true` and optional `trace_id`. Table/CSV use JSON-valued
cells so raw spelling and missing-versus-empty titles remain visible.
`--no-header` suppresses table/CSV headings; `--quiet` suppresses only the table
summary. Preview returns before mutation, serialization, defaults, Document stamp,
save or destination probing, including no-ops, failures and missing-parent
`--save-as` paths. Cache, network and telemetry writes are skipped. Normal writer
reporting remains its existing text even with `--json`, CSV or `--quiet`, followed
by its unchanged overwrite/save and content-change-dependent stamp behavior.

This is a raw existing-CLI source preview, not native UI, room-family, whole-file,
SPS or physical-effect evidence. Historical preview source basis:
`tests/ux/room-rename-preview-source.json` (unchanged, including its record of the
former unfiltered UUID-source bug). Source-only qualified-selector correction:
`tests/ux/room-selector-fix-source.json`.

#### Preview an existing control move

```bash
lox config control move file.Loxone --to-room "Beta" --title "Lamp [Alpha]" --dry-run --json
lox config control move file.Loxone --to-room "Beta" --type WeatherData --exclude InputRef --dry-run -o csv --save-as output.Loxone
```

`config control move` has this preview; `config xml move`, control rename and
wire/unwire still fail closed under global `--dry-run`.
The existing parser has `--type`, `--title`, `--exclude` and `--to-room`, not
`--from-room`. There is no new room creation, normalization or selection policy.
One immutable typed plan is shared by preview, the normal control-move writer,
and the unchanged public `move_to_room` API.

`--type` wins if both selectors are supplied. It resolves the room first, then
selects case-insensitive exact types with direct `IoData` children, honoring
case-insensitive type exclusions. All matches, including UUID mirrors, count in
pre-order; zero matches and already-matching room references still report and save
normally. `--title` instead admits one existing general element selector before
resolving the target room; fuzzy titles, `uuid:`, `gid:`, `Type:`, bracket-room
qualification and UUID mirror de-duplication keep their current behavior.
Exclusions are ignored in that branch. A title-selected object without direct
`IoData` still reports one selected object but changes nothing. No selector is the
existing “Specify --type or --title” error, without resolving the room.

Preview exposes the effective typed `selector`, supplied `exclude`, raw
`target_room`, resolved `room_uuid`, `count`, `changed` and existing writer
`report`. Each `selected` object contains its actual child-index `path`, raw
attributes and direct `io_data` entries. Those entries include their own actual
paths/attributes and `before_pr`/`after_pr`: absent `Pr` is `null`, distinct from
`""`. `created` means only that this existing `IoData` gains a missing `Pr`
attribute, never a new room or `IoData` element. An empty `io_data` list means no
such child exists. Only these direct `IoData.Pr` attributes are edited, in the
existing order; nested selections remain stable. UUIDs, category, wiring, flags,
unknown metadata, unmatched objects and unrelated `IoData` remain untouched.

JSON includes `dry_run: true` and optional `trace_id`. Table/CSV retain raw values
in JSON-valued cells; `--no-header` suppresses headings and `--quiet` suppresses
only the table summary. Preview returns before mutation, serialization, defaults,
Document stamping, destination probing or saving, including missing-parent
`--save-as`, zero matches, no-ops and errors. It writes no files, directories,
cache or telemetry and makes no network requests. Normal writer text output,
report-before-save, overwrite, BOM/line endings and content-change-dependent stamp
remain unchanged, even with JSON/CSV/quiet flags or a no-op.

Source-only raw CLI provenance: `tests/ux/control-move-preview-source.json`.
This is not native UI, room-family, whole-file, SPS or physical-effect evidence.
Historical room-rename artifacts and the qualified-selector correction above are
unchanged.

#### Preview existing room creation

```bash
lox config room add file.Loxone "Grüße 室" --dry-run --json
lox config room add file.Loxone "New room" --dry-run -o csv --save-as output.Loxone
```

Only the already supported `config room add` gains this admission. Its unchanged
grammar is `FILE NAME [--save-as FILE]`, with no parent, type or UUID override.
One private immutable creation plan feeds preview and the unchanged
`add_room(name) -> Result<String>` writer API. No DOM clone, trial mutation,
XML serialization or generic mutation framework is used to preview.

`parent` describes the actual existing XML root, `insert_after` describes the
last direct `C` child whose exact `Type` is `Place` or `PlaceCaption` (or `null`
when appending without an anchor), and `room` describes the planned `C` element.
Each includes its actual parsed child-index `path`, `element_name` and raw
attributes; missing keys remain absent, distinct from empty strings. The room's
path is its insertion position. A nested caption is not used as a parent or
anchor; an unusual root is neither rejected nor repaired.

The existing defaults and attribute order remain `Type="Place"`, `V="175"`,
generated `U`, exact requested `Title`, `WF="16384"`, with no children. The legacy
UUID format and random generator are unchanged. `uuid_ephemeral: true` and
`scope` explicitly state that this UUID belongs only to this invocation; preview
does not reserve it or promise reuse by a later execution. `report` contains the
existing writer report for this plan.

Duplicate detection remains the existing recursive, lowercased substring match
of `C Type="Place"` records having both `U` and `Title` (including empty `U`).
Missing metadata, case, Unicode, whitespace and empty names keep their prior
admission behavior. There is no trimming, Unicode normalization, exact-only
duplicate rule, new room policy or allocation change.

JSON includes `dry_run: true` and optional `trace_id`. Table/CSV use JSON-valued
cells; `--no-header` suppresses headings and `--quiet` suppresses only the table
summary. Preview returns before mutation, serialization/defaults, Document stamp,
destination inspection, save or directory/cache/network/telemetry writes, on
success or failure and with existing, same-input or missing-parent `--save-as`.
Parser, input-read, XML-parse and duplicate errors retain their existing order.
Normal execution still prints its legacy text even with JSON/CSV/quiet, reports
before saving, overwrites as before, and keeps BOM, line endings and stamp
behavior. Unrelated unaudited commands remain fail-closed.

Source-only raw existing CLI provenance: `tests/ux/room-add-preview-source.json`.
No native UI, room-family, whole-file, SPS or physical-effect claim is made.
Historical room-rename/selector/control-move artifacts and behavior are unchanged.

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
