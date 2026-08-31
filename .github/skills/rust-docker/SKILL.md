---
name: rust-docker
description: Build, check, run, or test this Rust workspace in Docker when Cargo is unavailable on the local node. Use for local Rust validation and reproductions without installing a host toolchain.
license: AGPL-3.0
metadata:
  author: eisber
  version: "1.0"
compatibility: Requires a running Docker daemon and PowerShell
allowed-tools: Bash
---

# Rust validation with Docker

Run Rust commands locally in the pinned `rust:1.91-bookworm` image. This matches
the workspace's `rust-version = "1.91"` requirement and avoids modifying the
host toolchain.

## Prepare once

From the repository root in PowerShell:

```powershell
docker info
docker pull rust:1.91-bookworm
docker volume create lox-cargo-registry
docker volume create lox-cargo-target
```

The named volumes preserve downloaded dependencies and compiled artifacts
between runs.

## Run Cargo

Resolve the repository path before mounting it:

```powershell
$repo = (Get-Location).Path
docker run --rm `
  --mount "type=bind,source=$repo,target=/work" `
  --volume lox-cargo-registry:/usr/local/cargo/registry `
  --volume lox-cargo-target:/work/target `
  --workdir /work `
  rust:1.91-bookworm `
  cargo test --workspace --release
```

Replace the final Cargo arguments as needed:

```text
cargo fetch --locked
cargo check --workspace
cargo build --workspace --release
cargo test test_name -- --nocapture
cargo run -- <lox arguments>
```

Pass Cargo directly as the container command. Do not wrap it in `sh -lc` or
`bash -lc`, because a login shell can replace the image's Rust `PATH`.

## Run against a temporary base worktree

For before/after regressions, create a detached worktree under the repository
and mount it independently:

```powershell
git worktree add --detach .tmp-base origin/main
$base = (Resolve-Path .tmp-base).Path
docker run --rm `
  --mount "type=bind,source=$base,target=/work" `
  --volume lox-cargo-registry:/usr/local/cargo/registry `
  --volume lox-cargo-target:/work/target `
  --workdir /work `
  rust:1.91-bookworm `
  cargo test test_name -- --nocapture
git worktree remove .tmp-base
```

Use a different target volume when base and head build artifacts must remain
strictly isolated.

## Limitations

The pinned official image includes `cargo` and `rustc`, but not `rustfmt`,
Clippy, or `rustup`. Use the repository CI or an image explicitly built with
those components for `cargo fmt` and `cargo clippy`; do not report them as
validated from this image.
