# Irrigation: rename or clear one existing zone name

`config irrigation-zone-set-name` changes the nested `zone/@nm`, not the
Irrigation block's Title or a duration's `Co/@Def`. Existing `set-param`
only edits direct Co defaults/the documented VI attribute subset; `xml
set-attr` selectors match C elements, not zone records. This is a distinct
persisted operation, not a generic attribute wrapper.

Source qualification uses frozen Loxone Config **17.2.10.21 AMD64**,
EXE SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
Addresses below refer to the existing canonical `code/functions/0xADDRESS.c`
exports. No new native execution or GUI roundtrip qualification is claimed.
The atlas's shared empty Qt dispatcher `0x140009180` is not operation proof.

## Actual owner, row and setter

Actual factory **`0x140432790`** switches on the control type: case
**0x1cc (460)** calls **`0x1407045a0`** (lines 3348–3353), then stores
that type at owner +0x88 (lines 4183–4185). The type initializer
**`0x140706f30`** writes the literal **Irrigation** to its type record;
the same owner's vtable slot 0x370 is **`0x140705dc0`**, returning that
literal. This is not a stripped-class-name-only atlas join.

The command branch `0x14019de60:408–409` dispatches command **0x7d2** to
`0x1401b20c0`. That function constructs and executes the dialog at lines
38–41. Actual constructor **`0x140b4f2c0:38–40`** dynamically casts the
control to **CControlObjectIrrigation**, stores it at dialog +0x70 and uses
`0x140706030` to obtain its existing zones. It snapshots each zone using
`0x1407057a0`, then binds the *live* zone providers to the view (lines 75–80).
The constructor is not the atlas dispatcher.

`0x1407045a0:23` initializes the zone count to **eight**. Each CZone has
physical zero-based index at **+0x20**, UTF-8 name at **+0x28**, and owner
pointer at **+0x40** (`0x140704690:19–23`). Bounded original-PE reads bind
the constructor's RIP-relative vtable to **0x141077b00**. Its slots include:

| Offset | Target | Meaning |
|---|---|---|
| 0x00 | 0x1400258e0 | three columns |
| 0x08 | 0x140705b70 | column 0 displays V(index+1); column 1 reads name |
| 0x38 | 0x140708190 | setter |
| 0x50 | 0x140707550 | column 1 is editable unconditionally |
| 0x88 | 0x1400258f0 | common empty regular-expression validator |
| 0xb8 | 0x140704810 | destructor |
| 0xc0 | 0x140025dc0 | common CDataWrapper construction |
| 0xc8 | 0x14000c610 | default false provider hook |

This reuses the already recovered common editable-provider/delegate route
documented in [Alarm entry names](alarm-entry-name-native.md). The actual
irrigation constructor installs no name-specific grid callback or uniqueness
check. The actual shared CDataWrapper vtable **0x140ec3770**, installed
by `0x140025dc0`, has slot **0xc8 = 0x14000c610** (no extra prevalidation);
slot **0xb0 = 0x140025b40** forwards the write to the provider.
**`0x140708190:22–25`**, column **1**, converts QVariant to QString
and calls **`0x14041db10`** on +0x28. That helper uses QString::toUtf8,
strlen and byte copying (lines 28–78), with no trimming, normalization,
case folding, duplicate scan, identity or duration update. Column 2's
separate duration setter is not exposed here.

## Acceptance, empty-name side effects and output captions

Edits are live inside the dialog. **`0x140b4f750`** restores snapshots
and duration values before QDialog::reject (lines 31–46); it is cancellation,
not an accepted-copy routine. For an accepted exec result,
**`0x1401b20c0:42–44`** marks the document modified and calls
`0x140469a90`, the common dirty/connected-control notification path.

### Persisted Template state and admission

The common command gate is **not merely opaque runtime host state**.
Document reader **`0x1405288f0:486–505`** enters the direct `Template`
child, sets owner **+0x111c = 1**, and reads **IfT** into **+0x1120**.
The dispatcher gate is **`0x14019de60:256–260`**, through
`0x14050e140`/`0x1405287e0`: +0x111c nonzero with +0x1120 zero is
locked. Irrigation command **0x7d2** is not excluded by `0x140624d00`.
These fields reflect persisted Document/Template configuration.

The accepted path **`0x1401b20c0` → `0x140469a90` → `0x14050be70`
→ `0x140525ab0`** also consults IfT and can set control bit
**0x20000000** with recursive propagation. The common serializer
**`0x14047e260:663–688`** persists bit 29 and **uuidParent**.
Changing only nm in a Template document would therefore either bypass
the native lock or omit accepted-edit derived changes.

Following the existing CentralLight link admission precedent, this
operation refuses **any direct Template child**, including IfT=true,
IfT=false and absent IfT, before mutation, no-op detection or saving.
It does not attempt template locking/propagation emulation.
It also requires exactly **one plain owning Document**, either the root
itself or a direct child of the root. A nested descendant is not a
substitute: the admitted Document must be the same one selected by
`stamp_document` and the established original-byte name/stamp guards.
Missing, duplicate, namespaced or non-owning Documents are refused.
This operation never performs an unstamped edit; old Alarm/shared
stamp-helper policy is unchanged.

Opening the dialog calls **`0x140708280`**. For an empty zone name, it may
copy the name of the first connected output object through virtual +0x1a8.
Therefore this CLI requires **all seven unselected names already nonempty**.
The selected name may be missing/empty: its transient opening-time autofill
would be overwritten by the explicitly selected rename/clear. This permits
restoring a CLI-cleared name without admitting autofill of another zone.
It does not imitate autofill, disable a zone, or rename another record.

The Irrigation primary vtable **0x141077be0**, slot **0x248**, is
**`0x140705830`**: connector indices **16..23** return the corresponding
zone's +0x28 name. **`0x140d54630:42–53`** uses that virtual result when
building the displayed connector caption. This is an on-demand derived
caption, not a serializer instruction to rewrite Co keys/UUIDs, physical
actors, OutputRef titles, wires or auxiliary references.

### Reachable empty input, not a guessed empty-string reset

The new empty branch is independently source-qualified; the earlier nonempty
setter proof alone was not sufficient. The dialog's actual view initialization
`0x140b4f2c0:63` calls **`0x140b66d90`**, which connects table Paste signal
**`0x140d3f4d0`** to **`0x140b6b170`** (lines 653–671).
Actual key handler **`0x140d43310:51–55`** emits that signal for the Paste
standard key and a valid model index, without checking clipboard text.
`0x140b6b170:28–45` reads the clipboard and directly submits a QVariant to
`0x140d39e10`, or dispatches tab/newline-containing text to **`0x140b6cb50`**.
For a clipboard containing just one newline, that latter function's
do-once first-cell path trims to empty and calls `0x140d39e10` at line 91;
its empty remainder checks at lines 97–103 then stop before another column
or row. This is a reachable one-name clear, not a duration edit.
Direct single-cell whitespace is not trimmed by `0x140b6b170`.

The actual write route has no empty-name rejection: wrapper +0xc8 remains
false, the shared pre-edit signal **`0x140c08180`** is only forwarding, and
the dialog installs no rejection callback. `0x140d39e10` forwards the empty
QVariant through wrapper +0xb0 to the known CZone setter/UTF-8 assignment.
This Paste route does not depend on assuming that Qt's imported line editor
will commit an empty string. The common editor's actual setModelData and
eventFilter slots are Qt import thunks `0x140dd1624`/`0x140dd163c`;
no imported Qt implementation body is claimed.

The actual OK connection **`0x140b4f800:183–196`** uses thunk
`0x1401c10fc`, an indirect call through dialog virtual **+0x180**.
Bounded original-PE reads bind the installed dialog table **0x141559ac0**:
+0x180 is **`0x1409179c6`**, the imported `QDialog::accept`, while +0x188
is the known snapshot-restoring Cancel implementation `0x140b4f750`.
There is no name check or old-name restoration on OK.

### Empty persistence, reload and later derived names

Zone-array initialization **`0x140706db0:29–49`**, installed at primary
virtual +0xf10, creates the eight CZone records and clears each +0x28 string.
The factory's established final initialization calls that virtual.
There is no localized zone-name constructor default.
`0x140708ff0:29` passes flags **1,1,0** for nm.
Shared reader **`0x1404c52f0:75–105,169–174`** clears its temporary string,
reads/unescapes the attribute and copies it even when missing/empty
(skip-empty-read flag zero). Its writer **lines 682–686** returns without
writing nm when the string is empty (omit-empty flag one).
Thus both absent nm and `nm=""` reload as empty storage, not a proven
reset-to-default or localized-name request.

Primary virtual **+0xf20 = `0x140707680`** is called during simulation
initialization by `0x140511a60:25–31` (caller `0x14015de80:82–85`).
Like dialog opening, its lines 34–45 copy the first connected output
object's virtual +0x1a8 string only when the zone name is empty.
This is connection-dependent later derivation, not constructor/read-time
locale regeneration. No graph/reference/title extraction framework is added.
For an empty name, **`0x140d54630:45–65`** skips the custom suffix altogether;
the existing base connector caption remains. The CLI does not invent its
locale-dependent text.

Explicit `--clear` removes only the selected nm and stamps the Document.
Missing/explicit-empty selected storage is admitted for restoration by
`--name`, and clearing it is an exact-byte no-op. Unselected empty storage
continues to be refused. Requested `--name ""` and whitespace-only `--name`
remain errors; no prior nonblank request contract is weakened.

## Persistence and conservative CLI shape

The same primary vtable's **0x40** slot is **`0x140708ff0`**, called by
the common XML save/load route **`0x14047e260:717`**. It visits the existing
zone array in order, entering each **`zone`** XML child by ordinal through
`0x1404c7600`, then serializes **ioIdx**, type **5**, from +0x20 and
**nm**, type **0x0f**, from +0x28 (`0x140708ff0:26–30`).
Bounded PE string reads witness `0x141078ff0 = "zone"` and
`0x141079000 = "nm"`. ioIdx is identity, not an editable priority or a
connector's Def. The CLI requires canonical ioIdx **0..7 in array
order**, without missing, duplicate or reordered records; default creation
and migration are not qualified. The first zone's zero ioIdx may be absent:
the shared reader clears its temporary string, the type-5 path calls
`0x140422760`, and that parser returns zero for empty input. The writer's
omit-zero flag is enabled. No other ioIdx omission, empty value, sign or noncanonical numeric spelling
is admitted or normalized.

The established type-0x0f native XML string-reader evidence is reused:
five named entities, ampersand last, not general numeric references or
standard XML whitespace normalization. Original raw names, owner identity,
ioIdx and save-stamp metadata are checked using the established alarm
name-byte helper, extended with exactly the Irrigation zone storage shape.
No new generic XML editing framework or speculative XML fallback is added.
All eight names are checked in their original storage; numeric references,
literal tab/CR/LF names, DTD/non-UTF-8 input and namespaced target/stamp
storage are refused atomically.

UUID-only owner selection avoids relying on DOM-normalized/sanitized Titles.
Both requested and selected stored owner U must be nonnil **native35**
(8-4-4-16), with an exact spelling match; RFC36 storage or request aliases
are not accepted. Before `require_one`, the operation requires exactly one
raw selector match and exactly one C object with that effective 128-bit
identity across the whole file, also counting case/RFC-grouping aliases.
It must lie inside the admitted Document. The global selector's UUID
deduplication is not used as uniqueness proof and is not changed.
Raw owner Type/U and each selected zone's raw ioIdx must match admission.
The original raw nm supplies previous_name and exact case-sensitive
equality. Thus the shared digit-attribute sanitizer cannot silently turn
`Wake 7am=ok` into `Wake _7am=ok`. Its global behavior remains unchanged.
Only the selected nm and existing Document stamp are byte-patched. All
other names, physical identity, duration defaults, unknown fields,
passwords, references, BOM, line endings and source spelling remain exact.
Decoded-name no-ops, including save-as, preserve the original bytes/stamp.
The requested nonblank single-line XML-safe Unicode limit is a conservative
CLI policy, not a recovered native grammar or length cap.

This is offline editing of the admitted non-Template file shape. No live
write, full native resave, undo/history or immediate UI/proxy-notification
equivalence is claimed. The persisted Template exclusion above is required
even for offline files; it is not waived as a GUI-only host-context policy.
