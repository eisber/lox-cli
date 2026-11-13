# DayTimer selected-entry deletion

Static recovery from Config 17.2.10.21 AMD64, PE SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
No native application execution was used. This is a bounded implementation,
not a claim that every DayTimer variant or overlapping schedule is supported.
The implemented operation is restricted to ordinary **digital** schedules.

## Concrete operation and ownership

- Factory `0x140432790`, case `0x170`, constructs DayTimer through
  `0x1407ac800`; its inline data at `+0xe50` uses the constructor
  `0x140536770`. The old XML/string leads `0x1407ad5b0`/`0x1407aee90`
  initialize connectors; they are not deletion evidence.
- The DayTimer-specific branch of `0x14019de60` opens `CDayTimerDlg`
  (`0x1400823b0`) with a working copy. The widget hit-test
  `0x1400813b0` matches the mode-vector index and time interval, returning
  a physical 40-byte record index; `0x1400802b0` stores that selection.
- Actual Delete-key caller `0x140083000` checks focus/button enablement.
  The dialog's MFC message map at `0x14161ae90` also routes
  `WM_COMMAND`, control 10321, to `0x140085410`. The latter requires a
  nonempty record buffer, snapshots undo state (`0x140081710`), and
  removes precisely the selected 40-byte record via `0x1404b3880`.
  A negative selection instead clears the entire buffer: **not exposed**.
  Enablement `0x140084740` tests nonempty, not a minimum of two entries.
- OnOK `0x140084660` calls normalizer `0x140081b10`, then working-copy
  writer `0x140537710`. The DayTimer caller copies back only on modal
  result 1 using `0x140537650`; `0x1404eedc0` rebuilds resolved mode
  pointers without removing the serialized mode list.
- DayTimer vtable `0x140efc218`, slot `+0x40`, resolves to
  `0x1407aeb80`, which dispatches inline data slot `+0x30`.
  Data vtable `0x140ffccd8`, slot `+0x30`, resolves to `0x140538450`,
  whose entry reader/writer is `0x1405385e0`. Thus this is the accepted
  DayTimer object's serializer, not an unrelated same-string lead.

## Storage consequences and conservative boundaries

`From`/`To` are minutes; `Ix` indexes the ordered UUID vector `Modes`.
There is no entry UUID in this record layout. Normalization sorts by `Ix`
then `From` (`0x140082090`), so XML ordinal is not stable identity.
The CLI resolves the mode UUID and exact interval, preserving XML order.

`0x1405385e0` derives `N` from serialized records, excluding default-valued
records for ordinary `M=0`. Integer fields use type 5 and omit-zero;
the generic reader `0x1404c52f0` clears the read string and converts it
with `0x140422760` (empty string becomes zero). `M` preserves missing
storage over constructor zero. Analog false forces each read entry's
value to 1; the constructor's default value is zero. Thus every digital
entry survives the ordinary default-value filter, irrespective of the
stored V spelling; deleting the last leaves the native zero default.
The CLI preserves surviving V spellings instead of normalizing them.

Analog true is deliberately unqualified: the recovered floating conversion
helper `0x140422510` loses its FP return in decompilation. Analog
default-valued-row filtering and its effect on N must be qualified before
extending this command. This is a source-proof dependency, not evidence that
the native operation or an analog setter is unsupported.

`0x1400830a0` resolves Modes UUIDs before editing, clearing entries on a
missing reference. The CLI instead rejects missing/duplicate/non-Mode
references and bad indices before mutation. Selected-row deletion leaves
Modes/default untouched, including when removing the final record. The
writer then omits zero N and emits no Entry. No last-entry replacement
or minimum count occurs in this recovered branch.

Native acceptance can merge, sort, split, or trim other rows. This CLI
requires separated intervals per mode and rejects other storage instead
of copying incidental normalization. It preserves unknown attributes,
survivor IDs/flags/conditions, unrelated schedules and parent wiring.
It does not implement clear-all, mode deletion, nonzero M variants,
malformed-count repair, or a new dry-run pathway.

Fresh native validation, if separately authorized: create synthetic
ordinary digital timers with two modes and separated entries;
select one interval, press Delete, accept, save, reopen, and compare the
selected record/count, surviving order/attributes, default, mode UUIDs,
and wiring. Repeat with the last entry and Cancel. No such execution or
runtime qualification is claimed here. Analog qualification is separate work,
not a prerequisite or approval wait for this supported digital operation.
