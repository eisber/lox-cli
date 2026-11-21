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

The original reviewed snapshot proves 159/522 catalog identities (30.46%): 76 exact and
83 aliases, with 24 candidates and 339 unresolved records. Thus 363 records
still lack proven identity. It joins 140/483 native classes to catalog records,
leaving 343 without that proof. Its 317 factory/XML associations are a separate
population, including configuration/project objects; none of these numbers is
simulator behavior coverage or overall completion.

The later Access-only extension proves **160/522 catalog identities (30.65%)**:
77 exact, 83 aliases, 23 candidates and 339 unresolved, leaving **362 unproven**.
It links **141/483 native classes**, leaving **342 unlinked**. Its **318**
factory/XML associations retain all 317 original associations and add only
CT454 `Access` (`CControlObjectAccess`, catalog row 396). The broader three-chain
proposal's Category and Place associations are not admitted; its proposed
320-association count is not this reviewed snapshot's count.

Use the existing optional flag with the separate Access-only report:

```powershell
python scripts\element-native-map.py `
  --inventory .reverse-engineering\ux\block-ux-coverage.json `
  --native-catalog .reverse-engineering\native-simulator\scope\17.2.10.21-primary-handler-candidates\catalog.json `
  --factory-proof .reverse-engineering\element-atlas\factory-access-only-admission-20261107T1442Z\report.json `
  --output .reverse-engineering\element-atlas\factory-access-only-admission-20261107T1442Z\native-map-reviewed.json
```

The admitted report SHA-256 is
`9771b8a288f333d9e3c93f77e0af7e540de895387b90bcdc169d19436309ce97`.
It derives only the independently endorsed Access association from supplement
`35e97bf952e8b6ff2977b22dfac7dad1b8f44c40b31d01a7d56a471ac1508bff`;
independent sign-off is
`672ca55af3cc8e4e3c31b5584797b12763ace8aabd0677e58e04f7efc11ea7d9`.
Parent replay passed 123 static checks and 121 proof byte-span assertions.
The normal-return identity assumes the documented Win64 preservation and
ordinary fresh-allocation contracts, supported by a pinned static runtime
witness, not a claim about a loaded DLL. There is no Access UX-rule, native
simulation or behavioral-parity admission. Historical reports remain unchanged;
three producer snapshots are digest-addressed beside the extension.

### Reviewed three-candidate extension

The next independently admitted cohort adds exactly these static identities:

| Catalog row | ControlType | Existing XML type | Native class |
|---|---|---|---|
| 359 | 412 | `2Point` | `CControlObject2Point` |
| 361 | 414 | `3Point` | `CControlObject3Point` |
| 452 | 505 | `EdgeDetection` | `CControlObjectEdgeDetection` |

The separate merged snapshot proves **163/522 catalog identities (31.23%)**:
80 exact, 83 aliases, 20 candidates and 339 unresolved, leaving **359 unproven**.
It links **144/483 native classes (29.81%)**, leaving **339 unlinked**. Its
**321 factory/XML associations** retain every previous association and add only
these three. The original 159 and Access-only 160 snapshots remain unchanged;
Category and Place are still not admitted.

The proof connects each selected numeric factory case to its actual allocated
root, final primary vtable and offset-zero COL, then the same table's `+0xf18`
descriptor-literal assignment. The old `+0x50` candidates and matching class
names are not identity getters or sufficient proof. The claim is conditional
on the independently reviewed ordinary Win64 normal-return, successful fresh
allocation and synchronous object-model assumptions for these exact chains.
It does not prove a loaded MFC DLL, allocation success, bytes surviving later
callbacks, persistent registry contents, serialization, UX or native behavior.

Independent admission SHA-256:
`9f97912870f92313585960afdcab77488def098e7e92016ea0a68ed0f619739f`.
The original proposal SHA-256 is
`65c1940f1c7d854542e72bc17ad25de334dbe710d213e858aee93ec88b46831a`.
The separately merged opt-in report SHA-256 is
`28e47977b739ec151f67fdccae4c9a012d4adfe1da59395704a592f9779825e8`.
Parent replay of the independent raw-PE checker passed 66 checks, including
51 flagged negative cases. No prior proof checker was used as its semantic
oracle.

Use the existing opt-in flag with the new separate report and output:

```powershell
python scripts\element-native-map.py `
  --inventory .reverse-engineering\ux\block-ux-coverage.json `
  --native-catalog .reverse-engineering\native-simulator\scope\17.2.10.21-primary-handler-candidates\catalog.json `
  --factory-proof .reverse-engineering\element-atlas\factory-three-reviewed-admission-20261108T0003Z\report.json `
  --output .reverse-engineering\element-atlas\factory-three-reviewed-admission-20261108T0003Z\native-map.json
```

Default four-proof extraction is unchanged; the general recognizer is not
broadened to admit other candidates. Generate the atlas in another new output
directory, never over the historical reports. These identity counts do not
change the 27-layout/134-method metadata assessment or establish an overall
behavior percentage. Revalidate each factory, object and literal witness
against the next release's image instead of inheriting these addresses.

### Reviewed BinDecoder extension

A further independently admitted relation adds only catalog row 301, CT339
`BinDecoder`, to `CControlObjectBinDecoder`. The separate merged snapshot
proves **164/522 catalog identities (31.42%)**: 81 exact, 83 aliases,
19 candidates and 339 unresolved, leaving **358 unproven**. It links
**145/483 native classes (30.02%)**, leaving **338 unlinked**. Its **322**
factory/XML associations retain all 321 previous associations. The original
159, Access-only 160 and three-candidate 163 snapshots remain unchanged.

The proof follows the actual allocated root and final primary vtable/COL to
the registry's `+0xf18` dispatch. Independent control-flow and register
dataflow checks establish that all selected normal returns reach the explicit
descriptor-string assignment after 31 loop iterations. The `+0x50` candidate
does not establish getter semantics. Ordinary Win64 normal-return, successful
fresh disjoint live allocations, ordinary CRT contracts and synchronous
object-model conditions remain explicit; register preservation is not proof
that pointed memory is unchanged. This admits an emitted assignment and
same-pointer/CT static registration-call relation, not literal persistence
across later callbacks, a persistent registry, loaded DLL binding, runtime,
serialization, UX or native behavior.

Independent admission SHA-256:
`fe8e026af388d6714012fb9624af980eba14723f75afda30350483ecdce6ef94`.
The original proposal SHA-256 is
`b89c63cead3f28ca0322213c0a6017b90d4d1f348755414a10aad4ed3b60ec72`.
Parent replay of the independent checker passed 50 total checks:
20 positive and 30 negative cases. Its own PE mapper and dataflow checker did not use the
author's analyzer as a semantic oracle.
The separately merged opt-in report SHA-256 is
`efc22928eb4d234740642ed014f5a25f5f26a36f569e380e4cfc24b830fe7d73`.

```powershell
python scripts\element-native-map.py `
  --inventory .reverse-engineering\ux\block-ux-coverage.json `
  --native-catalog .reverse-engineering\native-simulator\scope\17.2.10.21-primary-handler-candidates\catalog.json `
  --factory-proof .reverse-engineering\element-atlas\factory-bindecoder-reviewed-admission-20261108T0028Z\report.json `
  --output .reverse-engineering\element-atlas\factory-bindecoder-reviewed-admission-20261108T0028Z\native-map.json
```

Power, PulseBy, LightControllerH, DaylightController and ShadeRoof remain
candidates: their selected post-primary-store call effects are not closed.
Local matching literal sites do not discharge that missing root-memory
relation. No other candidate, contract, Qt metadata or behavior count changes.
Keep this extraction opt-in and re-establish the complete chain for a new
release rather than transferring this image's verdict.

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

Separately reviewed Qt header cohorts now cover 27 layouts and 134 class-local
metadata methods, with 618/645 layouts still unvalidated; the original frozen
snapshot and earlier 7/21 and 15/93 snapshots are unchanged.
See [the reviewed dialog cohort](../analysis/advanced-block-ux.md#reviewed-twelve-dialog-extension).
These metadata counts do not add catalog identities, GUI bindings or runtime passes.

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

## Current grid-power-offset CLI/unit evidence

A separate current CLI execution at source commit
`4c608d649a941050e3eb03fa50a125b266be6ae3` verifies only
`offset-grid-power.implementation` and `offset-grid-power.unit_test`.
The existing typed O implementation was not replaced and no command was added.
Thirteen isolated local-file CLI operations and four exact named tests cover
signed/zero persistence, parameter direction, invalid/nonzero-underflow
rejection, unrelated XML preservation, dry-run nonmutation and parser-wire
retention. All 50 recorded assertions passed.

The narrower collector/selector procedure was explicitly accepted only because
all existing O conditions were retained. Neither the monolithic CLI runner nor
its broad test selector was executed or claimed. Independent semantic review
reconstructed the logs, fixtures and binary bindings rather than trusting
the validator's free-text observations: 41 read-only checks, nine semantic
counterexamples and a strict digest negative. Review SHA-256:
`6bb916943aa6c783df1b894bdff594ecc04807304def027f6c1ee815ec31dbae`.
This is cooperative-toolchain evidence, not immutable OS execution attestation.

Use the exact unchanged contract copy and new result as additional arguments
to the catalog command, with a new output directory:

```powershell
--contract .reverse-engineering\ux\energy-manager2\offset-current-run-20261108T0057Z\contract.json `
--results .reverse-engineering\ux\energy-manager2\offset-current-run-20261108T0057Z\results.json `
--output .reverse-engineering\element-atlas\report-reviewed-offset-current-20261108T0112Z
```

The contract SHA-256 is
`74bd4f5931c73a2bfbcfaa15e2b7e16e92cfcce9b9b80fb756748fc8d37c7b29`;
the result SHA-256 is
`2166de52b0d3ce68bd440198337efffbef7584f4b7df78f44d271e8982d1f8b3`.
The contract copy preserves the existing Git-blob bytes. A CRLF Windows checkout
has a different digest and correctly fails with an older result pinned to the
LF bytes. Do not normalize the verifier, substitute a digest, or relabel that
historical execution as current.

The three original contract reports and their 37 capability instances remain
unchanged. Adding the separate 19-instance CLI snapshot produces **56 declared
capability instances**, including overlaps: implementation **1/56**, unit test
**1/56**, GUI oracle **0/56**, complete **0/56**. These are not 56 unique controls
or an overall completion percentage. Only EnergyManager2's evidence snapshots
change; all 522 catalog records and 483 native records remain unchanged.
The new CLI snapshot has two verified observations and **51 missing cases**,
including both GUI cases; its `complete` gate remains false. Parser evidence
does not establish simulator runtime, and no new native behavior pass is added.

The packet's `collect.py --verify` rehashes the frozen inputs and replays the
unchanged verifier without executing CLI/tests or overwriting evidence.
Fresh execution needs a new owned output, current source/build/fixture and
contract pins, a new run/record and every required O assertion/test. Preserve
the original GUI requirements and re-establish native binding evidence for a
new Config release rather than inheriting this image's verdict.

## Reviewed evidence outside atlas stages

Later assessments for the pinned 17.2.10.21 executable are retained
separately from the atlas's declared capability snapshots:

| Assessment | Established | Not established |
|---|---|---|
| Seven dispatcher exceptions (`lox-fck`) | 7/7 are distinct leaf tail-dispatcher code entries, with bounded control-flow and Qt-pointer evidence | Why the analyzer omitted them, their dynamic virtual destinations, widget behavior or additional verified controls |
| Bulk factory/XML proof (`lox-td6`) | Reviewed allocation, constructor, primary-vtable and XML-registration chains admit 159 catalog records | The remaining identities, simulator callback roles or behavioral/CLI parity |
| Access-only extension (`lox-2to`) | Independent instruction-anchor and bounded ABI review adds only CT454 Access, reaching 160 catalog identities and 141 linked classes | Category/Place admission, unconditional allocator behavior, Access UX rules or native/parity coverage |
| Repaired Lighting fixture (`lox-8xh`) | A fresh official process reopened the first native save and resaved it, preserving all 882 baseline identities, 75 Lighting connectors, four moods and the authored Program | Whole-project no-op, nested editing, simulator behavior or full Lighting-family parity |
| Initial official Lighting simulation (`lox-4tk`) | One disconnected native run started, progressed and stopped; three channel displays and their wired digital observers were bound and captured | ON/OFF input behavior, current mood, packed numeric values, repeated/recorded runs or Rust parity |
| Explicit digital-source fixture (`lox-u6b`) | A separate variant reused two public digital sources and passed native save/reopen/resave, preserving 882 baseline identities and adding ten | Public input delivery, ON/Reset behavior or parity; the first failed variant remains a failure |
| Visible digital-source switch (`lox-45q`) | One disconnected run delivered `On` source and independent controller input `Aus -> Ein -> Aus` through two complete clicks on the visible internal switch; three outputs turned on and remained on after `On` fell | Explicit Reset/all-off, raw packed values, current mood, timing, repeatability or Rust parity |

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
That initial run did not verify public input delivery or expected behavior.

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

The later bounded input-delivery report is
`.reverse-engineering\native-simulator\workstreams\st2-glyph-native-probe-20261107\followup-20261107T0958Z\portable-st2-glyph-selection-followup-result.json`
(SHA-256 `f945a5e86fc4811fdd55d93c2b5a1c468c3ea9908dbc8e03b4f78b08ad2a6429`).
Parent review verified all 103 manifest artifacts and the unchanged admitted
fixture. A separate earlier zero-runtime-click visibility blocker and its 95
artifacts remain unchanged; it is not retroactively a successful test.

Ordinary source-name/body selection made the small simulation switch immediately
left of the `VI` capsule visibly distinguishable against the selected background.
Selection was navigation, not input delivery, and does not prove a selection gate
or bitmap-cache cause. Each subsequent complete left click targeted a freshly
visible, physically owned switch, not an inferred rectangle or historical pixels.
This exercised the statically recovered `IoData St=2` public switch route; the
native observation does not isolate mouse-down from mouse-release timing.

Independent source and controller `On` readbacks changed `Aus -> Ein -> Aus`.
Channel displays changed from `WW(0%)` to `WW(29%)`, `WW(31%)` and `WW(33%)`;
their three digital observers changed from `Aus` to `Ein`. These outputs remained
on at the sampled `On`-low readback, so lowering `On` is not an all-off result.
Explicit Reset was not attempted because its own switch was not visibly bound.
Raw packed values, current mood, timing, repeated runs and Rust parity remain
unverified. These are bounded observations on this exact fixture, not a general
native-input selector or whole-family behavioral proof.

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
