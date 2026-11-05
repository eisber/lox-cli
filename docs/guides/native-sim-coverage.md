# Native offline-simulator BB evidence

`scripts\native-sim-coverage.py` imports **observed native basic-block entries**.
It does not drive the GUI, start a simulator, download tools, or contact a
Miniserver. Python 3.12+ and its standard library suffice.

## Validated collector

The initial console-only proof used the official
[DynamoRIO Windows 11.91.20756 release](https://github.com/DynamoRIO/dynamorio/releases/tag/cronbuild-11.91.20756),
asset `DynamoRIO-Windows-11.91.20756.zip`, asset ID `694132131`.
The downloaded archive matched GitHub's HTTPS asset digest:

```text
sha256:eb26ba44658cdcc3e165028da1a3fff7519805182edaead71668779ea0fe0d66
```

The release is a pinned upstream cron build, not a stable-release claim.
`drrun -version` returned `11.91.20756 -- build 0`. Its actual help documents
`-s`, `-pidfile`, `-t`, and the `--` application separator. The drcov client
**does not implement `-help`**: that probe was rejected. Its pinned upstream
[`drcov.c`](https://github.com/DynamoRIO/dynamorio/blob/cronbuild-11.91.20756/clients/drcov/drcov.c)
documents `-dump_binary` and `-logdir`; the real smoke accepted both.

The plain and instrumented owned `whoami.exe /?` console processes both exited
0 with identical stdout and empty stderr. The actual binary log had 26 modules,
11,573 records, and 1,007 target-module records. This proves only that bounded
console instrumentation worked on that machine; it does **not** prove Loxone
Config compatibility or simulator execution.

Keep the archive, digest receipt, help outputs, profile, console receipt and raw
logs private under `.reverse-engineering\native-simulator\coverage`. The notebook
collector has a separate private coverage root. Never publish vendor binaries,
module paths, recovered pseudocode, usernames, or raw logs.

An interactive runner, authorized separately by the coordinator, can construct
an argv list (not a shell command) from its verified private profile:

```python
argv = [
    profile["drrun_path"], "-s", str(timeout_seconds),
    "-pidfile", str(target_pid_file),
    *profile["args"], "-logdir", str(unique_log_directory),
    "--", str(pinned_application), str(owned_project),
]
```

`profile["args"]` is `["-t", "drcov", "-dump_binary"]`. Launcher options must
precede `-t`; subsequent options belong to the client. Verify the pinned
launcher/client/config/runtime hashes before use. The PID file identifies the
**application**, not the drrun launcher; independently check its executable and
ancestry while it is alive. Use a sanitized environment, finite timeout,
isolated offline fixture and graceful application exit so the log is flushed.
Clean up only processes owned by the run. Do not change security settings or
the application outbound firewall fence. Compare plain and instrumented native
outputs before treating a GUI capture as valid.

## Import

Only the format actually captured by that tool is accepted:

* `DRCOV VERSION: 3`, flavor `drcov`;
* Windows module table v5 with the exact standard columns, self-containing
  modules and zero file offsets;
* binary records `<IHH>`: segment-relative start, byte length, module ID.
  Windows's supported contiguous modules make these starts module RVAs.

Older v2 logs, Linux flavors, text BB tables, custom columns and segmented
modules are deliberately rejected rather than guessed. All counts, IDs, module
bounds, record sizes and exact payload length are validated. ID `65535` is the
documented unknown-module sentinel: its truncated PC is counted but never
mapped. Other undeclared IDs are errors. Limits are 128 MiB/log, 65,535 modules
and 4,000,000 BB records.

For structural inspection without any identity or coverage claim:

```powershell
python scripts\native-sim-coverage.py inspect $RawLog `
  --artifact-root $LocalArtifactRoot --output $UniqueInspection
```

For mapping, use the canonical `decompile-index.json` referenced by the private
`.reverse-engineering\latest.json`. No fresh Ghidra import or decompilation is
needed. Prefer **both** a hash-pinned local PE and a collection manifest:

```powershell
python scripts\native-sim-coverage.py import $RawLog `
  --artifact-root $LocalArtifactRoot --output $UniqueReport `
  --module-manifest $ModuleManifest --target-pe $PinnedPE `
  --decompile-index $FrozenIndex --tool-receipt $ToolReceipt
```

The manifest is created from measured collection evidence, not from guessed
module identity. Its `modules` array lists only target-image instances:

```json
{
  "schema_version": 1,
  "raw_sha256": "<actual raw-log SHA-256>",
  "modules": [
    {
      "module_id": 6,
      "path": "C:\\OwnedFixture\\owned.exe",
      "sha256": "<measured target binary SHA-256>"
    }
  ]
}
```

The target digest must equal the canonical index's `target_sha256`; the manifest
must also match the actual raw digest, exact module ID and normalized absolute
Windows path. A manifest is a collector assertion: drcov does not itself embed
a binary hash or cryptographically attest loaded memory.

`--target-pe` alone selects one matching basename, checks its SHA against the
index and matches runtime entry RVA, timestamp and checksum. Multiple matching
instances require a manifest. The on-disk PE supplies **ImageBase** independently
of ASLR. Runtime module spans may exceed on-disk `SizeOfImage`, and the actual
console log's `preferred_base` was its relocated load base; neither is silently
used as the static image base. The report exposes these differences. A smaller
runtime span or mismatched entry/checksum/timestamp is rejected.

Manifest-only imports require an explicit `--image-base`, particularly useful
for independently authored fixtures. Do not guess it for production captures.
The optional tool receipt records the official repository, pinned `release`,
archive `sha256` and identical `github_digest` (`sha256:<digest>`).

## Meaning and integration

The compact JSON report has `schema_version: 1`,
`kind: "native-basic-block-coverage"`, `application.sha256`,
`measurement.raw_log_sha256`, and `observed.target_bb_count`.
It binds the index, manifest and optional tool receipt by digest.
`artifacts[]` contains hashed relative paths beneath `--artifact-root`;
`artifact_root` is relative to the report's directory, so a fetched bundle can
be relocated intact. The raw log and both generated outputs must be beneath
that declared local root. Outputs are created exclusively, never overwritten.
Stdout contains only the report path, kind and counts.
For the joined native-feedback receipt, declare the common local repository
root as `$LocalArtifactRoot`; refs then begin with `.reverse-engineering\...`
(or `tests\ux\...` for public case definitions), not notebook host paths.

A separate private `*.bb-map.json` stores every target BB mapping and confirmed
function addresses/names, without pseudocode. Mapping uses **all exact inclusive
body ranges**, including noncontiguous bodies, converted to RVA intervals.
A function is confirmed only when the entire BB lies inside one of its exact
ranges and overlaps no other function. Partial boundary spans, holes, ambiguous
overlaps and unmapped spans remain explicit; they never become confirmed
function executions. Duplicate records are retained but are not hit counts.

`observed.executed_function_count` is explicitly **whole-executable**, not
simulator-specific. Qt stubs and initialization helpers have no inferred engine
role. `scope.status` remains `unknown`; `unknown_count` counts recognized
functions whose roles are unreviewed, not missing code. Static recovered-body
totals are not a native BB denominator. `coverage.basic_block_percentage`,
branch and source-line percentages remain `null`, and `full_engine_coverage`
remains `false` even if every observed BB maps. No static CFG denominator or
reviewed complete simulator scope exists yet. BB observations are not edges,
path coverage, or pseudocode-line coverage; `drcov2lcov` cannot manufacture those
from this index without genuine source/PDB mapping.

The next denominator step is a **targeted** retained REA
`analyze_function(procedure, detail=summary|complete)` export after runtime
simulator scope is reviewed. That existing operation provides local CFG
`basic_blocks`, exact inclusive body ranges and unresolved-call/terminal-flow
limitations. `get_call_graph(address, direction=forward|backward)` can connect
canonical procedures but is not a whole-CFG or indirect-call-completeness
guarantee. No new expensive whole-executable Ghidra import is required for this
milestone. Warning-bearing pseudocode bodies and unrecoverable-jumptable flags
are limitations, not missing native-coverage percentages or denominator proofs.

A first-feedback-loop gate may separately require nonzero **actual target**
BBs and matching plain/instrumented native outcomes. Passing that gate must
never be relabeled complete engine coverage.

Run the independently authored tests:

```powershell
python -m unittest discover -s scripts -p test_native_sim_coverage.py
```
