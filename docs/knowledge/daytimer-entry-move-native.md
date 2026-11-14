# DayTimer whole-interval mouse movement

Static recovery from Config 17.2.10.21 AMD64, executable SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
No native application, UI, simulator, save/reopen or runtime qualification
was performed. This is one fresh ordinary digital M=0, same-mode, Ctrl-up
body drag with one movement update, not arbitrary assignment of two endpoints
or a start/end API wrapper.

## Actual widget and mouse callers

`0x1400823b0` constructs the primary `CDayTimer` at dialog `+0x1d50`;
`0x140082a40` binds it to actual control **10316**. Initialization
`0x1400830a0` selects it at dialog `+0x22d0` and binds its `+0xe8` to
the working-copy data. The English dialog 10315 contains that button-class
control, style `0x50310040`, separately from From/To time fields.

Original PE bytes in constructor `0x14007efa0` install vtable
**`0x14161a868`**. Its slot `+0x60` at `0x14161a8c8` points to
`0x14007f1d0`, whose original LEA returns map `0x14161a850`.
That map's entries pointer is `0x14161a6d0`. Actual records are:

| Record | Message | Callback |
|---|---|---|
| `0x14161a710` | WM_LBUTTONDOWN `0x201` | `0x1400802b0` |
| `0x14161a730` | WM_LBUTTONUP `0x202` | `0x140080760` |
| `0x14161a750` | WM_MOUSEMOVE `0x200` | `0x140080870` |

This is not field callback `0x140084f40` or setter `0x140081a50`.
Hit-test `0x1400813b0` first matches mode and `From <= hit < To`,
returning a physical 40-byte record index with **kind 2** for body movement,
before its separate boundary-resize checks. Strictly separated rows give
one matching physical record. Down handler `0x1400802b0` stores selection
at `+0x11c`, hit minute at `+0x184`, original From/To at `+0x188/+0x18c`,
and sets `+700` to 1. With Ctrl up, it snapshots undo through
`0x140081710` without calling copy helper `0x140080650`, then captures
the mouse.

## Delta, clamp and grid are linked native semantics

Pointer conversion `0x140081310` and move handler `0x140080870` use integer
`((x + horizontal_scroll - left) * 1440) / (right - left)`.
Layout `0x140080030` sets `right - left` to 96 times its quarter-hour
pixel scale. The projected timeline is minutes, with outside-left sentinel
`-1` and outside-right value 1440, not independent HH:MM field input.

In kind 2, `0x140080870` subtracts the original hit minute from the
current projected minute, adds that delta to **both saved original
boundaries**, and writes selected From/To/Ix. Keeping the same row leaves
Ix unchanged. `--shift-minutes` exposes this signed timeline-minute delta.
The CLI fixes the grab to the selected interval's From point, a reachable
body hit; therefore its allowed delta is `-From-1 .. 1440-From`.
It does not invent a drag offset from two new endpoint fields.
The exact sequence is down, one move update, release, accept. It does not
replay an arbitrary multi-update mouse gesture: the native callback compares
against the current row before snapping, so an earlier right-edge update
that changed duration can affect a later update. The fresh down branch does
not mutate the existing record, making the comparison pair unambiguous here.

The native sequence is:

1. If proposed From is negative, set From to 0 and To to the original
   duration. Otherwise, if To exceeds 1440, set To to 1440 and From to
   `1440 - original_duration`.
2. With Ctrl up, `+700` stays 1 and selects **15-minute** snapping.
   If the clamped pair and same mode are already unchanged, no snapping
   occurs, including an off-grid original interval.
3. Otherwise snap **From**, not the delta: remainder 1..7 rounds down,
   8..14 rounds up. Shift To by the same correction, **except downward
   snapping does not change To when To is already 1440**.

For example, 02:00–03:00 shifted by 8 pointer-minutes becomes
02:15–03:15, not 02:08–03:08. A duration-eight-minute interval clamped
to 23:52–24:00 snaps to 23:45–24:00: duration becomes fifteen minutes.
Thus this operation does not promise unconditional duration preservation.
Upward snapping after a right clamp can instead put To above 1440; this
implementation refuses that result, rather than inventing a second clamp.
Positive original duration makes the native equal-endpoint repair unreachable
in the admitted body-move branch.

These are new caller semantics absent from minute-exact independent
`timer-entry-set-start/end`. The implementation directly patches the linked
result with one save stamp and one atomic destination write; it never
publishes an intermediate endpoint file.

## Release, acceptance and unchanged physical ownership

`0x140080760` calls normalizer `0x140081b10` on body-drag release.
The existing accepted path remains `0x140084660` → `0x140081b10` →
`0x140537710`, DayTimer caller `0x14019de60` → `0x140537650`, and mode
pointer rebuild `0x1404eedc0`. Owned serialization remains
`0x1407aeb80` → `0x140538450` → `0x1405385e0`. These are reused closed
facts from the [From](daytimer-entry-start-native.md),
[To](daytimer-entry-end-native.md) and
[deletion](daytimer-entry-delete-native.md) investigations.

Both original and final rows must retain strictly increasing `(Ix, From)`
order, and the final interval must stay strictly separated from all other
same-mode rows. Then release/accept normalization cannot merge or touch rows;
the analog trim branch is disabled, and `0x140081ee0`/`0x140082090`
sorting leaves physical order unchanged. Reorder, merge and count repair
are refused, not silently approximated. Digital V=1/default=0 keeps the
same serialized rows and N. Modes, IDs, NeedActivate, V spellings, unknown
metadata, wires and every unselected row remain unchanged.

The owned writer emits From/To from offsets 0/4 with integer type 5 and
zero omission. A changed From zero removes its attribute; a positive
changed From inserts or replaces it. Unchanged boundary spellings are
preserved. No persisted flag/reference field is added or inferred.

## Admission and file contract

The existing DayTimer helpers enforce exact native Type, effective native35/
RFC mode identities and unique ownership, ordinary nonzero WF including
16384, one plain owning root/direct-child Document, Template/uuidParent/WF29
refusals, guarded raw/DOM agreement and incrementable signed32 DateS.
Before timeline calculation, the move caller additionally requires the timer
immediately beneath an exact native `C Type="Page"`, either directly in that
owning Document or through one exact `C Type="Program"` directly in the
Document. Folder, alias/case, extra or incorrectly ordered ancestors are
refused before no-op, stamping or any destination write. The existing
root/direct-child Document formats are unchanged; these additional ancestry
checks apply only to movement, not the older typed boundary, removal,
activation or replacement operations.
Movement also refuses ancestor IfT state and global owner/mode
identity aliases through C/Co/Ref. Global identity attributes cannot be
duplicated. The published processing-instruction validator is reused, not
forked: legitimate prolog/PI/comments are preserved, malformed PIs are
refused before no-op/output. DTDs, namespaces and trailing element aliases
are refused. Safe unrelated raw-identical attributes remain untouched.

Only changed selected boundaries and Document Date/DateS are patched,
preserving raw BOM, EOL, quotes and unrelated bytes. Every guard precedes
no-op detection and any destination write. Effective no-ops are byte-exact;
`saved_to` is null without a write, and a no-op `--save-as` writes exact input
bytes. Missing stamps are inserted only on a real change.
The existing exclusive sibling-staging/atomic-rename writer is reused.
Input-path save-as aliases and foreign staging entries are refused without
touching input or new/existing output.

`--room`, JSON, CSV, `--quiet` and `--no-header` follow existing commands.
`--dry-run` remains unsupported and rejects before file I/O. This is an
offline edit, not deployment. Analog/M4, Ctrl copy, mode changes, new entries,
resize handles and unstable normalization are outside scope.
Shared sanitizer issue **lox-n37w** remains open: guarded disagreement is
refused locally; unrelated 12-hour metadata is preserved without a global
loader fix or raw-tuple fallback.
