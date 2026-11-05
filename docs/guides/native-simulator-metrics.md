# Official simulator: recovery and validation metrics

The official offline simulator is the oracle. `lox-sim` is a separate
implementation, not a substitute for native observations.

## Separate the denominators

| Layer | Measurement | Completion evidence |
|---|---|---|
| Recognized executable | Exported function bodies / recognized nonexternal functions | Version-bound export index and research database |
| Simulator scope | Reviewed engine, clock, dispatcher, handlers and dependencies | Exact module identities, function ranges, classification evidence and unresolved boundaries |
| Native execution | Executed basic blocks and intersected function ranges | Genuine recorder logs mapped to the matching binary and analyzed ranges |
| Static coverage | Executed static blocks / reviewed static blocks | Compatible block definitions and an explicit, reviewed CFG denominator |
| Behavior | Required scenarios with native observations / required scenarios | Case definitions, fixture identity, actual output samples and retained evidence |
| CLI parity | Implemented and differentially verified capabilities / required capabilities | Implementation, focused tests and matching native behavior/validation/feedback |

These are not interchangeable. Exporting every recognized function does not
recover original source, establish correct pseudocode, identify the simulator,
or validate every behavior. Recorder blocks are not branches or pseudocode
lines. Counts of intersected functions do not mean those functions were fully
executed.

The current corpus baseline for Config **17.2.10.21** has **53,088** exported
bodies and **7,339** external classifications. **12,685** bodies contain warning
comments; **6,758** contain unrecovered-jumptable/indirect-jump warnings.
These text flags include thunks and are not an incorrect-code percentage.
Simulator-owned function and static-block denominators remain **unknown**.

## Immutable evidence joins

Run the scorecard from the repository root. Output paths must be new; old
successes and failures are never overwritten.

```powershell
python scripts\official-sim-scorecard.py baseline `
  --output .reverse-engineering\native-simulator\baseline-NEW.json

python scripts\official-sim-scorecard.py compare `
  --artifact-root . `
  --receipt .reverse-engineering\native-simulator\plain-1.json `
  --receipt .reverse-engineering\native-simulator\plain-2.json `
  --receipt .reverse-engineering\native-simulator\instrumented.json `
  --coverage .reverse-engineering\native-simulator\coverage.json `
  --output .reverse-engineering\native-simulator\scorecard-NEW.json
```

The first-loop gate requires two distinct plain receipts and at least one
instrumented receipt for the same application, case definition and fixture.
All must report successful native execution and identical output samples at
the same named phases. Actual elapsed times are retained; this comparison does
**not** establish full timing or trace equivalence.

Every artifact is hash-checked beneath the explicit local artifact root.
Observations must cite actual native-readable, UI Automation or OCR evidence.
An instrumented receipt must bind a raw log whose matching analysis reports
nonzero target execution and mapped functions. Duplicate receipts, altered
artifacts, mismatched application hashes and unsupported sources are rejected.
Blocked/unreadable output is not a pass.

Each case retains its definition/fixture hashes and matched per-recording
basic-block/function counts. Those counts are not added into a fictitious
coverage percentage. Exit status is `0` for a successful baseline or established
loop, `1` for valid evidence with an unestablished loop, and `2` for invalid input
or an unwritable output.

The gate measures whether a bounded feedback loop works. It always leaves
`engine_complete: false` and unknown coverage percentages as `null`. Hashes
establish byte association, not proof against a dishonest evidence producer.
Raw projects, recorder logs, screenshots and vendor code remain private under
`.reverse-engineering`; only independent tooling and synthetic protocols belong
in Git.

## Turning traces into recovered functionality

1. Compare controlled startup-only and simulation executions to nominate
   candidate functions. A trace difference is a lead, not proof of engine
   ownership: GUI work also changes when simulation starts.
2. Record loaded-module identities and classify exact function ranges as
   engine, clock, dispatcher, block handler, UI adapter, dependency or unknown.
   Keep unresolved indirect calls and unclassified modules visible.
3. Use REA `analyze_function(procedure, detail="complete")` for selected
   functions. It already exposes local CFG blocks, assembly, pseudocode,
   references and limitations. `get_call_graph` provides canonical
   caller/callee relationships, but is not an indirect-call-complete CFG.
   Do not repeat the whole-program import merely to obtain selected CFGs.
4. Recover state layouts, input/output routing, clock semantics, transitions
   and validation predicates into independent behavioral contracts. Add
   boundary, invalid-input, timing, reset and multi-block interaction cases.
5. Replay native cases, implement the CLI/model behavior and compare actual
   outputs. Keep counterexamples and reduce failing fixtures without removing
   the original evidence.

Only publish a coverage fraction when its denominator and omissions have been
reviewed. Never shrink scope to improve a percentage. A release update changes
the binary identity and requires fresh mapping and native replay; old evidence
remains historical.

All desktop work uses the dedicated notebook and owned offline Config process.
No production connection, OS clock change or recurring analysis job is needed.
