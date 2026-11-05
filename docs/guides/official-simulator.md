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

The pinned external launcher interface is supplied by the
[coverage collector](native-sim-coverage.md). Target PID files, image hashes,
creation time, session and launcher ancestry are checked; a launcher PID is
never mistaken for the GUI PID. No instrumentation is installed by the runner.

## Evidence and limits

Each invocation creates a unique local directory below
`.reverse-engineering\native-simulator\replay` and an independent notebook
directory below `C:\ProgramData\LoxOracle\native-replay`. Original fixtures
and earlier runs are never overwritten. Child environments are whitelisted;
MCP, GitHub and API tokens are not inherited by Config or DynamoRIO.
Each new run also uses an explicit unique simple project basename,
`input-<run-id>.Loxone`, recorded in `request.fixture_filename`. The copied
bytes retain the identical pinned fixture SHA; no XML identities are changed.
This prevents cross-run `input.Loxone` autosave-name collisions after
client-requested exit. Historical `input.Loxone` receipts remain readable;
old backups are neither restored nor deleted. Any unexpected recovery dialog
still blocks execution rather than being clicked through.

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

`--ocr-profile` performs this admission automatically after a fresh replay has
closed its owned GUI and fetched its evidence. Instrumented success also
requires an actual nonempty flushed log; target-BB/function mapping and
plain/instrumented comparison remain independent collector/scorecard gates.

This pilot is not complete engine coverage, a universal allocation formula,
closed physical-plant validation, or a claim of full simulator parity.

Offline tests:
`python -m unittest discover -s scripts -p "test_official_sim*.py"`.
