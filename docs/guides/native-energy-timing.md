# Native EnergyManager2: bounded timing semantics

This is a **version-bound static contract with sparse exploratory native
observations**, not an implemented simulator, original source recovery, or a
new native behavioral pass. No vendor code, disassembly, project files or trace
excerpts are published here.

## Identity and evidence

Reviewed Config **17.2.10.21 AMD64**, executable SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`;
analysis ID
`6a0520329af17d7b9fdaafe69398c62c454d9da838fc9f9c8d0bdd8d17bb15ae`.
Addresses below are **RVAs**, with preferred image base `0x140000000`.

Canonical bodies were hash-checked against the frozen index, then relevant
instructions and data were independently read from the pinned PE using the
existing scope tooling and Capstone 5.0.6. Private evidence is retained under:

```text
.reverse-engineering\native-simulator\timing-semantics\17.2.10.21-static-v1
```

Its semantic review links the query receipts, canonical bodies, byte evidence,
selected direct-flow graphs, fixture inspection and proposed countercases by
hash. Direct graphs do not resolve virtual calls, prove live object identity,
or establish an engine denominator. See [scope](native-simulator-scope.md),
[metrics](native-simulator-metrics.md) and the
[native replay requirements](official-simulator.md).

## Fields and units

Let `L` denote one native load record. Constructor RVA `0x613dd0` establishes
12 records starting at object offset `0x1008`, with stride `0xf0`. Record order
is priority order; the separately stored output index preserves association.
This is an object-layout fact, **not an invocation recipe**.

| XML / state | Offset in `L` | Bounded meaning |
|---|---:|---|
| `OIdx` | `+0x70` | Zero-based input/output association, not priority |
| `MDay` | `+0x74` | Daily requested minimum, integer **seconds** |
| `MinRt` | `+0x78` | Minimum run floor, integer **seconds** |
| `SwPwr`, `MaxPwr` | `+0x80`, `+0x88` | Switch-on/rated power, double kW |
| `RT` | `+0x90` | **Preparation time**, integer **seconds** |
| `An` | `+0x94` | Digital versus analog load-output mode |
| `tpR` | `+0xa8` | Daily deadline selector |
| Current / previous allocation | `+0xb0`, `+0xb8` | Internal allocated power, not boolean pin values |
| Derived actual load power | `+0xc0` | Feedback-derived kW |
| Completed daily run total | `+0xc8` | Integer seconds |
| Current activation timestamp | `+0xd0` | Clock milliseconds; zero means no current episode |

Serializer RVA `0x61c8e0` binds the actual XML attribute strings to those
fields. The generic read path at RVA `0x4c6500` stores an integer directly:
there is no minutes-to-seconds conversion at this XML boundary. Its
decompiler's apparent floating-point temporary is misleading; the pinned
instruction stores `EAX`, not a converted float.

The separately GUI-verified [configuration contract](../../contracts/energy-manager2.json)
still applies: preparation defaults to **60 s**, its GUI/setter lower bound is
30 s, and GUI minimum-runtime minutes are converted to stored seconds.
The daily fixture explicitly overrides preparation to 30 s. Negative minimum
runtime has been accepted by the GUI; this review does not invent a
nonnegative validator or establish overflow/negative-time behavior.

## Preparation, minimum run and recalculation are different

**Confirmed static ordering:** activation helper RVA `0x614200` stores the
activation timestamp first, then extends a **shared block preparation
deadline** using that load's `RT` in seconds converted to milliseconds.
Existing episodes do not rearm that deadline on every allocation update.
Consequently this path does **not** delay the minimum-run timestamp until
preparation finishes. A preparation-plus-minimum-run explanation must not be
asserted from the late ON sample.

The timing predicates are distinct:

| Anchor RVA | Static finding |
|---|---|
| `0x618e10` | With a nonzero episode timestamp and nonzero `MinRt`, the release-eligibility predicate becomes true only when elapsed **whole seconds are strictly greater** than `MinRt`. |
| `0x6194b0` | An already-running digital load is retained while that minimum floor has not passed, before ordinary power-based stop decisions. |
| `0x618ff0` | Analog allocation also consults the minimum floor; its power adaptation and lower-priority interactions are separate branches. |
| `0x614838`–`0x6148aa` | Ordinary full allocation is deferred while either shared deadline is still future. Explicit force, priority-change and Recalc-edge conditions can bypass that deferral. |
| `0x61ba80` | Without a preparation deadline, an expired fallback is advanced by **60,000 ms**. A separate **500 ms** wake is requested; that does not mean full power allocation every 500 ms. |

The shared deadlines are object offsets `+0x1b70` (preparation) and `+0x1b78`
(fallback). A new load can extend the preparation deadline for the block, not
just itself. The countdown output uses the later applicable future deadline,
rounded up to seconds. Enqueue wrapper RVA `0x430120` forwards tagged deadlines
to the previously identified scheduler; queue presence is not proof of an
exact callback time.

### Frame clock for a passive entry observation

The already reviewed dispatcher passes frame context `0x1419ac930` as the
handler's second argument, **RDX** on the normal AMD64 call path. The frame
clock is the **u64 at `RDX + 0x38`**: dispatcher clock destination
`0x1419ac968` is exactly that offset. Handler RVA `0x614636` preserves RDX in
R13; its two deadline comparisons read `[R13 + 0x38]` at RVAs `0x614844` and
`0x614861`. This is a static pointer/field relationship, not a captured live
frame binding. `+0x30` is a separate entry gate, not this clock.

The minimum-run/accounting helpers separately call the clock reader rather
than receiving this frame value. Those samples need not be bit-identical to
the frame timestamp. Neither value is proven monotonic or a fixed-step dt.

There is no positive-power guard in the activation helper itself. An eligible
zero-rated load can briefly start a zero-allocation episode, arm preparation,
then have its episode closed by output update. Thus zero kW alone does not
disable a slot or prove it cannot affect shared timing. The daily XML retains
two additional used/connected zero-rated records; their eligibility and the current
power budget must not be ignored when attributing evaluation cadence.

**Minimum runtime is not an automatic OFF timer.** Once eligible, a load can
remain ON because power, daily obligation, priority or manual selection still
requests it. Conversely, ordinary scheduler deferral can postpone evaluation
past the floor. No minimum-floor-specific wake is established in the reviewed
paths. All elapsed-time statements assume the normal increasing clock path;
the recovered clock is adjusted local wall time, **not proven monotonic**.

### Daily obligation

Daily helper RVA `0x619800` compares the positive remaining `MDay` seconds
against the nonnegative seconds left to the selected local deadline. When due,
it requests rated power before normal surplus allocation. Selectors 4–27 map
to 01:00–24:00; selector 14 therefore means 11:00. Selectors 0–3 use separate
document time fields; their astronomical calculation is not recovered here.

Daily accounting helper RVA `0x616430` adds completed episode seconds to the
current timestamp-derived episode duration. Output updater RVA `0x61b360`
closes that episode when its internal allocation is zero. These paths count
**manager-commanded run episodes**, not a separately measured integral of the
status input or consumed energy. The feedback value does not gate the elapsed
calculation in those helpers.

The day-marker helper RVA `0x616a10` clears the reviewed per-load runtime state
when the frame's day marker differs. It does not by itself prove calendar,
DST, midnight-continuity, persistence or full application-reset behavior.

## Feedback, stopping and forced Off

The [native documentation](../kb/energy-manager-2.md) explicitly supports:

- **Digital source:** status OFF/ON. An ON source is translated into allocated
  or rated load power, not interpreted as the literal value **1 kW**.
- **Analog source:** actual consumption in **kW**.
- **Unconnected status:** assume the allocated load power.

The handler's feedback branch at RVAs `0x6146f0`–`0x614767` independently
distinguishes connectedness and source kind via helper `0x45ef20`, then either
reads the analog value or converts digital state to load power. Source-kind
classification and the load's `An` output-mode setting are not the same test.
Feedback is processed before the full-allocation deadline gate.

Therefore the existing **digital** daily plant's `Or(OL1, 0).Q -> IL1`, with
`Gpwr = 2.5 * Or.Q`, is consistent; no feedback-unit bug was demonstrated.
Changing it to an analog source of literal value 1 would mean 1 kW, whereas
analog `2.5 * Or.Q` represents the same 2.5 kW plant. No fixture was changed.
`Ppwr` is documented as UI-only and is absent from the reviewed core
power-balance expressions. The older `energy-interfaces-analysis.md` overview
is not authoritative for connector direction, defaults or units.

For eligible automatic digital loads, RVA `0x6194b0` contains separate
switch-on/rated comparisons, a small power tolerance, and a distinct
permitted-import branch when switch-on power is below rated power. Minimum
floor expiration alone does not choose the OFF branch. The complete analog,
priority, storage and multi-load allocator is **not** claimed recovered.

The dominating input is called **Off** in the documentation and stored as
connector key **`Reset`** in this fixture/build. Helper RVA `0x616d00`:

- returns early from the cycle while the input is held;
- on its detected high-state change, clears per-load runtime/allocation/manual
  state and updates outputs without consulting `MinRt`;
- does **not** establish unconditional clearing of both shared deadlines.

Thus a minimum floor is not protection against forced Off. Releasing Off is
not proven to resume immediately or preserve completed daily credit. The
previous SOC comparison exercised the inactive Off path, not this reset edge.

## Four fresh exploratory timing witnesses

On **2026-11-06**, four separately declared plain native runs retained the same
transparent daily plant, identities, wiring and five observation windows.
Only the first Load's stored `RT`/`MinRt` varied. Canonical CLI inspection
verified seconds: RT 30/60, and minimum runtime absent-as-zero/60/120.
The CLI's minimum-runtime argument is minutes; the saved fields, not that
argument's numeric spelling, establish the seconds used here.

| Stored RT / MinRt (s) | 20–40 s | 70–90 s | 130–140 s | 170–175 s | 200–210 s |
|---|---:|---:|---:|---:|---:|
| 30 / 0 | 0 | 1 | **1** | 0 | 0 |
| 60 / 0 | 0 | 1 | **0** | 0 | 0 |
| 30 / 60 | 0 | 1 | 1 | **0** | 0 |
| 30 / 120 | 0 | 1 | 1 | **1** | **1** |

These are actual `OL1` samples, not asserted expectations or continuous
intervals. Each retained pin/reference agreement, elapsed capture interval,
native clock evidence, verified running state and closed owned cleanup:

- RT30/Min0: ON at **131.628 s / 11:00:12**, OFF at
  **170.640 s / 11:00:51**.
- RT60/Min0: already OFF at **130.731 s / 11:00:10–11**; its preceding
  ON sample was **70.728 s / 10:59:10–11**.
- RT30/Min60: ON at **130.639 s / 11:00:11**, OFF at
  **170.643 s / 11:00:51**.
- RT30/Min120: still ON at **171.672 s / 11:00:52** and
  **201.976 s / 11:01:22**. Its last capture interval was
  **201.722–202.229 s**; no release was observed.

The earlier OFF sample with RT60 contradicts the naive rule that increasing
preparation time must always postpone OFF. The observations are consistent
with differing preparation/fallback evaluation phases, but **do not prove
that causal mechanism**, an exact release time, or a preparation-delayed
minimum-run start. Identical broad Min0/Min60 samples do not mean their
minimum-runtime settings are generally equivalent.

Every definition was **exploratory**, with no expected output assertions:
**zero new native passes**, no repeatability loop, and no promotion of these
receipts into a reviewed contract. One execution per vector is not an
independently repeated result. No fifth zero-load control was executed.

Private joined evidence:

```text
.reverse-engineering\native-simulator\cases\20261106T063931Z-timing-four-plain-b48c58631f84\joined-summary-v2.json
```

Helper provenance is deliberately split. Runs 1/2 retain run-local staged
PowerShell/runner sources and separate admission-time file snapshots; run 2's
retained controller-file/runner hashes differ from run 1. An admission-time
controller-file copy does **not** establish the Python bytes loaded at launch.
Runs 3/4 used a private, exact `aa28dba` Git-source snapshot,
including local imports and staged remote helpers. Their deployed UX backend
was independently matched to that revision's CRLF representation without
changing the existing profiles. Earlier executed bytes are **not**
retroactively described as that later snapshot, and current shared sources
are not evidence of earlier execution.

## Remaining discriminating observations

The original static proposals below are retained as historical hypotheses;
the four witnesses above used their separately predeclared five-window
protocol. Unexecuted follow-ups remain **proposals**, not authorized jobs.
Use fresh exploratory definitions, actual native clock/output readback and
immutable fixture/definition hashes. Never change expectations after seeing a
sample. The private countercase artifact declares exact windows and controls.

1. **Preparation boundary:** retain existing `MinRt=0` daily fixtures and
   compare `RT=30` versus `RT=60` at elapsed **125–140 s**
   (native **11:00:05–11:00:20**). This distinguishes their intermediate
   recalculation phases that the broad OFF/ON/OFF samples miss.
2. **Minimum floor:** compare only `MinRt=60` versus `120`, keeping `RT=30`,
   sampling **165–175 s** and **195–205 s**. An extended **225–240 s** window
   for the unchanged 120 s fixture is a separate observer-budget proposal:
   it is outside the current replay runner's 210 s phase limit.
3. **Feedback units:** on a separately reviewed 3 kW surplus / 2.5 kW digital
   load plant with no daily obligation, compare digital status with analog
   2.5 kW feedback, and optionally the intentionally incorrect analog 1 kW
   alternative. Observe after activation and after preparation.
4. **Forced Off:** only after a bounded native input-control protocol is
   reviewed, assert Off during an observed ON episode shorter than `MinRt`,
   sample held state, then separately observe release/requalification.

One conditional static schedule is consistent with the historical late ON:
activation near elapsed 60 s, preparation evaluation near 90 s, then ordinary
evaluations near 150 s and 210 s. At 150 s the 120 s floor would still hold;
the historical sample at 201.574 s and new sample at 201.976 s could precede
the next evaluation. **Those are not
observed transitions or a promised release time.** Actual activation time,
live object/UUID attribution, force conditions, queue servicing and release
must still be measured. Engine completeness and coverage percentages remain
unknown.
