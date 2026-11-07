# Versioned element atlas

The atlas is a generated, read-only view over the existing UX inventory, native
class census and evidence contracts. It does not add a command per control,
execute Config, connect to a Miniserver, or establish new GUI/runtime passes.
Keep generated reports and all vendor inputs under ignored
`.reverse-engineering\`; do not commit them.

## Generate a catalog report

Run from the repository root in PowerShell:

```powershell
python scripts\element-atlas.py `
  --inventory .reverse-engineering\ux\block-ux-coverage.json `
  --application-version 17.2.10.21 `
  --contract contracts\energy-manager2.json `
  --contract contracts\shading-recovery.json `
  --contract contracts\hvac-recovery.json `
  --reference contracts\light-controller2.json `
  --reference tests\ux\hvac\schema-integration.json `
  --connector-map docs\schemas\connector-map.json `
  --output .reverse-engineering\element-atlas\report
```

Open `.reverse-engineering\element-atlas\report\index.html` locally. Its filter
searches the catalog, native-class and shared-mechanism tables without a server
or external resources. `atlas.json` contains the complete linked evidence and
exception queues; `catalog.csv` and `native-classes.csv` provide flat exports.
Repeated runs with identical input bytes produce identical output bytes.

Optional `--native-map` and `--ux-map` add version-matched extraction sidecars.
Every sidecar must pin the exact inventory bytes and its other source files.
The generator checks those digests, executable/version identity, record
membership and unique identities before writing the report.

For the current frozen corpus, generate the native sidecar with:

```powershell
python scripts\element-native-map.py `
  --inventory .reverse-engineering\ux\block-ux-coverage.json `
  --native-catalog .reverse-engineering\native-simulator\scope\17.2.10.21-primary-handler-candidates\catalog.json `
  --output .reverse-engineering\element-atlas\native-map.json
```

Then add `--native-map .reverse-engineering\element-atlas\native-map.json` to
the atlas command. Its four reviewed factory identities are version-bound;
other stripped-class-name matches remain candidates. The native table retains
all classes, with primary vtable and `+0x50` candidate addresses, even when no
catalog association exists.

To opt into the later reviewed bulk factory proof, generate a separate sidecar:

```powershell
python scripts\element-native-map.py `
  --inventory .reverse-engineering\ux\block-ux-coverage.json `
  --native-catalog .reverse-engineering\native-simulator\scope\17.2.10.21-primary-handler-candidates\catalog.json `
  --factory-proof .reverse-engineering\element-atlas\factory-identity\report.json `
  --output .reverse-engineering\element-atlas\factory-identity\native-map-reviewed.json
```

Use that output as the atlas's `--native-map`, with a separate report directory
such as `.reverse-engineering\element-atlas\report-reviewed-factory-20261107`.
Without `--factory-proof`, the original four-proof behavior is unchanged.
Only explicitly reviewed report digests are admitted; live inventory, native
catalog, executable and evidence sources must match. Two historical producer
inputs are retained beside this report under `sources\<SHA256>`, not silently
substituted with later producer bytes.

The reviewed snapshot proves 159/522 catalog identities (30.46%): 76 exact and
83 aliases, with 24 candidates and 339 unresolved records. Thus 363 records
still lack proven identity. It joins 140/483 native classes to catalog records,
leaving 343 without that proof. Its 317 factory/XML associations are a separate
population, including configuration/project objects; none of these numbers is
simulator behavior coverage or overall completion.

Generate shared UX mechanism leads with:

```powershell
python scripts\element-ux-map.py `
  --inventory .reverse-engineering\ux\block-ux-coverage.json `
  --output .reverse-engineering\element-atlas\ux-mechanisms.json
```

Add `--ux-map .reverse-engineering\element-atlas\ux-mechanisms.json` to the
atlas command. Groups describe metadata membership and static leads, not verified
widget bindings, rule enforcement, or runtime coverage. Executable pointers with
no recognized canonical function remain explicit unresolved leads. Their
existence does not prove a decompilation export failure. Missing required inputs
or contradictory source identities remain extraction errors.

For the frozen snapshot, 205/522 records have candidate or metadata-based UX
mechanism leads. The extractor's narrower 177-record candidate population
excludes 28 metadata-only associations. Its 692 parameter entries are all
catalog parameters belonging to those 177 records, not recovered GUI fields.
None of these populations measures verified UX coverage.

## Identity and evidence

`catalog_index` is the source record identity, not a numeric ControlType ID.
Each generated record ID combines the inventory snapshot digest and that index.
The current inventory has 522 records but only 487 distinct numeric IDs; all
duplicates remain separate. Changing snapshot bytes deliberately changes the
snapshot-scoped IDs. Cross-version equivalence needs separate evidence.

The 483 native classes are a different population. Preserve unjoined classes
and class-name candidates; a name match is not a factory/XML binding, a verified
editor, or proof of simulator behavior. Unknown record kinds remain
`not_classified`: rows without XML names include project/documentation entries.

Compatible `--contract` and optional `--results` files go through
`verify-ux-contracts.py` and its `enrich_inventory` identity join. Cases keep
their original required levels and observations; the atlas does not combine
historical runners into a new passing run. Contract reports with invalid
evidence remain explicitly invalid, and the command exits 2. Invalid primary
inputs or sidecars fail before report generation.

Several contract snapshots for the same block can be linked independently.
Each is joined with `enrich_inventory` separately; their cases and gates are
never unioned. Summary stage denominators count capability **instances across
snapshots**, including overlaps, not unique product capabilities. Supply the
matching `--results` explicitly to display existing executed evidence; omitted
results are missing, not implied failures or passes.

For example, the independently preserved Energy CLI subset can be added with:

```powershell
--contract tests\ux\energy-manager2\cli-contract.json `
--results .reverse-engineering\ux\energy-manager2\cli\cli-contract-results-20261105T090457-7fad3e6d.json
```

These are additional arguments to the atlas command, not a new execution or
proof that subsequent code changes were tested by that historical runner.

`--reference` accepts other version-pinned JSON snapshots, including the
inspection-oriented Lighting format. References are not verified execution.
Newer metadata can sit alongside an older assessment without rewriting it:
the HVAC integration reference and current connector map expose the repaired
56-key shape, while historical contract scope remains visible and GUI/runtime
parity stays unverified. A count match alone is not connector equivalence.

Original inventory CLI/UX observations are retained as historical snapshots.
Current connector metadata is separately digest-pinned repository data, not a
claim that every associated CLI operation or native behavior works.

## Reviewed evidence outside atlas stages

Five later assessments for the pinned 17.2.10.21 executable are retained
separately from the atlas's declared capability snapshots:

| Assessment | Established | Not established |
|---|---|---|
| Seven dispatcher exceptions (`lox-fck`) | 7/7 are distinct leaf tail-dispatcher code entries, with bounded control-flow and Qt-pointer evidence | Why the analyzer omitted them, their dynamic virtual destinations, widget behavior or additional verified controls |
| Bulk factory/XML proof (`lox-td6`) | Reviewed allocation, constructor, primary-vtable and XML-registration chains admit 159 catalog records | The remaining identities, simulator callback roles or behavioral/CLI parity |
| Repaired Lighting fixture (`lox-8xh`) | A fresh official process reopened the first native save and resaved it, preserving all 882 baseline identities, 75 Lighting connectors, four moods and the authored Program | Whole-project no-op, nested editing, simulator behavior or full Lighting-family parity |
| Initial official Lighting simulation (`lox-4tk`) | One disconnected native run started, progressed and stopped; three channel displays and their wired digital observers were bound and captured | ON/OFF input behavior, current mood, packed numeric values, repeated/recorded runs or Rust parity |
| Explicit digital-source fixture (`lox-u6b`) | A separate variant reused two public digital sources and passed native save/reopen/resave, preserving 882 baseline identities and adding ten | Public input delivery, ON/Reset behavior or parity; the first failed variant remains a failure |

The dispatcher report is
`.reverse-engineering\element-atlas\dispatcher-classification\report.json`
(SHA-256 `5e76b9a65e07498f1d0bbe754c063f188e120afb10f8acebd9b3fe4ac1a28327`).
The seven entries remain absent from the canonical census; classifying their
entry kind does not supply missing canonical body inputs or change the atlas's
raw census diagnostics. Virtual destinations remain a separate recovery task.

The reviewed factory report is
`.reverse-engineering\element-atlas\factory-identity\report.json`
(SHA-256 `c6f1ec984727cfe373a54cf0efbb2079a886e988e7ba3928398a2ebbd2d32d7b`).
Catalog admission requires both ControlType and the existing explicit XML name
to match; 164 catalog rows with other factory leads remain unadmitted.
The 148 rejected/unknown factory cases are preserved rather than filled from
names or the unrelated `+0x370` descriptor slot.

Lighting evidence is retained under
`.reverse-engineering\native-simulator\workstreams\lighting-envelope-recovery\visual-continuation-20261107\portable-lighting-roundtrip-evidence.json`
(SHA-256 `47a711c9f7c8afcf6e1684c1108891c1b5211f0fdfb293bce40294abfd537979`).
The producer is pinned to commit `0c9f259`. Initial native import added
identities/connectors and normalized mood CIDs; those migrations are recorded,
not waived. The fresh native-baseline resave preserved the Lighting graph but
changed one permission `WF` from `6307840` to `16384`. That exact project delta
remains explicit. Earlier failed fixtures and refused attempts remain separate
historical counterexamples.

The initial simulator observation report is
`.reverse-engineering\native-simulator\workstreams\lighting-native-oracle-20261107\portable-lighting-native-observations-final.json`
(SHA-256 `c5d9cab0b616bd11aa4a1717623d4fcd2edd2d1991a4c73199923e0be6e9f588`).
The unchanged fixture's `AQ1`, `AQ2` and `AQ10` displayed `WW(0%)`; their
three digital observers displayed `Aus`, not packed numeric lighting values.
The visible clock progressed while running and remained unchanged across two
stopped samples; this does not recover internal `dt`. Control-preemption
boundaries remain explicit, with no continuous-behavior inference across them.
Public input delivery and expected behavior remain unverified.

The optional fixture producer mode `--harness digital-sources --baseline FILE`
uses the immutable native-resaved baseline rather than changing the default
fixture. Its reviewed readiness report is
`.reverse-engineering\native-simulator\workstreams\lighting-digital-source-fixture-20261107\portable-lighting-digital-source-fixture-result.json`
(SHA-256 `bc93afb94e32532327e1f5220e5fcef80b4f68e7d1c44d1b3d9ec5f21833c5e0`).
The accepted native file has SHA-256
`674905cda0c10ffff853e7090507f696747c8b0fb8f3149479b96cff924823ce`.
Its existing digital `VI4`/`VI5` sources reach separate `On` and `Reset` inputs
through native-preserved references. All 75 Lighting connectors, moods and
three observers remain intact; native normalizations are enumerated, not
treated as a whole-project no-op. The first representation lost its references
on native save and is retained as a failed counterexample. Regenerated files
must receive their own native validation rather than inherit this exact file's
readiness result.

These assessments do not automatically advance contract stages or the overall
completion percentage. Raw projects, bytes, disassembly and workstation
receipts remain ignored and private.

## Repeating after an update

First regenerate the version-matched raw corpus and UX inventory using the
existing reverse-engineering pipeline. Regenerate native/UX sidecars from those
new bytes, then run the atlas with the new application version and explicitly
reviewed contracts/references. Old executable identities and stale sidecars
are rejected, not silently carried forward. No recurring job is installed.

Report source-limited counts and remaining annotations independently:
XML-name annotations, parameter/XML-key mappings, class/record leads and
declared contract evidence. Overall UX/behavior completion stays `null` because
neither the catalog nor a handler-slot census closes the behavioral denominator.
Missing linked verification does not mean a control has never been exercised.
