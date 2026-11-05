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
Each must retain a distinct original native `execution.run_id`. Appending an
OCR admission does not create another execution, and two analyses of one
capture cannot count as independent repeats.
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

### Bounded file-only output recognition

`scripts\official_sim_ocr.py` supplies `recognize_literal(image_path, box,
profile, new_output_directory)`. It never accesses a desktop or takes an
expected scenario value. The caller must establish the actual simulator phase,
owned screenshot, header anchor and correct value rectangle.

The initial German pilot uses privately extracted Tesseract **5.4.0.20240606**,
Pillow **11.3.0**, and the official `tesseract-ocr/tessdata_fast` German model
at revision `87416418657359cb625c412a48b6e1d6d41c29bd`. Both executable and
model are hash-pinned in a private `native-tesseract-ocr-profile`. Pillow is
an optional runtime dependency; missing tooling explicitly blocks recognition.
No global tool installation or image upload is required.

One frozen preprocessing recipe removes green wire pixels, takes minimum RGB,
enlarges 4x bicubically and adds a white border. German single-line recognition
must produce exactly `Ein` or `Aus` with confidence at least 90. Other words,
multiple words, clipping, low confidence and missing identities are blocked;
there is no fuzzy correction or default-OFF inference. Confidence is an OCR
engine score, not a calibrated probability.

Actual positive native captures yielded `Ein` above 95, including an independently
wired reference and a separate capture. Some negative glyphs yielded exact
`Aus` below 90 and therefore remain **unadmitted**. This is a bounded positive
pilot, not complete ON/OFF readback coverage. Source image, rectangle, processed
image, raw TSV, tool/model hashes and observer versions are retained.

### Computer-use and vision models

Computer-use models can locate controls, select useful crops, suggest native
zoom actions and diagnose dialogs that accessibility APIs cannot expose.
They are advisory observers, not replacements for native simulation or
automatically trusted pass/fail judges.

Candidates reviewed against official sources:

| Candidate | Relevant use and limitation |
|---|---|
| [Qwen3-VL](https://huggingface.co/Qwen/Qwen3-VL-4B-Instruct) | Local vision, multilingual OCR and PC GUI grounding; first candidate when weights already exist |
| [UI-TARS](https://github.com/bytedance/UI-TARS) | Screenshot-based native GUI agents; actions still need the owned-process driver guards |
| [Fara1.5](https://github.com/microsoft/fara) | Open-weight screenshot-based agent family; current model cards focus on browsers, not verified native Qt behavior |
| [OmniParser](https://github.com/microsoft/OmniParser) | Structured screen-element detection/captioning for navigation; no guarantee of exact tiny-value transcription |

Benchmark on blind native crops without providing expected scenario values.
Include both state words, blank, clipped and wrong-region controls; retain exact
match, false-state decisions, abstention and latency. Pin the model/runtime and
keep local inference loopback-only for private captures. A model's self-reported
confidence is not equivalent to the OCR engine score. Model suggestions alone
must not promote blocked native evidence to a pass.

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
