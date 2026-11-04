# Repeatable local Loxone Config analysis

`scripts\reverse-loxone.py` uses a local REA checkout and Ghidra to inventory
Loxone Config and export pseudocode for every function the provider discovers.
It never executes Loxone Config, connects to a Miniserver, or uploads evidence.
All binaries, recovered text, databases, logs, and provider installations belong
in the Git-ignored `.reverse-engineering` directory, not in commits or CI
artifacts. Publish independently implemented interoperability changes and
synthetic regression fixtures instead of recovered proprietary code.

Run analysis on demand. No scheduled task or background watcher is required or
installed by the analysis command.

## Prerequisites

On Windows x64, use Python 3.12+, REA's supported Node.js version, Ghidra 12.1+
and a compatible full JDK. Ghidra's input and temporary volumes must be local
NTFS, without junctions in admitted paths.

For the existing `E:\work\rea` checkout:

```powershell
Set-Location E:\work\rea
npm ci --no-audit --no-fund
npm run build:cached
```

If building reports missing Windows native controls, follow
`E:\work\rea\native\windows\README.md`: restore only the native addon and its
manifest from the published REA package whose version **exactly** matches that
checkout. Never substitute another version. REA verifies its digest and ABI.
Ghidra and Java are separate caller-installed prerequisites.
Build the local archive decoder with `cargo build --release` in `lox-cli`
before the first run; `--lox` can select an existing compatible `lox.exe`.

## Run manually after an update or interruption

From `E:\work\lox-cli`, using the actual installed provider directories:

```powershell
python .\scripts\reverse-loxone.py --rea-root E:\work\rea --ghidra-dir E:\work\lox-cli\.reverse-engineering\tools\ghidra_12.1.4_PUBLIC --java-home E:\work\lox-cli\.reverse-engineering\tools\jdk-21.0.13.0.2+1
```

Rerun this same command whenever you want to analyze an installed Config update
or continue an interrupted export. No separate reset or version-selection step
is needed. The process runs in your terminal until it finishes or you stop it.

The default target is the installed `LoxoneConfig.exe`. Repeat `--binary` to
select additional installed vendor PE filenames, for example
`--binary LoxoneConfig.exe --binary LoxoneMonitor.exe`. Third-party Qt and
Chromium libraries are frozen and hashed but are not decompiled by default.
`--install-dir` selects another installation.

The pipeline freezes the entire installation under its content digest and
checks for changes during capture. Future updates, including resource-only
updates, create separate snapshots. Existing snapshots are verified before
reuse. Tool/runtime/bridge digests and analysis settings select an independent
analysis directory. An unchanged completed analysis is reused.

Every installed root-level `.LxRes` catalog is decoded through `lox config
extract`, with telemetry explicitly disabled. Output lengths and CRC32 values
are checked against their headers and XML is parsed. The analysis-level
`resources` directory retains every decoded locale, extraction diagnostics,
hashes, and a structured `block-catalog.json` from `tdc_DEV.LxRes` when present.
Numeric control/connector identities, defaults, templates, and group directions
remain intact. Resource connector labels are not assumed to be XML wire keys.

One persistent REA session per binary shares the expensive Ghidra import
across inventories and decompilation. The default JVM heap is 6 GiB; cold
analysis has a two-hour allowance, and individual decompiles have a 60-second
deadline. Set `--heap`, `--startup-timeout`, and `--decompile-timeout` when
needed. These are limits, not estimates of runtime.

An interrupted export resumes using REA's append-only journal and verifies
existing function files before skipping them. Ghidra's database is ephemeral:
restarting the process repeats import and auto-analysis, even when exported
functions can be reused. Interrupting before export begins leaves no function
journal to resume. Failed functions remain explicit and are retried on the next
manual run. Concurrent runs cannot share the output root.
Exit status 0 means the selected discovered functions were exported or
classified external, 2 means partial results, and 1 means pipeline failure.

## Read the results

Start with `.reverse-engineering\latest.json`. It identifies the version,
analysis directory, status, counts, and each binary's research database.

| File in a binary's analysis directory | Content |
|---|---|
| `overview.json` | REA's provider observations and limitations |
| `list_*.json`, `segments.json`, `imports.json` | Inventories or retained-inventory summaries |
| `inventory.evidence.json` | Complete original inventory Evidence bundle |
| `code\decompile-index.json` | All discovered functions, identities, outcomes and caveats |
| `code\decompile-journal.jsonl` | Resumable per-function export history |
| `code\functions` | Address-named recovered pseudocode, kept local |
| `complete.evidence.json` | Final retained REA Evidence, including export manifest |
| `research.sqlite3` | FTS5 index of verified recovered function bodies |
| `topics.json` | String-match leads for formats, block semantics and protocols |
| `rea.stderr.log` | Provider diagnostics |

`installation-diff.json`, when present in the version directory, compares file
content identities with the previous selected version. It is not a semantic
function matcher: addresses and generated names can move between releases.

Search a `research.sqlite3` file with ordinary SQLite:

```sql
SELECT address, name, path
FROM functions
WHERE functions MATCH 'DayTimer OR LightController2';
```

Topic matches are leads, not verified behavior. Trace their references and
control flow through REA before drawing conclusions. Use synthetic configs to
compare any independent implementation with observed behavior.

## Optional scheduling (explicit opt-in)

Skip this section for manual operation. Only run the registration helper if you
explicitly want Windows to launch repeated analyses automatically.

```powershell
.\scripts\register-loxone-analysis.ps1 -ReaRoot E:\work\rea -GhidraDir E:\work\lox-cli\.reverse-engineering\tools\ghidra_12.1.4_PUBLIC -JavaHome E:\work\lox-cli\.reverse-engineering\tools\jdk-21.0.13.0.2+1
Get-ScheduledTaskInfo -TaskName LoxCli-LoxoneConfig-Analysis
```

This registers an hourly, limited-privilege Windows task for the current
interactive user. The user must be logged in and the machine awake.
`-IntervalHours` changes the cadence. Existing task definitions are backed up
locally before replacement; `-WhatIf` previews registration. It does not update
Loxone Config, REA, Java, or Ghidra.

Unattended output is appended to `.reverse-engineering\scheduled.log`; check
`latest.json` and the task's last exit status for failures. A provider upgrade
or changed executable triggers fresh analysis rather than presenting old
results as current. A failed/interrupted export is resumed on the next check.

To stop an active scheduled run and remove its schedule without deleting saved
analysis:

```powershell
Stop-ScheduledTask -TaskName LoxCli-LoxoneConfig-Analysis
Unregister-ScheduledTask -TaskName LoxCli-LoxoneConfig-Analysis -Confirm:$false
```

## Resource metadata

The installed `tdc_*.LxRes` and `tdd_*.LxRes` files in Config 17.2.10.21 use
the same LoxCC container magic as config archives. The CLI's existing
decompressor can extract these directly:

```powershell
.\target\release\lox.exe config extract "C:\Program Files (x86)\Loxone\LoxoneConfig\tdc_DEV.LxRes" --save-as .reverse-engineering\blocks.xml
.\target\release\lox.exe config extract "C:\Program Files (x86)\Loxone\LoxoneConfig\tdd_DEV.LxRes" --save-as .reverse-engineering\devices.xml
```

Keep the decoded catalogs local too. Format magic alone does not establish
schema compatibility with a later release.

## Coverage limits

Complete export is **not** complete original-source recovery. Stripped names,
optimized/inlined code, unresolved indirect calls, and unidentified functions
can remain. The observed binary references a PDB, but that does not make its
symbols available. Provider counts and failures measure discovered-function
coverage only; preserve REA's original limitations and uncertainty.

Pipeline regression tests use synthetic files and no proprietary executable:

```powershell
python -m unittest discover -s scripts -p test_reverse_loxone.py
```
