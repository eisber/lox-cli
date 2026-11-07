# Command Reference

Loxone Config-as-Code CLI. All commands operate on local `.Loxone` XML config files — no live Miniserver connection required for editing, simulation, or validation.

Global flags: `--json` (shorthand for `--output json`), `-o`/`--output json|csv|table`, `-q`/`--quiet`, `--no-color` (also respects `NO_COLOR` env var), `--no-header`, `-v`/`--verbose` (request logging, `-vv` for request+response bodies), `--dry-run`, `--non-interactive` (fail instead of prompting; implied by `--json`), `--trace-id <ID>` (correlation ID for agent tracing), `--ctx <name>` (use a specific context)

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
lox config users file.Loxone                      # list user accounts
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
| `--mode digital\|analog` | Set `An` and native mode-customization marker `Anc` |
| `--preparation-seconds S` | `RT`, integer seconds 30..86400; default 60 |
| `--min-runtime-minutes M` | `MinRt`, integer minutes converted to signed 32-bit seconds; negatives are permitted |
| `--daily-minutes M` | `MDay`, integer minutes/day 0..1440 converted to seconds |
| `--deadline CHOICE` | `tpR`: `sunset`, `dusk`, `sunrise`, `dawn`, or hourly `01:00`..`24:00`; all 28 choices GUI-save observed |
| `--rated-power KW` | `MaxPwr`, finite kW >=0 |
| `--switch-on-power KW` | `SwPwr`, finite kW >=0; zero substitutes rated power |
| `--icon BUILTIN` | Exact case-sensitive canonical filename or native/RFC ID from `energy icons`; optional `builtin:` prefix |
| `--no-icon` | Remove only the matched Load's `ico`; mutually exclusive with `--icon` |

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
```

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
lox telemetry enable                       # opt in to anonymous usage analytics
lox telemetry disable                      # opt out
```

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
