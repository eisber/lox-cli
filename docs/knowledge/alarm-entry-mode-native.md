# Alarm Clock entry per-mode checkbox

Static-source qualification for `config alarm-entry-set-mode`, following the
published Add, Active and Delete operations. This is one existing ordinary
entry and one checkbox, not global operating-mode management. No native
execution or GUI roundtrip was performed; two historical qualifications
remain two, with zero new qualifications.

## Reused ownership, caller and persistence

The [Active checkbox recovery](alarm-entry-active-native.md) already joins
the actual type-9 `CCheckBoxItemDelegate` event callback through role 10
(unchecked 0 / checked 2), generic model, retained column wrapper and provider
slot `+0x38` to **`0x14093d140`**. Its enabled/checkable guards, cancellable
pre-edit notification and modified/refresh notification are reused, not
recovered again. A current model row is not a persisted alarm identity.

The owner factory case `0x1d8` / type 472, `CControlObjectAlarmClock`,
owner array `+0xde8`, dialog working array `+0x90`, modal result 1-only
copy (`0x14019de60` → `0x14062b010` / `0x14062b1b0`) and serializers
`0x140631280` / `0x140631470` are also reused. Cancel does not copy the
working array back. Native project undo history is not emulated.
The [Delete](alarm-entry-delete-native.md) and
[Add](alarm-entry-add-native.md) qualifications retain their existing scope.

## Newly recovered checkbox and bit identity

- Provider type **`0x14093bd20`** returns checkbox type **9** for columns
  **3–12** on ordinary records. Eligibility **`0x14093ca00`** allows these
  columns only when the record is not internal; internal records permit
  only columns below 3. The setter has its own matching internal guard.
- Header loop **`0x14093d2b0`** assigns column `c` resource `c + 0x2c08`.
  Its omitted fifth argument is resolved by frozen instructions:
  `0x14093d494` writes **9** to `[rsp+0x20]`, and the call at
  `0x14093d4a9` forwards the loop column in `r8d`.
- For column `c`, setter **`0x14093d140`** passes **`c - 3`** to
  **`0x1406305f0`** after `QVariant::toBool`. That helper computes
  `mask = 1 << bit`, then ORs the mask for true or ANDs its complement
  for false into record **`+0x30` (`Modes`)**. It writes no other field.
- Getter **`0x14093bd60`** calls **`0x14062e0e0`**, which reads that
  same Modes bit. The decompiler omits the getter's second argument;
  frozen bytes `41 8d 50 fd` establish `edx = r8d - 3` before the call
  at `0x14093bdad`.

Thus native bit **0** is mask **1**, dialog column **3**, header resource
**`0x2c0b`**; bit **9** is mask **512**, column **12**, resource
**`0x2c14`**. All intermediate bits use the same proven mapping.
No labels were extracted or guessed. In particular these indices are
not asserted to be weekdays, global mode Ids, UUIDs or raw columns.
The CLI accepts only the precisely established numeric bit identity 0–9.

## Boolean and Active effects

In the mode branch of **`0x14093d140`**, checking a bit sets Active
(`+0x2c`) to true **only if the old Modes is zero**. After the helper,
it sets Active to false **only if the resulting Modes is zero**.
There is no other Active write in this branch.

For admitted **nonzero old and resulting masks**, both conditions are
false: checking a mode on a disabled entry does **not** reactivate it,
and clearing a nonfinal bit does **not** disable it. Only the requested
bit changes. The accepted row copy preserves these fields; the edit
does not touch date/time/name sorting keys, N, Id/U or references.
Nonzero resulting Modes bypasses the zero-mask serializer fixup.

## Final-bit boundary and supported storage

Natively clearing the final bit yields Modes=0 and Active=false in
the working record. The reused row serializer **`0x140631470`**
also contains a **post-field** Modes=0 → Modes=`0x3ff`, Active=false
fixup. Its presence does not establish that the normalized in-memory
state is what the same save writes; this operation does not infer a
writer/read-flag direction or claim a final-zero persisted roundtrip.

The CLI therefore **refuses clearing the final bit before mutation**,
whether the entry is active or disabled. It never writes zero Modes
or invents normalization. `alarm-entry-set-active --no-active` can
disable the alarm without changing its modes.

The existing typed admission is unchanged: V=175, actual consistent N,
one protected Int=true/Id=0 standard, unique non-nil native-35 stored U
and unique Ids, nonzero ordinary Ids, undated rows, strictly increasing
valid Time below 86400, nonzero ten-bit Modes, and strict booleans.
Only the selected ordinary entry is editable. RFC-36 is a request alias
only, not accepted stored U. Legacy/default repair, equal-time sorting,
internal editing and zero masks remain excluded.

Exact semantic no-ops retain bytes, lexical spellings, omitted defaults
and stamp, including save-as. A changed operation only rewrites the
selected Modes attribute, plus the normal Document save stamp;
Active spelling/absence, N/Id/U/name/time, all other rows, unknowns,
standard alarm, wires and auxiliary Ode/Le/Entry U references survive.
Auxiliary file roles and GUI roundtrip remain unqualified.
Errors precede mutation/destination writes. Existing room, save-as and
BOM/EOL-aware output contracts apply. Global dry-run is fail-closed.
