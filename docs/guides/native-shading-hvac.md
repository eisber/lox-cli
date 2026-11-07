# Native shading/HVAC preparation

**Status: preparation only; every oracle case is PROPOSED and unexecuted.**
Neither family is complete. No simulator algorithm, central CLI registration,
schema, observer, hardware binding or production configuration was changed.
The full shading/HVAC parity tasks remain open.

## Version and evidence boundary

This preparation targets Config **17.2.10.21**, AMD64, image base
`0x140000000`, executable SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
The authoritative corpus is `.reverse-engineering\latest.json`.
Canonical bodies were read through `Corpus.body`, which verifies their indexed
hashes; the on-disk PE hash, COL self-RVA, primary object offset, base lineage
and exact slot pointers were independently checked with the existing
`scripts\native-sim-scope.py` helpers. No new Ghidra import or vendor ABI call
was made.

Private evidence:

* `.reverse-engineering\native-simulator\families\shading-hvac\source-ledger.json`
  — executable/index/body hashes, canonical body ranges, COL/pointer byte
  hashes, source joins and limitations.
* `...\fixture-generation.json` — exact actual CLI identity, commands,
  authored inputs, fixture hashes, UUID/channel roles and local inspections.

The public contracts and inventories point to the exact private ledger hash.
They contain independently authored summaries and value-free shapes, not
vendor pseudocode, resource XML, screenshots or runtime traces.
Read JSON as UTF-8 with optional BOM (`utf-8-sig`).

There are 53,088 exported bodies in this corpus. The existing catalog's
284 deduplicated primary handler candidates are **not** a supported-block
count, XML ID table or full simulator denominator. Engine completion is
always false for this work.

## Exact types and static handler joins

| Family | Verified XML type / CT | Native class | Primary COL | Primary vtable | Slot `+0x50` |
|---|---|---|---|---|---|
| Shading | `JalousieUpDown2` / 349 | `CControlObjectPush2Jalousie2` | `0x141675028` | `0x140f80470` | `0x1407fd870` |
| Room/HVAC | `HeatIRoomController2` / 469 | `CControlObjectHeatIRoomController2` | `0x14167dec0` | `0x141039f18` | `0x14064b110` |

The XML type IDs come from the current canonical repository XSD. The native
join is stronger than a name prefix: each validated primary table has
object offset zero and `CControlObject` at primary offset zero in the decoded
RTTI hierarchy. **The same table's `+0xf18` slot** points to registration
`0x1407ff450` for shading and `0x140658900` for HVAC. Those hash-verified
bodies explicitly register the respective XML type string and connectors.
The target `+0x50` pointers are executable exact canonical entries.

The primary `+0x40` methods are `0x140800700` and `0x140666bc0`,
respectively. The former delegates an embedded serializer; the latter names
room-controller attributes and a nested `Timer` serializer.

**Confidence:** confirmed static primary lineage, registration join and slot
pointer; candidate native dispatcher role; **unknown** actual fixture-bound
instance dispatch and behavior. Energy's fixture-bound class IDs, frames,
callback pointers and timing behavior are not transferred to these classes.
Shading's reviewed `+0x80`/`+0x88` methods mention clock helper
`0x1404b6de0`; that does not establish runtime clock ownership.

`Thermostat` appears in older room recipes but is not a verified XML alias
for this version. The HVAC target connector is **`Input`**, not `Setpoint`.
`Temp` is measured room temperature; `TempO` is outdoor information.

## Surface inventories and the HVAC blocker

* `tests\ux\shading\surface-inventory.json`: 9 inputs, 8 parameters,
  4 outputs, plus an explicitly unresolved embedded advanced-settings
  serializer. Core and native registration agree on the 21 keys/order.
* `tests\ux\hvac\surface-inventory.json`: native registration order for
  16 inputs, 20 parameters and 20 outputs; named advanced storage and
  unresolved delegated records.

**Verified HVAC schema blocker:** the current emitter and pinned CLI generate
52 connectors, while this native registration declares 56. Missing keys are
`RtD`, `AQh3`, `AQc3`, `AQhc3`. Parameter ordering and numbered-output ordering
also differ. The native registration creates three interleaved numbered
heat/cool/combined groups; extra keys in the core direction map are not
evidence of native support. This workstream does not edit the central map or
manufacture missing UUIDs. A GUI repair that regenerates connectors is not a
successful preservation round-trip.

### Narrow four-connector follow-up

`tests\ux\hvac\missing-connectors-proposal.json` adds an **unapplied**
structural proposal with exact private PE-window/body and resource hashes.
The four missing keys are statically supported:

| Key | Direction / core `t` | Native descriptor default | Physical unit |
|---|---|---|---|
| `RtD` | input / `I` | zero | unknown; absent from this block's pinned TechDoc IO rows |
| `AQh3` | output / `O` | zero | not declared in pinned resource; unknown |
| `AQc3` | output / `O` | zero | not declared in pinned resource; unknown |
| `AQhc3` | output / `O` | zero | not declared in pinned resource; unknown |

The same primary table's registration names `RtD` at `0x141003dd8`; its
non-index flag class matches the resource-documented `Reset`/`DisMv` inputs.
Machine bytes establish the formatted output loop's indices 1, 2 and 3.
Pinned `tdc_ENU.LxRes` independently places those patterns in an Output
group with `Combines=3` and zero defaults. Neither the group rows nor their
nested IO rows declare `Unit`/`UnitId`; a numeric range is not a physical unit.
The constructor stores the zero floating-point registration argument in the
descriptor field that its serializer explicitly calls `Default`.

The private proposal changes only this type's connector list/order and adds
`t.RtD=I`; the third-output `t` entries already equal `O`. It does **not** force
serialized `Co/@Def=0`, because omission rules are unverified. No full
`docs\templates.json` entry is proposed: the type is absent and unrelated
template attributes/defaults cannot be invented. No shared file was changed.
The initial seeds, protocols and private fixtures remain byte-identical; a
centrally corrected CLI needs a fresh, separately hashed disposable fixture.

The HVAC serializer names `UUIDTimer`, `AllS`, `AQDig`, `ComfHC`, `SourceCap`,
`Objects`, `PrioH`, `PrioC`, `Cap`, `DMtc`, `ExT`, `custRU`, `custRI`,
`srcName`, `DefFan`, `DefFanNP`, `DefAir`, and version-guarded `Ssh`.
`Timer` is nested; other embedded serializers remain unresolved.
Names/path anchors are known, but record encodings, source masks, omission,
defaults, version migration and hardware/plant requirements are not.

Unit labels are catalog evidence only. Timing parameters without a label,
the active temperature system, manual-position scaling and heating/cooling
duty units require native calibration. Conflicting output metadata is not
resolved by assuming either a percentage range or a voltage range.
`EndUp`/`EndDown` register `fullup`/`fulldown` command tokens: they are **not**
established physical end-stop sensor inputs.

## Minimal disposable fixtures

The two public seeds are independently authored document/room/page shells:

```powershell
lox config add --type JalousieUpDown2 --title "Prepared Shading" --room "Shading Lab" --page "Shading Lab" tests\ux\shading\seed.Loxone --save-as .reverse-engineering\native-simulator\families\shading-hvac\shading-minimum.Loxone --json
lox config add --type HeatIRoomController2 --title "Prepared HVAC" --room "HVAC Lab" --page "HVAC Lab" tests\ux\hvac\seed.Loxone --save-as .reverse-engineering\native-simulator\families\shading-hvac\hvac-minimum.Loxone --json
```

The actual executable used was `target\release\lox.exe`, SHA-256
`3912fc2e9d9537e4495638041095a159411b2114ebf54d319d18335a6abf14cf`.
It was not rebuilt or replaced. Its behavior is recorded separately from the
current source: it omitted corner coordinates; its layout command did not
supply them. Only these new private fixtures received independently chosen
`Px2`/`Py2` values through `config xml set-attr`. Every generated UUID and
connector role survived those edits.

Each fixture contains one controller, no circuit and no hardware. Inputs are
reserved for a future **official** simulator input path, which is not yet
established. No physical sensors, end stops, plant or capabilities were
fabricated. If native execution requires a circuit, use existing lab sensors
and record their genuine dependency closure; do not replace missing hardware
with pretend bindings. `tests\eval\fixture.Loxone` was hash-checked unchanged.

| Private fixture | SHA-256 | Core connectors | Local check |
|---|---|---:|---|
| `shading-minimum.Loxone` | `433478d55327235667c23903645e279fc520baf3e6db209cf2fb5186cb67d457` | 21 | 0 errors, 1 expected unwired-input warning |
| `hvac-minimum.Loxone` | `8e44a25612f371881a6f5b729d0c061fa0c62ca15ede9aa578d2475df8b49f7d` | 52 | 0 errors, 1 expected unwired-input warning |

Both local `describe`, `connectors`, `check`, `validate`, and `surfaces`
commands exited zero. `validate` reported zero warnings. These are **file-only
inspection results**, not native admission or behavior results: its generic
“all connectors are wired” diagnostic does not mean this intentionally
unwired block is executable. HVAC remains blocked by the independently
verified native shape discrepancy.

All authored parameter values and exact commands are in the family protocol
and private generation manifest. They are experiment inputs, not recovered
vendor defaults. UUID generation is nondeterministic: regenerating a fixture
requires new byte hashes, a new role ledger and new calibration.

## Three queued cases per family

The protocols are preparation records, **not directly runnable runner cases**.
`runner_ready` is false. No case carries a PASS result.

| Protocol | Case | First obligation |
|---|---|---|
| `tests\ux\shading\protocol.json` | `shading.admission-calibration` | Save/reopen, exact identities, timing/input/position calibration |
| | `shading.reversal-interlock` | Both directions visibly execute; no simultaneous activation; proposed reversal delay bounds |
| | `shading.stop-hold` | Stop deactivates direction outputs; proposed bounded/stable position |
| `tests\ux\hvac\protocol.json` | `hvac.admission-calibration` | Resolve native/core shape and establish mode/temperature/source prerequisites |
| | `hvac.target-feedback` | Distinguish explicit target from measured temperature through official target feedback |
| | `hvac.presence-demand` | Bounded heating trend in confirmed heat-only mode with declared presence/temperature inputs |

Each case declares setup, clock, input changes, independently chosen expected
bounds, a wall-time cap and limitations. Expectations are **hypotheses until
official native execution**, not simulator specifications. The final HVAC
pair changes both presence and temperature and explicitly does not isolate
presence parity. No full motor, lamella, safety, comfort, PI/PWM, schedule,
predictive or plant algorithm is reconstructed.

## GUI queue and calibration prerequisites

No window inspection, UIA, pixels, screenshots, input or GUI execution was
performed by this workstream. The coordinator owns the lease; shading/HVAC
has no GUI authorization in this phase.

Energy's **later correction** reports that the backend and application paths
exist; the SSH transport account does not establish the authorized GUI actor.
The earlier missing-backend/username-mismatch report is superseded, not an
established blocker. The coordinator must verify the actual admitted profile.
This workstream did not inspect the endpoint or attempt any repair.
Original worker protocol bytes/hashes remain in private evidence snapshots.
Public proposed protocols use environment-neutral private-profile requirements;
no host/account identity or an erroneous environment blocker is published.

Before granting any future lease, the coordinator must establish:

1. Coordinator-admitted isolated desktop identity, pinned Config PE, required existing tools and
   the frozen helper baseline
   `.reverse-engineering\native-simulator\workstreams\054ce95-oracle-baseline\manifest.json`.
   Bind the actual staged helper bytes per run; never retroactively label a run.
2. Exact private fixture byte hash, UUID/reference roles and native import
   without silent repairs. Resolve the HVAC central schema blocker first.
3. Official input/readback paths, genuine source/plant prerequisites and
   separately fixture-bound native progress. Missing hardware is blocked.
4. Official simulator clock, restart behavior, temperature/duty/position units
   and settling. No OS-clock or private-memory manipulation.
5. Adequate observation resolution. A coarse screenshot cannot establish a
   subsecond, all-cycle interlock; inability to obtain an approved observation
   path blocks that expectation.

Use the existing admitted ENTRY-only observer profile only as a read-only
reference; no observer changes or claimed Energy ABI portability are
authorized. Keep vendor code, XML, traces, screenshots and any calibration
captures private and ignored. Do not close full-family tasks from these
static ledgers or local CLI checks.
