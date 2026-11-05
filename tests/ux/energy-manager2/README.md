# EnergyManager2 offline pilot

`field-bindings.json` is the initial recovery snapshot of the **verified subset**,
pinned to Loxone Config 17.2.10.21 and its executable SHA256. Later observations
below do not rewrite that snapshot. The coverage-gates manifest is
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
cases without converting unknowns into passes. Preserve original contract,
result and runner snapshots. Changed contract bytes require a new derived
snapshot with compatible actual evidence; updating a hash alone is not proof.

## CLI-to-GUI configuration follow-up

`gui-cli-cases.json` defines a bounded configuration batch, not a complete parity
contract. Its raw evidence is under
`.reverse-engineering\ux\energy-manager2\gui-cli-followup`. The original runner
records **54 passes, 1 failure and 8 not-run cases**. The failure exposed a shared
XML writer bug: CRLF attributes accumulated literal `&#xD;` text in unrelated notes.

After fixing the writer, a fresh run under
`note-fix-recheck-20261105T085750Z` records **9 passes, no failures or not-run cases**:
the note-preservation recheck and all eight negative-input probes. Once/twice CLI
edits preserve the original note; fresh GUI saves match the original-GUI baseline.
The original failed run remains unchanged. These are separate partial records,
not a combined strict-completion result.

The batches observed all 28 deadline choices, customization-marker transitions,
icon select/cancel, dirty-cancel rollback and unused-slot reordering. Negative
fixtures used hash-pinned exact-byte fault injection into disposable official
projects, not a competing XML parser or supported CLI mutation API:

- Missing/extra slots become twelve; malformed load IDs are regenerated.
- Dangling/malformed icon references are cleared.
- Duplicate load IDs/output indexes and `tpR=28` persist after GUI save. Opening
  and saving a project therefore does not establish valid identities or bounds.
- One GUI minute-to-second overflow wrapped to a negative stored value. The CLI
  retains checked rejection; GUI filtering/overflow is not a CLI validation model.

All observed behavior is version-bound and limited to the exact recorded inputs.
No SPS runtime, physical output or full configuration-scope completion is claimed.

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
- Storage-priority/day-cycle execution and MaxSpwr-zero runtime meaning remain
  unverified. Numeric spelling/field cross-products and complete configuration
  scope still require review; the follow-up does not establish general CRUD or
  malformed-input equivalence. No production upload, restart or live call was used.
