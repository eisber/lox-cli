# DayTimer existing-interval Activation required

Static qualification of Config 17.2.10.21 AMD64, PE SHA-256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
No native application, UI, runtime, or save/reopen experiment was performed.
This operation edits one existing ordinary **digital** interval's configured
`NeedActivate` flag; it is not enable/disable, runtime activation, or insertion.

## Newly resolved field-specific joins

- MFC map entry `0x14161ae70` routes `WM_COMMAND` control 10324 to
  `0x1400853d0`. Data exchange `0x140082a40` binds this control to the
  dialog's `CButton` at `+0x1270` (HWND at `+0x12b0`).
- Existing English resource DLL SHA-256
  `42c775360009da4e3bb18ed35d4fbfcfe70cdbcc909a78fee9e1c45d0de2ad36`,
  dialog 10315, language 2048, resource RVA `0xa74c8`, has control 10324
  labelled **Activation required**, style `0x50010003`. Its low type bits
  are `BS_AUTOCHECKBOX`, not a three-state checkbox. The callback sends
  `BM_GETCHECK` (`0xf0`) and passes its result directly to `0x1400819c0`:
  the ordinary checkbox states are 0/1, without an invented Boolean adapter.
- The already-inspected setter guards the selected physical record index,
  snapshots undo state at `0x140081710`, and writes the supplied 32-bit
  value to selected 40-byte record offset `+0xc`.
- The existing owned serializer `0x1405385e0`, ordinary `M=0` branch,
  reads `NeedActivate` into that same offset using type 0 and flags
  `(attribute=1, omit-zero=1, preserve-missing=0)`. On write it calls the
  helper only for a nonzero flag, so false is omitted.
- Generic helper `0x1404c52f0` clears the read string before lookup.
  Its type-0 reader compares with `TRUE` at `0x140ff3bb4` using
  `0x14041eb10` (`_stricmp`). Thus any case of true reads as 1;
  absence, empty, false, and other nonmatching text read as 0.
  The writer uses lowercase `true` at `0x1416147f8`; its omit-zero
  branch also skips false. The CLI intentionally accepts only absent or
  case-insensitive true/false, rejecting other explicit spellings rather
  than silently using the native nonmatching-text fallback.

## Reused ownership, identity and count proof

Selection, acceptance/copy-back, serializer ownership, digital defaults,
and native/RFC UUID equivalence are reused from
[the deletion qualification](daytimer-entry-delete-native.md), not newly
recovered operations. Acceptance is `0x140084660` → `0x140537710`, with
DayTimer caller `0x14019de60` copying back via `0x140537650`.

The serializer derives `N` from record count and the record's value versus
the default, never from `NeedActivate`. In the qualified ordinary digital
branch the reader forces V to 1 and the default is zero; changing the flag
does not add/remove/reindex records or require recounting N.

Both CLI operations share the same exact mode-UUID-plus-interval selector,
reference validation, native count checks, and separated-interval domain.
Native 35-character IDs and RFC aliases compare without rewriting request,
Modes or Mode.U spellings; alias duplicates remain ambiguous.
Analog, nonzero M, touching/overlapping intervals, count repair, clear-all,
mode removal and acceptance normalization are not added by this operation.

Actual edits preserve every field except selected `NeedActivate` and the
normal Document save stamp. Semantic no-ops deliberately retain original
bytes, including an explicit false or mixed-case true. These are CLI
preservation policies, not claims of native full-file serialization parity.
