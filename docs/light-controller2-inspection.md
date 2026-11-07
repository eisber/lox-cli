# LightController2: initial read-only slice

```powershell
lox config lighting inspect fixture.Loxone "Fixture Lighting" --room Study -o json
lox config lighting inspect fixture.Loxone "uuid:lighting"
lox config lighting inspect fixture.Loxone "uuid:lighting" -o csv --no-header
```

This local-only command exposes every **stored** mood `Qn`, numerically ordered
`AQn` connectors, exact downstream wire references, and nested element attributes.
It reuses the existing mood listing helper. It never saves, rewrites, connects to
a Miniserver, supplies missing output values, or assigns an actuator role.

JSON includes raw attributes, IDs, unknown nested records, document version
metadata and diagnostics. `status=read_only_unverified` and `gui_verified=false`
are deliberate: recovery is bound to Config **17.2.10.21**, executable SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`,
not proof that an arbitrary input file came from that executable. Table/CSV
warnings remain visible with `--quiet`.

Units, color encodings, channel roles, `NameAI`/`CapAI`/`PresM`, `FC`, `HCL`,
`PSD`, `LSConfig` and sequencing remain **unsupported for interpretation/editing**.
Missing/duplicate connector identities and malformed mood IDs are reported
explicitly, not repaired. XML text/comments are not reproduced in the structured
attribute inventory; the original file is untouched.

Existing `config moods`, `config set-mood-color`, and `config add-mood` remain
unchanged prior art, not newly GUI-verified capabilities. See
`contracts/light-controller2.json` for exact canonical binder hashes, inventory,
and the queued four-execution GUI protocol. The synthetic fixture is independently
authored and **not native evidence**. The first leased native configuration
attempt is now closed and blocked, not a roundtrip PASS. Full lighting oracle work
and its Energy parity dependencies remain open.

## Dedicated lab preparation and queued protocol

The original `tests\ux\light-controller2\synthetic.Loxone` is a DOM countercase,
**not** the GUI project. Its hash and contents are unchanged.

`tests\ux\light-controller2\prepare-lab.py` independently authors a disposable
`ControlList/Document` envelope with room/category containers, an unconnected
all-zero-serial lab `LoxLIVE`, a referenced `Program/Page`, BOM/CRLF, and fresh
native-format UUIDs. The pinned existing CLI adds the controller's 71 connectors,
three wired `StateV` observers and four moods. All identities/references are
checked offline. CLI validation is **not proof of native controller loadability**:
the native run below proved only the imported document caption.

Prepared files:

- `.reverse-engineering\native-simulator\workstreams\lighting-preparation\batch-02\manifest.json`
- `.reverse-engineering\native-simulator\workstreams\lighting-preparation\batch-02\Lighting-Slice-Lab-20261106.Loxone`
- `.reverse-engineering\native-simulator\workstreams\lighting-preparation\batch-02\checkpoint.json`

The exact four-execution queue is
`tests\ux\light-controller2\gui-protocol.json`: native load/inventory/no-op;
one prior-art `Q2` edit; sparse-output countercase; cancel/no-op. Actual lighting
selectors must be discovered under the lease, not guessed beforehand.
The prepared variant's `--color 43` is an existing CLI color argument, **not**
raw `Q2=43`: offline readback verifies that only `Q2` changes from `1610612751`
to `1610612753`. No GUI unit or actuator role is inferred from those integers.

**Corrected identity status:** Energy's newer explicit preflight verifies
the authorized private-profile notebook/actor/active-console identity, with
both pinned application and backend present. Its earlier absent-backend/actor-mismatch
report was incorrect: the SSH transport account does not establish the GUI actor. No desktop or
tunnel repair was indicated. The subsequent explicitly leased Lighting run
independently verified that notebook identity, exact application/backend hashes,
normal console desktop and the retained outbound block fence. Its owned process
is now closed and the lease returned. Any future attempt still needs an
**explicit new parent lease**; previous authorization is not reusable.

## Initial native configuration attempt: blocked

Private receipt:
`.reverse-engineering\native-simulator\workstreams\lighting-preparation\gui-batch-20261106T132251Z-receipt.json`
(SHA-256 `24cd5a2c4d62dc7f6d2117eaafe2fbab1de5b9c279eed51a729345c762a22124`).

One native process was launched, with **zero simulator starts**. Owned MFC/MDI
captions identified `Disposable Lighting Slice Lab 20261106`; the generic
application header was not accepted as controller identity. The frozen Save As
operation timed out. Native filename and Save controls were then identified
semantically, with exact owned filename readback; Save and keyboard Enter
alternatives still yielded no output in the exact owned run folder.

After identity-checked closure of PID `16104` (forced cleanup explicitly
recorded), only the transferred original XML remained, with unchanged hash.
No native-saved XML exists to inspect or reopen. Thus post-save mood values,
UUID/container/channel references, native fixups and controller identity are
**unknown**. For example, offline Slice A stores `Q1=1610612740`,
`Q2=1610612751`, `Q10=1610612761`; no corresponding native-after values are
claimed. The remaining edit/sparse/cancel cases were not run after this gate
failed.

An offscreen UIA rectangle also produced non-finite JSON during broad dialog
inspection; the strict frontend rejected it. The raw report was retained
privately, not normalized into a PASS or used to weaken shared helpers.
The unresolved blocker is native import/export or driver binding, **not an
established CLI value bug**. No channel-role, nested-edit, runtime or full-family
capability was promoted. Raw screenshots, traces and native artifacts stay
private and ignored.
