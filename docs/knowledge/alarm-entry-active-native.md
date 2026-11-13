# Alarm Clock entry Active checkbox

Static-source qualification for `config alarm-entry-set-active`, independently
following the published deletion operation. No native execution, GUI roundtrip
or new runtime qualification was performed. The two historical qualifications
remain two.

## Actual checkbox caller to persistent record

The existing owner/type binding and accepted-copy/serializer route are reused
from [deletion qualification](alarm-entry-delete-native.md), not retraced:
factory case `0x1d8`, `CControlObjectAlarmClock`, owner array `+0xde8`,
`CAlarmClockDlg` working array `+0x90`.

- `0x14093d2b0` creates column zero with delegate type **9**. The fifth
  argument is omitted by the decompiler's inferred helper signature; frozen
  instructions at `0x14093d310` store 9 to `[rsp+0x20]` before the call
  at `0x14093d325`. `0x140b60ab0` forwards that argument to `0x140d33a00`.
- `0x140d426a0` installs per-column delegates selected by `0x140d34800`.
  Factory `0x140d348d0`, case 9, constructs `CCheckBoxItemDelegate`
  through `0x140da99f0`. Its actual vtable `0x1415efde8`, slot `+0x90`,
  points to checkbox event callback **`0x140da9ad0`**.
- The callback requires enabled/checkable flags and a valid check-state
  value, reads Qt role **10**, and handles checkbox mouse events and keys.
  Toggle helper **`0x140da9a30`** maps checked 2 to unchecked 0 and
  otherwise to 2, calling the model's `+0x90` with role 10. Explicit
  plus/minus key branches in the callback similarly write 2/0.
- Actual generic model constructor `0x140d3afe0` sets vtable
  `0x1415cf610`; its `+0x90` is **`0x140d3d540`**. That accepts only
  roles 2/10, calls `0x140d39e10`, and emits `dataChanged` on success.
  `0x140d37410` resolves the current source row and column to its cell;
  this is not a persisted XML ordinal.
- `0x14093cf60` exposes undated records through providers whose primary
  `+0x30` points at the working alarm record. Registration `0x140b60630`
  builds `CEditableDataProvider::CDataWrapper` cells via `0x140025dc0`.
  Each wrapper retains its provider and column identity.
  Wrapper vtable `0x140ec3770`, setter `+0xb0`, is **`0x140025b40`**;
  it forwards the retained column to provider slot `+0x38`.
  The actual provider secondary vtable `0x1411be5e0` resolves that slot
  to **`0x14093d140`**, not a name-only candidate.

For column zero, `0x14093d140` converts the supplied QVariant with
`toBool` and writes record **`+0x2c` (Active)**. If and only if the
new value is true and **`+0x30` (Modes)** is zero, it sets Modes to
`0x3ff`. False does not clear or repair Modes. Getter `0x14093bd60`
returns a bool; `0x140d35b10` maps that to checkbox state 0/2.

## Validation, cancellation and side effects

Provider eligibility `0x14093ca00` requires a record and permits ordinary
columns; even internal alarms permit columns 0–2 natively. This command
intentionally supports **ordinary entries only**, not internal-standard
editing. Wrapper eligibility `0x140025b90` forwards that predicate.
Generic edit `0x140d39e10` initializes a successful result via
`0x14060b960`; this wrapper's optional prevalidator `+0xc8` is
`0x14000c610` (false), so there is no alarm-specific prevalidation prompt.
The generic pre-edit notification may cancel an edit, before the setter.

Post-edit notification follows `0x140b6a7b0` / `0x140b6a4f0`.
The dialog's actual `0x14093cb40` connection marks it modified and
refreshes selection/button state; `0x140b69aa0` only refreshes model
data. There is no additional persisted alarm/reference mutation in the
column-zero setter. This operation does not expose or emulate native
project undo history.

The reused dialog accept validator returns true. Only modal result 1
copies the working array back (`0x14019de60` → `0x14062b010`);
Cancel cannot change the owner's array. Row copy `0x14062b1b0` preserves
Active/Modes/Id/U and resets only nonserialized transient `+0x54`.
The array serializer `0x140631280` writes its unchanged count N and rows.
For already strictly ordered undated records, changing Active cannot
affect the date/time/name sorting keys.

Row serializer `0x140631470` passes Active to shared `0x1404c52f0`
as type 0 with omit-false enabled. That helper reads absent values as false,
writes `true` for true, and omits false. The CLI therefore removes Active
when disabling a previously true entry, while exact semantic no-ops retain
the original bytes, attribute presence/spelling and stamp.

`0x140631470` separately normalizes zero Modes to `0x3ff` and Active
to false after processing the fields. This is **not** the enable-only
setter branch. The command rejects zero Modes in every row rather than
performing load normalization, default repair or an activation migration.

## Deliberately bounded storage

Both operations share the unchanged published admissibility constraints:
V=175, consistent N, one existing Int=true standard with Id=0, unique
non-nil **native-35 stored U** values and unique Ids, ordinary Ids nonzero,
undated rows, valid strictly increasing Time, nonzero 10-bit Modes, and
strict true/false-or-absent boolean storage. RFC-36 is accepted only as a
CLI UUID argument alias, never as stored XML. Neither deletion's domain nor
the supported storage is widened.

Within this subset, only the selected entry's Active changes. N, all Id/U
spellings, Modes, other fields and rows, the internal standard, wiring and
unknown nodes remain unchanged. Auxiliary Ode/Le attributes and Entry U
children are preserved without reference repair; their file role and
native GUI roundtrip remain unqualified. Errors occur before mutation or
destination writes. Actual saves use existing BOM/EOL-aware output and
Document stamping; save-as retains the original file.
