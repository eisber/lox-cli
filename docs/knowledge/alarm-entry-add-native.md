# Alarm Clock: add one ordinary entry

Static-source admission of `config alarm-entry-add`, on top of the published
[deletion](alarm-entry-delete-native.md) and [Active](alarm-entry-active-native.md)
contracts. No new native execution or GUI roundtrip qualification: the two
historical qualifications remain two.

## Actual Add caller, not metadata dispatch

Dialog constructor `0x14093be80` constructs the Add button at dialog `+0xc0`,
sets its Add icon, and connects its actual clicked signal to **`0x14093ca20`**.
The absent staticmetacall export is not invented or used as a body.

`0x14093ca20` calls **`0x140629cf0(dialog+0x90, 0x5460)`**. It wraps the
returned record in an actual `CAlarmClockEntryDataProvider`, retaining the
record at primary `+0x30`, registers the row through `0x140b60630`, retains
the provider, selects the returned row key through `0x140b6e100`, marks the
dialog modified and refreshes selection through `0x14093d660`.
Selection translates that row key to a current row with `0x140d397c0`;
neither row position nor that key is persisted as the alarm identity.

## Constructor, identity and insertion

`0x140629cf0` allocates a 0x60-byte `CAlarmClockEntry`. It initializes:

| Field | Native initial value |
|---|---|
| U (`+8`) | fresh value from `0x1404f2b00` |
| Id (`+0x18`) | smallest unused nonnegative integer |
| Year/Month/Day (`+0x1c/+0x20/+0x24`) | zero |
| Time (`+0x28`) | caller's 0x5460 = 21600 seconds |
| Active (`+0x2c`) | true |
| Modes (`+0x30`) | 0x3ff |
| Name (`+0x38`) | localized resource 0x2f93 |
| Int (`+0x50`) / transient (`+0x54`) / Sa (`+0x58`) | zero |

Id allocation starts at zero and restarts the entire existing-record scan
after each collision. It is not maximum Id plus one. The record is appended
at the actual array count through `0x1404b3b40`. Its resize helper
`0x1404b3e70` dynamically grows storage. Neither this path nor the actual
clicked callback supplies a fixed application count cap. This does **not**
establish unlimited capacity; the CLI uses checked signed count/Id
representation, in addition to its strictly distinct valid-time domain.

Native UUID allocator `0x1404f2b00` calls `0x1404f2780`: application-derived
time words, an incrementing 16-bit process counter, 0xffff and machine data.
The CLI does not impersonate that process/machine allocator. It generates
a fresh random non-nil identity, checks against all existing U attributes,
and formats the native **8-4-4-16** representation. The reused actual UUID
reader `0x1404c4ae0` accepts that 35-character grouping, not stored RFC-36;
the shared type-0x19 writer in `0x1404c52f0` emits that same grouping.
Accepted row copying preserves the allocated U; it is not left nil for a
later serializer-generated replacement.

## Proven follow-on edits, not guessed composite defaults

Unedited Add initially uses 06:00 and a localized name. Typical files already
have a standard alarm at 06:00, which would require native equal-time/name
sorting. Rather than silently invent that policy or a localized default,
the command requires an explicit name and time and follows the actual
column-one and column-two editor routes before accept:

- Provider type getter `0x14093bd20`: column one is type 0 (common delegate),
  column two type 0x1b. Value getter `0x14093bd60` supplies a QString and a
  QTime respectively.
- Factory `0x140d348d0` constructs the common delegate through
  `0x140da4620`, and the time delegate through `0x140dabf00`. Frozen
  constructor/vtable bytes resolve common create-editor to `0x140da5450`
  and its model commit to the imported `QStyledItemDelegate::setModelData`
  thunk `0x140dd1624`; no external Qt body is claimed.
  Common create-editor calls Qt's standard editor factory.
- Actual time delegate vtable `0x1415f09d8` has create-editor `+0x68` =
  `0x140dabf30`, load-editor `+0x78` = `0x140dabfe0`, commit `+0x80` =
  **`0x140dac080`**. Commit obtains the editor's QTime, builds a QVariant,
  and calls model `+0x90` with role **2**.
- The already-proven actual model `0x140d3d540` accepts role 2. Its
  `0x140d39e10` → wrapper `0x140025b40` → actual provider `0x14093d140`
  route retains the source record and column, not a guessed XML index.
- For column **1**, `0x14093d140` converts to QString and calls
  `0x14041db10` on record Name. That helper uses QString::toUtf8.
  For column **2**, it converts to QTime, checks validity, then writes
  `second + (minute + hour*60)*60` to record Time. Neither branch
  modifies Id/U/Modes/Active/Int/Sa/date fields or auxiliary references.

Only these two follow-on columns are admitted. No general new-entry field
editor, modes/day-mask constructor, internal/default migration or date editor
is inferred. Names must be nonempty single-line XML text as a conservative
CLI restriction, not a recovered native text-length policy. Explicit time
syntax is locale-independent HH:MM[:SS]; no locale display string is parsed.

## Accepted copy and persistent serializer

The owner factory/type, deep working copy and accepted-only route are reused,
not rediscovered: owner `+0xde8` → dialog `+0x90`; only modal result 1
copies back through `0x14062b010` / `0x14062b1b0`. Cancel cannot copy the new
record into the owner. Row copy retains serialized fields and resets only
transient `+0x54`.

Reused `0x140631280` sorts and writes actual N and each row through
`0x140631470`. The date/time/name comparator is `0x14062e1e0`; within this
command's undated, strictly distinct-time subset, ordering is determined by
Time alone. Equal times are rejected even when names differ. False/zero
fields use the native omission defaults; midnight Time=0 is omitted.

The shared admission checks remain unchanged for deletion and Active:
V=175, actual N, exactly one Int=true/Id=0 standard, unique native-35 non-nil
U/Ids, ordinary nonzero Ids, undated, valid strictly increasing Time, nonzero
10-bit Modes and strict boolean storage. The new row also satisfies them,
so existing typed Active/removal operations can immediately use its U.

Only one new row and N are changed. All old rows, unknown children, wiring,
Ode/Le and Entry U references are preserved; their auxiliary file role and
native GUI roundtrip remain unqualified. Existing save-as, BOM/EOL handling
and document stamp behavior are reused. Validation precedes mutation and
destination writes. No live deployment, native undo emulation or runtime
qualification is claimed.
