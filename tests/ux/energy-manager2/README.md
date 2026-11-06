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

The CLI now accepts all four named deadlines and hourly `01:00`..`24:00` values.
`run-cli-cases.py` exercises all 28 mappings against the actual CLI. The committed
`cli-contract.json` remains the earlier frozen CLI-only snapshot, not a live
description of current support. Derive a new snapshot from a new runner directory
with `scripts\snapshot-energy-cli-contract.py`, using unique `--contract` and
`--results` paths. That operation does not import the separate GUI observations.

## Bounded built-in icon list/select/clear

The CLI now lists the **623 loaded filled built-ins** without a project:
`lox config energy icons --output json`. Existing-load `set` accepts exact
canonical filenames/native IDs with `--icon`, or explicit `--no-icon` to remove
only that record's `ico`. It does not clean up definitions or other references.
Native and RFC ID aliases compare by exact identity, never a GUID range/prefix.
Inspection preserves raw unknown values and distinguishes known built-ins from
unresolved/malformed references; it does not claim custom payload validity.

This static catalog is archive version **20261009**, SHA256
`11656e55212ffbf0153f9075ee21da28da04f8c23d5d5e6efe945f4782391df4`,
independently versioned from the Config executable. A newer Documents archive can
override the installed archive. `docs\schemas\loxone-icons.json` contains only
factual identifiers/filenames and provenance. The frozen raw archive, localized
tags, artwork, native exports and research report remain ignored under
`.reverse-engineering\ux\energy-manager2\icon-recovery\20261105T093122Z`.
See `COMMANDS.md` for the manual standalone generator/refresh runbook; catalog
refresh does not require rerunning a decompiler or the bulk recovery pipeline.

The generator is tested with independently authored tiny ZIP catalogs, not
vendor assets. CLI/Rust tests cover exact fan selection, native/RFC case aliases,
rejected disabled/absent/malformed/empty/zero/custom selections, compound-patch
rollback, clear closure, unknown preservation and list output/provenance.
**Missing:** a fresh official GUI explicit-clear save/reopen case, verified custom
selection/import/replace/delete and full configuration/runtime scope. Exact
membership is not Qt SVG rendering proof. No custom mutation or image decoding
is enabled by this slice, and no GUI evidence is imported into its CLI-only run.
Original contracts/results and old runner directories stay frozen.

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

## Storage-power incompleteness diagnostic

`lox config energy inspect FILE BLOCK --output json` now exposes typed
`storage_power_feedback` (`missing`, `not_triggered`, or `unknown` with a reason).
The single `missing_storage_power` diagnostic follows the recovered Config
17.2.10.21 callback, for valid unique connectors:

```text
Soc connected
AND MaxSpwr unconnected, default 0
AND (MinSoc connected OR MinSoc default nonzero)
```

This tests **connections and defaults**, not whether the current Soc is below
MinSoc or what Spwr currently reports. A wired zero-valued MaxSpwr differs from
an unwired zero default. An unwired zero MinSoc does not trigger the dependency;
a connected zero-valued MinSoc does. Zero edits remain accepted. This warning
neither prohibits simulation nor establishes a universal runtime-zero sentinel.

Static proof independently decodes pinned on-disk PE instructions for
`0x140618e80`, links EnergyManager2's primary RTTI vtable slot `+0x1358`, and
recovers property ID `400 + connector index21 = 421` from `0x14042e610`.
Connector `+0x38` is the connection count; `+0x50` is `Co/@Def`.
The native reader uses `Nc`; inconsistent `Nc/In`, missing/duplicate bindings,
invalid defaults and malformed/zero source IDs are explicitly unknown. This is
not a general dangling-reference resolver or whole-block completeness check.

The ignored static proof and freshly prepared six CLI fixture hashes live under
`.reverse-engineering\native-simulator\workstreams\energy-closeout`. The first
two prepared fixture hashes exactly match the original native zero-warning and
positive-control fixtures. Their original receipts remain immutable; they are
not fresh executions or proof of the additional four countercases.
`storage-power-feedback-cases.json` declares the six native cases; its original
preregistered bytes are retained separately in the ignored workstream directory.
**No fresh feedback assertion passed.** Two GUI launches attempted the first
case: a nonunique basename hit an owned recovery question without recovery input;
the corrected unique name loaded the right public project on its cover page.
After guarded focus and invoking the exact observed Project Trees button, the
custom `LxTreeCtrl` (`24702`) exposed no UIA selection items or Energy Manager
property controls. No guessed clicks or helper changes were attempted. The five
remaining cases are not_run; both owned PIDs closed, no simulator started, and
the GUI lease was returned. A deterministic native page/block target binding
needs coordinator review before further GUI execution.

The corrected retained preflight proves actor/console session1, Default desktop,
the exact executable/backend hashes and two outbound fences on the dedicated
notebook. An earlier empty-array result was incorrectly interpreted as missing
backend/actor mismatch; that interpretation is explicitly corrected, not a
remaining blocker. The original static proof stays immutable; the contract
references its corrected derived proof and does not claim GUI parity.

The new contract feedback capability references static proof only. Earlier
capabilities and receipts are not relabeled or merged, and the overall contract
remains incomplete. Exact new contract bytes require a new actual runner/result;
old result hashes must not be patched to pretend they cover the new capability.

Focused validation, using a private isolated Cargo target and actual binary:

```powershell
$env:CARGO_TARGET_DIR = "$PWD\.reverse-engineering\native-simulator\workstreams\energy-closeout\target"
cargo test config_edit::energy_tests::
cargo clippy --bin lox -- -D warnings
cargo build
python tests\ux\energy-manager2\test_storage_power_feedback.py --binary "$env:CARGO_TARGET_DIR\debug\lox.exe"
```
