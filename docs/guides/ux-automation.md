# Deterministic UX automation

`scripts\loxone-ux.py` sends a bounded recipe as **one** MCP PowerShell tool
call, after protocol initialization. Python 3.12+ and its standard library are
the only client requirements. This companion tool deliberately defaults to
compact, ASCII-escaped JSON rather than the main Rust CLI's table output.

This is an interaction shortcut, **not independent GUI parity proof**. A
successful batch does not establish that vendor storage, CLI edits, and GUI
behavior agree.

## Private setup and ownership

Use an explicitly owned, separately launched Loxone Config process with a
public/synthetic project on a dedicated test desktop. Never target a primary
work desktop or a real vendor/customer project. The tool does not launch
applications, install software, create tunnels, start agents, or deploy scripts.

Manually deploy the reviewed `scripts\loxone-ux.ps1` backend using your existing
authenticated SSH connection. Keep that backend available in the MCP peer's
interactive Windows PowerShell session. Start/stop the authenticated MCP peer
and any SSH forwarding manually using your established lifecycle; close them
when finished. Do not expose an unauthenticated listener.

The known peer interface is Windows-MCP 0.8.8 / FastMCP 4.1.6, stateless HTTP,
MCP protocol `2025-06-18`, with `PowerShell` arguments `command` and integer
`timeout`. The client does not run `tools/list` or discover screenshot tools.
Protocol and backend response schema mismatches fail closed.

Create a profile **outside the repository**, with private permissions:

```json
{
  "schema_version": 1,
  "mcp_url": "http://127.0.0.1:18765/mcp",
  "token_file": "${LOX_UX_TOKEN_FILE}",
  "backend_path": "C:\\OwnedTools\\loxone-ux.ps1",
  "remote_artifact_root": "C:\\PrivateUxRuns",
  "expected_exe_sha256": "<64-hex-SHA256-of-the-owned-Loxone-Config-executable>"
}
```

Replace the SHA placeholder with the full verified hash. Alternatively omit
`expected_exe_sha256` for read-only identity discovery. `token_file` expands
environment variables and `~` locally and must resolve to an absolute path.
It contains only the bearer token, not JSON or an Authorization header.
Backend/artifact/save paths are remote absolute Windows drive paths, not paths
on the Python host. Each request gets a unique directory below the artifact
root. The backend creates it; the client does not create local run files.

```powershell
$env:LOX_UX_PROFILE = "$HOME\.lox\ux-profile.json"
$env:LOX_UX_TOKEN_FILE = "$HOME\.ssh\owned-ux-token"
$ownedPid = 1234 # Replace with the PID of your separately launched test process.
python scripts\loxone-ux.py --pid $ownedPid status
python scripts\loxone-ux.py --pid $ownedPid run tests\ux\recipes\client-example.json
```

Plain HTTP is accepted only on loopback (for an existing SSH tunnel); other
endpoints require HTTPS. Redirects and inherited HTTP proxies are disabled.
Profiles, tokens, private endpoints, and request headers are not printed.
Keep profiles, projects, and backend artifacts out of commits.

## Short commands and recipes

Common flags `--profile`, `--pid`, `--expected-sha`, and `--verbose` may appear
before or after the subcommand. `--profile` overrides `LOX_UX_PROFILE`; a PID is
always explicit. `--expected-sha` overrides the profile pin, never a conflicting
recipe pin.

| Commands | Arguments |
|---|---|
| `status`, `windows` | Owned target identity / owned windows only |
| `inspect` | Optional `--window-class`, `--selector` JSON or `--name`; `--limit` 1–50, `--depth` 0–5 |
| `focus`, `invoke` | Optional window class and selector |
| `set-value VALUE`, `keys VALUE` | Optional window class and selector; keys uses SendKeys syntax |
| `wait` | Required selector; optional exact `--value`; `--timeout-ms` 1–30000 (default 5000) |
| `energy-open` | Open the selected Energy Manager through its semantic Settings group |
| `energy-read ROW COLUMN` | Read a consumer-grid cell |
| `energy-set ROW COLUMN VALUE` | Set a supported editable cell |
| `energy-select ROW COLUMN INDEX` | Select a supported combo-box choice |
| `energy-accept`, `energy-cancel` | Explicitly finish or cancel the owned energy dialog |
| `save-as PATH` | Save to an absolute, **nonexisting** remote Windows path |
| `run RECIPE.json` | Execute 1–64 validated steps in order |

Selectors use only `automation_id`, `automation_id_suffix`, `name`, and
`control_type`; values are exact strings, not expressions. Obtain selectors
from an owned, bounded inspection/report for the **same executable version**.
Input selectors must be unique. Read-only inspection may return several
matches, bounded by `--limit` and `--depth`. No fixed desktop coordinates are
stored in recipes.

Energy column arguments accept either indexes or readable field names:
`priority`, `output`, `mode`, `icon`, `name`, `preparation`, `minimum-runtime`,
`daily-runtime`, `deadline`, `switch-on-power`, and `rated-power`.
`name` is column **4**, while `mode` is column **2**. The backend checks the
native 12-row/11-column grid and header binding, then uses `GridPattern.GetItem`;
it never indexes the subset of cells visible at the current DPI/scroll position.

```powershell
python scripts\loxone-ux.py --pid $ownedPid energy-open
python scripts\loxone-ux.py --pid $ownedPid energy-set 1 name "Test load"
python scripts\loxone-ux.py --pid $ownedPid energy-select 1 deadline 27
python scripts\loxone-ux.py --pid $ownedPid energy-cancel
python scripts\loxone-ux.py --pid $ownedPid run tests\ux\recipes\energy-edit-cancel-check.json
```

The specialized Energy binding is pinned to Config **17.2.10.21** and the
checked-in recipe hash. Settings-group and grid-header labels currently cover
the observed English and German UIs; other languages fail rather than guess.
The block must already be selected: diagram/tree navigation remains a separate
visual or version-verified procedure. `energy-open` is idempotent when that
dialog is already open and uses the property scroll bar's range API when needed.
Dialog accept/cancel waits for closure. Save As uses exact standard native
filename/control IDs and verified filename readback, not localized label guesses.
Existing output files are never overwritten.

Some controls do not expose a writable UIA ValuePattern. `set-value` reports that
explicitly; it does not pretend an unsupported edit worked. Qt cell display names
can also lag the actual model. Energy reads mark them **non-authoritative**:
validate a GUI-saved project with `lox config energy inspect` before claiming
storage equivalence. Successful UI actions are not domain assertions.
Off-screen input is refused. Automatic grid scrolling and complete legacy-tree
object navigation are not yet provided; do not replace those failures with fixed
pixel offsets.

Recipes are closed JSON data, not scripts: no evaluation, shell fragments,
templates, arbitrary PowerShell, online commands, or `launch` operation.
Example shape for a version-specific recipe:

```json
{
  "schema_version": 1,
  "name": "My independently checked owned workflow",
  "requires": {"exe_sha256": "<full-64-hex-executable-hash>"},
  "steps": [
    {"op": "status"},
    {"op": "windows"},
    {"op": "inspect", "limit": 8, "depth": 1}
  ]
}
```

The checked-in read-only `client-example.json` is independently authored,
contains no vendor project, and is not a version-verified interaction recipe.
Input operations require a hash from the profile, CLI, or recipe. Conflicting
recipe pins fail **locally**, before authentication or remote execution.
Selectors verified against one Config build must not be reused on another
build by changing only the hash.

Recipe operation names use backend spelling: `set_value`, `save_as`,
`energy.read`, `energy.set`, `energy.select`, `energy.accept`, `energy.cancel`;
other names match the short commands. Energy rows are 1–12, columns 0–10,
selection indexes 0–27. These bounds do **not** make every column editable:
column-specific support and semantic checks belong to the backend.

Before any input, the backend must verify the owned PID, known executable
hash, foreground ownership, unique enabled selectors, and occlusion guards.
It stops at the first failure. GUI batches are not atomic and are not rolled
back automatically; inspect partial results before deciding what to do next.
The client sends JSON and the backend path as UTF-8 base64 data, never as
interpolated PowerShell code.

The primitives are for trusted offline recipes, **not a security sandbox**.
Generic keyboard shortcuts can navigate application menus; only use them in the
owned disconnected project. No production deployment/connection workflow is
provided. Failure capture uses the verified owned window via `PrintWindow`, never
an unrestricted desktop fallback, and skips capture when identity is unverified.

## Results, failures, and measured overhead

Default output retains `ok`, completed/failed step counts, structured errors,
bounded read results, backend elapsed time, private artifact paths, and
transport metrics. Bounded inspections are returned directly, including an
exact single-control `inspect --limit 1` read, when they fit the same read
budget: 2048 serialized ASCII JSON bytes per result and 8192 bytes shared across
included read results. Non-read step values and full results remain in the
backend's private `report` and `actions` artifacts. Reads exceeding either
budget, including oversized inspections, explicitly return `omitted: true`,
`reason: "read_size_limit"`, and their report reference; data is not silently
truncated. `--verbose` explicitly returns the full backend JSON reply,
not the MCP envelope, headers, or profile. Treat this output as private too.

No automatic screenshots are requested by the client. Backend failure capture
is subject to backend policy; this client has no screenshot command.

Exit code is 0 only for a valid backend `ok: true` response. A JSON-RPC error,
missing/mismatched response ID, invalid HTTP response, missing PowerShell
status, or **nonzero PowerShell status even when MCP `isError` is false**
fails. Backend `ok: false` preserves partial step results and returns exit 1.
Result `index` and `failed_step` are zero-based recipe step indexes.
There are no automatic retries: a connection interruption may occur after
input, so inspect the requested run directory on the owned machine first.

`transport_metrics` counts HTTP requests, JSON-RPC calls (excluding
notifications), notifications, MCP tool calls, submitted request body bytes,
consumed response body bytes, client transport elapsed milliseconds, and
`output_bytes` (the exact ASCII JSON line including newline). Headers, TLS,
TCP framing, model tokens, and imagined speedups are **not** counted. A normal
batch has 3 HTTP requests: initialize, initialized notification, one tool call;
that is 2 JSON-RPC calls, 1 notification, and 1 MCP tool call.

Run the offline regression tests without contacting any endpoint:

```powershell
python -B -m unittest discover -s scripts -p test_loxone_ux.py -v
python -B scripts\loxone-ux.py --help
```

On Windows, `test_loxone_ux_backend.py` also runs GUI-free PowerShell regressions
for delayed windows, exact value waits, window-bound keys and same-process
occlusion. It does not send desktop input.

## Measured manual check

Launch the independently authored `tests\ux\fixtures\automation-window.ps1` on
the dedicated desktop and explicitly supply its PID and executable hash:

```powershell
python scripts\check-ux-driver.py --profile $env:LOX_UX_PROFILE --pid $ownedPid --expected-sha <fixture-executable-sha256> --output .reverse-engineering\driver-check.json
```

This encodes the four-step value roundtrip, the same four separate calls, stale
pin rejection, ambiguous-selector stop, and unchanged-after-failure readback.
It preserves previous outputs. One final measured run used **1 MCP tool call, 854
stdout bytes, 1.87 s** for the batch, versus **4 calls, 2,475 bytes, 5.61 s** for
the same fixture operations separately. This is one controlled sample, not a
statistical benchmark, a model-token count, or a full Config performance claim.

The real pinned Config build also executed the edit/cancel/reopen recipe and
edit/accept/Save As. Independent native CLI inspection confirmed the saved name
and deadline `27` (`24:00`). Its 11-step edit/cancel/reopen procedure used one MCP
tool call and a 1,350-byte summary in the observed run. Raw projects, captures,
reports and notebook details
remain in ignored/private storage; this does not complete the family parity gate.
