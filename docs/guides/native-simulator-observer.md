# Bounded passive native entry observer

## V5 entry-only prototype and current admission status

The current independently authored V5 client uses `drmgr` BB insertion at exact
selected application entry PCs and `dr_insert_clean_call(..., save_fpstate=true)`.
It reads `dr_get_mcontext` and bounded `dr_safe_read` samples; it has no drwrap
dependency, post hooks, return-address replacement or context writes. The
source/build/profile and console artifacts are separately versioned; V4
DLL/profile/receipts and their schema remain immutable.

V5 profile metadata is `backend: "drmgr-entry"`, `client_version: 5`,
`supports_return: false`, `supported_observations: ["ENTRY"]`,
`format_schema: "passive-entry-observer-jsonl-v2"`. It requires the pinned
launcher, drconfig, runtime, client and drmgr; it does **not** require drwrap.
Controller and notebook preflight reject every `post=1` configuration for V5.
`read_config` retains its flat identity/budget/site API.

Native entry replay for **every backend, including V4**, requires literal `late_attach_verified: true`
in the pinned profile **before staging or application launch**. Profile
creation defaults this field to `false`; missing, false, numeric or string
values cannot admit native replay. No native-late-admitted V4 or V5 profile currently exists.
An immutable unadmitted profile copy may materialize `false`, but existing
proof profiles must not be rewritten. Console/file-only diagnostics remain
available with unadmitted profiles; V4 log parsing/console diagnostics and
plain/drcov behavior are unchanged. Existing V4 profiles are diagnostic-only
under the native launcher because vendor TLS failures are confirmed.

Schema 2 HEADER declares that backend/version and `supports_return=false`.
SITE records contain `selected` and `instrumented`, **not `wrapped`**.
Selection means an exact validated target RVA; instrumentation means a clean
call was emitted there. A selected unexecuted site may retain
`instrumented=false`; an observed ENTRY must refer to an instrumented site.
RETURN records are invalid for schema 2. Schema-1 V4 parsing remains separate.

The new synthetic fixture checks real registers/caller/prefixes plus XMM0
arguments/results, XMM6–15, nonvolatile GPRs, a live x87 sentinel, MXCSR and
main/native-timer-thread static TLS. Its normal-launch, bad-read, budget and
nudge-exit checks passed. **Two late-attach console attempts failed**, the
diagnostic repeat retaining actual owned target and attacher exit codes
`0xc0000005`. Thus **V5 is not console-admitted or native-ready**. No vendor
V5 GUI case is established. The underlying late-attach transparency failure
remains unresolved; a normal-only proof cannot substitute for the required
late-attach gate. No protection/security setting was weakened.

The remainder documents the preserved V4 drwrap interface and schema-1
evidence; it must not be interpreted as V5 RETURN support.

## Preserved V4 backend

Reproducing V4 requires its retained source snapshot, not the current source
tree. The current CMake project builds V5 and its independently authored MASM
sentinel fixture with the existing MSVC assembler.

`tools\native-sim-observer` is an independently authored DynamoRIO `drmgr` /
`drwrap` client. It samples **actual selected main-module entry contexts**:
RCX, RDX, R8, R9, RSP, the safely read stack return address, thread ID, site ID,
entry PC and module base. It never invokes a vendor function, changes an
argument/result/context, or writes application memory. No GUI operation,
private vendor ABI, process-wide memory scan or environment dump is implemented.

This evidence is **not** drcov, a call graph, BB/branch/line coverage, fixture
UUID attribution or simulator parity. Register roles and object/frame layouts
remain unknown until separately established. A sampled first pointer is only a
**vtable candidate**, not an assertion that the register identifies an object.
No UUID search, decoding heuristic or nearby-GUID binding is implemented.

## Public tool provenance and build

Use the already verified official DynamoRIO Windows **11.91.20756** archive:

```text
eb26ba44658cdcc3e165028da1a3fff7519805182edaead71668779ea0fe0d66
```

The implementation was grounded in that archive's public `samples\wrap.c`,
`ext\include\drwrap.h` and `include\dr_modules.h`, plus the pinned public
drcov nudge/exit example described in [native-sim-coverage.md](native-sim-coverage.md).
The sample's allocation/result-mutation behavior is **not** reproduced.
No return-address replacement, function replacement or context-setting API is
used. Return hooks are opt-in and may miss exceptional/unmatched returns.

Use an existing x64 MSVC environment and CMake; no tool installation is required.
Keep environment setup and build in the **same** `cmd.exe` process:

```text
call "<installed VS>\VC\Auxiliary\Build\vcvars64.bat"
cmake -S tools\native-sim-observer -B .reverse-engineering\native-simulator\observer\build -G "NMake Makefiles" -DCMAKE_BUILD_TYPE=Release -DDynamoRIO_DIR="<verified SDK>\cmake"
cmake --build .reverse-engineering\native-simulator\observer\build
```

Put the pinned SDK's `ext\lib64\release\drmgr.dll` and `drwrap.dll` alongside
the resulting observer DLL. Keep all SDK files, compiled DLLs, console fixtures,
profiles, memory-rich logs and receipts in ignored private storage. The profile
creator verifies fixed upstream launcher/runtime/extension hashes and records
the independently compiled client hash.

## Console proof, not a GUI admission

The console-only CLI requires the authorized notebook's **Session 0**. Its
independently authored `observer_fixture.exe` exports two no-inline functions.
Known synthetic object/frame markers and fixture IDs have a known layout;
this says nothing about vendor object layout. The fixture independently reads
its own caller address with `_ReturnAddress()`, allowing exact observed
return-RVA comparison rather than merely checking an address is in the image.

```powershell
python scripts\native-sim-observe.py profile `
  --tool-root "<verified DynamoRIO root>" `
  --client "<owned observer directory>\native_sim_observer.dll" `
  --output "<new private profile>.json"

python scripts\native-sim-observe.py console-proof `
  --profile "<private profile>.json" `
  --fixture "<owned observer directory>\observer_fixture.exe" `
  --output "<new private proof directory>"
```

This checks actual register values, caller return RVA, stack alignment, readable
object/frame/vtable prefixes, explicit address-1 rejection, known raw clock RAX,
event and unique-pointer limits, exact plain/observed synthetic stdout equality,
and bounded nudge exit/complete flush. It also starts a **normal owned fixture**,
witnesses its initialization, attaches the pinned public launcher with an
independent **30-second** deadline, then lets the synthetic fixture make known
calls. This validates pre-existing-module iteration and registration deduplication.
An owned file trigger is used only by the synthetic fixture; it is not a vendor
execution mechanism. Every console helper uses `CREATE_NO_WINDOW`, sanitized
environment, finite waits and held process handles. PID, image hash, creation
time, session and parent lineage are checked; cleanup is exact-handle-only.

Successful console evidence does **not** establish Config compatibility, output
parity, clock parity or UUID-to-instance attribution. Only the GUI owner may run
the separately authorized post-start experiment. Vendor startup/SimulationStart
must remain excluded; do not replace the proven native-only late-attach recipe
with launch-time instrumentation.

## Integration API (no replay changes)

Load `scripts\native-sim-observe.py` with `importlib.util`; it has no import-time
Windows API calls or execution. The principal functions are:

| API | Responsibility |
|---|---|
| `read_pe(path)` | Read-only AMD64 identity, section/export metadata and file SHA-256. |
| `validate_profile(profile)` | Check launcher, drconfig, runtime, extensions and client bytes against pins. |
| `config_bytes(pe, client_sha256, sites, max_events=256, max_objects=64, prefix=64)` | Generate bounded ASCII config; validate selected executable-section RVAs. |
| `launch_argv(profile, config_path, log_path, config_sha256, pidfile, *, attach_pid=owned_pid, seconds=40)` | Pure argv construction; **does not** validate/adopt/launch a process. |
| `parse_log(path, expected=None)` | Strict bounded JSONL parser; require complete SUMMARY, wrapped sites, coherent reads/return associations and optional expected identities/sites. |

`launch_argv` also supports `application=...` for the synthetic launch proof.
The vendor integration must use **attach**, never that launch alternative.
The caller must validate and retain its owned live process handle, creation
identity, image pins, session/desktop and independent attach/watchdog deadlines.
Hash-check the actual config bytes and all tool components immediately before
execution. Revalidate the exact same live process before any nudge. Never
interpret a launcher PID as the application PID.

Each site is `{"id": integer, "rva": integer, "post": 0|1, "mask": 0..15}`.
Snapshot mask bits `1/2/4/8` select **only** RCX/RDX/R8/R9 as read addresses.
Unselected registers are still recorded but not dereferenced. Set `post=0`
unless a return hook is separately justified; synthetic clock success alone
does not prove a vendor clock hook safe.

The exact late-attach shape is:

```python
[
    profile["drrun_path"], "-s", "40",
    "-attach", str(verified_owned_pid), "-takeover_sleep", "-takeovers", "32",
    "-c", profile["client_path"],
    str(config_path), str(new_log_path), config_sha256,
]
```

Use native subprocess argv, never shell concatenation. The client iterates
pre-existing modules after registering module-load events, deduplicates the
same matching module instance, and wraps only selected RVAs in the main image.
It checks basename, PE timestamp and SizeOfImage; SHA-256 declarations are
independently checked by the owner, not computed over live memory by the DLL.
The newly created **zero-byte log** signals client initialization after this
iteration, not successful native behavior or finalized evidence. Events are
retained in a fixed bounded client buffer and written on exit only.

The documented client-0 cleanup interface is:

```python
[
    profile["drconfig_path"], "-nudge_timeout", "5000",
    "-nudge_pid", str(verified_owned_pid), "00000000", "1",
]
```

Apply an additional independent **eight-second** subprocess timeout. This
client explicitly registers a nudge callback: argument `1` requests
`dr_exit_process(0)` and its exit callback flushes the observation log. Other
arguments do nothing. This is **process exit**, not a flush-only live snapshot
or normal vendor shutdown. Recheck held-handle exit and parse the finalized log.
A partial, empty or unflushed log is a failure, never fabricated observations.
No registry-wide client registration, all-process nudge or name-based kill is
needed. Single-client observer mode is the only proven configuration; drcov
coexistence is not claimed.

`prepare` generates a config and exact launch plan **without executing it**:

```powershell
python scripts\native-sim-observe.py prepare `
  --profile "<verified profile>.json" --target "<locally readable pinned PE>" `
  --site "1:371e10:0:4" --site "2:371ef0:0:1" `
  --site "3:15ef30:0:1" --site "4:614600:0:1" `
  --max-events 256 --max-objects 64 --prefix 64 `
  --output "<new private plan directory>"
```

Those are selected entry RVAs, **not** a proven callback→worker→document→handler
chain. The plan's launch alternative is for explicit console tooling only:
the GUI owner must construct the attach argv with its verified owned PID.
Remote profile paths must be readable on the host performing profile validation;
do not bypass validation just because they are not controller-local paths.

## Format schema: `passive-entry-observer-jsonl-v1`

UTF-8 JSONL, complete newline-terminated records, at most **16 MiB**:

1. `HEADER`: `schema=1`, `kind=passive-entry-observer`, target/client/config
   SHA-256, main-module basename/base/SizeOfImage/timestamp and match status;
   declared event/object/prefix limits.
2. One to sixteen `SITE` records: ID, RVA, post flag, register mask, wrapped status.
3. Zero to 4,096 `ENTRY`/`RETURN` records with contiguous zero-based `seq`.
4. Exactly one terminal `SUMMARY`.

The separately hash-bound profile supplies upstream tool/runtime/extension
hashes, version, units and limitations. The proof receipt binds that profile,
script, fixture, exact argv, live process identities and finalized raw log hashes.
The GUI integration must retain the equivalent joins; header hash declarations
are not independent authenticity proof.

`ENTRY` contains site/thread, context status, PC, all five requested registers,
return-read status and (only when readable) return address. There are four
register-indexed `samples`. Each contains raw address, read status, byte count,
hex prefix and optional safely read first-pointer/vtable-candidate bytes.
No unavailable pointer/context/return value is replaced with a zero value.
Disabled samples have explicit status `0` and no bytes.

| Read status | Meaning |
|---:|---|
| 0 | Not selected / no complete first pointer available for a second read |
| 1 | Entire configured bounded request read successfully |
| 2 | Partially readable request; only actual returned bytes retained |
| 3 | Unreadable; no bytes retained |
| 4 | Unique-pointer budget exhausted; no read attempted |

`RETURN` references its original entry sequence and records raw RAX only when
context is available. Register/return values are raw unsigned 64-bit patterns,
addresses are application virtual addresses, RVAs are byte offsets. **No vendor
clock unit, epoch, monotonicity or frame-clock identity is inferred.**

`SUMMARY` records event/unique-pointer totals, dropped events, unreadable reads,
partial reads, `truncated_reads` (the same **short safe-read** counter),
object-limit read refusals, unavailable contexts and whether nudge requested
exit. Full object sizes are unknown: a successful fixed prefix is not a
complete-object read and does not establish how many unrequested bytes exist.

Limits are hard capped at 16 sites, 4,096 events, 512 unique pointer values and
**256 bytes per prefix read**, plus an eight-byte candidate-vtable read and
eight-byte stack-return read. Invalid pointers count against the finite unique
budget. Repeated pointers may be sampled again until the event budget fills;
subsequent entries only increment dropped counters and perform no memory reads.
Samples are not atomic snapshots and can race target writes. Exit flushing is
bounded by the owner's deadline; exceeding it blocks the capture.

Focused synthetic parser/configuration tests:

```powershell
python -m unittest discover -s scripts -p test_native_sim_observe.py -v
```
