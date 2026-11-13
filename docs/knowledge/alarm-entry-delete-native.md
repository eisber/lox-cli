# Alarm Clock configured-entry deletion

Static recovery only: Config 17.2.10.21 AMD64, PE SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
No native execution or additional native qualification is claimed.

## Concrete route

- Factory `0x140432790`, case `0x1d8` (472), constructs
  `CControlObjectAlarmClock`, primary vtable `0x140edf5e0`, with
  `CAlarmClockArray` at `+0xde8`. This is not a class-name-only binding.
  The existing registry builder `0x1404ba670` calls the same object's
  slot `+0xf18`: here `0x14062d410`. It unconditionally writes
  `AlarmClock` to descriptor `+0x18`; its final `0x1404bea20` changes
  only descriptor `+0x104`. The builder registers that descriptor by
  numeric type and string. XML reader `0x140487c90` looks up Type and
  calls the factory with descriptor `+0x10`; writer `0x14047e260`
  obtains the numeric descriptor through `0x1404493a0` and writes Type.
  The atlas's older conservative factory extractor left this complex
  registration function unjoined; the operation uses the concrete chain.
- `0x14019de60`, branch `0x1d8`, dynamically casts to that actual owner,
  constructs `CAlarmClockDlg` through `0x14093b5e0`, and loads the
  owner's array through `0x14093d100` (deep copy, then sort).
- Metadata names dispatcher `0x140c086b0`, but its body is not exported.
  Actual constructor helper `0x14093be80` independently proves the
  deletion callback: the delete-icon button at dialog `+0xc8` is connected
  directly to `0x14093cb80`; its shortcut clicks the same button.
- `0x14093cb80` obtains `selectedRows`, maps model indices to stable
  row keys with `0x140b69820` (including proxy mapping), then obtains
  each provider through `0x140b66970`. The provider's `+0x30` points
  to the actual copied alarm record, as established by `0x14093cf60`.
  This is **not the XML ordinal**. Only records whose `+0x50` (`Int`)
  is zero can be deleted. `0x14093d660` applies the same condition to
  button enablement. Dated records (`Year != 0`) are not shown.
- Pointer-identity helper `0x14062e880` destroys the matched array record
  and removes precisely one pointer through `0x1404b3960`; it does not
  renumber survivors, create a replacement, or impose a minimum count.
  The callback removes the corresponding provider and marks the dialog
  modified. An empty selection or protected record produces no deletion.
- Dialog accept `0x140930dc0` calls virtual validator `+0x1a8`.
  The actual dialog vtable `0x1411be6c0` resolves this to
  `0x14092fd80`, a thunk returning true: no minimum/default repair here.
  Only modal result 1 causes the caller to deep-copy the accepted array
  back with `0x14062b010`; Cancel leaves the owner unchanged.
- Owner vtable slot `+0x40` is `0x140631150`, reached by
  `0x14047e260`. It serializes the same `+0xde8` array through
  `0x140631280`. The array sorts and writes `N` from its actual size;
  entry vtable `0x141032b08`, slot `+0x10`, is `0x140631470`.
  Fields are `Id`, `Name`, `Active`, `Time` (seconds), `Year`, `Month`,
  `Day`, `Modes` (10-bit mask), `Int`, `Sa`, and `U`. Surviving Id/U
  values are copied unchanged; transient `+0x54` is reset by the copy
  helper but is not among those persisted fields.

## Defaults, references, and admitted subset

`0x140629e50` initializes an **empty newly constructed owner**, creating
an internal standard alarm and an ordinary disabled alarm. This is not an
OnDelete last-row replacement. `0x14062a1c0` creates/repairs the protected
standard alarm (`Int=true`, `Id=0`, Time=21600), possibly relocating an
ordinary Id=0 and updating its supplied reference. The configuration reader
`0x140631150` invokes that migration only for versions below 155.
Neither deletion nor accepted copying invokes it.

Stored alarm `U` values must use native **35-character 8-4-4-16** grouping.
The native UUID reader `0x1404c4ae0` requires length `0x23` and dashes at
positions 8, 13 and 18. A stored RFC 36-character spelling fails that reader,
leaving a newly loaded alarm's UUID zero; `0x140631470` then generates a
replacement UUID. Accepting stored RFC spelling would therefore overclaim native identity
preservation. The CLI rejects it in **any configured alarm row of the selected
owner**, including a survivor or the internal standard, before mutation.
Both native and RFC spellings remain accepted as `--entry-uuid` argument
aliases only; the validated comparison key does not rewrite stored spelling.

The same owner's auxiliary serializer slot `+0xf28` is `0x14062e670`,
which also writes alarm-ID references `Ode` (`+0x1030`) and `Le` (`+0x10dc`),
with constructor defaults of -1, and invokes `0x14062e4a0` for `Entry`
children keyed by `U`. At runtime, `0x14062a3b0` assigns these owner
references from the current alarm's Id, without renumbering that alarm.
OnDelete and accepted-array copying do **not** touch these owner references
or auxiliary `Entry` children. Where present in the edited XML, `Ode`,
`Le`, and `Entry U` are preserved unchanged even if they reference the
removed alarm. This is not dangling-reference repair or a promise of
reference validity. Their auxiliary file role and native GUI roundtrip remain
unqualified; no reference repair or additional refusal is invented.

`alarm-entry-remove` deliberately supports V=175, an existing unique
internal standard alarm with Id=0, unique non-nil native-35 stored entry UUIDs and Ids,
undated entries, valid second-of-day times already in strictly increasing
order, nonzero 10-bit Modes, and explicit true/false or absent flags.
This avoids legacy migration, missing-UUID generation, Modes=0
normalization, dated/invisible rows, and locale-sensitive same-time/name
sorting. These are CLI qualification boundaries, not invented native
minimum-count or deletion rules.

The CLI identifies one existing record by `U`, accepting native
8-4-4-16 and RFC 8-4-4-4-12 **argument** aliases case-insensitively. It removes
only that child and decrements N. Deleting the final ordinary alarm leaves
the original internal standard alarm and N=1, **not a new default**.
It refuses deleting Int=true, malformed/ambiguous storage, and a missing
entry without writing either input or save-as destination. No dangling
reference repair is invented: surviving Ids, UUIDs, Modes, Dev and wiring
remain untouched, as do unknown attributes and sibling nodes. The existing
save path updates document save stamps and retains BOM/line endings.

Clear-all, alarm creation, dated/legacy/default repair, live alarm APIs,
and configurations lacking the protected standard record are not exposed.
