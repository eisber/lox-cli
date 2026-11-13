# Alarm Clock: change one existing ordinary entry's Time

Static-source admission of `config alarm-entry-set-time`. The published
[Add](alarm-entry-add-native.md), [Active](alarm-entry-active-native.md),
[mode](alarm-entry-mode-native.md) and [deletion](alarm-entry-delete-native.md)
proofs are reused, not restarted. No new native execution or GUI roundtrip:
the two historical qualifications remain two.

Time syntax is strict `HH:MM[:SS]`, 0–86399 seconds. **Seconds are a CLI
extension over valid native QTime/storage, not qualified native GUI seconds
entry.** Native editor `0x140dabf30` uses `QLocale::timeFormat`; its
format-type argument is elided. No timezone or locale parsing policy is
inferred from the setter's arithmetic.

## Existing-record edit and the new ordering joins

The reused actual time delegate `0x140dac080` obtains the editor's QTime
and commits role 2 through model `+0x90`. The actual route
`0x140d3d540` → `0x140d39e10` → `0x140025b40` →
`0x14093d140`, column **2**, retains the existing record pointer.
After QTime validity, the setter writes only record `+0x28`:
`second + (minute + hour*60)*60`. It does not allocate or replace a row,
rewrite U/Id, or change Active/Modes/Name/date/Int/Sa/references.
The setter itself performs no array reorder; model `0x140d3d540`
subsequently emits `dataChanged`.

The ordering result is not inferred by symmetry with Add:

1. Actual owner caller **`0x14019de60`** calls **`0x14093d100`** with
   owner `+0xde8`. This helper deep-copies into dialog `+0x90` with
   `0x14062b010`, then calls **`0x140630bc0`** before building providers
   through `0x14093cf60`. Thus dialog initialization physically sorts.
2. On modal result **1** only, that caller obtains the working array with
   **`0x140558c10`**, whose entire body returns dialog `+0x90`, and copies
   back through `0x14062b010`. The copy walks the actual source sequence,
   appending one copied record per source slot; it does not sort.
   `0x14062b1b0` preserves the record's Id/U and serialized fields.
   Cancel does not copy back. This getter/copy does not establish that
   a display-row number is an identity or that proxy sorting reorders
   the working array.
3. Array serializer **`0x140631280`** calls **`0x140630bc0`** before
   writing actual N and iterating the physical records; its read branch
   also calls it after loading the records. The sort is therefore not
   merely a display/proxy sort, nor deferred until reload.
4. **`0x140630bc0`** builds a temporary array of the same record pointers
   and calls `0x14062e1e0` with destination **array+8** and bounds
   **0..count-1**. The latter is a merge-sort helper, not just a
   stand-alone comparator: it compares date fields, then Time in seconds,
   then Name on equal Time/date. Its final merge calls
   **`0x1404b3a90`** on that destination. That helper writes a pointer
   directly into the backing array's indexed slot. Thus serialization
   physically changes array order without reconstructing the records
   or renumbering their identities.

Generic notification/proxy display reordering immediately after the edit
remains unqualified; no claim that an accepted working sequence is already
sorted is needed. The persisted distinct-time result is established
independently by the serializer's unconditional sort on its write branch.
Native full save may perform other nonrequested normalizations; the CLI
does not claim whole GUI byte/model resave equivalence or emulate undo.
The owner `+0x238` receiver remains outside this operation's qualification.

## Minimal typed edit and conservative admission

The selected row must be ordinary even though the native time setter also
allows the internal row. All existing alarm-command storage admission
checks remain unchanged: V=175, actual N, exactly one protected
Int=true/Id=0 standard, unique nonnegative Ids, nonzero ordinary Ids,
strict native-35 nonnil unique stored U, undated records, initially strictly
increasing valid Time, nonzero ten-bit Modes, and strict boolean storage.
`--entry-uuid` accepts native-35/RFC-36 aliases without changing stored U.

On a real edit, only Time changes (zero is omitted using the reused
serializer default), and existing alarm records are physically ordered
by their distinct Time values. All record data travels with its identity;
non-entry XML children stay in their slots. N, the standard record's
fields, unknown nodes, wiring, Ode/Le and Entry U references are unchanged.
Their auxiliary file role and native GUI roundtrip remain unqualified.
The resulting file satisfies the other typed alarm commands immediately.

An equal requested time belonging to another row is conservatively
refused, including the standard alarm's time. This avoids inventing
native Name/locale ties; it is not a native prohibition claim. A semantic
no-op does not rewrite, reorder or update the stamp, preserving lexical
Time spelling or absence and exact original bytes; save-as copies those
bytes. Real changes use existing BOM/EOL/stamp handling. Validation
precedes mutation and any destination write. No live operation, default
repair, legacy/date migration, identity generation or reference fixup is
exposed.
