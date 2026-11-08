# Version-bound native UX campaign

`scripts\ux-campaign.py` is a standard-library, **file-only** queue and evidence
collector. It never opens a transport, starts Config, runs `lox sim`, injects
input, provisions machines, or grants desktop authority. Its default `plan`
operation performs no input. Beads remains the task tracker (`lox-7cq7`, epic
`lox-gav4`); the append-only journal is experiment data, not another task system.

## What the 100 definitions actually mean

The generated catalog is
`tests\ux\campaigns\config-17.2.10.21-mixed100.json`. It pins Config
**17.2.10.21**, SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`,
the existing executor sources, source case paths/hashes, and individual
canonical definition hashes.
Repository source descriptors explicitly use `sha256-lf-source-bytes` (CRLF
normalized to LF) so the catalog is reproducible across Git checkouts.
**Native artifacts, fixture bytes, executable and actual deployed-helper
identity proofs always use raw-byte SHA-256**; source-text normalization is
never applied to native receipts or used to relax a runtime pin.

Exact mix:

| Phase | Definitions | Complete existing executor interface |
|---|---:|---:|
| Configuration | 70 | 1 |
| Simulator | 30 | 30 |
| Total | 100 | **31** |

**31 is interface compatibility, not 31 end-to-end runnable or qualified
workers/cases.** Actual guarded readiness, fixture identity, captures and the
private receipt-envelope producer are still required. The 100 experiment
definitions and their conditions are unchanged by the qualification fix below.

Configuration consists of the exact reviewed Energy edit/cancel/reopen-dialog
recipe, all 63 existing Energy configuration follow-ups, four Lighting
roundtrip procedures, and the Shading/HVAC admission procedures. These cover
28 explicit deadline indexes, names/modes/customization markers, cancellation,
priority/identity preservation, units, non-finite/scientific/overflow text,
malformed records/references, nested moods and sparse outputs. Source procedures
and concrete input values are retained, not replaced with guessed selectors.
The catalog also records explicit independently authored seeds/variants for
names, marker transitions, cancellation, record corruption and note linebreaks.
These are proposed **inputs**, not previously observed GUI outcomes. Icon
choices remain bounded same-version native-inventory operations; their widget
and exact reference bindings are explicitly unqualified.

**69 cases are `needs-executor`, not runnable.** The missing adapter is a guarded
native Save As / close / reopen lifecycle with retrieved before/after XML,
semantic diff, and owned captures. Empty action stages additionally need
version-calibrated widget/variant binding: Energy scalar editors, icon picker,
priority controls and malformed-project admission; Lighting mood/channel
selectors; Shading admission/timing units; HVAC admission and its unresolved
56-versus-52 connector discrepancy. Existing Energy edit primitives do **not**
complete those workflows. No inferred GUI defaults or CLI acceptance rules
are native oracles. These configuration cases remain exploratory.

Simulator definitions are **six explicitly identified plain repetitions of
five existing protocols**, not 30 new behaviors: original and reordered
three-load priorities, storage at/below its threshold, and the daily observer
baseline. Each retains the exact fixture SHA, settings/inputs, expected outputs
and native sample windows of its source. Native Or observations belong to those
Energy fixtures; this does not establish a general logic-family oracle.
Calibration images/profiles and original projects remain external/private.
There is no new instrumentation or interaction while attached.

**Actual native executions performed to implement this queue: 0.**
Executor compatibility is not worker readiness, fixture admission, or a pass.
Whole-product behavior percentage remains `null`; static identities/body
counts and Python test results are not native UX coverage.

## First local checkpoint

```powershell
python scripts\ux-campaign.py validate
python scripts\ux-campaign.py plan
python -m unittest discover -s scripts -p test_ux_campaign.py -v
```

The first configuration canary is
`config-energy-dialog-cancel-canary`. It uses
`tests\ux\recipes\energy-edit-cancel-check.json` unchanged: selected Energy
block, open/focus, row-1 name edit, deadline index 27, Cancel, reopen the
**dialog**, read and Cancel. It is not file-close/reopen or a persistence pass.
Successful native action receipts are classified as **observed**, because
the helper explicitly marks UIA grid text non-authoritative.

## Private bindings and exclusive workers

Keep all queue data/handoffs/raw artifacts under an ignored
`.reverse-engineering\ux-campaign\...` directory or an external private root.
Use a **new directory** for each queue; initialization/generation never
overwrites existing history. Private profiles and endpoints are never
required in the public catalog.

An existing guarded authority must separately provide a scheduling binding.
This is collector metadata referring to that authority's **actual retained
admission receipt**, not a new native readiness API:

```json
{
  "schema_version": 1,
  "kind": "ux-campaign-worker-binding",
  "worker": "lab-a",
  "physical_identity_sha256": "<verified opaque physical-desktop identity hash>",
  "application": {"version": "17.2.10.21", "sha256": "<exact executable hash above>"},
  "authority": "<existing guarded authority receipt producer>",
  "admitted": true,
  "valid_until": "<authority-issued UTC timestamp>",
  "admission": {"path": "<relative private receipt>", "sha256": "<actual file SHA-256>"},
  "resources": {"shared-host-a": 1},
  "capabilities": ["energy-dialog", "guarded-native-input", "owned-capture"]
}
```

Only `lab-a`, `lab-b`, and `notebook` are accepted aliases. They must bind
**distinct verified physical desktop hashes**. Three aliases on one Notebook
are rejected. Resource names/capacities come from the external authority:
VM workers sharing a gated host name the same resource and capacity; an
independently qualified Notebook names its independently verified resource.
A failed VM readiness check does not create 100 attempts or block an unrelated
Notebook. Legitimate shared-host capacity limits remain enforced.

**Integration prerequisite:** the external guarded authority must produce
these binding/capture envelopes from its actual receipts. There is no public
importer for private readiness/proof formats in this change. A caller-written
`admitted: true`, status success, or three aliases does not establish readiness.
The queue binds receipt bytes and scheduling metadata; it cannot authenticate
an arbitrary producer or replace live source/native/physical-input, session,
foreground, overlap and offline-fence guards. Those remain authoritative on
**every** executor operation. Do not populate bindings from CLI/unit output.

```powershell
$q = ".reverse-engineering\ux-campaign\fresh-run"
python scripts\ux-campaign.py init --queue $q
python scripts\ux-campaign.py admit --queue $q --artifact-root $privateRoot --binding $admittedBinding
python scripts\ux-campaign.py batch-plan --queue $q --worker lab-a --limit 5
python scripts\ux-campaign.py claim --queue $q --worker lab-a --canary
```

`claim` returns a unique run ID and retains dispatch intent before any external
action. It does **not** execute input or count a native attempt. Each adapter
requires a completed compatible native canary per worker; simulator admission
does not qualify Energy configuration, or vice versa.

### Independent qualification runs, not extra campaign experiments

At **queue creation**, before any admission, the `created` journal event
predeclares six qualification identities and their unique dispatch `run_id`s:
one for each of the three workers and two adapters. Their static identities
include `qualification-notebook-loxone-ux-recipe`,
`qualification-lab-a-loxone-ux-recipe`, and
`qualification-lab-b-loxone-ux-recipe`. Each binds the adapter, worker, approved
source-template IDs/digests, definition hash and **maximum one dispatch**.
`plan` exposes the definitions; `batch-plan` exposes the selected worker's
predeclared run ID. These are scheduling/evidence identities, never grants of
native input authority.

`claim --worker notebook --canary` dispatches that worker's predeclared
**qualification** run using the unchanged Energy cancel recipe. Lab workers
can independently qualify the **same configuration adapter** through their
own already-declared identities. They are not consuming, cloning or retrying
the Notebook's run. Success consumes the qualification identity, **not** the
100-case experiment definition. A subsequent ordinary `claim` can dispatch
the original configuration experiment once globally; even consuming that
experiment does not delete another worker's unattempted qualification.

Each pair gets only one qualification attempt, regardless of which declared
simulator template it uses. Failure/UNKNOWN never creates a replacement,
changes workers, or selects another template as a retry. Terminal control and
UNKNOWN still park/quarantine the physical worker and shared resources.
Another worker requires its own independent actual admission, explicit
canary claim and distinct native execution receipt. A bare admission flag
cannot open the adapter gate. Native execution IDs cannot be reused across
qualification runs or experiments.

Stats' top-level case/attempt/completion/pass/field counters are for the
**100 experiments only**. `qualification` reports separate qualification
dispatches, evidenced native attempts and completions; per-worker summaries
have the same split. `all_native_attempts` / `all_completions` explicitly sum
both categories. For example, three successful configuration qualifications
mean **0/100 experiments attempted and 3 qualification completions**, not
three completed experiments or 103 executed cases. Qualification observations
still do not establish semantic or persistence passes.

Old journals lacking the predeclared plan are not silently migrated or
reclassified. They remain readable with their original catalog/accounting;
new admission/dispatch/handoff is refused. Retain any old captures under
their original run identities. A fresh queue is permissible only after the
external authority has resolved prior ownership/quiescence; it is not an
escape from UNKNOWN.

New journals also require `handoff_identity_version: 2`. Each Config handoff
retains `expected_target_pid` from its emitted `--pid` in the append-only
journal. Captured request, reply and identity proof must bind that exact PID,
not merely agree with each other. A legacy missing PID is UNKNOWN; never
backfill it from a mutable private binding. Resolve ownership/quiescence and
create a new queue instead. No unavailable process-start identity is invented.

## Executable handoff, not a new launcher

```powershell
python scripts\ux-campaign.py handoff --queue $q --run-id $runId `
  --private-binding $privateBindings --output $newPrivateHandoffDirectory
```

For the configuration canary, the private bindings contain `worker`, explicit
owned `pid`, `ux_profile` (the **existing** `loxone-ux.py` profile path), and
`fixtures.energy-configuration` (local exact baseline XML path). The private
guarded lifecycle must already have opened a fresh disposable native-loadable
copy and selected the Energy block under the existing fence. The queue does
not navigate or launch it. `handoff.json` contains this actual existing argv:

```text
python scripts\loxone-ux.py --profile <private-profile> --pid <owned-pid>
  --expected-sha <pinned-hash> --deadline-seconds 300 --verbose run <recipe.json>
```

The generated recipe is validated by the existing driver's `validate_recipe`
and its argv by the real argument parser. The existing helper's guards,
unsupported operations, explicit Cancel, deadlines and stop-on-first-error
behavior are unchanged. No `force`, guard-skip, direct messages or ABI fallback
is provided.

### First concrete configuration execution and receipt bridge

The existing command below executes the exact bounded configuration recipe;
it is not a new lifecycle/widget adapter or a 100-case completion promise:

```powershell
python scripts\loxone-ux.py --profile $existingOwnedProfile --pid $ownedPid `
  --expected-sha 7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3 `
  --deadline-seconds 300 --verbose run tests\ux\recipes\energy-edit-cancel-check.json
```

Only the **sole designated GUI owner**, under its existing actual guarded
authority, may execute it on its already-owned disposable project. A canary
already in flight must finish under that owner: do **not** run this command,
query the desktop, launch another process, or reconstruct a new handoff for
the same execution. This implementation performed no native run; the current
owner's actual GUI receipt remains pending until separately received.
For prospective queue-managed runs, claim and materialize the handoff **before**
execution, then execute its exact argv once. Do not invent a PID or infer
application ownership from a successful session-metadata probe.

Minimal bridge for the parent/existing capture producer:

| Retained data | Required original source |
|---|---|
| Campaign run, purpose, qualification ID/hash | Exact predeclared claim and emitted handoff; not a new identity synthesized after a failure |
| `native-request` | Actual backend artifact-directory `request.json`, including exact target PID/hash, steps and `artifact_dir`; not a request reconstructed from the recipe |
| `native-receipt` | Actual backend `report.json`, validated with the existing driver's exact `validate_reply` schema |
| `native_run_id` | That actual request's `artifact_dir`, unique for this execution |
| Guard, fence and identity proof | Actual retained authority/process/session/physical-desktop/fence evidence for that same process and fixture, with raw digests |
| Observations, owned screenshot, XML/diff | Actual captured artifacts; no synthesized UI values, helper image, saved XML or golden outcome |

`--verbose` stdout includes caller `transport_metrics`; it is **not** the
backend's exact report schema. Retain it separately if useful and point
`native-receipt` at the actual backend `report.json`, rather than treating the
stdout wrapper as a native receipt. Existing collection/transfer remains the
sole owner's approved private route; this queue does not fetch files.

The cancel-only recipe does not save/close/reopen a project. Before/after XML
must be the actual disposable file bytes captured at those points, and the
diff must retain that limited scope—not fabricate a native save. UIA cell
text is non-authoritative; a fully bound successful reply is **observed**,
not a persistence or domain pass. If any actual identity/capture artifact is
unavailable, retain UNKNOWN/blocked instead of constructing proof from PID.
Full Save As/reopen, new widget bindings, profile/protocol migration and the
private envelope producer remain outside this implementation.

Simulator bindings use `sim_profile`, `transfer_profile`, `artifact_root`,
optional `ocr_profile`, and `fixtures.<fixture_alias>` from the catalog.
Exact fixture bytes must match. The adapter stages a fresh case definition
**below** the supplied artifact root, as required by `official-sim.py`, then
emits its existing `--profile --transfer-profile --case --fixture --mode plain
--artifact-root` argv. Private calibrated artifacts must exist at their
original relative hash-bound paths below that root. Missing OCR/calibration
or UI readback remains blocked; it never becomes a default OFF/pass.

The external worker executes the emitted argv only under its existing
authority and bounds; the public queue never runs it. Keep a qualified worker
warm within a compatible batch: fresh baseline file copy per case and an
already admitted VM baseline per batch, **not** snapshots/reboots per click.
Only one case is claimed per desktop at a time; claims across at most three
distinct desktops are atomic.

## Capture groups and evidence admission

The external collector produces `schema_version: 1`,
`kind: "ux-campaign-capture"`, `evidence_origin: "native-config-ui"` envelopes.
Each binds the returned `run_id`, `worker`, `physical_identity_sha256`,
`case_id`, `definition_sha256`, the canonical `handoff_sha256`, actual
`fixture_sha256`, and the executor's distinct `native_run_id`.
New dispatches also require the exact `purpose`, `qualification_id`, and
`qualification_definition_sha256` from the handoff (the latter two are `null`
for a campaign experiment). Relabeling a qualification as an experiment, or
replaying one worker's receipt as another worker's qualification, is rejected.
`artifacts` is a list of `{kind, path, sha256}` entries relative to the private
artifact root. Required kinds are:

* `guard-receipt`, `fence-proof`, `identity-proof`;
* `native-receipt`, `native-request`, `observations`;
* `before-xml`, `after-xml`, `xml-diff`, `screenshot`.

Identity metadata must bind the campaign/native run IDs, exact application,
fixture and physical identity, actual `pid`, absolute Windows executable
`path`, `start_time`, positive interactive `session`, and the actual
`guard_receipt_sha256` / `fence_proof_sha256` artifact digests.
This metadata must come from the existing authority's actual process/fence
proof, never be synthesized from a PID alone.

For configuration, retain the original verbose backend reply and backend
request, not compact output. The original validator checks the actual
PID/hash/request steps; successful actions are observations, **not** verified
persistence. The backend does not automatically screenshot successful
recipes: the separately guarded capture producer is required.

For simulator cases, retain the existing `official-native-replay` receipt
and all its manifest artifacts, exact native case definition and request.
The collector checks bytes/digests, original native run/PID/start, application,
fixture, samples and readback-source membership. It reuses the existing
readback calibration validator and `judge`, respecting original blocked
results rather than substituting a status string. Only that native assertion
path can produce `verified-pass` / `verified-fail`.
Synthetic input in actual Config UI can be native evidence; synthetic helper
receipts, local CLI/unit tests and their `status: pass` are not.

```powershell
python scripts\ux-campaign.py collect --queue $q --run-id $runId `
  --artifact-root $privateRoot --capture $captureEnvelope
python scripts\ux-campaign.py collect-batch --queue $q `
  --artifact-root $privateRoot --captures $captureEnvelopeArray
python scripts\ux-campaign.py stats --queue $q
```

Batch review is of retained capture groups, not a new coordinator proof per
click. `collect-batch` accepts 1–20 independently bound envelopes and still
validates each native receipt; batch acceptance never promotes actions to
passes. A private worker can repeat claim/handoff/capture/collect sequentially
within a bounded batch without requiring per-value coordinator review.
An identical already-collected envelope may be audited again in a capture
group: artifact bytes are rechecked, the original result is returned, and no
new journal result, attempt, completion or pass is added. A changed envelope
cannot replace a terminal result.
`batch-plan` is advisory and does not reserve or execute cases; a concurrent
claim always chooses the next actually available compatible case.

## Terminal stops and UNKNOWN

`USER_CONTROL` / `CONTROL_UNAVAILABLE` require the actual guarded error
receipt, end the case and park the worker. Any invalid/missing evidence at
collection is retained as `UNKNOWN`, not silently requeued.
Guard-only stops prove no native start. Fully validated, handoff-PID-bound
partial native receipts retain a proven start (`completed_steps > 0`) even
when terminal: they count an attempt in the appropriate qualification or
experiment category, but remain blocked, parked and unqualified—not completed
or passed. Missing/invalid partial proof remains UNKNOWN.
For timeout or missing caller response:

```powershell
python scripts\ux-campaign.py unknown --queue $q --run-id $runId `
  --reason "Caller timeout; remote completion UNKNOWN"
```

The dispatch intent and shared resource reservation survive indefinitely:
there is **no** lease TTL, automatic retry, blind reset or success fallback.
Other independently qualified workers may continue if their actual shared
resources permit. A stale scheduling admission only blocks new work; it does
not reclaim an active/unknown job.

Only separately verified quiescence can release a quarantined resource:
`quiescent --worker ... --proof ... --artifact-root ...`. The proof descriptor
must hash a private `ux-campaign-quiescence-binding` containing the exact
`run_id`, physical identity, `quiescent: true`, and a hash-bound existing
`guard_receipt`. Even then the failed case stays consumed and the worker
**stays parked**; a fresh queue/binding is required for explicit readmission.
A crashed queue writer leaves `writer.lock`; do not delete it based on age.
Verify process/quiescence separately before any manual recovery.
One queue is authoritative for this campaign. Creating a fresh directory does
not release a prior UNKNOWN resource: the external authority must first verify
its quiescence before granting any new binding for that physical desktop.

Experiment stats and separate qualification stats distinguish dispatch intents, evidenced native starts/attempts,
observations/completions, unique cases/control fields, verified failures and
passes, blocked/UNKNOWN, and actual elapsed runs/hour per worker. Missing
native proof is not counted as a known native start. Plans/readiness probes
contribute zero attempts. Six repeated protocols are not six times the
behavior coverage.

## Catalog maintenance

The catalog is generated deterministically from the reviewed source files.
To inspect a candidate, generate it to a **new** review path:

```powershell
python scripts\ux-campaign.py catalog --write .reverse-engineering\ux-campaign\catalog-candidate.json
```

Review changes before replacing the versioned catalog. Existing queues bind
the old catalog hash and reject drift; never overwrite historical journals,
raw evidence, fixtures or native receipts to make a new definition fit.
