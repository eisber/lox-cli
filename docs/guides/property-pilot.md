# Small SDK property pilot

This is an opt-in execution adapter, **not** a new desktop authority or a
replacement for the file-only [campaign queue](ux-campaign.md).
`scripts\ux-experiment.py` uses the existing profile validation and
`loxone_ux_transport.MCPClient`. It does not change SDKToolGate, InputLedger,
profiles, controllers, policy, DPI awareness, or the Rust CLI.

The separate `config-property-pilot-20261109-v3` catalog predeclares five
**inputs to one protocol**, not five different proven behaviors:

| Immutable case ID | Proposed Title |
|---|---|
| `pilot-title-ascii-20261109-v3` | Energy ManagerPilot |
| `pilot-title-unicode-20261109-v3` | Energy Manager Wärme |
| `pilot-title-spaces-20261109-v3` | Energy Manager Pilot Room |
| `pilot-title-punctuation-20261109-v3` | Energy Manager (Pilot-4) |
| `pilot-title-long-20261109-v3` | Energy Manager Pilot Persistence Observation Five |

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
`tests\ux\campaigns\config-property-pilot-20261109-v3.json`. The command above
regenerates it to a new private destination, so it works on a fresh checkout;
generation deliberately refuses to overwrite any existing file.
Queue creation predeclares one non-retryable qualification run
per worker for this adapter; each worker/version needs its own actual generic
case canary. SDK readiness alone does not qualify anything. Admission, claim
and handoff retain the existing exclusive worker/resource rules.

Unpublished v1/v2 catalogs, IDs and source snapshots are retained as private
draft provenance, not shipped as unusable manifests. Reviewed source fixes
require fresh v3 IDs/pins; no previous journal or definition is rebound.

The private handoff binding additionally provides `catalog_path`,
`artifact_root`, `executable_path`, `document_path`, `before_xml` (relative raw
artifact descriptor), `authority_module`, and `authority_source_sha256`,
alongside `worker`, existing `ux_profile` and
`fixtures.property-baseline`. The authority must independently stage an owned
disposable document with the exact fixture bytes. The executor never writes
native XML. Do not copy private bindings, artifacts or vendor files into Git.

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
This pin is not authentication or a grant of desktop authority.

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
