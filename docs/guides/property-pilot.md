# Small SDK property pilot

This is an opt-in execution adapter, **not** a new desktop authority or a
replacement for the file-only [campaign queue](ux-campaign.md).
`scripts\ux-experiment.py` uses the existing profile validation and
`loxone_ux_transport.MCPClient`. It does not change SDKToolGate, InputLedger,
profiles, controllers, policy, DPI awareness, or the Rust CLI.

The separate `config-property-pilot-20261110-v4` catalog predeclares five
**inputs to one protocol**, not five different proven behaviors:

| Immutable case ID | Proposed Title |
|---|---|
| `pilot-title-ascii-20261110-v4` | Energy ManagerPilot |
| `pilot-title-unicode-20261110-v4` | Energy Manager Wärme |
| `pilot-title-spaces-20261110-v4` | Energy Manager Pilot Room |
| `pilot-title-punctuation-20261110-v4` | Energy Manager (Pilot-4) |
| `pilot-title-long-20261110-v4` | Energy Manager Pilot Persistence Observation Five |

Each pins the original raw fixture, Config version/image, target UUID/type,
Title, authored input, and executor sources. Nothing edits the old mixed100
catalog/journal or credits its standalone historical observations.
**Native runs performed by this adapter for this delivery: zero.**

## File-only preparation

```powershell
$catalog = ".reverse-engineering\ux-campaign\property-pilot-$([guid]::NewGuid().ToString('N')).json"
python scripts\ux-campaign.py catalog --pilot --write $catalog
$q = ".reverse-engineering\ux-campaign\new-property-pilot"
python scripts\ux-campaign.py --catalog $catalog validate
python scripts\ux-campaign.py --catalog $catalog init --queue $q
```

The same current catalog is checked in as
`tests\ux\campaigns\config-property-pilot-20261110-v4.json`. The command above
regenerates it to a new private destination, so it works on a fresh checkout;
generation deliberately refuses to overwrite any existing file.
Queue creation predeclares one non-retryable qualification run
per worker for this adapter; each worker/version needs its own actual generic
case canary. SDK readiness alone does not qualify anything. Admission, claim
and handoff retain the existing exclusive worker/resource rules.

Unpublished v1/v2 catalogs, IDs and source snapshots are retained as private
draft provenance, not shipped as unusable manifests. The reviewed v3 patch
and catalog remain external provenance. The v4 dialect/source-closure fixes
require fresh IDs, pins and queue; no previous journal or definition is rebound.

The private handoff binding additionally provides `catalog_path`,
`artifact_root`, `executable_path`, `document_path`, `before_xml` (relative raw
artifact descriptor), `authority_module`, and `authority_source_sha256`,
alongside `worker`, existing `ux_profile` and
`fixtures.property-baseline`. The authority must independently stage an owned
disposable document with the exact fixture bytes. The executor never writes
native XML. Do not copy private bindings, artifacts or vendor files into Git.

### Preparation without command-line payloads

The reusable `ux_property_executor.prepare_fixture(...)` is a separate,
preparation-only API; **do not call it from an uncertain native run**:

```python
result = prepare_fixture(
    existing_client, approved_input_file, expected_raw_sha256,
    predeclared_new_remote_document, private_artifact_root, preparation_id,
    transfer=existing_strict_transfer,
)
```

Prefer the existing `official-sim.py:Transfer` with its already protected
profile: strict known hosts, batch mode and identity-only authentication.
It sends only local/remote file paths through `scp` (the existing SFTP
transport), never fixture data in a process argument. The helper creates
the predeclared new directory, snapshots exact local input bytes, retains
intent/ack, transfers that file, then verifies remote bytes through a short
standard-SDK PowerShell request. No new transport/profile is invented.

If explicitly predeclared instead, `transfer=None` uses bounded 4,096-character
SDK `FileSystem` base64 chunks, first write refusing overwrite and subsequent
writes explicitly appending. Each intent and raw acknowledgement is fsynced
before proceeding. A short decode uses `CreateNew`, checks the decoded raw
hash before writing, and reads back the actual file. Large incompressible
input never enters a PowerShell/Python command argument; gzip is not relied on.
Final proof includes raw hash/length, first bytes, UTF-8 BOM, CRLF and lone
CR/LF counts. The source bytes are never normalized.

Every preparation PowerShell request and executor PowerShell dispatch checks
UTF-16LE `-EncodedCommand` base64 length **before sending**: at most 30,000
encoded characters plus a reserved 2,048-character wrapper, below Windows'
32,767-character limit. `isError: false` is not success when actual PowerShell
Status Code is nonzero. Any unknown chunk/transfer remains retained and is
never replayed; re-entering the same preparation ID refuses locally. A new
ID is not permission to evade a stopped case or repeat an uncertain effect.

`prepared` proves input staging only: zero GUI launches, input or native
completion credit. It never writes an expected native save result. Current
GUI ownership, source closure, actual root/focus and qualified selectors are
still separately required before any native execution.

## External authority integration prerequisite

The approved existing authority must expose this Python callback:

```python
snapshot(client, context, stage, expected_identity, control, display, input_intent)
```

It may use **read-only** standard UIA/OS observations through the existing
transport. It must not issue input. It returns a relative `{path, sha256}`
descriptor for its actual retained UTF-8 JSON receipt under the private
artifact root. The executor reads and hashes those bytes; a producer-supplied
digest or `admitted: true` is not accepted instead of that receipt.
The explicitly trusted module's raw source pin is retained in the handoff.
This pin is not authentication or a grant of desktop authority. The loader
reads the module once, verifies its **raw** bytes, then compiles those same
bytes; it does not hand a checked path back to a loader that re-reads it.
Private observer and transitive helper code need their own reviewed closure.

The receipt echoes `context` (run, definition, worker, physical desktop,
application, resources, input deadline), `stage`, `input`, and the canonical
hash of the actual live `DisplayInventory` response. It contains fresh
`source_utc`, actual `control_state`, `outcome: "available"`,
`evidence_origin: "native-config-ui"`, retained `source_guards` and `policy`,
and `facts`. The provider must actually verify these against its existing
source/native/physical-input, session, identity, ownership, resource and
per-worker policy receipts on every operation. They are **not caller flags**.
Notebook and VM policies need not be identical. No public private-security
provider or substitute admission oracle is supplied here.

`facts` carries exact `identity` (PID, nonzero main root, unmodified
precision-labelled `start_identity`, session, executable path/application).
Stage `loaded` also binds `document_path`; splash HWND zero is not loaded.
`initial-open`/`fresh-reopen` require zero existing Config processes.
`closed`/`final-closed` require zero processes and
`original_identity_absent` equal to the exact previous identity.

Before each input, `guard` facts include current `foreground`, uniquely
resolved `selectors` for the requested declarative binding, their actual
read-only UIA selector, `visible: true`, and dynamic `physical_bounds`.
`target_visible` and `foreground_visible` must both be actually observed true
(including `target_visible` at `loaded`). For a point input,
`hit` must equal `{physical_point: [x,y], identity: ...}` for that bounds
center, mapped and hit-tested using the actual per-worker display/observer
geometry. No fixed PID, coordinates, scale, forced foreground, direct UIA
Invoke/SetValue, private ABI or output forcing is used.
A normal visible-target focus click is allowed; subsequent inputs require
the exact foreground root. `value` facts include `uia_value` with `binding`,
`value`, exact fresh `identity`, and `method: "read-only-ValuePattern"`.

Invisible foreground/target roots, or different foreground/target roots in
the same PID, stop **before SDK input** as `blocked` /
`AMBIGUOUS_WINDOW_ROOT`. Same PID is not an owner-chain proof. Optional
`window_relationships` preserves actual fresh read-only Win32 owner/ancestor
chains, input focus, visible-hit root, expected PID/exact creation identity,
window class and UI state in `blocked_diagnostics`. Missing observations
remain explicitly unavailable. A guessed MFC/Qt ownership relationship or an
owner boolean never permits input, and no forced-foreground fallback exists.
Any future interpretation requires independently reviewed live relations;
historical facts from an already-closed process do not prove them.

**Current concrete integration gap:** historical Notebook observations prove
the unnamed Edit's exact ValuePattern and bounds, but do not supply a
qualified dynamic header/property selector binding. VM selector facts are
unavailable. The trusted authority must resolve those bindings from actual
live facts and refuse missing/ambiguous ones. Do not substitute historical
coordinates or deploy a new server. Operator historical runs are not tests
of this adapter. Parent review and explicit GUI ownership precede live use.

## Execution and evidence

After ordinary admission/claim/handoff, only the sole separately authorized
GUI owner may execute the **exact argv in `handoff.json`**. There is one
generic entry, not one command per control.

The deterministic protocol is ordinary document launch; select block and
property; normal SDK Type; official Ctrl+S; retrieve raw XML; normal Alt+F4
and prove zero processes; ordinary fresh-PID reopen; select/read exact UIA
value; retrieve raw XML/screenshots; finally normal close and prove idle.
PowerShell requires actual Status Code zero, not merely `isError: false`.
An unchanged/no-write save is not silently replaced by Save As; that would
require a separately predeclared route.

Every SDK dispatch has fsynced intent before the call and retained raw ack
after it. Admission rechecks current dispatch/quarantine/ownership and appends
execution-start under the same existing queue writer lock; each subsequent
intent rechecks that live dispatch too. Each authority observation is a checkpoint. Exceptions, uncertain
receipts, takeover, changed roots and partial runs stop; re-entry never
replays a run. Native start is separately `false`, `true`, or `null`
(unknown). Reservations remain through collection; UNKNOWN/takeover requires
external verified quiescence. This is queue-local exclusion, not a claim of
global cryptographic client exclusion or a kernel-atomic cross-system claim.

Results are `complete`, `reject`, `fail`, `blocked`, or `UNKNOWN`.
`complete` means the exact Title persisted with saved/reopened raw-byte
equality and fresh read-only UIA proof. The file-only collector independently
rechecks artifacts, identities, checkpoints, SDK acks and the full XML diff,
and records **observed**, never a semantic/behavioral pass. Every positional
attribute, attribute order, text/tail and child order change is preserved;
raw files remain authoritative, including byte-only formatting changes.
The read-only XML comparator accepts UTF-8 Loxone digit-prefixed attribute
names such as `12hTF` by adapting only lexical attribute-name tokens in memory
and restoring their original names in the parsed view. Attribute order,
unknown fields, comments, CDATA values and processing instructions survive;
reserved-name collisions and DTD/entity declarations are refused. No artifact
is rewritten, normalized, reserialized, or hashed through this adaptation.
Unexplained changes are not sorted away, ignored or declared equivalent.
The capture's `atlas_reference` links family/field/case/definition to retained
collector evidence; it does not automatically publish or promote atlas facts.
Synthetic test origins are refused by native collection.

Definitely-no-effect blocked/reject outcomes may be collected without native
origin only when their start/result, handoff and acknowledged read-only intents
are durably bound. Any effect intent or synthetic-origin closeout is refused.
These stops stay parked, receive no native/qualification/completion credit,
and are distinct from uncertain native completion. Typed exception, stage,
provider reference and traceback artifacts remain private; entry and journal
closeout failures are retained, never silently swallowed.

```powershell
python -B -m unittest discover -s scripts -p "test_ux*.py" -q
```

## One finite operational cycle

The first milestone is **five actual clean executor completions on one
independently authorized desktop**, not five definitions or mock receipts.
Use one qualification completion followed by the first four normal
experiments; qualification remains separately counted. Each starts from its
owned exact baseline. If five distinct authored inputs are required, run all
five experiments after qualification (six completions total).

Only after independent source acceptance, actual selector observation,
current owner/effect authority and the worker's real prerequisite gate:

```powershell
# $privateRoot and these existing bindings are parent-reviewed private data.
$catalog = "tests\ux\campaigns\config-property-pilot-20261110-v4.json"
$q = Join-Path $privateRoot "fresh-v4-queue"
python scripts\ux-campaign.py --catalog $catalog init --queue $q
python scripts\ux-campaign.py --catalog $catalog admit --queue $q --binding $admission --artifact-root $privateRoot
$claim = python scripts\ux-campaign.py --catalog $catalog claim --queue $q --worker notebook --canary | ConvertFrom-Json
if (-not $claim.ok) { throw "No dispatch; retain exact refusal." }
$run = $claim.run_id
$handoffDir = Join-Path $privateRoot ("handoff-" + $run)
python scripts\ux-campaign.py --catalog $catalog handoff --queue $q --run-id $run --private-binding $executionBinding --output $handoffDir
# DATA ONLY: pass handoff.json and its journal-bound digest to one fresh agent.
# Sole GUI owner independently reviews authority, then executes its exact argv.
# ux-experiment.py performs strict collection; do not collect it a second time.
python scripts\ux-campaign.py --catalog $catalog stats --queue $q
```

For each subsequent case the coordinator claims **without** `--canary`,
materializes a new handoff, and launches one fresh one-case agent. No fixed
worker pool, automatic replay, alternate-template canary, or new queue to
evade a failure. Continue only after actual `complete` + collected `observed`,
proven final zero processes, retained full XML delta and independent review.
A stopped worker remains blocked/UNKNOWN; unrelated authorized resource
lanes can progress, but shared-host resource reservations are not waived.
Waiting for capture/input prerequisites is not a dispatch or admission.

The existing Title bindings are logical requirements, **not qualified UIA
selectors**. `xml_uuid` is XML identity, never an invented UIA selector.
Painted Bezeichnung/graph Title text does not prove an exposed UIA element.
Duplicate editor IDs require the actual `Allgemein` ancestor, selected-type
header and unique `CSettingEditorText` observation. Do not execute this draft
while those facts are absent. A normal navigation step or scalar `O` variant
requires an explicit new contract/case/source version and before-failing
tests; this Title protocol is not silently relaxed to fit another route.

For every accepted observation, review the private `atlas_reference` and
full XML changes before proposing an atlas or CLI correction. Fixes are
separate reviewed changes; altered execution sources require a new catalog
and case identities. Old standalone runs never become executor runs.
