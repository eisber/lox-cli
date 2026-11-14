# DayTimer existing-interval From boundary

Static recovery from Config 17.2.10.21 AMD64, executable SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
No native application, UI, save/reopen, simulator or runtime qualification
was performed. This is one new start-boundary operation, not a second API
for replacement, deletion, activation or the existing end-boundary edit.

## Independently recovered From branch

The public baseline `52156dc948390fb395a3df351553d7f60510097d` provides
`timer-entry-set-end`, removal, activation and legacy `timer-schedule`.
The latter replaces all entries; generic C attribute/property/parameter
commands cannot select an Entry. Delete/re-add loses its physical identity,
metadata and order. None preserves an existing entry while changing only From.

The English resource dialog 10315 has label 10317 **From** followed by
SysDateTimePick32 control **10318**, style `0x50010029`. Data exchange
`0x140082a40` binds it to dialog `+0x10a0`. The actual original-PE MFC
record at **`0x14161ae30`** routes WM_NOTIFY, DTN_DATETIMECHANGE
(`0xfd09`), control 10318 to **`0x140084f40`**. This is a different record
and callback from To (`0x14161ae50` → `0x140085190`).

Setup `0x1400836e0` sends DTM_SETFORMATW (`0x1032`) with **HH:mm**
at `0x14161b2f0` to both time controls. The From callback independently
reads `+0x10a0` and `+0x1188`, obtains hours/minutes and computes
`hour * 60 + minute`, without rounding. `0x140084aa0` populates these
controls from the selected 40-byte record's From/To; stored To 1440 is
displayed as midnight.

`0x140084740` enables From for a selected physical record, and enables To
for ordinary M=0. In **this enabled-To branch**, new From strictly before
the nonzero displayed To reaches `0x140081a50` with To unchanged. Displayed
To zero is restored to 1440. From's computed value is not remapped:
**From 00:00 stays zero**, independently of the To midnight convention.
Crossing/collapsing From can adjust To; disabled To can preserve duration.
Neither coupled branch is exposed.

The selected setter `0x140081a50` snapshots undo, writes record `+0`
and `+4`, invalidates the widget and posts `0x49e`. Its qualified caller
supplies unchanged To, so only From changes. The field callback does not
invoke schedule normalization.

## Acceptance, sorting and zero-default serialization

Selection, accepted working-copy ownership and the ordinary digital defaults
are reused from [deletion](daytimer-entry-delete-native.md) and
[end editing](daytimer-entry-end-native.md):
`0x140084660` → `0x140081b10` → `0x140537710`, accepted DayTimer caller
`0x14019de60` → `0x140537650`, resolved mode rebuild `0x1404eedc0`,
and owned writer `0x1407aeb80` → `0x140538450` → `0x1405385e0`.
No new acceptance or serializer ownership claim is inferred from a string.

The From change affects a sort key, unlike the end edit. Accordingly the
normalizer and sort are checked for the proposed branch: `0x140081b10`
cannot merge strictly separated same-mode rows; the analog trim branch is
disabled. `0x140081ee0` sorts using `0x140082090`, comparing Ix and then
From. Requiring both original and proposed rows to retain strictly increasing
`(Ix, From)` order, and strict separation from all same-mode neighbors,
keeps every physical row, count and order unchanged.

The actual entry writer `0x1405385e0` passes record `+0` with the original-PE
ASCII key **From** (`0x140ffcd3c`) to `0x1404c52f0`, type 5, attribute=1,
omit-zero=1, preserve-missing=0. That helper's writer case 5 returns without
emission for zero. Thus a changed midnight start **removes** an explicit
From attribute; a changed positive start inserts one if previously omitted.
The existing integer reader's missing value is zero. This is writer evidence,
not a constructor/default guess. An unchanged effective zero preserves
explicit raw zero spellings or omission rather than normalizing the file.
Digital V=1/default=0 keeps all records in the writer's N/default filter;
the strict From<To guard also avoids the reader's equal-endpoint repair.

## Admission and preservation

The implementation reuses DayTimer entry selection and the end operation's
owning-Document, Template/WF29/uuidParent, raw/DOM agreement, mode identity,
signed-32-bit DateS and byte-patching checks. Ordinary nonzero WF 16384
remains admitted by the same source-qualified caller/accepted/writer facts.
It additionally ensures one balanced, namespace-free root, rejects trailing
element aliases/DTD storage, and refuses duplicate guarded identities.
Safe unrelated value-identical raw attributes are preserved.

Only selected From and Document Date/DateS are patched, retaining quotes,
BOM, EOL, unrelated attributes, modes and UUID spellings, To, N, NeedActivate,
V, wiring and all unselected entries. Missing stamps may be inserted on an
actual edit. All admission runs before no-op detection or any destination
write. Present DateS must be nonempty ASCII digits in `0..2147483647`
(exclusive upper bound), allowing one increment. No-op save-as copies exact
bytes; no-write JSON reports `saved_to: null`.

The shared sanitizer issue **lox-n37w** remains open and unfixed. Guarded
raw/DOM disagreement, including digit-like quoted text, is refused; unrelated
legitimate 12-hour metadata is preserved. No general raw-XML fallback,
preview mechanism, analog conversion, nonzero-M mode, wrap/clamp, reorder,
merge or native runtime behavior is implemented or qualified.
