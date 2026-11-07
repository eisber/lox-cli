# Version 1 UX evidence contract

`ux-contract.schema.json`, `ux-results.schema.json` and
`ux-runner-record.schema.json` define the wire format. The stdlib verifier applies
these schemas and the semantic gates below; producers must not invent another
interpretation. Unknown keys/types, duplicate JSON keys/IDs and nonfinite numbers
are errors. `synthetic-example.json` demonstrates shape only, not a real contract.
`synthetic-runner-record.example.json` demonstrates an explicitly synthetic,
not-run record. Neither example establishes any Loxone behavior.

## Producing a contract and observations

1. Pin `application.version` and the SHA256 of **the executable bytes**. Use the
   numeric `control_type` and exact `block_type`; names are not fuzzy identities.
2. Declare `scope.status` (`unknown`, `incomplete`, `complete`) and describe the
   reviewed surfaces/conditions and omissions. `complete` requires GUI source
   evidence; a catalog export cannot establish the denominator.
3. Give each capability a stable ID and kind (`parameter`, `nested_record`,
   `condition`, `validation`, `feedback`). `binding.status` is `known`, `unknown`
   or `ambiguous`. Only known bindings have a `path` (XML storage notation).
   Keep unknown bindings and omission distinct from defaults, null or zero.
4. Preserve source evidence separately as `catalog`, `native` or `gui_oracle`,
   with artifact paths and SHA256. No raw vendor payload or production config
   belongs in a committed manifest. Keep such artifacts ignored/local.
5. Define cases independently: stable `id`, `capability_ids`, human-readable
   `condition`, `procedure`, `expected`, and exact `evidence_kind`
   (`implementation`, `unit_test`, `gui_oracle`). Link required cases from each
   capability's `required_case_ids`; links must be reciprocal. **Each capability
   requires all three levels**, with distinct cases for actual CLI implementation
   proof, executed unit tests, and official GUI oracle observations. Describe omitted
   values, boundary/invalid inputs, nested rows, cross-field rules and visible
   feedback where applicable. Do not reduce them to one generic "works" case.
6. An **actual runner** records its ID, runner identity, timezone-aware timestamps,
   application identity, `synthetic` boolean, `kind`, `operation` and
   `observations`. Observations use the result case fields: `id`, `outcome`
   (`pass`, `fail`, `not_run`), `evidence_kind`, `actual`, `artifacts`.
7. Results pin `contract_sha256` (exact manifest bytes), repeat block/application
   identity, copy runner fields into `run` and reference its `record` artifact.
   `cases` must equal the record's observations by ID. Passes need nonempty
   artifacts. Missing observations remain missing, never implied passes.

Hash files **after** saving them (`Get-FileHash -Algorithm SHA256`, lowercase the
output). Byte hashes change with line-ending normalization/reformatting; regenerate
results when the manifest changes or is checked out with different line endings.
Do not hand-author a passing runner record to make this gate green.

Artifact references are `{ "path": "relative\\artifact.json", "sha256": "64 lowercase hex" }`.
Both slash styles are accepted relative to `--root`; absolute paths, traversal
and symlink escapes are rejected. The runner record is JSON, not a fabricated
pass certificate. `kind=gui_observation` and `synthetic=false` are necessary for
GUI verification. Here `synthetic` describes **execution**, not input provenance:

| Execution | Input project | `run.synthetic` / record `synthetic` | GUI evidence |
|-----------|---------------|--------------------------------------|--------------|
| Actual official Config GUI run | Safe generated/synthetic or public sample | `false` | Valid for observed conditions |
| Mocked/simulated/fabricated run | Any input | `true` | Never verified |

GUI runs **must** use disposable synthetic/generated or public-sample projects,
not production input. A real GUI run on such a project can establish Loxone
behavior; a mocked execution cannot. Unit tests here fabricate execution records
to test acceptance/rejection and never run Config, including positive gate tests.
No `fixture_origin` field is needed: describe the safe project in `operation`
and reference it in artifacts. GUI source references must include the actual
record and at least one passed GUI case artifact.

### Rust integration shape examples

These illustrative capabilities are **not** recovered real-block bindings.
Use `kind=parameter` for a scalar and `kind=nested_record` for a child list;
an ordering obligation belongs in independent case `condition`/`expected`,
not an invented `ordered_child_list` kind. The shape below uses explicit unknown
source evidence and cannot establish parity:

```json
{
  "capabilities": [
    {"id":"scalar","kind":"parameter","binding":{"status":"known","path":"Example/@Value"},"source_evidence":[],"required_case_ids":["scalar.roundtrip"]},
    {"id":"loads","kind":"nested_record","binding":{"status":"known","path":"Example/Load"},"source_evidence":[],"required_case_ids":["loads.order-roundtrip"]}
  ],
  "cases": [
    {"id":"scalar.roundtrip","capability_ids":["scalar"],"condition":"Changed scalar","evidence_kind":"gui_oracle","procedure":"Edit Value in a disposable project; save and reopen in official Config.","expected":"The serialized attribute and reopened GUI retain the changed value."},
    {"id":"loads.order-roundtrip","capability_ids":["loads"],"condition":"Ordered child-list replacement","evidence_kind":"gui_oracle","procedure":"Set two distinct Load records to order B,A; save and reopen in official Config.","expected":"Serialized children and reopened rows retain B,A order and row identity; unrelated children are preserved."}
  ]
}
```

This is a fragment: wrap it with `schema_version`, `application`, `block_type`,
`control_type`, and explicit `scope` from `synthetic-example.json`. Known bindings
in real manifests require recovered evidence; the example names are fictional.
The fragment declares only GUI cases and is intentionally partial: add separate
required `implementation` and `unit_test` cases for **both** capabilities before
claiming CLI parity. Do not relabel a GUI observation to fill those missing levels.

## Gate and reporting

```powershell
python scripts\verify-ux-contracts.py --contract contracts\synthetic-example.json --report
python scripts\verify-ux-contracts.py --contract contracts\energy-manager2.json --results .reverse-engineering\ux\energy-results.json
python scripts\verify-ux-contracts.py --contract contracts\energy-manager2.json --results .reverse-engineering\ux\energy-results.json --inventory .reverse-engineering\ux\block-ux-coverage.json --report
```

JSON output reports diagnostics, cases/conditions, individual capabilities and
stage counts (catalog, native, known binding, implementation, unit tests, GUI,
complete). Stage denominators are **declared capabilities**, not GUI field counts,
native function counts or compiler coverage. Empty denominators yield `null`,
never 100%. Overall percentage stays `null` unless scope is verified complete.
Each capability needs verified source, known binding and required cases covering
**implementation AND unit_test AND gui_oracle**. **All** required cases must pass
and verify at their exact evidence level (and all declared GUI cases must verify).
Missing required levels produce explicit diagnostics, even if optional cases at
that level passed. Stage counts remain independent: complete GUI observations
can coexist with zero implementation/unit proof and **zero CLI-complete parity**.
Unit/implementation passes cannot substitute for GUI roundtrips, and GUI passes
cannot substitute for CLI implementation or executed unit tests.
Unknown/incomplete scope, missing/failed/not-run cases, mocked GUI execution,
stale identities/contract bytes or mismatching artifacts prevent completion.

An actual combined runner may execute and record all three stages using the
existing schema; record each stage's genuine operation/outcome and artifacts.
Existing GUI-only results remain valid partial oracle evidence. Neither missing
CLI stages nor not-run cases may be filled with fabricated results.

The verifier accepts **one result per exact contract snapshot**, not a union of
independently executed runs. `tests\ux\energy-manager2\cli-contract.json` is a
separate incomplete CLI/unit subset: precise producer definitions and omissions
are retained, original GUI case definitions remain required but missing, and
only catalog/native recovery sources are attached. Its results copy a genuine
CLI runner's observations unchanged. Existing GUI contract/results remain
independent and untouched. Fresh combined GUI/CLI/unit proof against one reviewed
snapshot is needed later; copying old GUI passes into a CLI runner is not proof.
`scripts\snapshot-energy-cli-contract.py` derives this narrow snapshot from
digest-checked immutable producer sources and refuses to overwrite snapshots.
Choose the reviewed run explicitly with `--run-directory`, supply a new local
`--results` path, and optionally override `--contract` for historical snapshots.
Pre-review records remain historical evidence tied to their own source bytes;
they must not be presented as proof of subsequent fixes. Final integration
requires a fresh post-review run and its immutable case-definition snapshot.

The composed post-review CLI snapshot uses run
`energy-cli-20261105T090457-7fad3e6d`, whose immutable definition SHA256 is
`09fc48b371891154911283bf839d8887ded79b5c5b194dc6f4a3102d2445ae7d`.
Its local results are
`.reverse-engineering\ux\energy-manager2\cli\cli-contract-results-20261105T090457-7fad3e6d.json`:
38 unchanged CLI/unit observations cover 19 narrow capability mappings;
15 original GUI case definitions remain missing. Native-ID, underflow and
simulator-input regressions are retained in the actual producer definitions.
This is partial evidence only, not a merged or complete UX-parity result.

Exit codes: `0` complete; `1` valid input but strict gate incomplete; `2` invalid
input/schema/artifact/identity. `--report` returns `0` for honest, structurally
valid partial reports, but never hides invalid input (`2`). The default target is
Config 17.2.10.21, executable SHA256
`7b55e3ef18401574ab967c5c6a0140cd5a66dc728521ebba04ad53093577d8b3`.
Change targets only with both `--application-version` and `--application-sha256`.
`verify_files(contract_path, results_path=None, root=..., application=None)` is the
same producer/integration API. Inventory enrichment is read-only: output retains
metadata and adds verified evidence; unmatched/ambiguous entries stay unverified.
Unmatched/ambiguous supplied contracts fail the strict gate. The top-level gate
covers supplied contracts only; the entire catalog remains incomplete and has no
GUI percentage, even if every supplied contract passes.

## Trust boundary

Digests prove artifact integrity/association, **not** that a GUI was executed or
that the producer's assertions are true. Review the actual runner operation,
original observations and artifacts. Complete scope is a reviewed claim tied to
GUI evidence, not automatically discovered by this verifier. A lying runner,
omitted surface or badly chosen expected outcome remains outside this gate.
Passing narrow cases does not establish behavior beyond their named conditions,
nor runtime/Miniserver safety. No live server operation is part of verification.
