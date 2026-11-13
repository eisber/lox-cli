# Alarm Clock: rename one existing ordinary entry

`config alarm-entry-set-name` edits the existing nested record's Name, not
the owning control's Title. Generic `config control rename` only sets Title.
The published Add/Active/mode/deletion/Time identity, owner acceptance and
physical-sort proofs are reused. There is no new native execution or GUI
roundtrip qualification; the two historical qualifications remain two.

## Actual string and validation route

Name is column **1**, type **0** (`0x14093bd20`). The common delegate's
actual createEditor (`0x140da5450`) obtains role **0x104**; model
`0x140d3b820` calls `0x140d38690`, then wrapper `0x140025c70` forwards
to provider slot 0x88, **`0x1400258f0`**. This returns a default-constructed
empty QRegularExpression, not a name-specific grammar or uniqueness check.
The actual common vtable's setModelData slot is the imported Qt
QStyledItemDelegate implementation; its body is not recovered here.

The reused role-2 write route is `0x140d3d540` → `0x140d39e10` →
`0x140025b40` → **`0x14093d140`**, column 1. The actual CDataWrapper
vtable's optional prevalidation slot 0xc8 is `0x14000c610` (false).
The generic pre-edit signal forwards through `0x140c08180`; the alarm
dialog installs no Name-specific rejection callback. Its post-edit handler
`0x14093cb40` marks the dialog modified, refreshes the row and updates
standard-alarm selection feedback. It does not rewrite other record labels
or references. This conclusion is not inferred from the deletion validator.

The setter obtains `QVariant::toString` and calls **`0x14041db10`** on
record **+0x38**. That helper calls **QString::toUtf8**, takes bytes through
`strlen` and copies them into CLxStringU. It performs no trimming,
case folding, normalization, comparison or duplicate-name scan. For valid
Unicode without embedded NUL, the CLI preserves the string exactly.
Malformed UTF-16 conversion is outside Rust string input and is not emulated.
The native setter assigns even an equal value; the CLI deliberately avoids
rewriting an equal decoded value. Equality is exact and case-sensitive:
canonically equivalent but differently encoded Unicode is still a change.
No uniqueness rule is invented; duplicate names are allowed.

## Reader, defaults, entities and empty writer behavior

Entry serializer **`0x140631470`** uses Name key **0x14160f190**,
type **0x0f**, flags **1,0,0**, field **+0x38**.
`0x140631280` read construction initializes that CLxStringU empty, not
with Add's localized resource. `0x1404c52f0` clears its temporary string,
reads the attribute, applies `0x1404c0000`, and copies it with
`0x14041d880`. Missing and explicitly empty Name therefore read as empty;
there is no localized-name fallback. On write, the omit-empty flag is
**zero**, so this serializer retains even an empty Name attribute.

The native XML tokenizer **`0x1404c2be0`** stores quote-delimited attribute
slices; **`0x1404c0750`** retrieves those raw bytes. The string reader
**`0x1404c0000`** replaces the five named XML entities, with ampersand
last, rather than applying a general numeric-reference decoder or XML
attribute-whitespace normalization. The CLI consequently guards the
selected raw Name before standard-XML normalization: numeric character
references and literal tab/CR/LF storage are refused as
`target-name-storage`. DTD-dependent/non-UTF-8 input and namespaced
target/stamp attributes are also refused rather than approximated. Escaped literal
text such as `&amp;#65;` remains ordinary text, not a numeric reference.
Raw target selection follows the existing nested entry's document ordinal,
not a potentially duplicated name or a globally unique U assumption.

## CLI subset and preservation

Requests must be nonempty, nonblank, single-line XML-safe Unicode.
Controls, U+2028/U+2029 and U+FFFE/U+FFFF are refused. This safety subset
is a CLI policy, not native grammar; no native length cap is claimed.
Clearing is not exposed even though the recovered string assignment and
serializer can represent empty Name. Missing/empty stored Name can be filled.

All existing storage admission remains unchanged: V=175, exact N, one
protected Int=true/Id=0 standard, unique nonnegative Ids, native-35 nonnil
unique stored U, ordinary selected row, undated records, strictly increasing
valid Time, nonzero ten-bit Modes, and strict booleans. Request UUID aliases
do not rewrite stored U. Since admitted Time values are distinct, changing
Name cannot affect physical date/time/name sorting; no reordering is done.

Only the selected Name and normal Document save stamp change. Id/U,
Time/Active/Modes, all other flags/unknown fields, N, standard record,
wiring, Ode/Le and Entry U references are retained. A semantic no-op bypasses
serialization/stamping entirely, including save-as as an exact source copy.
Real edits patch only the selected raw Name value (inserting Name if absent)
and Document Date/DateS values using the existing stamp calculation. All
other source bytes, including unrelated numeric-reference Name storage,
quote spelling, BOM and line endings, remain untouched. This avoids
standard-XML decoding of unselected fields during a whole-document resave.
Validation precedes mutation and
any destination write. Full native resave/model/undo behavior, immediate
proxy notifications, owner +0x238 receiver and auxiliary file roles remain
unqualified; no reference repair or label cascade is invented.
