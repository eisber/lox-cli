# DayTimer existing-interval To boundary

Static recovery from Config 17.2.10.21 AMD64, executable SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
No native application, UI, save/reopen, simulator or runtime qualification
was performed. This adds **one end-boundary edit**, not an arbitrary atomic
From/To assignment, drag/move operation, insertion or analog conversion.

## Public duplicate preflight

The full public implementation was checked at baseline `205f749`, before
native queries. `TimerSchedule` in `src/commands/config_cmd.rs` calls
`ConfigEditor::add_daytimer_entries` in `src/config_edit/wiring.rs`.
Both daytime and overnight branches remove **every** direct Entry and write
replacement To-only rows. They discard selected-row metadata and other rows;
this is not the same preservation operation.

Generic XML SetAttr/SetProperty use `require_one` and `matches_selector`;
the latter explicitly rejects elements other than C. Native Entry rows have
no UUID and cannot be selected, even in a one-row schedule. SetProperty
creates SET children, SetParam changes Co.Def (or a small attribute allowlist
in the editor helper), Remove removes by UUID, and Move changes room bindings.
None implements this operation. Existing deletion/activation and legacy
replacement contracts are unchanged.

## Newly resolved caller, units and mutation

- English resource DLL SHA-256
  `42c775360009da4e3bb18ed35d4fbfcfe70cdbcc909a78fee9e1c45d0de2ad36`,
  existing dialog 10315/language 2048/RVA `0xa74c8`, labels control 10320
  **To**. Its class is SysDateTimePick32, style `0x50010029`.
  Data exchange `0x140082a40` binds it to dialog `+0x1188`.
- Actual MFC entry `0x14161ae50` routes WM_NOTIFY/DTN_DATETIMECHANGE
  (stored code `0xfd09`) for control 10320 to **`0x140085190`**.
  This is neither activation callback `0x1400853d0` nor deletion callback
  `0x140085410`.
- Setup **`0x1400836e0`** sends DTM_SETFORMATW (`0x1032`) to both time
  controls with UTF-16 **`HH:mm`** at `0x14161b2f0`.
  The callback reads hour and minute, computes `hour * 60 + minute`, and
  imposes no 5/15-minute rounding. Minute granularity is therefore
  field/caller evidence, not an inference from stored integers.
- **`0x140084740`** enables To when a physical record is selected in
  ordinary M=0 mode. The special M=4 fixed-duration disablement is outside
  this operation.
- The exposed branch of `0x140085190` keeps the selected From unchanged.
  A new nonzero To strictly after From passes directly to
  **`0x140081a50`**; GUI 00:00 is represented as 1440. Invalid/crossed
  input instead takes native clamp/rollover branches: these are **not**
  reimplemented or exposed as zero-length/wrapped intervals.
- The setter requires selection >=0 and supplied endpoints <=1440,
  snapshots undo via `0x140081710`, and writes selected 40-byte record
  offsets `+0`/`+4`. It then invalidates the widget and posts `0x49e`;
  `0x140084aa0` refreshes the selected fields. There is no normalizer in
  this field callback. Only To changes in the qualified branch.

## Reused acceptance and conservative domain

Ownership, hit-test identity, working-copy acceptance, resolved mode-pointer
rebuild, native/RFC UUID comparison, digital defaults/count and owned
serializer qualification are reused from
[deletion](daytimer-entry-delete-native.md) and
[activation](daytimer-entry-activation-native.md), not newly qualified work.
OnOK remains `0x140084660` → `0x140081b10` → `0x140537710`;
the DayTimer branch of `0x14019de60` copies back with `0x140537650`.
The owned record writer is still `0x1405385e0`.

The normalizer can merge, sort, trim or remove records. This operation
requires already sorted `(Ix, From)` records and separated intervals,
and refuses a new To touching/crossing another row in the same mode.
From and Ix do not change, so a supported edit leaves the sort key and
record cardinality unchanged. No intermediate two-field normalization
proof is asserted. Both shrinking and extending the To boundary are useful
instances of the same actual control branch.

Digital reads force V=1 against default zero; a To edit does not affect
the default filter or N. The CLI preserves N, V spellings, NeedActivate,
Modes order/spellings, wires, unknown attributes and other rows. A positive
changed To is written as decimal minutes. No zero/default From rewriting
or whole-record normalization is done.

Ordinary WF `16384` is retained: the recovered To enablement, caller,
setter, normalizer and owned schedule writer do not gate on bit `0x4000`.
Frequency in fixtures alone is not the justification. The accepted
`0x1401b6f50` path writes a comparison diagnostic at `+0x17c` via
`0x140476890`, not WF at `+0x2e8`.
Template lock and inherited bit-29/uuidParent collateral follow the
[existing common Template qualification](irrigation-zone-name-native.md#persisted-template-state-and-admission).
Direct Template documents and template-owned ancestors are refused,
including on no-op; template locking/propagation is not emulated.

## Byte preservation and explicit limits

The same DayTimer identity/admission helper is reused. The new operation
additionally requires one exact, namespace-free, owning Document at the
root or directly below it, before no-op detection. It checks original
attributes against the admitted DOM for selection, references, intervals
and stamp metadata, then patches only selected To and Document Date/DateS.
BOM, EOL, quotes, child order and all other bytes remain unchanged.

Missing Document `Date` / `DateS` attributes are allowed and inserted on
an actual change; a missing Document itself is refused. Before no-op
detection, stamping or output, a present `DateS` must contain only ASCII
decimal digits and represent a nonnegative signed-32-bit value strictly
below `2147483647`. This leaves room for the stamp increment and preserves
valid raw spellings on no-op. The operation-local guard reuses the known
bound already enforced for input inversion; it does not change the global
stamp helper or admission for any older operation.

The shared loader sanitizer issue `lox-n37w` is not fixed here. Disagreement
in guarded storage (including digit-like attribute text) is refused.
Unrelated value-identical duplicate attributes are preserved rather than
globally rejected or serialized away; conflicting/guarded duplicates are
refused. Namespaces, DTD-dependent storage and unsupported encodings are
not admitted. A same-value request is an exact raw-byte no-op, including
noncanonical numeric spellings, but still undergoes every admission check.
All refusals occur before touching input or any new/existing save-as file.

The unmodified, ordinary digital schedule in
`tests/eval/golden/configs/Schedule-Based AC Control.Loxone` is covered
directly: its normal WF, Document metadata (including digit-named fields),
other controls, schedule rows and references are not stripped to gain
admission. This test is CLI preservation validation, not native execution.
