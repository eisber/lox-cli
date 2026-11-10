# Version 5 revision 2: rendered body + native inspector

This is the small `sdk-property-hybrid-v1` selection adapter on the existing
property executor, queue and `MCPClient`, not a new scheduler, desktop authority
or per-control CLI. **No v5 native completions or live qualification are claimed.**
The first milestone is one independently admitted real canary, then five clean
executor completions before proposing more scale.

The public method is explicitly `hybrid-image-inspector-v1`, with strategy
`exact-stable-body-v1`. It does not claim that a rendered block has a UIA UUID.
An exact, globally unique frozen-XML Title must resolve to the declared Type
and UUID; the current native inspector must independently corroborate that
Title and the reviewed selected-type header **before editing**.

## What stays frozen

Published v4 definitions, source pins, journals and the legacy guard contract
remain unchanged. Replay v4 on commit `73bbe975468e30ed4efbfee59692dd45a94a178c`.
The shared source changes intentionally make its source-pinned catalog refuse
on this checkout. Do not repin old cases or turn standalone observations into
v5 runs. The new catalog is `config-property-hybrid-20261110-v5-r2`, with new case,
adapter, source and per-worker qualification identities.

```powershell
python -B scripts\ux-campaign.py catalog --pilot --write <new-private-v5-catalog.json>
python -B scripts\ux-campaign.py --catalog tests\ux\campaigns\config-property-hybrid-20261110-v5-r2.json validate
```

The five Title values remain proposed inputs. The declared input SHA must match
the actual private fixture, including BOM and CRLF, and its Title/type/UUID
cardinality must be verified. No vendor fixture, image, crop, credentials,
runtime binding or effect grant is published here.

## Prelaunch authority boundary

A v5 private handoff requires `execution_binding` with exactly three relative
raw `{path, sha256}` descriptors under its private artifact root:

* `effect_authority`: independently accepted real grant, scoped to this run,
  definition, worker, physical identity, application and resources, with future
  `valid_until` and scope `native-property-execution`.
* `runtime_binding`: those same run/worker/physical/application fields, method,
  exact document/executable, authority RAW source pin and the exact two other
  descriptor values (`effect_authority`, `selector_evidence`).
* `selector_evidence`: independently reviewed actual image/native-inspector
  evidence described below.

The complete descriptor bundle is bound into the actual append-only handoff
and its separate journal digest. Mutating a JSON file and supplying a new hash
cannot replace that retained anchor. Every observation/input rechecks its raw
bytes and grant expiry. Scheduling admission still uses the existing real
resource receipt and TTL; an allowed boolean, old sample or local clock
extension never creates fresh admission.

When ownership uses a Beads issue lease, the active case controller renews its
own lease with `bd --actor <owner> heartbeat <issue> --json` and checks the
canonical `bd show` receipt. Repeating `update --claim` is idempotent and does
not renew an existing lease. The read-only authority verifier must reject an
expired lease, not renew it or invent a later expiry.

Before any launch, the separately accepted source-pinned authority must expose:

```python
preflight(context, execution_binding)  # returns actual private {path, sha256}
```

It must independently verify the **real parent-held grant and current owner**
against its existing authority, not accept a kind string, self-supplied hash
or `admitted: true`. The retained receipt must echo the complete context and
descriptor bundle, have fresh timezone-aware `source_utc` and `outcome:
"available"`. Missing callback, missing provenance, changed binding or stale
grant stops before launch. The library does not mint or authenticate a real
grant; source review/pins are not effect permission.

The private bootstrap needs a newly accepted v5 source closure: the six public
executor source descriptors now include `ux_hybrid_selection.py`. The loader
checks the declared public LF-source digest and compiles the same read bytes.
Do not silently expand a five-source v4 bootstrap or rewrite its anchors.
All private, native, image and authority pins remain **raw**, never LF-normalized.

## Actual selector evidence

The evidence pins method, strategy, capture-source SHA, and
`mutable_title_excluded: true`, backed by actual independently reviewed crops:

* `body_template`: original `source_image`, exact `template` descriptor and
  `[left, top, right, bottom]` crop. It is an interior BODY/ICON anchor, not
  editable Title text, a header, an arbitrary mask or a whole mutable-title
  target. The original/crop pixels are independently rechecked.
  Mandatory integer `safe_click_box: [left,top,right,bottom]` and
  `safe_click_offset: [x,y]` are template-relative. The offset must lie inside
  that reviewed box, and the transformed point must remain inside it and the
  canvas. The contextual template center is never assumed to be safe.
* `baseline_label`: separately retained original/rendered baseline Title crop.
* `inspector_label_template`: separately retained actual Bezeichnung crop.
* `canvas`: actual `selector` fields and an ordered direct-parent `ancestors`
  prefix. This scopes the rendered search; it does not invent block UIA.
* `inspector`: actual `selected_header` selector/ancestor pattern, explicit
  reviewed `xml_type` correlation, the actual `editor` selector/ancestor
  prefix and `row_ancestor_index`. Nonempty `label_semantic`, `native_locale`
  and `xml_attribute: "Title"` explicitly map the actual rendered property
  (for example German Bezeichnung or English Name) to XML. The label is the
  reviewed interpretation of the pinned rendered label crop, **not an observed
  UIA Name/Value**. Painted labels may expose empty names or truncated pixels;
  never fabricate a full native name. These patterns must come from actual
  observations, not guessed automation names or XML UUIDs.

The existing optional file-only Pillow decoder is used; there is no OCR,
external image service, live SDK change or new native dependency. Only bounded
RGB/opaque-RGBA original PNGs are admitted. Exact RGB candidates are searched
only inside the measured canvas. Zero or multiple candidates refuse. There is
no fuzzy score, learned template, automatic crop rebinding or confidence waiver.
Missing Pillow explicitly refuses in preflight before launch. The stdlib-only
CI skips image-dependent tests, while a separate stdlib test verifies that
missing-dependency refusal. No dependency is silently treated as success.
Body and baseline label must come from the same raw frame, stay inside its
reviewed `baseline_canvas_crop`, and not overlap. This proves geometric crop
association only; their semantic association still requires independent review.

## Owned capture, not a global Screenshot dependency

V5 lifecycle images use the existing read-only authority callback at stage
`capture`; they do **not** dispatch the SDK's global `Screenshot` tool.
`facts.owned_capture` has method `reviewed-owned-window-png-v1`, exact identity,
raw `image` descriptor, reviewed capture-source SHA, explicit `native_method`
and actual `native_success`, measured `window_bounds`, mapping and raw
`capture_ack` descriptor. The evidence's nonempty `capture_methods` list is
independently reviewed (for example the actually accepted PrintWindow2 route).
The image must cover those measured owned-window bounds. The actual retained
normal PowerShell/MCP reply must have `isError: false` **and** Status Code zero.

The same native-method/success/source/ACK/window proof accompanies
`facts.hybrid` and `facts.inspector`. The source-pinned private callback must
parse the actual native helper result, verify its raw PNG hash, exact process/
root and measured mapping, and retain every real request and raw ACK before
emitting these facts. A producer boolean alone is not capture admission.
The collector rechecks original PNGs, mapping, ACK status and retained proof.

The immutable evidence also requires `native_result_contract` version
`printwindow-native-result-v1`, native method `PrintWindow2`, and reviewed
JSON field paths. `capture` maps SHA, returned, method, flags, width, height
and pixel-to-physical transform to the actual raw `capture` object.
`before`/`after` independently map PID, root, start, session, executable/image
hash, project path, physical rectangle and measured calibration success to
their respective raw objects; they may not alias the other phase.
Actual PrintWindow method, flags 2, returned true, exact PNG hash/dimensions,
root/process/project and calibration must agree with the normalized facts.
Status zero plus a nonempty JSON object is **not** capture proof.
Start encoding is explicitly `identity-object` or `kernel-utc-100ns`; the
latter converts exact seven-digit UTC fractions to FILETIME ticks without
discarding the seventh digit. Mapping is reviewed/pinned, never learned.

This supports a reviewed read-only PrintWindow artifact through normal SDK
PowerShell without changing the SDK server. Use the already accepted,
strictly staged helper with a short read-once/hash/execute-the-verified-bytes
wrapper. Do not inline a large helper into a command argument. Each worker
needs its own accepted provider, actual policy, SID/session/source and measured
calibration. A VM's 96-DPI or SID assumptions do not qualify another machine.

## Live read-only callback contract

The existing `snapshot(...)` callback receives two additional context fields:
`selection_method` and **current** `expected_title`. It must echo both.
Initial selection/Type uses BEFORE. After the acknowledged Type, subsequent
guards and fresh-PID reopen use AFTER. Never reuse the old private callback's
hard-coded BEFORE on every guard.

A target pointer guard supplies `facts.hybrid`:

* exact current `identity`, method, expected Title, original baseline SHA,
  pinned `capture_source_sha256`, and actual raw owned PNG `image` descriptor;
* current scoped `canvas` selector, ancestor chain, visibility and
  `[left, top, width, height]` physical bounds and actual runtime ID;
* `mapping` bound to that identity and actual display-inventory hash, with
  measured axis-aligned `pixel_to_physical: [sx,0,tx,0,sy,ty]`, equal positive
  scales, and calibration method `measured-owned-image-v1`. Its actual image
  endpoints `[[0,0],[width,height]]` must agree with the retained measured
  physical endpoints. No DPI setter or guessed scale is used.

The computed point must lie inside both the unique body anchor and canvas.
`facts.hit` must bind that exact point to the exact PID/start/root identity
through actual native UIA/Win32 physical hit evidence.
`hit_canvas_runtime_id` must also bind the actual UIA FromPoint ancestor chain
to that canvas. This is canvas identity, never invented block identity.
Normal visible-target pointer activation remains separate from keyboard readiness; hidden/same-PID
ambiguous roots retain the existing refusal.

For property/Type/keys and fresh value proof, `facts.inspector` supplies:

* exact identity/current expected Title and actual `selected_headers` census.
  Exactly one native header must match its selector and ordered ancestors;
* current `editors` census. Exact rendered-label matching inside each scoped
  native row must qualify exactly one editor; it is not selected by expected
  value. Duplicate Title IDs/classes elsewhere do not qualify without that
  row's actual label pixels. Duplicate qualifying rows/editors refuse.
  It carries actual runtime ID, selector, visible/enabled state, `read_only:
  false`, `method: "read-only-ValuePattern"`, exact current value and actual
  keyboard-focus observation;
* actual matching `label_row_runtime_id` for the reviewed row ancestor,
  current `label_source_image`, `label_crop` and
  `label_image`, plus the same measured mapping contract. The exact label
  pixels must be from that owned image and lie inside the editor's actual
  reviewed ancestor's current row bounds. The semantic label mapping belongs
  to reviewed evidence, not a fictional native text read.

Property/value point receipts also carry the chosen editor's actual runtime ID
and selector: binding only the duplicated automation ID is insufficient.
Before Type, the editor must actually have keyboard focus. Every Type/Shortcut
also requires exact target foreground and `facts.input_focus` with that full
identity, nonzero actual HWND, `thread_active_root`, visible/enabled state.
The unchanged ToolGate/InputLedger remains authoritative on every SDK call.

## Stops, collection and first real handoff

Read-only observation need not pretend the target is foreground. It records
the actual foreground truthfully; it never creates keyboard authority.
`ready` **or** `ai` is valid ControlStatus observation; `user`/preemption never
permits override. A refusal or uncertain call stops the run, retains intent/raw
ack and remains parked. A later real READY/handback does not resume a terminal
v5 run or replay its input. Any separately authorized manual pause-aware route
is a different case, not automatic recovery here. Never infer event origin,
empty input ledger, self-echo or permission from state/generation alone.

The existing lifecycle remains select, inspect, normal Type/save, raw XML,
ordinary full close, fresh-PID reopen, stable-body select and exact AFTER
inspector/XML corroboration, then ordinary close. Original images, derived
crops and metadata are retained separately. The strict collector recomputes
exact image matches, before-Type checkpoint ordering, inspector/correlation,
binding pins and true keyboard-focus facts. Synthetic fixtures receive no
native qualification or completion credit.
Actual following dispatch arguments, including template-safe or scoped
inspector Click/Type locations, must exactly match the recomputed proof.
This generic protocol uses Ctrl+S. A separately qualified ordinary SaveAs
storage case does not qualify this adapter or silently add a SaveAs fallback.

Missing real effect grant, accepted v5 private provider/bootstrap, actual body
template/canvas/mapping provenance, or live inspector bindings must remain
explicit blockers. Only the sole separately authorized worker may execute the
exact sealed argv for the first new canary. Parent review and existing four
repository quality gates precede publishing or any live validation.
