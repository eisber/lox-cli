# Toolkit validation

Use the repository entry point instead of copying a per-feature qualification
script. Python 3.11+ is required. Compiler targets and result manifests must be
outside the source checkout; keep them on an approved local cache volume.

```powershell
python scripts\validate-toolkit.py dev --offline `
  --target-dir E:\work\lox-campaign-cache-3f18a6ad\parent-target `
  --results E:\work\lox-campaign-cache-3f18a6ad\validation\checks.json

python scripts\validate-toolkit.py gates --offline --test-threads 1 `
  --target-dir E:\work\lox-campaign-cache-3f18a6ad\parent-target `
  --results E:\work\lox-campaign-cache-3f18a6ad\validation\checks.json
```

| Phase | Checks |
|---|---|
| `dev` | Locked workspace `cargo check` |
| `gates` | Formatting, warnings-as-errors clippy, release workspace build, complete workspace tests |
| `packages` | **Fresh** workspace Cargo package verification, existing archive-content checks, actual standalone simulator tests and benchmark smoke |
| `native` | Current native library build, then existing Python/Java22/Kotlin artifact/error/resource checks, or verified exact-artifact proof reuse |

`--features` is applied to development and final gates. `--test-threads` is
explicit and part of the gates' identity; the default remains one until a
controlled workload comparison justifies a different value. `--force` runs a
phase even if an identical successful result is retained. It is not a retry
policy: failed checks return their original nonzero status and stop that phase.

## What can be reused

One compact JSON manifest records each phase's logical commands, raw source
hashes, Cargo configuration, toolchain identity, a non-disclosing environment
hash, command timings, logs and output hashes. Only a completed success with all
retained artifacts unchanged can be reused. A source, command, feature,
environment, toolchain or output change invalidates the corresponding key.
Missing or modified logs are not successful evidence.
Unsupported Cargo execution/local-source overrides explicitly disable result
reuse; checks still run. The script does not infer a custom compiler wrapper's
transitive implementation inputs.

Both executions use the same working-directory and compiler-target identity.
This intentionally does not import historical campaign receipts, infer identity
from a crate version, or transfer success between unrelated checkouts. Cargo
still manages its ordinary dependency/compiler cache; the result manifest is
not a replacement build system.

Validation uses a private configuration with telemetry disabled and external
HTTP proxies closed. Only the complete Rust test gate permits owned loopback
traffic. The 16 MiB Rust test-thread stack setting is limited to that gate, not
native tests or package checks. No Miniserver, context, user token or real-device
access is required.

## Release package boundary

```powershell
python scripts\validate-toolkit.py packages --offline `
  --target-dir E:\work\lox-campaign-cache-3f18a6ad\parent-target `
  --results E:\work\lox-campaign-cache-3f18a6ad\validation\checks.json
```

This phase always allocates a fresh package compiler target, including when
source names and versions match a previous build. It never uses a cached
successful result to skip genuine clean-package verification. The existing
`verify-packages.py` and `test-simulator-package.py` remain authoritative; the
standalone extraction/compiler identity includes the actual archive digest.

Preserve actual archives, unpacked source, locks, logs and selected proof
artifacts before retiring an idle package compiler cache. The entry point
does not delete caches, move live targets, publish packages or push commits.
Required independent review and publication-specific behavioral proofs remain
separate obligations.

## Native proof boundary

`native` currently supports the existing Windows AMD64 builder:

```powershell
python scripts\validate-toolkit.py native --offline `
  --target-dir E:\work\lox-campaign-cache-3f18a6ad\parent-target `
  --results E:\work\lox-campaign-cache-3f18a6ad\validation\checks.json `
  --maturin C:\tools\maturin.exe --java-generator C:\tools\uniffi-bindgen-java.exe `
  --java-home C:\tools\jdk-22 --kotlin-compiler C:\tools\kotlinc\bin\kotlinc.bat
```

Use your installed tool paths. Versions and platform requirements of
`build-archive-bindings.ps1` are unchanged. Kotlin uses the same generated Java
API, not a separate implementation.

Every invocation first asks Cargo to build the **current** release library.
Only identical library bytes, the conservative complete source inventory,
toolchain/environment
and all retained generated artifacts/fixtures/proof outputs can reuse the
previous native success. A changed library cannot inherit success merely because
the binding source is unchanged. The library inside the actually qualified
Python package must match the current build before a result becomes reusable.

Native-library compilation time and native proof execution time are separate.
Source mutation during either compilation or qualification prevents recording
success. This does not claim reproducible native binaries, a stable ABI, another
platform's qualification, or general CLI/API parity.
Kotlin compiler JARs and the Java runtime implementation are hashed in addition
to their launchers. Native command records and every top-level proof log remain
required, including allocation checks and the current-library build.
