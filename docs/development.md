# Building, measuring, and packaging

## Local iteration

Use the existing development profiles; no reduced test coverage or release
optimizations are needed for a normal edit:

```powershell
cargo check --workspace --locked
cargo test --workspace --locked
cargo build --workspace --locked
```

Run a targeted test selector first when it covers the edit. Before committing or
pushing, the required integration gates remain `cargo fmt --check`,
`cargo clippy -- -D warnings`, `cargo build --release`, and `cargo test`.
CI additionally checks the workspace on Windows, Linux, and macOS.

The Windows build script reserves the existing 8 MiB stack for the `lox`
executable. Its only file input is `build.rs`; changing documentation, scripts,
or task metadata must not rerun that script and relink the executable. Target
selection is managed by Cargo. The static MSVC CRT settings are unchanged.

CI caches are separated by workload, architecture, and installed compiler
version. A lockfile change can reuse the same compiler/workload dependency
cache; lint and release-test jobs no longer compete to save one Linux cache.
Build and test commands use `--locked` so CI cannot silently resolve a different
dependency graph. Cache speedups have not been measured on hosted runners.

## Reproducible measurements

Do not clean a shared target or benchmark a checkout that another worker edits.
Create a private source worktree at the commit being measured:

```powershell
git worktree add --detach .reverse-engineering\bench\baseline HEAD
cargo fetch --manifest-path .reverse-engineering\bench\baseline\Cargo.toml --locked

python scripts\measure-build.py --label before-check-cold `
  --cwd .reverse-engineering\bench\baseline `
  --target-dir .reverse-engineering\bench\target-check `
  --output .reverse-engineering\bench\measurements.jsonl --jobs 8 `
  -- cargo check --workspace --locked --offline
```

For independently cold check/test/build measurements, use a different **new**
target directory for each workload. Repeat each command for multiple warm
samples. Dependency downloads are a separate preparation step, not build time.
The measurement script binds its target directory to the source working
directory and rejects a subsequent run from another location. Archive extraction
can preserve identical file timestamps: reusing a target after relocating such
sources was observed to retain an old compiled `CARGO_MANIFEST_DIR`, so use fresh,
source-specific targets for archive install/test probes too.
An existing nonempty target without a measurement-source marker is rejected
rather than silently adopting artifacts of unknown origin.
For source-edit invalidation, change one leaf Rust source file in the private
worktree, run the same workloads, and restore the file. Also measure a README
edit to distinguish compiler invalidation from accidental build-script
invalidation. Compare equivalent source edits, profiles, features, job counts,
and command flags before and after a change.

The standard-library-only measurement script records command, source commit,
working-tree status and tracked diff, toolchain, relevant environment, elapsed
time, and exit status. Windows CPU and peak process-family working-set memory
are sampled every 100 ms by default: short-lived processes and between-sample
peaks can be missed, so CPU is a lower bound, not total system utilization.
Other platforms explicitly report resource metrics as unmeasured. Measurement
and sampling overhead is included in elapsed time. Logs and source patches stay
beside the JSONL output; `.reverse-engineering` is ignored and never packaged.
An interrupted run terminates only its own command process family.

## Source packages

The root package explicitly includes source, embedded schemas, runtime
documentation, contract files, Rust integration tests, UX fixtures, licenses,
and the eval cases read by `lox blocks`. Historical eval reports, golden
exports, task databases, local reverse-engineering assets, CI files, and
development scripts are not application package inputs. The simulator package
includes its sources, tests/fixtures, and benchmark target.
The CLI preserves its original sanitized `tests\eval\fixture.Loxone`.
The simulator ships one byte-identical 52,165-byte copy as
`tests\fixtures\eval.xml`, used by integration tests, benchmark smoke tests,
and the trace test's fallback. Required tests no longer read parent-directory
fixtures. Large optional developer corpora are not distributed: the existing
integration/equivalence corpus checks and explicitly ignored manual alarm smoke
test retain their original behavior.
Whitelist entries are rooted with `/`: Cargo's unanchored `README.md` pattern
also matches a nested `.beads/README.md`, which the actual archive check caught.

Verify **actual archives**, not just `cargo package --list`:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) ".reverse-engineering\bench\target-package"
$env:CARGO_BUILD_JOBS = "8"
cargo package --workspace --locked
python scripts\verify-packages.py `
  .reverse-engineering\bench\target-package\package\lox-cli-0.14.2.crate `
  .reverse-engineering\bench\target-package\package\lox-sim-0.1.0.crate
```

`--allow-dirty` may be added for a deliberate uncommitted manifest experiment;
do not use it for a release. On the tested Cargo 1.99.1,
`cargo package --workspace` orders `lox-sim` before `lox-cli` and stages the
local simulator archive in a temporary Cargo registry to build the CLI archive.
The normalized CLI manifest correctly keeps the versioned registry dependency;
it does not ship a working dependency on the repository's `lox-sim` directory.
CI builds these archives and checks their inventory for private assets and
missing literal `include_str!`/`include_bytes!` files.
The release workflow also retains package build verification instead of
publishing with `--no-verify`; its publish commands now enforce the lockfile.
CI additionally tests the simulator directly from its archive:

```powershell
python scripts\test-simulator-package.py `
  --package-dir .reverse-engineering\bench\target-package\package `
  --work-dir .reverse-engineering\bench\standalone-simulator `
  --target-dir .reverse-engineering\bench\target-standalone-simulator `
  --offline --jobs 8
```

The script chooses the current simulator version from Cargo metadata (or accepts
an explicit `--archive`), checks the archive, and runs ordinary `cargo test`
followed by `cargo test --bench sim_bench` in Criterion's smoke-test mode. Source
and target directories are keyed by the archive's SHA256. A one-package workspace
boundary prevents the enclosing checkout from claiming the extracted crate;
it uses the archive's own lockfile and the standalone edition-2021 resolver.
The crate is not modified and no parent fixtures are copied. Previously extracted source must remain
byte-identical. Tests retain the archive's own lockfile; no companion package,
live repository data, feature switch, or test omission is required.

For an offline install probe before registry publication, unpack **both**
archives beneath an ignored project-local directory. Create a disposable
workspace there with the two package directories as members, resolver `3`, and
a `[patch.crates-io]` entry pointing `lox-sim` to the unpacked simulator. Copy the
repository lockfile to that workspace. Its disposable manifest is:

```toml
[workspace]
resolver = "3"
members = ["lox-cli-0.14.2", "lox-sim-0.1.0"]

[patch.crates-io]
lox-sim = { path = "lox-sim-0.1.0" }
```

On Windows also copy the unpacked CLI's `.cargo` directory to that workspace
root to retain the packaged static CRT configuration. The simulator now carries
its own required fixtures; no workspace-level fixture reconstruction is needed.
Run the install command
from the disposable workspace, using absolute paths for the package/install
directories and an isolated `CARGO_TARGET_DIR`:

```powershell
$bench = Join-Path (Get-Location) ".reverse-engineering\bench"
$env:CARGO_TARGET_DIR = Join-Path $bench "target-install"
Push-Location (Join-Path $bench "install-source")
cargo install --path (Join-Path $bench "install-source\lox-cli-0.14.2") `
  --root (Join-Path $bench "install-root") --debug --locked --offline
& (Join-Path $bench "install-root\bin\lox.exe") --version
& (Join-Path $bench "install-root\bin\lox.exe") --help
cargo test --workspace --locked --offline
Pop-Location
```

This verifies installation from the archives with the unpublished companion
package supplied locally. It is **not** proof that `cargo install lox-cli` works
against crates.io. Registry availability and publication policy remain separate
release prerequisites. No package or license declaration is changed to bypass
them: the CLI retains `AGPL-3.0-or-later AND LicenseRef-Commercial`, and `lox-sim`
retains `AGPL-3.0-or-later`.

## Recorded Windows experiment (2026-11-05)

Source: `8f96466696517aca10472d8dc99e12d714990d49` (unchanged Rust sources from
`ce7a517`, plus Beads initialization). Host: 32 logical processors, approximately
64 GiB RAM, `x86_64-pc-windows-msvc`; rustc 1.99.1 (`58cf8f9c3`), Cargo 1.99.1
(`5f94df478`), default development profiles, existing static CRT flags, 8 Cargo
jobs. Locked dependencies were fetched first after an offline attempt reported
missing cached crates; downloads are excluded below.

Each cold workload had its own empty target directory:

| Workspace workload | Cold elapsed (s) | Sampled family CPU (s) | Sampled peak working set (bytes) |
| --- | ---: | ---: | ---: |
| check | 35.003 | 228.969 | 1,522,974,720 |
| test | 65.887 | 375.844 | 2,277,793,792 |
| debug build | 54.209 | 253.734 | 1,973,846,016 |

For the paired comparison, only `build.rs` changed; package whitelists and
other workers' source changes were **not** included. Commands were
`cargo <check|test|build> --workspace --locked --offline -v`. Each cell is the
median of three samples; all successful paired samples, including outliers,
were retained. The source edit appends a harmless comment to
`src\commands\blocks_cmd.rs`, invalidating the same compilation unit before and
after; the documentation edit appends a comment to `README.md`.

| Workload | Warm before → after (s) | Source edit before → after (s) | README edit before → after (s) |
| --- | ---: | ---: | ---: |
| check | 0.832 → 0.669 | 1.573 → 1.339 | 1.584 → 0.651 |
| test | 2.392 → 2.169 | 8.189 → 6.960 | 8.043 → 2.153 |
| debug build | 0.787 → 0.675 | 4.278 → 3.538 | 3.761 → 0.657 |

Verbose Cargo logs confirm that every README-edit sample previously reran the
build script and recompiled `lox`; none did either after the change. Required
Rust-source recompilation still occurs. There is no claimed cold-build or
hosted-CI speedup, and no dependency/profile change was justified by these
measurements. Shared-page working sets can be counted in multiple processes;
these are sampled aggregate working sets, not unique physical memory usage.
Elapsed timing includes sampling overhead and ordinary shared-host variation.

Raw commands, source diffs, elapsed/CPU/memory samples, Cargo logs, and the
aggregation live locally under `.reverse-engineering\bench` in
`baseline.jsonl`, `comparison.jsonl`, and `summary.json`. The initial baseline
also retained an anomalous 12.242-second warm build; the independent paired
comparison above is the basis for the reported medians.

Actual Cargo archive inventories on the same frozen source assets:

| Package | Files before → after | Uncompressed file bytes before → after | Compressed archive bytes before → after |
| --- | ---: | ---: | ---: |
| lox-cli | 2,500 → 2,350 | 32,888,978 → 19,425,660 | 5,443,467 → 3,166,213 |
| lox-sim | 41 → 40 | 1,003,391 → 1,003,585 | 178,306 → 178,328 |

The CLI's uncompressed package shrank approximately 41%, principally by
removing historical exports/reports rather than runtime assets. Both anchored
archives passed Cargo's actual package build verification and the inventory
checker; no `.beads` or `.reverse-engineering` path remained. The simulator's
minor byte increase is manifest metadata, not an iteration-speed claim.
These controlled counts do not include other workers' subsequent source or
contract changes, or this new development guide; CI checks the final integrated
archives again.

The unpacked-workspace probe preserves archive hashes and the reconstructed
workspace/lockfile description in
`.reverse-engineering\bench\install-reconstruction.json`; this, rather than the
enclosing repository's Git status, identifies the unpacked source. Install/test
command and resource samples are in `package-verification.jsonl`. Failed
intermediate experiments are retained: the unanchored README leak, the missing
shared fixture, and stale cached manifest paths after source relocation were
detected rather than hidden or worked around by skipping tests.

Final proofs from the reconstructed archives, using new source-bound targets:

| Command | Elapsed (s) | Sampled family CPU (s) | Sampled peak working set (bytes) | Result |
| --- | ---: | ---: | ---: | --- |
| `cargo install --path <unpacked-cli> --root <local-root> --debug --locked --offline` | 44.333 | 221.969 | 1,939,652,608 | installed |
| `cargo test --workspace --locked --offline` | 60.594 | 341.188 | 3,514,888,192 | 925 passed, 2 ignored, 0 failed |

Installed `--version` and `--help` both exited successfully from the isolated
install directory, outside the source checkout. All 62 archived Rust-source,
build-script, and license/notice files checked against the frozen edited
snapshot matched byte-for-byte. Four in-memory package-verifier regression
tests, `rustfmt --check build.rs`, shared-target rejection, and cross-source
target-reuse rejection also passed. No crates were published, and final shared
integration/release gates are still the coordinator's responsibility.

Source-package CI adds a correctness gate and costs an additional build;
separate workload caches use more storage. Neither change is claimed to reduce
total hosted-CI wall time. Local registry staging is not a publication or a
license-policy approval. Cargo installs the binary, not external documentation;
`lox docs` still uses the existing repository/binary-directory/`~/.lox/docs`
lookup and manual documentation-install workflow.
