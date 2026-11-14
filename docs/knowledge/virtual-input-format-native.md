# Existing analog VirtualIn display-format edit

Static recovery from Config 17.2.10.21 AMD64, executable SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
This is an offline XML operation, not a native application or runtime pass.

## Actual property, not creation storage

Factory type `0x47` is `CControlObjectAnalogIn`, primary table `0x140ee1d58`.
Its slot `+0x12f0` points to `0x1404a9970`, which calls `0x14042bc40`.
The latter constructs `ControlSettings::CSetting` property `0x410`, kind 4:
getter `0x14044c050`, string setter `0x140474ef0`, verifier `0x14047cdd0`.
The resource IDs are `0x38c5/0x38c6`; no label-only native-map inference is used.
The Display accessor is slot `+0x548 -> 0x140449a20`, owner `+0xec8`.

Registration requires analog slot `+0x470` and a Display, and consults
`+0x7a8 -> 0x140444fa0`. For this exact table, slots `+0x960` and `+0xb80`
are constant false, eliminating its effective-owner and device-parent branches.
The separate visibility capability and its unresolved `+0xc40` graph are not
reused. The alternate read-only registration branch is excluded by requiring
absent/zero `EnoceanSubType`; the CSetting owner `+0x11c8` gate is constant false.

The real type-`0x47` descriptor in `0x1404a8a60` registers `Q` with
`0x2c000000` flags, then `Qm`. Connector analog slot `+0x68 -> 0x14045ee90`
delegates to owner `+0x498 -> 0x1404645f0` for Q; that branch reads
`Analog` at owner `+0xe2c`. `0x1404ab7e0` binds that field to XML `Analog`.
These are operation-specific capability witnesses, not constructor-default or
WF-frequency eligibility assumptions.

## Accepted string path and normalization

The setting handler `0x140748370` resolves the selected property via
`0x140746a20`; parent redirection and child companion controls are excluded.
Kind 4 in `0x140748be0` converts to string and removes newlines, then calls
`0x1407458e0`. Verification `0x14074a0a0` applies the generic string check
`0x1404d3860` and property verifier. The generic check transforms case-insensitive
`<script`, `<div`, `<style`, and exact `<!--`; transformed inputs fail verification.
`0x14047cdd0` applies `0x1404210e0`, replacing bytes 1–31 with dots and marking
transformed input invalid. Its machine predicate loads an **unsigned** byte,
decrements its low byte, and compares unsigned against 30. Bytes 128–255 do not
enter the replacement branch. The accepted mutation helper `0x140748940` invokes
the bound string setter, which copies the Unit string and applies that same
control-byte replacement.

The kind-4 GUI conversion is `0x14041db10`: `QString::toUtf8`, `strlen`,
then byte-preserving `memcpy`. No normalization or unit-catalog lookup is on
this path. The literal UTF-8 bytes for `°²³µμ€` therefore survive the verifier
and setter unchanged. In particular, micro sign and Greek mu are not aliases.
The attribute writer `0x1404bfdd0` copies the string, escapes XML syntax bytes,
and stores it using `strdup`/`memcpy`, without converting high bytes.

No Unit-specific placeholder parser is on this recovered property path.
The CLI deliberately offers only the existing ASCII suffix characters plus
those six literal Unicode characters, with the existing conservative 128-byte
limit measured in UTF-8, not characters. This is not an arbitrary-Unicode,
HTML, template, physical-unit validation, or universal-rendering claim.

## Actual GUI commit, including empty input

The concrete CSetting table `0x141030980`, slot `+0x40 -> 0x140623670`,
maps kind 4 to editor kind 0. Property-panel factory `0x140b08d60` constructs
`CSettingEditorText` via `0x140d017a0`. Its QLineEdit `textEdited` callback
`0x140d02c20` marks the editor dirty independently of its text.
`editingFinished -> 0x140d02c80` constructs a QString QVariant from the actual
line-edit text and calls `0x140b12750`, with no nonempty check.
The kind-4 path through `0x140b08830/0x140b089f0` has no combo-entry restriction.
`0x140b0f090` rejects null/invalid/unchanged variants, then dispatches the
concrete CSetting through `0x140748b80 -> 0x140748be0`.

The null guard is not an empty-string refusal. In the shipped Qt6Core.dll
(SHA-256 `b98eea394a95563879b614dbd4432ce074b0fceeb154c3bae252f4e7a9376a40`),
the QString QVariant constructor at RVA `0x10ad70` initializes a nonnull type
header independently of QString contents. `isValid` at RVA `0x5980` accepts
that header. `isNull` at RVA `0x10caf0` checks the null bit and pointer-type
flag; QString's metatype at RVA `0x982c90` has flags `0xc007`, not pointer flag
`0x800`. Thus even an empty/null QString payload is a valid, nonnull QVariant.
After deleting nonempty text, the actual callback can commit empty Unit.
The empty verifier loop performs no replacement; the setter copies empty.

`0x1406252d0` snapshots the previous value for the setting handler, and
`0x140624ba0` computes a change/default marker without writing Unit or a
persisted customization flag. Neither is a ResetToDefault callback.
The recovered Unit setter does not change StateOnly, Mult, or another Display
field. GUI undo/notifications and native whole-document resaving are not
implemented by the CLI.

## Default and independent state

The setter does not trim whitespace, choose a unit catalog entry, change scale
or range, or reset to a default. Constructor `0x140423fe0` initializes Unit to
`<v.1>`, but existing Display load `0x140487130` reads Unit as a string even when
the attribute is absent: missing/empty Unit becomes empty, not that default.
Missing Display is a different branch and is not admitted.

Type-15 string serialization in `0x1404c52f0` omits empty Unit when requested;
nonempty spaces are retained. The XML entity helper `0x1404c0000` decodes named
entities, not arbitrary numeric references. The CLI refuses numeric-reference
guarded metadata instead of guessing equivalence.

Explicit `--clear`, mutually exclusive with FORMAT, follows that proven
empty-input branch. A changed clear removes only the selected Unit attribute
and updates the Document stamp. Already absent/empty Unit is an exact-byte
no-op after all admission checks. It does not construct a new Display or
restore `<v.1>`, and does not claim inherited/default-reset semantics.

`StateOnly` is property `0x413`, separate setter `0x140474b50`, Display `+0xc`.
The Unit setter and selected accepted string path do not invoke it. Its existing
absent/true/false spelling is preserved, not forced true from CLI creation code.
The analog serializer calls the actual Display table's first slot,
`0x140487130`, which writes Unit and StateOnly independently.

## Deliberate boundary

The writer patches only Unit and the admitted Document stamp, not the native
whole-document resave. Namespace/template/instance/device/inherited-child
contexts, noncanonical identities/layouts, other flag states, and shared
digit-name sanitizer rewrites are refused before no-op or save-as. No visibility,
rights/reference recomputation, hardware change, deployment compiler, or global
formatter is implemented.

Raw admission also independently requires exactly one balanced, complete XML
root, rejecting hidden trailing controls/documents and out-of-root text/CDATA
before a no-op or write. Outside-root XML whitespace/comments/processing
instructions and inside-root CDATA remain supported. This check is local to
the format operation; it does not change the shared loader.
