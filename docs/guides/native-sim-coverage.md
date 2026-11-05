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

The pinned stock drcov client has **no live flush-only nudge**. Its public
[`event_nudge`](https://github.com/DynamoRIO/dynamorio/blob/cronbuild-11.91.20756/clients/drcov/drcov.c)
accepts argument `1` as a process-exit request; the exit callback invokes
`drcovlib_exit()`, which dumps the table. `drcovlib_dump()` exists as a public
library API but is not exposed by this stock client's nudge interface. Do not
guess other nudge arguments or call private application functions.

The installed `drconfig -help` documents a PID-specific candidate for the
single-client profile: `drconfig -nudge_timeout 5000 -nudge_pid <owned-target-pid>
00000000 1`. This requests **exit**, not a snapshot followed by continued
simulation, and is not equivalent to normal application shutdown. An initial
PS1 smoke was blocked by the existing script-execution policy; that policy was
not changed or bypassed. After explicit authorization, a normal managed-Python
native-argv smoke on an owned Session-0 loopback `PING.EXE` process verified this
exit path: the nudge and launcher returned 0, the nudge took 0.145 seconds, and
the held target handle confirmed exit without forced cleanup. A zero-byte
live log became a strictly validated
103,299-byte log with 12,484 records and 156 target-module BBs.

The verified `bin64\drconfig.exe`, version `11.91.20756 -- build 0`, matched its
entry inside the digest-pinned official archive:

```text
sha256:a6daed6ea20e4a00f6c76460c77e648a0ae1de6c9ac9839c7b9e5658a8ce08d3
```

The smoke checked the target's image hash, process creation time, Session 0,
and launcher lineage while holding its process handle. The nudge had an
independent eight-second subprocess timeout in addition to
`-nudge_timeout 5000`. Raw logs, strict local inspection, receipt and method
profile are private evidence, not simulator coverage. This console success
does not establish that the same exit path works in a blocked GUI process.
Any separately authorized runner must validate actual results, bound the wait,
verify the current owned target PID, and never use process-name-wide or
all-process nudges. A zero-byte or truncated file is a failed capture, never
substitute coverage.

## Bounded Windows startup compatibility investigation

The following are **documented candidates, not proven Config fixes**, verified
against the installed 11.91.20756 help and that release's public source. Do not
silently replace the original collector profile; preserve each variant's argv,
hashes, measurement interval and native-output comparison.

1. **Separate core injection from the coverage client.** Upstream
   [debugging guidance](https://github.com/DynamoRIO/dynamorio/blob/cronbuild-11.91.20756/api/docs/debugging.dox)
   explicitly recommends running without a client. A bounded diagnostic argv is
   `[drrun, "-s", "60", "--", exe, project]`. Success here is not coverage; failure
   helps distinguish a DR-core/injection issue from a drcov-client issue.

2. **Preserve diagnostics instead of an unreadable DR notification.** The
   archive already contains the Windows x64 debug runtime. A separate diagnostic
   launch may put `-debug -msgbox_mask 0 -stderr_mask 15 -loglevel 1 -logdir
   <DR-log-directory>` before `-t drcov`; put the client's own `-logdir
   <BB-log-directory>` after `-t`. Redirect stderr to an owned file and retain
   both log directories. `-loglevel`/`-logmask` require the debug runtime.
   `-msgbox_mask` controls DR notifications only, not client `dr_messagebox()`
   calls or arbitrary application dialogs. Do not suppress assertions, lower
   checks, or change OS security/mitigation settings to make a run appear valid.

3. **Attach after a plain, owned process has initialized.** This is supported
   experimentally on **Windows**, not just Linux: the pinned
   [Windows deployment section](https://github.com/DynamoRIO/dynamorio/blob/cronbuild-11.91.20756/api/docs/deployment.dox)
   gives a running-process example, and the public Windows
   [`dr_inject_process_attach()` API](https://github.com/DynamoRIO/dynamorio/blob/cronbuild-11.91.20756/core/lib/dr_inject.h)
   and [frontend implementation](https://github.com/DynamoRIO/dynamorio/blob/cronbuild-11.91.20756/tools/drdeploy.c)
   agree. For the matching x64 launcher:

   ```python
   argv = [
       drrun, "-s", "90", "-attach", str(verified_owned_pid),
       "-takeover_sleep", "-takeovers", "32",
       "-t", "drcov", "-dump_binary", "-logdir", str(unique_log_directory),
   ]
   ```

   There is no `-- exe project` tail: the application already exists.
   Actual Windows help warns that blocking syscalls can prevent takeover;
   `-takeovers` defaults to 8, and `-takeover_sleep` sleeps one millisecond
   between attempts. The value 32 above is bounded experimental tuning, not an
   upstream compatibility promise. An **independent outer watchdog is still
   required**: Windows attach uses an unbounded debug-event wait internally
   before normal run-time limiting is reached. Only the GUI owner may initiate
   this on its verified owned PID in the interactive session. Record successful
   takeover and coverage's post-attach interval; startup execution is absent.
   No claim is made for older DR releases or every Windows build.

4. **Do not confuse `-late` with post-GUI attach.** The same release documents
   `-late` for new launches. Its Windows implementation selects thread-based
   injection rather than the default mapping path; it does not wait for Qt
   initialization or an arbitrary simulation phase. `-attach` is the documented
   existing-process mechanism.

5. **Qt exclusions are a risky diagnostic, not a coverage guarantee.** The
   pinned public
   [`native_exec_list` option](https://github.com/DynamoRIO/dynamorio/blob/cronbuild-11.91.20756/core/optionsx.h)
   accepts a **semicolon-separated** module-name list before `-t`, for example
   `["-native_exec_list", "Qt6Core.dll;Qt6Gui.dll;Qt6Widgets.dll"]` only after
   checking the actual dependency names. Native execution is enabled by default
   on x86/x64, but upstream explicitly warns of issues and limited testing.
   It does not remove DR injection/private-loader initialization. Calls back
   from excluded modules can miss instrumented application code:
   `native_exec_retakeover` defaults to false and
   [callback-retakeover completeness remains an open issue](https://github.com/DynamoRIO/dynamorio/issues/978).
   Therefore do not infer missing engine behavior from absent BBs, label this
   an official Qt fix, or turn this into a full-engine denominator. No
   verified Qt-specific cure was found in the bounded public search.

### If DR remains incompatible

Microsoft's official
[TTD command-line recorder](https://learn.microsoft.com/en-us/windows-hardware/drivers/debuggercmds/time-travel-debugging-ttd-exe-command-line-util)
supports user-mode Windows launch and PID-specific `-attach`, `-noUI`, explicit
`-out`, bounded `-maxFile`, and PID-specific `-stop`. It is a feasible separate
native-recording experiment after a missing-tool probe, provenance/license
review and coordinator approval—not a fallback already validated here.
The notebook probe found no TTD/WinDbg AppX packages, commands on PATH, or CDB
at the common SDK debugger locations. No new tools were installed.

TTD needs elevation, can incur roughly 5–20x or greater recording overhead,
and produces private memory-rich traces and large replay indexes. Stopping
recording does not unload TTD from the target; close the owned application at
the end. Keep the capture interval short, validate native outputs under
recording, and retain real recorder diagnostics. If it is incompatible with
existing protection settings, stop; do not disable those protections.

TTD is **not drcov format**. The current importer must reject it. A future
validated replay/address exporter and identity/RVA adapter would be needed;
instruction or call observations must not be relabeled BB/branch/line coverage
without establishing their actual units. Existing Windows WPR/ETW CPU sampling
can help diagnose hangs, but samples are not exhaustive BB execution coverage.

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

For post-init/post-start collection, the manifest may additionally contain
`capture`: `instrumentation_start` (`post-init` or `post-start`),
`startup_coverage: false`, `simulation_start_coverage` (necessarily false for
post-start), `interval_utc: {start, end}` with timezone-aware timestamps,
`native_case_status` (`pass`, `blocked`, `failed`), `native_running_verified`,
`run_id`, `cleanup_kind`, and nonempty `source_artifacts` refs (`path`, `sha256`).
Source refs must be relative to the declared local root and match actual files.
The importer preserves this hash-bound context as `measurement.capture` and
includes its evidence in `artifacts[]`. Interval bounds do not timestamp
individual BBs. Nonzero mapped BBs do not upgrade a blocked native case or
establish that simulation advanced after attachment.

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
