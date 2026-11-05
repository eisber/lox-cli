# EnergyManager2 offline pilot

`field-bindings.json` documents the recovered **verified subset**, pinned to
Loxone Config 17.2.10.21 and its executable SHA256. The coverage-gates manifest is
`contracts\energy-manager2.json`; it intentionally remains incomplete.

`synthetic.Loxone` and `native-cases.json` are independently authored native/CLI
cases. They contain no vendor project, credentials, household configuration or
alarm settings. They are **not GUI golden projects** and must not be counted as
official GUI roundtrips.

## Actual offline oracle evidence

The owned official GUI ran against a disposable public heating-rod sample,
disconnected from any Miniserver. Raw inputs, saved outputs, UIAutomation
observations, screenshots, comparisons, runner record and results stay under
`.reverse-engineering\ux\energy-manager2`, never in commits.

Run the coverage verifier with `--contract contracts\energy-manager2.json
--results .reverse-engineering\ux\energy-manager2\results.json --report`.
Strict mode must remain incomplete; report mode may expose genuinely verified
cases without converting unknowns into passes. Rehash result provenance after
changes to the contract bytes (including checkout newline normalization).

## Critical distinctions

- `O` is offset-grid-power **parameter**, stored in `Co[@K='O']/@Def`.
- Scalar GUI decimal comma becomes invariant XML decimal point.
- `MinRt` and `MDay` store seconds; the GUI edits integer-converted minutes.
  `RT` instead is preparation seconds, with GUI range 30–86400.
- `OIdx` binds L1–L12, not priority. Priority is ordered `Load` children.
  Reordering must preserve Id, OIdx, icon, complete record attributes and wires.
- Negative minimum runtime is actually accepted by this GUI; do not invent
  a nonnegative constraint. `SwPwr > MaxPwr` is also allowed.
- A zero switch-on edit substitutes rated power. Zero rated power is accepted
  and produces pink feedback without clearing a connected load's `U=true`.
- GUI save upgraded the public sample (`V=172` to `178`, colors and input flag
  encodings). Whole-file byte identity is **not** claimed. Reorder preservation
  was compared against the GUI-saved baseline, without stripping UUIDs.
- Storage-priority/day-cycle execution, MaxSpwr-zero runtime meaning, numeric
  overflow/non-finite handling, malformed references and structural add/delete
  remain unverified. No production upload, restart or live call was used.
