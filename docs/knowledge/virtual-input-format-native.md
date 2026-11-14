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
transformed input invalid. The accepted mutation helper `0x140748940` invokes
the bound string setter, which copies the Unit string and applies that same
control-byte replacement.

No Unit-specific placeholder parser is on this recovered property path.
The CLI deliberately offers only the documented bounded ASCII no-transform
subset. This does not classify every native placeholder, promise that an
arbitrary suffix is a recognized physical unit, or reproduce universal rendering.

## Default and independent state

The setter does not trim whitespace, choose a unit catalog entry, change scale
or range, or reset to a default. Constructor `0x140423fe0` initializes Unit to
`<v.1>`, but existing Display load `0x140487130` reads Unit as a string even when
the attribute is absent: missing/empty Unit becomes empty, not that default.
Missing Display is a different branch and is not admitted.

Type-15 string serialization in `0x1404c52f0` omits empty Unit when requested;
nonempty spaces are retained. The XML entity helper `0x1404c0000` decodes named
entities, not arbitrary numeric references. The CLI refuses empty/reset requests
and numeric-reference guarded metadata instead of guessing equivalence.

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
