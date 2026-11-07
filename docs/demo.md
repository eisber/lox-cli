# README terminal demo

The [30-second GIF](assets/demo.gif) renders **actual `lox` subprocess output**,
not screenshots of a private installation. All server responses come from a
small HTTP mock bound to `127.0.0.1` on an automatically assigned port. The
light is an in-memory integer; the room and identifiers are synthetic. The
clock is fixed mock data, not a live Miniserver clock.

The [`record-demo.py` source](../scripts/record-demo.py) contains the reusable
command sequence, mock endpoints and independently authored XML fixture. The
[captured transcript](assets/demo-transcript.json) records commands, complete
combined output, exit codes and the verified HTTP request sequence. Command
typing, page transitions and reading pauses are editorial playback timing;
output is only line-wrapped, never rewritten or replaced with canned CLI text.

## Reproduce

Requires Python 3.10+, Pillow (11.3.0 used for the checked-in GIF), a monospace
TrueType font, and a freshly built `lox` binary. Pillow is also used by existing
repository image scripts; no recording service, browser or global configuration
is needed. The default font is DejaVu Sans Mono; pass `--font` for another local
monospace font. Fonts are not redistributed.

From the repository root in PowerShell:

```powershell
cargo build --bin lox
python scripts\record-demo.py --lox target\debug\lox.exe
```

For a private Cargo target, pass its absolute `lox.exe` path instead. On
macOS/Linux, use the corresponding native paths and executable without `.exe`.
The script writes `docs/assets/demo.gif` and `docs/assets/demo-transcript.json`.
It creates then removes its own `target/readme-demo` sandbox, refusing to reuse
an existing directory. It does not read your Miniserver config: inherited
`LOX_*` variables and proxy settings are discarded, config/cache/home are
isolated, and telemetry is explicitly disabled. No secrets are required.

Generation fails if a command unexpectedly fails, mock requests differ from the
allowlist, on/off readback is wrong, the template is not written, output clips,
the GIF does not decode as 21 frames / 30 seconds, or its size exceeds 2 MB.
Run the focused safety tests with
`python -m unittest discover -s scripts -p test_record_demo.py`.
These tests need only the standard library: the optional clipping/rendering test
explicitly skips if Pillow or its font is unavailable. Actual GIF generation
requires both and fails with setup guidance rather than claiming success.

## Current command scope

Issue #2 originally requested `status`, `ls`, `on/off`, `config template`,
`library ls` and `config push`. Current main has no top-level `status`, `ls`,
`on/off` or `library` commands. This demo therefore shows:

| Intent | Current command shown |
|---|---|
| Server inspection | `lox live time` (mock clock, **not** health/status) |
| List controls | `lox config controls demo.Loxone -o json` (local file) |
| On/off | `lox live set 'Demo light' on/off --write`, then `live get` |
| Room template | `lox config template demo.Loxone hallway --room Studio` |
| Block library | `lox blocks ls --category lighting --limit 3` |
| Deployment safety | `lox config push demo.Loxone --non-interactive` |

**The final push deliberately exits 1 because `--force` is absent.** It does not
upload, stage, reboot, reload, or contact FTP. This is a demonstration of the
deployment guard, not a successful hardware deployment. Do not add `--force` or
`--reboot` to the recording sequence. Applying the template only edits synthetic
XML; it does not establish hardware bindings or prove native runtime behavior.
