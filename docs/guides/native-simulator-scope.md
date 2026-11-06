# Native simulator: reviewed function scope and refinement

This is **function identification**, not a recovered simulator, ABI, behavioral
contract, or engine-coverage percentage. The public files contain independent
tooling and synthetic fixtures only. Bodies, dossiers, CFGs and review evidence
stay ignored under `.reverse-engineering\native-simulator\scope`.

## Version and evidence

Reviewed Config **17.2.10.21 AMD64**, executable SHA-256:
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
Frozen analysis ID:
`6a0520329af17d7b9fdaafe69398c62c454d9da838fc9f9c8d0bdd8d17bb15ae`.
Preferred image base is `0x140000000`; RVAs subtract that base.

The successful post-attach capture identifies 1,553 executed **whole-executable**
functions, not 1,553 simulator functions. The shorter failed-probe capture is a
comparison lead, not a behavioral pass or proof of permanent deadlock. Neither
capture covers application startup or SimulationStart. Neither records call
edges, object identities, per-block timestamps or invocation counts.

The private first review (preserved unchanged) is:

```text
.reverse-engineering\native-simulator\scope\review-17.2.10.21.json
.reverse-engineering\native-simulator\scope\17.2.10.21-first-reviewed\ledger.json
```

The ledger binds the review, executable, canonical index/database/bodies, source
trace summaries/maps, exporter and selected CFGs by hashes. It preserves exact
inclusive ranges, RVAs, observation counts, role classification, uncertainties,
next probes, and evidence limitations. Its 25 entries include rejected
hypotheses; **25 is not a simulator-function denominator**.

## First-review actionable identities

All roles below follow canonical-body review, not just string matching.
“Confirmed” is restricted to the stated narrow static fact, not native-case
equivalence. Inferred roles still need dynamic object/call-edge attribution.

| Address | Status | Bounded finding |
|---|---|---|
| `0x140371ef0` | Inferred | Timer-worker adapter directly calls cycle dispatcher under running/unpaused gating. |
| `0x14015ef30` | Inferred | Mixed document/simulation cycle dispatcher; produces frame context and invokes control virtual slots. |
| `0x14050a840` | Inferred | Due-event selector used repeatedly by dispatcher. |
| `0x1404cb7a0` | Inferred | Tagged deadline enqueue/update; related near/far queues. |
| `0x1404b6de0` | Confirmed static reader | Offset-adjusted wall-clock path used by dispatcher and block timing, **not proven monotonic**. |
| `0x14015de80` | Candidate | Start/resume flags, date-offset setup and timer arming; absent from post-start traces. |
| `0x14015e900` | Candidate | Timer stop and date-offset clearing; not proof of full block-state reset. |
| `0x140614600` | Inferred | **EnergyManager2 virtual cycle handler**, tied to pinned PE class RTTI and executed in both traces. |
| `0x140616a10` | Inferred | EnergyManager2 day-marker state-reset helper. |
| `0x140616d00` | Inferred | EnergyManager2 Off-held gate and edge-reset helper; edge path not established by observed presence. |
| `0x14061b360` | Inferred | EnergyManager2 per-load output updater. |
| `0x14061ba80` | Inferred | EnergyManager2 deadline/countdown update helper. |

Independent static MSVC x64 RTTI links the primary
`CControlObjectEnergyManager2` vtable at `0x14102dd10` to handler
`0x140614600` through slot `+0x50`. Its constructor anchor is `0x140613dd0`.
Byte decoding also identifies dispatcher indirect callsites `0x14015fb80` and
`0x14015fd49` using that slot. These relationships are actionable probes,
**not a captured caller-to-callee edge or fixture UUID-to-this-pointer binding**.
The handler has 108 contained BB records / 70 distinct entries in the successful
capture and 37 / 33 in the shorter capture. These are not hit counts or coverage.

Important rejected leads:

- `0x140996ec0` / `0x140996f20`: Qt SimulationSettings metacast stubs.
- `0x140bae420` / `0x140bae450`: Qt action/widget construction, not engine roots.
- `0x14085e850`: executed Energy-related strings, but RTTI identifies
  **LoadManager**, not EnergyManager2.
- `0x1405dab30`: executed **EnergyFlowMonitor** handler, not EnergyManager2.
- The separate **Energy** class has primary vtable `0x140f200c0`, slot `+0x50`
  handler `0x1407b6ca0`; it was not observed in the successful capture. Do not
  confuse an EnergyManager2 fixture with the Energy class.

### Version-bound object UUID layout

For this exact executable, canonical construction and independently decoded
formatter instructions establish a 16-byte control ID at primary object offset
`0x8c`. The formatter at `0x1404f23e0` reads a little-endian u32 at `+0x8c`,
little-endian u16 values at `+0x90` and `+0x92`, then eight bytes in ascending
address order at `+0x94..+0x9b`. Its display is the native `8-4-4-16` form.
This is recovered storage/formatting evidence, not an assumed RFC UUID layout.

The EnergyManager2 constructor at `0x140613dd0` calls the common constructor
at `0x140422e00` before installing its primary vtable. A 256-byte bounded
entry sample therefore includes the reviewed ID field without following
arbitrary pointers or scanning process memory.

Static evidence is retained privately under
`.reverse-engineering\native-simulator\instance-layout`. A live identity claim
still requires the exact handler entry, class/vtable evidence and decoded ID
matching the independently inspected fixture. Merely finding GUID-like bytes
near a pointer is insufficient. New executable versions must re-establish this
layout; the offsets are not a cross-version ABI.

For the same version, worker instructions load the document pointer from timer
object `+0x48` before the direct dispatcher call. Its return address is
`0x140372063`. The two reviewed dispatcher-to-handler calls return at
`0x14015fb83` and `0x14015fd4c`. These are precise checks for captured return
addresses, not reconstructed call-stack guesses.

The normal Energy handler receives the static frame at `0x1419ac930` through
RDX. Its u64 timestamp is at frame `+0x38`, corresponding to `0x1419ac968`.
This is normal-path clock milliseconds, not a simulator timestep. Minimum-floor,
accounting and countdown helpers separately read the clock, so their samples
need not equal this frame timestamp. Live pointer/value associations still
require an admitted recording; static offsets alone do not establish them.

### Interrupted native entry probe

The first v4 multi-site entry recording captured callback, worker and dispatcher
entries on one native thread. Callback R8 equalled worker RCX, their captured
return addresses matched, and both sampled the expected `CSpsTimer` vtable.
The timer's sampled `+0x48` document pointer equalled dispatcher RCX; its return
address matched `0x140372063`.

Config then exited before post-attach clock/output admission. A separate
Energy-only entry probe also exited, without observing its selected entry.
These preserved register associations are **interrupted-run diagnostics**,
not proof that the observer is transparent or that a native parity gate passed.
Energy UUID/frame binding and the cause of the early exits remain open.
Exact application-error records subsequently locate both vendor access
violations at dispatcher RVA `0x15ef8f`, a TLS-chain dereference. The entry-only
v5 alternative also fails a stronger late-attach console TLS control while
passing normal launch. The fault location is known; the underlying cause and
a transparent late-attach solution are not.

## Reproducible local queries

No target execution, GUI, Miniserver, hosted analysis or recurring work is used.
The query command opens the frozen FTS database read-only and prints identities,
not bodies. `--observed` checks executable/index/map hash binding first.

```powershell
python scripts\native-sim-scope.py query --fts 'CControlObjectEnergyManager2'
python scripts\native-sim-scope.py query --fts 'FUN_14015ef30'
python scripts\native-sim-scope.py query --fts '"0x6928"' `
  --observed .reverse-engineering\native-simulator\coverage\20261105T212805Z-69d0d1231264445b82b3d83ab54c0d0f\analysis.json

$latest = Get-Content .reverse-engineering\latest.json | ConvertFrom-Json
$pe = Join-Path $latest.version_directory 'source\LoxoneConfig.exe'
python scripts\native-sim-scope.py rtti --pe $pe --type CControlObjectEnergyManager2
```

RTTI recovery validates COL signature/self-RVA/type-descriptor linkage before
reporting vtable slot pointers. It does not instantiate a vendor object or claim
every secondary-table pointer is a valid handler.

The private review JSON contains `target_sha256`, `analysis_id`, `image_base`
and `functions`. Each function requires `address`, `roles`, `classification`
(`candidate`, `inferred`, `confirmed`, `rejected`), `facts` and `uncertainties`.
Optional `rtti_names`, `next_probes`, confidence notes and supporting artifact
hashes preserve manual review. Tooling does not assign roles automatically.

## Selected CFG export, with explicit omissions

REA already supports `analyze_function(procedure, detail=complete)` with local
CFG/basic blocks and `get_call_graph(address, direction=forward|backward)`.
The blocker here is **retained state**, not an absent feature: the frozen session
was closed, its recorded project lifetime was ephemeral, and no retained project
was located in the searched project/cache directories. Its original import took
about 4,170 seconds. No costly whole-program reimport was performed.

The optional fallback reads pinned PE bytes with **Capstone 5.0.6** and exports
genuine independent **entry-rooted, direct-control-flow** instruction graphs.
These are explicitly **not REA/Ghidra CFG exports**. Calls split local blocks;
call-return fallthrough is assumed. Indirect jumps, exception edges, unknown
no-return behavior, exact-boundary exits and undecoded bytes remain explicit.
No switch destinations are guessed. Decompiled lexical call mentions are
separately labelled and never substituted for a canonical call graph.

Install the optional missing dependency locally, if needed:

```powershell
python -m pip install --target .reverse-engineering\native-simulator\scope\dependencies capstone==5.0.6
python scripts\native-sim-scope.py export `
  --review .reverse-engineering\native-simulator\scope\review-17.2.10.21.json `
  --trace .reverse-engineering\native-simulator\coverage\20261105T212805Z-69d0d1231264445b82b3d83ab54c0d0f\analysis.json `
  --trace .reverse-engineering\native-simulator\coverage\20261105T211248Z-0e182d6a57dc4c1493f628ecd383e713\analysis.json `
  --output .reverse-engineering\native-simulator\scope\17.2.10.21-first-reviewed `
  --pe $pe --cfg --dependency-dir .reverse-engineering\native-simulator\scope\dependencies
```

Output outside ignored scope storage is rejected. Without `--cfg`, the core
query/review/export tooling needs only Python's standard library. The focused
tests use synthetic PE, RTTI, branch/call bytes, FTS and trace fixtures:

```powershell
python -m unittest discover -s scripts -p test_native_sim_scope.py -v
```

CFG tests skip explicitly if the optional dependency is unavailable. Existing
ledger files cannot be overwritten: choose a fresh output directory to reproduce
an export. Each new export retains a hashed copy of its exporter.

### First-review boundaries

- Dispatcher canonical decompilation is badly corrupted after unresolved switch
  `0x14015f991`. The independent graph preserves that unresolved jump and 422
  undecoded body bytes; observed records also expose a missed direct-graph
  region. Apparent bad-data/port-I/O pseudocode is **not semantic evidence**.
- Timer callback label `0x140371e10` is outside the frozen exact function ranges.
  Its ownership and callback-to-worker edge remain unresolved.
- Indirect calls include ordinary import/Qt boundaries as well as dispatch.
  Their counts do not measure simulator correctness.
- Genuine selected CFGs do **not** establish a complete static engine denominator.
  Function, basic-block and behavioral coverage percentages remain unknown.
- Next bounded probes: bind EnergyManager2 fixture UUID to live class/dispatch
  destination; sample independent Off/held/release channels; then investigate
  day/reset and deadline behavior. Clock contracts (`lox-a4s`) and a broader
  handler inventory (`lox-bd0`) remain separate work.

## Static refinement v2: switch and callback

The new review/export does **not** rewrite the first ledger:

```text
.reverse-engineering\native-simulator\scope\17.2.10.21-refinement\review-v2.1.json
.reverse-engineering\native-simulator\scope\17.2.10.21-refinement\export-v2.1\ledger.json
.reverse-engineering\native-simulator\scope\17.2.10.21-refinement\refinement-v2.1-diff.json
```

### Proven switch, not a guessed handler table

Pinned instructions establish an unsigned `type_id - 2` bound of 318 before
the jump at `0x14015f991`. The 319-byte selector table at `0x1401600d4` contains
only two selector values. They index two **zero-extended u32 RVAs** at
`0x1401600cc`:

| Selection | Inputs | Destination |
|---|---:|---|
| 0 | 81 bounded type IDs | `0x14015f993`: optional type-specific pre-processing |
| 1 | 238 bounded type IDs | `0x14015fb39`: common dispatch path |
| Out of range | IDs outside 2–320 | `0x14015fb39` |

Both destinations are executable, decode at valid instruction boundaries, and
have exactly the selected canonical owner. The exporter checks the complete
audited instruction/dataflow form, guard escape, known incoming direct edges,
table bounds, zero-extension, exact owners, and instruction-boundary conflicts.
Unsupported forms fail; invalid entries remain partial with unresolved edges.
No arbitrary review-supplied target list is accepted.

This switch selects **pre-processing**, not a block-handler address.
Computation still uses each object's virtual slot `+0x50`. That distinction
supports a narrow confirmed static dispatcher contract, not original-source or
behavioral equivalence. The refined dispatcher graph has 286 call-split blocks,
4,505 decoded canonical body bytes, no undecoded body bytes, and no remaining
unresolved local jump at this switch. Recovering pre-processing exposes one more
indirect virtual call: **42 indirect calls** remain, versus 41 in the partial
first graph. Those calls, exception
transfers, dynamic instance attribution, and engine denominator remain unknown.

### Unwind metadata and actual callback flow

The AMD64 exception directory contains 50,449 whole-executable
`RUNTIME_FUNCTION` records. These are unwind regions, **not automatically
logical functions or a simulator denominator**. Dispatcher unwind interval
`[0x14015ef30, 0x140160213)` includes its table data; the canonical code body ends
at `0x1401600c8` inclusive.

No unwind record covers callback `0x140371e10`. Its exact entry is nevertheless
anchored by the timer-arm callback-address load at `0x140115609`. Independently
decoded reachable bytes end at `0x140371e29`; the supplied decode window is not
promoted into function ownership.

The callback null/sentinel-checks its third argument and **indirectly
tail-dispatches virtual slot `+0x10`**. There is no direct callback-to-worker
call. Canonical construction, pinned RTTI and the `CSpsTimer` vtable
`0x14165e438` conditionally link that slot to worker `0x140371ef0`. The worker
has an actual direct call at `0x14037205e` to dispatcher `0x14015ef30`.
Its three unwind regions are explicitly connected by `CHAININFO`; no
nearest-start ownership is guessed.

Six of the successful capture's **34 unmapped records** lie in the callback
region, representing three distinct BB entries. The shorter capture also has
six records / three distinct entries there. Collector statuses stay unchanged:
this is observed noncanonical code, not six newly identified functions or
captured dynamic call edges.

### Narrow clock facts, not a monotonic-clock contract

Canonical date helpers and pinned constants establish normal-path local OLE
date **days**, a day-valued date offset, conversion through 24 × 3,600,
rounding to seconds, then × 1,000 plus separately sampled local milliseconds.
The result is milliseconds relative to local-date 2009-01-01. Start/resume and
stop stores to the same date-offset global are byte-verified.

Separate date/millisecond sampling is not an atomic snapshot. Monotonicity,
DST/UTC behavior, invalid/negative-date handling, acceleration and native reset
behavior remain unverified. Offset clearing is not full block-state reset.

Reproduce the refinement into a **fresh** output directory with the earlier
export command, replacing `--review` with the v2 review path and `--output`
with a new ignored scope directory. Optional function `switches` specify only
guard/site addresses; the tool proves destinations from bytes. Optional
`regions` retain explicitly bounded noncanonical callback evidence.
`rtti --slot 0x10` selects the timer's actual virtual slot.

The focused suite now has **29 synthetic tests**, including switch dataflow
rejection, incomplete targets, boundary/owner conflicts, unwind chains and
half-open bounds, callback tail-dispatch, unmapped-status preservation, byte
facts and immutable exports.

The remaining dynamic probe is precise: bind callback user pointer/vtable and
worker document identity, then bind the EnergyManager2 fixture UUID/this/vtable
to the observed `+0x50` handler destination and frame clock. Static evidence
alone does not close that edge.

## Primary-vtable handler candidate catalog

The version-bound private catalog is:

```text
.reverse-engineering\native-simulator\scope\17.2.10.21-primary-handler-candidates\catalog.json
```

This is a foundation for `lox-bd0`, **not** a supported-block/type-ID inventory
or a list of recovered block algorithms. It scans self-RVA-validated AMD64 COLs,
decodes their class hierarchies/base descriptors, and requires exact
`CControlObject` type-descriptor ancestry. Class-name prefixes do not establish
lineage. The base class itself is reported separately from derived classes.

Only COL offset-zero primary tables are considered for dispatcher slot `+0x50`.
A validated candidate additionally requires the unique nonvirtual
`CControlObject` subobject at offset zero, normal readonly table storage,
executable first/selected pointers, and a unique exact canonical entry target.
Interior targets, range gaps/overlaps, unavailable metadata and incompatible
layouts remain unknown/invalid rather than acquiring nearest-start ownership.
Secondary COLs are retained separately without interpreting their `+0x50`
slots as primary dispatcher handlers.

For the pinned executable:

| Measurement | Count |
|---|---:|
| COL-backed derived classes with verified lineage | 483 |
| Validated primary-table candidates | 482 |
| Deduplicated validated handler addresses, all exact canonical entries | 284 |
| Raw primary-table+0x50 values with exact owners, including rejected layout | 285 |
| Validated handler addresses observed in successful / shorter capture | 15 / 9 |
| Shared validated handler addresses | 23 |
| Unknown/incompatible primary candidate | 1 |
| Secondary COL observations preserved | 393 |
| Unknown-lineage COL observations | 0 |

The primary-layout exception is `CControlObjectPresenceDetection`:
its `CControlObject` subobject is at **+88**, not zero. Its primary slot is
therefore not promoted into the dispatcher-handler catalog; its secondary
tables are retained. This is a layout exclusion, **not** proof the class is
unsupported.

The largest shared slot target is `0x140009180`, referenced by 116 derived
classes. Its canonical body is a return-only stub. Counting that address as
116 different algorithms—or interpreting its observed execution as 116
observed class instances—would be incorrect. Equal slot addresses in verified
ancestor classes are recorded as sharing evidence; source inheritance versus
identical code folding is not inferred.

Every class keeps XML type name, XML type ID and simulator support **unknown**.
Handler observation means mapped function-body/entry-byte presence, not a
captured class instance or dispatch edge. The successful trace still has
**34 unmapped records**; callback observations and all original maps/ledgers
remain unchanged.

The catalog command requires only Python's standard library:

```powershell
python scripts\native-sim-scope.py catalog --pe $pe `
  --trace .reverse-engineering\native-simulator\coverage\20261105T212805Z-69d0d1231264445b82b3d83ab54c0d0f\analysis.json `
  --trace .reverse-engineering\native-simulator\coverage\20261105T211248Z-0e182d6a57dc4c1493f628ecd383e713\analysis.json `
  --output .reverse-engineering\native-simulator\scope\17.2.10.21-primary-handler-candidates
```

Use a fresh output directory for reproduction; existing catalogs are immutable.
The catalog binds the executable, canonical index, traces/maps and retained
exporter by hashes. Five additional synthetic cases cover lineage versus
lookalike names, secondary/nonprimary/invalid targets, shared observations,
immutable version binding, and exact range gaps/overlaps: **34 tests total**.
