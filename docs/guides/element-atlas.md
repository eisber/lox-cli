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
