# Official offline simulator replay

`scripts\official-sim.py` records the **official Loxone Config** built-in
offline simulator. It never uses `lox sim`, a substitute model, a Miniserver
connection, or a headless/private vendor ABI.

The bounded pilot pins Config 17.2.10.21 by SHA-256 and a public-sample-derived
fixture by SHA-256. Case definitions contain procedures and expectations, not
vendor project files. Obtain the fixture separately and inspect its settings
with the canonical `lox config energy inspect`/`connectors` commands first.

## Private prerequisites

Use the authenticated notebook UX profile documented in the UX toolkit.
The notebook must be logged in and unlocked; its interactive MCP PowerShell
context must be distinct from the workstation running the CLI. Keep the
executable-scoped `LoxOracle-Config-Offline` outbound block enabled for every
profile. The runner checks it and refuses to adopt existing Config processes.

Provide a separate private transfer profile:

```json
{
  "host": "YOUR-NOTEBOOK",
  "port": 2222,
  "user": "YOUR-TRANSFER-USER",
  "desktop_host": "YOUR-DEDICATED-NOTEBOOK",
  "identity_file": "~\\.ssh\\YOUR-KEY",
  "known_hosts_file": "~\\.ssh\\YOUR-KNOWN-HOSTS"
}
```

SSH/SCP only copies owned files. All GUI execution occurs through the
authenticated interactive MCP endpoint, never SSH Session 0 or the local
desktop. Strict host-key checking and batch-mode authentication are mandatory.
The backend verifies the selected notebook's computer name and refuses the
controller's computer name before launching or touching any GUI.
Read-only console/input-desktop and foreground-integrity checks run before
vendor launch. A locked/noninteractive desktop or higher-integrity foreground
input owner blocks replay with an explicit operator-action reason. The runner
never stops input-sharing software, switches input desktops, raises privileges,
or changes foreground/security policies. An operator must resolve that
external condition before retrying.

```powershell
python scripts\official-sim.py --profile PRIVATE-UX.json `
  --transfer-profile PRIVATE-TRANSFER.json `
  --case tests\ux\native-simulator\energy-storage-at-finite.json `
  --fixture PRIVATE-storage-at-finite.Loxone `
  --capture-scope canvas `
  --ocr-profile PRIVATE-OCR.json
```

For a separately authorized instrumented replay, append
`--mode drcov --instrumentation-profile PRIVATE-VERIFIED-COVERAGE-PROFILE.json`.
Launch-time instrumentation remains the default. The explicit
`--instrumentation-start post-init` alternative starts the same owned Config
normally, then attaches the pinned Windows DynamoRIO **before Simulation starts**.
It excludes application startup from coverage. A separate 30-second attachment
deadline and exact-instance process-handle watchdogs bound the target and
attacher; DynamoRIO's `-s` alone does not bound Windows injection.
The explicit alternative `--instrumentation-start post-start --observer-mode
native-win32 --exit-dump-profile PRIVATE-VERIFIED-EXIT-DUMP.json` first starts
Simulation normally and verifies its pre-attach UIA state/clock. It caches the
exact root/canvas/clock geometry **before** attachment, then performs no further
UIA provider/tree calls or input, including cleanup. Only guarded Win32
identity/foreground/unchanged-geometry checks and owned physical ROI captures
follow attachment. Both application startup **and SimulationStart** are excluded
from this coverage interval.

The verified instrumented pilot uses:

```powershell
python scripts\official-sim.py --profile PRIVATE-UX.json `
  --transfer-profile PRIVATE-TRANSFER.json `
  --case tests\ux\native-simulator\energy-storage-at-finite.json `
  --fixture PRIVATE-storage-at-finite.Loxone `
  --ocr-profile PRIVATE-OCR.json --capture-scope canvas `
  --mode drcov --instrumentation-start post-start `
  --observer-mode native-win32 `
  --instrumentation-profile PRIVATE-VERIFIED-COVERAGE-PROFILE.json `
  --exit-dump-profile PRIVATE-VERIFIED-EXIT-DUMP.json
```

The full-UIA/earlier-attachment paths remain explicit alternatives, not the
verified instrumented recipe. A one-second `WM_NULL` timeout was insufficient
as a liveness proxy; the native-only recipe requires actual clock advancement
instead and retains all ownership/foreground/capture guards.

### Experimental passive entry observations

The explicit `--mode entry-observer` uses the independently authored
[bounded entry client](native-simulator-observer.md), **not drcov**. Its owned
synthetic-console proof includes normal startup followed by late attachment,
register/prefix/caller/RAX checks and its own public nudge exit/flush callback.
That proof does **not** establish vendor compatibility, output/clock parity,
fixture UUID binding or a complete call chain. No vendor entry-observer case
has been verified merely by adding this mode.

```powershell
python scripts\official-sim.py --profile PRIVATE-UX.json `
  --transfer-profile PRIVATE-TRANSFER.json --case PRIVATE-BOUNDED-CASE.json `
  --fixture PRIVATE-FIXTURE.Loxone --ocr-profile PRIVATE-OCR.json `
  --mode entry-observer --instrumentation-start post-start `
  --observer-mode native-win32 --capture-scope canvas `
  --instrumentation-profile PRIVATE-OBSERVER-PROFILE.json `
  --entry-config PRIVATE-CANONICAL-OBS1.cfg `
  --exit-dump-profile PRIVATE-OBSERVER-PROFILE.json
```

For this mode, `--exit-dump-profile` must contain the **same observer profile
bytes** as `--instrumentation-profile`; an old drcov exit-method profile does
not prove the new client's callback. Plain/drcov defaults and their existing
exit-method interface are unchanged. The first entry experiment must finish
its final phase/window within **90 simulator seconds**. Only post-start,
native-win32, canvas and pinned OCR admission are accepted. Attachment has an
independent 30-second deadline and a 120-second DR limit, retaining the existing
exact-instance target/attacher handle watchdogs and all native-only
foreground/geometry/ROI guards. No UIA or input follows attachment.

The runner copies the observer Python module, exact ASCII configuration and
source profile into the immutable run. **On the notebook**, it verifies every
actual tool/client/extension path with `validate_profile`, reads the actual
pinned AMD64 PE, and requires byte-identical `config_bytes` regeneration.
`read_config(Path_or_bytes)` is the shared pure decoder for controller,
launcher and independent receipt readers: it returns a flat dictionary of
target/client/config hashes, module name/size/timestamp, event/object/prefix
budgets and ordered `{id,rva,post,mask}` sites. It does not invent executable
sections. The initial integration accepts only the reviewed callback, worker,
dispatcher, EnergyManager2 handler and clock entry RVAs, with return hooks
disabled except an explicitly selected clock RETURN hook. Its RAX remains raw;
synthetic hook safety is not vendor compatibility or a clock-contract proof.
Prefixes remain bounded to 256 bytes; selecting an address as a
read source does not establish its ABI role or UUID layout.

Client and extension DLLs are separately hash-checked and snapshotted beneath
the run's `entry\tools`; the staged profile retains their pins and points to
those copies. The profile constructor itself does not copy files. Remote
profile paths are never treated as readable controller-local paths or bypassed
during notebook validation.

`official_sim_runner.entry_inputs(request)` is a read-only notebook preflight:
it verifies the original profile/config/source and actual PE without staging,
launching or attaching. It accepts the controller's exact normalized legacy
`after_first_calculation`/`wait_seconds` phase; named phases still require their
finite `latest_seconds` windows. The 90-second entry-case limit is unchanged.

The V5 `drmgr-entry` prototype uses schema-2 ENTRY-only records and no drwrap
dependency. Its profile declares `supports_return=false`, so **every post hook,
including clock RETURN, is rejected before launch**. V4 retains schema 1 and
its separately documented opt-in clock hook. Normal V5 FP/SIMD/static-TLS
console checks passed, but both late-attach console attempts failed: V5 is
**not console-admitted/native-ready**. See the observer guide; no paired vendor
V5 case or equivalence is claimed.

Native entry replay for **every backend, including V4**, refuses to stage or launch unless the pinned profile contains
literal `late_attach_verified: true`. Profile creation defaults to `false`;
the existing diagnostic-only profiles, including ones missing the field, are
blocked before application launch. **No native-late-admitted V4 or V5 profile exists.** This
readiness gate does not prevent retaining blocked console/interrupted log
diagnostics. Legacy V4 log parsing/console diagnostics and plain/drcov remain
compatible; known-failing V4 native entry launch is no longer exempt.

The PowerShell replay invokes the same existing launcher worker through
`--record-launch REQUEST.json`. Every new run retains immutable raw
`sanitized-launcher.stdout.log`, `sanitized-launcher.stderr.log` and
`sanitized-launcher.json` with the worker's actual exit code, source/request
hashes and precise failure reason. A worker that cannot start or times out has
an explicitly unknown exit code, not a fabricated success. These diagnostics
join the normal artifact manifest; pre-launch failures never imply a target
was created. Older failed receipts are not rewritten to add missing logs.

`entry\observations.jsonl` is separate from coverage logs. A zero-byte file is
initialization readiness only. Cleanup uses the observer profile's exact
client-0/argument-1 nudge with five-second DR and independent eight-second
helper deadlines, labeled `PASSIVE_observer_exit_flush_not_live_snapshot`.
This requests observer-client process exit/flush, **not** a drcov dump, live
snapshot or normal vendor shutdown. Forced cleanup cannot establish a passing
entry capture.

The original `official-native-replay` identity/run ID and functional sample
assertions are retained. The receipt adds `entry_logs` and `entry_observation`,
including the exact `parse_log(expected=...)` identities/sites/budgets,
validation status, counts, seen/missing site IDs and hash-bound `profile`,
`configuration` and `parser_source` artifact references. Complete strict
parsing and at least one actual context-readable ENTRY are required in
addition to real post-attach clock/output proof. Empty, partial, altered or
unknown-context observations remain blocked. Missing requested sites are
explicit even when other entries were captured. Entry observations never
become BB coverage, drcov `loop_established`, or an automatic UUID-binding claim.

The pinned external launcher interface is supplied by the
[coverage collector](native-sim-coverage.md). Target PID files, image hashes,
creation time, session and launcher ancestry are checked; a launcher PID is
never mistaken for the GUI PID. No instrumentation is installed by the runner.

## Portable reviewed protocols

The following public JSON files are byte-identical aliases of the verified
private reviewed definitions, preserving their case IDs and definition hashes.
Their scoped Git attributes retain LF bytes across checkouts:

| Protocol under `tests\ux\native-simulator` | Scenario |
|---|---|
| `energy-multi-original-reviewed.json` | Original three-load priority order |
| `energy-multi-third-first-reviewed.json` | Third Load moved to first priority, Id/U/OIdx retained |
| `energy-daily-observer-baseline.json` | Single-observer closed-feedback daily OFF/ON/OFF baseline |

They publish factual UUIDs, hashes, expected phases and calibrated rectangles,
**not** vendor XML, screenshots, models, hostnames or credentials. They are
portable protocols, not self-contained vendor-project bundles.

Obtain the exact SHA-pinned fixture separately, plus the private calibration
image/header/evidence files at their declared ignored root-relative paths.
Preserve those files byte-for-byte when relocating the verified evidence
bundle below a new artifact root. Tool/model and authenticated notebook/transfer
profiles are separate private prerequisites; a private operational manifest
records the exact working argv and paths without copying secret values.
Missing or altered calibration evidence intentionally blocks **before GUI
launch**. Do not remove references, edit digests or bypass verification to
make a clean checkout appear independently runnable. Public/private aliases
compare by their identical immutable definition bytes; changing an alias
requires a separately reviewed case version and fresh native evidence.
Portable relative references do not broaden the pinned application/model or
raster geometry: an incompatible native layout still blocks.

## Evidence and limits

Each invocation creates a unique local directory below
`.reverse-engineering\native-simulator\replay` and an independent notebook
directory below `C:\ProgramData\LoxOracle\native-replay`. Original fixtures
and earlier runs are never overwritten. Child environments are whitelisted;
MCP, GitHub and API tokens are not inherited by Config or DynamoRIO.
Each new run also uses an explicit unique simple project basename,
`input-<run-id>.Loxone`, recorded in `request.fixture_filename`. The copied
bytes retain the identical pinned fixture SHA; no XML identities are changed.
This avoids cross-run `input.Loxone` basename collisions, but **does not
guarantee no recovery prompt**: Config also shares a project recovery identity.
Historical `input.Loxone` receipts remain readable; source fixtures and previous
run artifacts are never restored, overwritten or deleted.

The default `--recovery-policy block` retains and blocks every startup modal.
The explicit `--recovery-policy owned-skip` permits one narrowly reviewed
exception per run: the exact German automatic-backup question naming the
**current unique `request.fixture_filename`**, on the verified owned
PID/session/hash/exact `#32770` GA_ROOT, with the actual Yes ID 6 and No ID 7.
Only that dialog is activated, and only its guarded actual **No** control is
clicked. Before/after pinned fixture hashes, modal text/pixels, validation,
foreground/control identity, action intent and disappearance proof are
retained immutably. Unknown text, another filename, extra/ambiguous controls,
multiple/repeated dialogs, identity changes or altered fixture bytes block.
There is no Yes/restore path, backup deletion or unrelated-project search.
When no exact owned backup path is exposed, the runner does not search for it.
Native No itself may clear the lab recovery entry; cache preservation is not
claimed. This opt-in does not authorize security prompts or foreign input.

Receipts bind application, case-definition and fixture hashes; artifact paths
are relative to the declared local repository root, not notebook paths.
Notebook absolute paths occur only in remote metadata. The CLI prints one
compact result and receipt path; exit 0 means pass and exit 2 means blocked/fail.

Native progress requires an enabled offline pause control, disabled LIVE, and
two different native simulator clock readbacks. Wall-time waits allow the
first real calculation interval; they are **not native stepping**, a known
internal dt, or a complete timing trace.
For the native-only post-start observer, pre-attach running state alone is
insufficient: two separately captured post-attach clock PNGs must be at least
three seconds apart, strictly recognized as actual English `HH:MM:SS` at
confidence ≥90, and show different values. Only the subsequent file-only
admission can mark native running verified; clock images, timestamps and raw
recognizer evidence remain bound to the original run.
Activation verifies the exact owned `GA_ROOT` (never `GA_ROOTOWNER` or a
same-PID substitute), retaining HWND/parent-chain proof. A bounded fallback can
click only a hit-tested blank owned title-tab container; foreign windows and
buttons are never clicked. Captures refuse foreign top-level overlap and use
physical coordinates consistently; no whole-desktop fallback is used.
`--capture-scope canvas` narrows the capture to the uniquely identified owned
native programming canvas, inset by **64 physical pixels on all four edges**
of its exact UIA/main-window intersection, then bounded by the virtual screen.
This is not a DPI-scaled inset or automatic occluder clipping.
Its native HWND/root, full bounds, screen origin and actual capture rectangle
are retained. This avoids an outside-canvas MCP control bar without hiding or
whitelisting it; any real foreign overlap **inside** the chosen rectangle
still blocks. Only verified DWM-cloaked or global-alpha-zero windows are
noncomposited exceptions, never `WS_EX_TRANSPARENT` alone. Rejected captures
retain first-occluder process/class/geometry/cloaking/alpha metadata, not
foreign window text or pixels. Capture geometry and all checked above-root
foreign windows, including nonintersecting right/bottom control bars, remain
in separate diagnostic artifacts.

### Strict German output readback

The current positive pilot uses the optional, file-only
`official_sim_ocr.py` helper: Pillow plus a separately obtained, SHA-pinned
Tesseract 5.4 executable and German `deu.traineddata`. The runner neither
installs nor downloads them. A private profile has this shape:

```json
{
  "schema_version": 1,
  "kind": "native-tesseract-ocr-profile",
  "language": "deu",
  "executable_path": "ABSOLUTE-PATH-TO-TESSERACT",
  "executable_sha256": "ACTUAL-64-HEX-DIGEST",
  "model_path": "ABSOLUTE-PATH-TO-deu.traineddata",
  "model_sha256": "ACTUAL-64-HEX-DIGEST",
  "header_model_path": "ABSOLUTE-PATH-TO-eng.traineddata",
  "header_model_sha256": "ACTUAL-64-HEX-DIGEST"
}
```

Tool/model files must lie under the declared common artifact root. The fixed
recipe whitens green wire pixels (`g > r+30 && g > b+30`), takes minimum RGB,
resizes 4× bicubic, adds a 24-pixel white border, and uses German PSM 7 with
default dictionaries. It admits only exact `Ein`/`Aus` at confidence **≥90**,
with independent agreement between the OL1 pin and its wired reference.
No expected scenario value is supplied to OCR; no fuzzy correction, alternative
recipe selection, pixel-colour state inference, or default OFF is permitted.

Windows OCR proposes the actual `Energy Manager` header geometry; it does not
admit output states or fuzzy title corrections. If its title is misread, a
unique two-word `Energy` proposal with the pinned geometry is independently
verified using the pinned English model, four physical pixels of crop padding,
and exact `Energy`/`Manager` words at confidence ≥90. Actual TSV word boxes are
mapped back into source pixels (4×/border24 inverse with floor/ceil), and a
new derived anchor is retained without rewriting the original Windows OCR.
The English model's bytes/hash join the artifact manifest when used.
The bounded raster binding supports the observed 107×14-pixel
header on this pinned build/public fixture. Crop positions are relative to that
actual header, not desktop coordinates. Other geometry is explicitly blocked.
Low-confidence `Aus` observations remain unadmitted.

The default `literal_recipe: "minimum-v1"` remains unchanged. The explicit
opt-in `"dual-gray-tight-v2"` supports the bounded OFF expansion: it accepts
only the calibrated **full** 25/27-pixel-wide, 17/18-pixel-high rectangles,
internally trims three physical pixels from the right, and always runs both
fixed minimum-RGB and 25%-minimum/75%-luminance views. Both use the same green
wire cleanup, 4× bicubic enlargement, white border and German PSM 7. Callers
must not pretrim again. Exact state words still need confidence ≥90;
contradictory state words (even a low-confidence contradiction), high-confidence
unknown words and multiword results block admission. All candidate evidence
is retained. This is one declared recipe, not expected-state-driven selection.
File-only holdout validation does not substitute for fresh native executions.
The bounded Soc49/MinSoc50 counterpart has now produced two independent plain
runner-admitted native OFF observations and one matching post-start recorded
observation, each after at least 70 elapsed seconds. Actual pin/reference
`Aus` confidence was 94.645/93.650. The recorded run's two clock reads exceeded
92 confidence and advanced six native seconds over 6,475 ms. An earlier
recorded attempt's low-confidence clock reads remain blocked and retained;
they were not corrected or relabeled. Recorder mapping and the joined coverage
gate remain separate from this bounded output admission.

UIA snapshots, application-only physical-pixel captures, header anchors,
processed crops, raw TSV, tool/model pins, helper source snapshots and actual
recognizer records are retained. Empty/ambiguous readback produces **blocked**.
Initial/progress snapshots remain separate from asserted output phases. Forced
cleanup is recorded and can invalidate drcov flushing; only
hash/start-time-verified owned targets and launchers close.
The native-only observer instead first requests the console-verified public
PID-specific DynamoRIO exit-and-dump method with pinned `drconfig`, an
independent eight-second timeout and a held exact-instance target handle.
This is recorded as **client-requested exit-and-dump**, never vendor-graceful
shutdown. Only a verified target exit plus an actual nonempty flushed log
admits that cleanup method. Exact owned termination is the final fallback and
is explicit; missing/empty coverage remains blocked.

### Immutable admissions and independent repeats

OCR can be repeated without driving a desktop:

```powershell
python scripts\official-sim.py --admit-receipt ORIGINAL-PRIVATE-RECEIPT.json `
  --ocr-profile PRIVATE-OCR.json
```

This writes a **new** correlated admission receipt; it never rewrites the
original blocked receipt, capture, or definition. `execution.run_id` remains
the original actual backend run identity. Two analyses of one run are not
independent replays. The case definition is copied byte-for-byte before launch,
and receipts bind that immutable run-local copy—not a mutable public case file.
Each admission also copies the OCR helper first and loads that retained source
as a fresh uniquely named module. Literal, header and clock recognition all
use that same snapshot; a helper cached before a long GUI replay cannot
silently differ from its recorded `observer_source_sha256`.

`--ocr-profile` performs this admission automatically after a fresh replay has
closed its owned GUI and fetched its evidence. Instrumented success also
requires an actual nonempty flushed log; target-BB/function mapping and
plain/instrumented comparison remain independent collector/scorecard gates.

This pilot is not complete engine coverage, a universal allocation formula,
closed physical-plant validation, or a claim of full simulator parity.

## Bounded channel and phase expansion

Legacy schema-1 cases retain their `wait_seconds` (70..120), single `OL1`,
and `after_first_calculation` behavior. Expanded schema-1 cases instead declare
`channels` (distinct `OL1`..`OL3`) and one to six `sampling_phases`:

```json
{
  "channels": ["OL1", "OL2", "OL3"],
  "sampling_phases": [
    {"name": "settled", "at_seconds": 70, "latest_seconds": 95}
  ],
  "expected_outputs": {"settled": {"OL1": 1, "OL2": 0, "OL3": 0}}
}
```

This is a protocol example, **not an established allocation result**. Each
phase starts at 20..200 elapsed seconds and ends within 30 seconds, no later
than 210 seconds. Windows must be ordered and nonoverlapping. Capture intervals
and their midpoints must fit the declared window; missed windows are blocked,
not silently shifted. Runs retain a finite 300-second target watchdog
and 270-second transport budget. Startup, attachment, observation and cleanup
retain their separate bounds. These windows measure actual native execution;
they are not an internal dt or an exact native step API.

An optional phase `simulator_time_window: ["10:59:10", "10:59:30"]` additionally
requires an actual clock observation within those same-day bounds.
Post-attach named phases use the shared pinned English clock recognizer on
their separately retained clock PNGs; there are still no post-attach UIA calls.
`simulator_start_time: "10:58:00"` requests **only the native simulator-local
control before Start**. The runner unchecks native “Use current time”, requires
an owned writable ValuePattern, retains requested/before/after values, and
blocks on unsupported editing or mismatched readback. This edit binding is
not yet established by a fresh native replay. It never sets the OS clock/date,
changes a calendar during execution, or synthesizes missing native clock values.
Native writable-ValuePattern reports have in fact ignored the requested time;
those attempts remain blocked. An explicit case
`simulator_time_edit: "verified-sections"` instead uses the owned focused
clock's HOME plus 0/3/6 character navigation, one nonwrapping arrow probe per
field, and bounded adjustment/readback. Exactly the intended field must change;
the other two must remain unchanged. Actual probe/results are retained.
A prior one-RIGHT-per-section candidate changed the hour instead of the minute
and was rejected, not inferred successful. The revised 0/3/6 recipe is unit
tested and has now established exact native 10:58:00 with “Use current time”
OFF before Start. Actual named captures reached 10:58:20, 10:59:10 and 11:00:30.
Output admission for that daily fixture remained blocked: its fanned-out OL1
did not provide an exact readable pin state. Clock/control success alone is
not an OFF→ON→OFF behavioral pass.

Additional channels are not inferred from evenly spaced rows. A private case
must supply `readback.channel_boxes`, mapping each channel to distinct
`pin`/`wired_reference` rectangles `[left, top, right, bottom]` relative to the
actual header. `readback.calibration` must retain hash-bound `image` and
`header_anchor` artifact refs plus a nonempty `review` of the actual row/wire
geometry. Source PNG, exact 107×14 header binding and rectangle bounds are checked before
launch. Rectangles cannot be reused, and each pin/reference pair must not
overlap. Neighbouring channel crops may share reviewed blank padding; that
does not authorize reusing another channel's glyph. Optional hash-bound
`calibration.evidence` refs retain the source calibration and individual
reading records. All calibration artifacts join the
original immutable receipt. Each channel calls the same strict file-only
recognizer independently and needs pin/reference agreement at confidence ≥90.
Missing or ambiguous channels block; actual differing digital values fail.
No expected output is passed to the recognizer.

Use `purpose: "exploratory"` with `expected_outputs: {}` while testing an
unreviewed timing/priority hypothesis. All declared phase/channel observations
are still retained, but this purpose **cannot pass** or count toward a feedback
gate. Preserve the exploratory case/receipts and any contrary evidence when
authoring a separately versioned reviewed assertion. In particular, a fixed
`Gpwr` is not a closed plant: multi-load fixtures must explicitly declare grid
and individual-load feedback assumptions, persistent Load Id/U/OIdx mappings,
and priority order. Historical daily-plant screenshots and simplified
`lox sim` behavior do not establish new automated native timing/allocation passes.

### Observed bounded multi-load order

Two privately pinned fixtures now have complete OL1..OL3 pin/Or.I1 agreement:
an exploratory native execution followed by a separately defined, fresh
reviewed native assertion for each order. Exploratory receipts remain blocked
as discoveries; neither original receipt nor case definition was rewritten.

The declared native plant has 4 kW PV, three digital 2.5 kW loads, RT 30 s,
zero minimum runtime/daily minimum, and native `Or(OLn, 0)` observers feeding
the individual-load feedback. Native grid math is
`Gpwr = -4 + 2.5 * sum(observer.Q)` kW, not constant grid power.
Stable Load Id/U/OIdx and all graph wiring/positions are retained when moving
the third Load record to first priority.

| Priority order by persistent OIdx | Actual OL1, OL2, OL3 in 70..95 s window |
|---|---|
| 0, 1, 2 | 1, 0, 0 |
| 2, 0, 1 | 0, 0, 1 |

Each reviewed fresh replay matched its preceding discovery. This establishes
only these sampled native fixture outcomes, not a universal allocation formula,
all priority/power combinations, physical plant parity, exact transitions or
zero-delay feedback. Each order now also has **two independent passing plain
receipts and one passing post-start recorded receipt for the same reviewed
definition/fixture**, without counting discovery receipts as pass repeats.
Actual post-attach clock advancement and nonempty flushed logs are retained.
Independent BB imports/scorecard joins and engine completeness remain separate.

### Observed bounded daily timing

The original daily fixture's fanout/ellipsis readback remains blocked. A new
private revision gives OL1 only one native `Or.I1` observer (`I2=0`) and fans
`Or.Q` out to **all** former OL1 consumers: individual-load feedback, grid
math and State. Existing parallel non-OL1 sources, Energy settings and Load
Id/U/OIdx are retained. The native plant is no-PV closed feedback:
`Gpwr=2.5*observer.Q` kW, daily minimum 60 s and deadline 11:00.

Actual new PNGs established independent pin/Or.I1 OFF→ON→OFF readback using
the unchanged strict recipe. Their discovery/calibration did not retroactively
pass the initial legacy-ROI receipt. A separate reviewed definition then
passed a **fresh** plain native replay:

| Named sample | Actual simulator-local clock | OL1 |
|---|---|---|
| Before daily window | 10:58:21 | 0 |
| Daily active window | 10:59:12 | 1 |
| After deadline window | 11:00:31 | 0 |

That unchanged reviewed definition/fixture now has two independent passing
plain executions and one passing post-start recorded execution. All three
state samples and their actual native clock windows were admitted; recorder
mapping and the independent feedback join remain separate.

Start time was independently read back as 10:58:00 with native “Use current
time” OFF before Start; the OS clock/date were untouched. These samples bracket
activity, not an exact transition instant, exact 60-second duty, internal dt,
continuous day integration or physical-plant validation. Separate MinRt120
and RT60 fixture versions were then observed without declaring expected
outputs in advance:

| Variant | Actual sampled OL1 values and simulator-local clocks |
|---|---|
| MinRt120, RT30 | 0 at 10:58:21; 1 at 10:59:12; 1 at 11:00:35; 1 at 11:01:01 |
| MinRt0, RT60 | 0 at 10:58:21; 1 at 10:59:12; 0 at 11:00:32–33 |

Canonical inspection confirms that only the intended MinRt or RT Load storage
differs between these observer fixtures. These variant receipts remain
**exploratory observations, not asserted passes**. The MinRt120 release time
was not captured, continuous ON between samples is not established, and the
RT60 sample agreement does not recover general preparation-time semantics.
One separately defined exploratory extension adds a 200..210-second sample
without changing the original MinRt case. It still observed OL1 ON at
11:01:21–22 (201.574 elapsed seconds). This is another sampled point, not an
observed release or proof of complete 120-second minimum-runtime semantics.

Offline tests:
`python -m unittest discover -s scripts -p "test_official_sim*.py"`.
