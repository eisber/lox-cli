# lox-core

Headless Rust library (Rust 1.91 minimum), currently available from this
workspace by path dependency:

```toml
[dependencies]
lox-core = { path = "path/to/lox-cli/crates/lox-core" }
```

```rust
use lox_core::configuration::archive::{
    ArchiveError, compress_loxcc, extract_and_decompress,
};

fn roundtrip(bytes: &[u8]) -> Result<Vec<u8>, ArchiveError> {
    let archive = compress_loxcc(bytes);
    extract_and_decompress(&archive)
}
```

## Available now

- `configuration::archive::{compress_loxcc, decompress_loxcc,
  extract_and_decompress}`: in-memory LoxCC/LxRes and backup ZIP byte codecs.
- Typed `ArchiveError`/`Result`, including header/back-reference fields and
  underlying ZIP/I/O errors with member context.
- CLI `lox config extract` and `lox config compress` use this implementation.
  CLI filesystem/path/output/error-envelope/context handling and acquisition
  baselines remain in the binary, as do deployment/sidecar/snapshot trust guards.

The library does not read files, select a connection context, print, start a
runtime or parse/normalize XML. Arbitrary decoded bytes are preserved, including
BOM, mixed line endings, duplicate attributes and non-UTF-8 data.

## Compatibility, not strict validation

The decoder preserves the existing CLI's permissive format acceptance. It does
not verify LoxCC CRC32 or declared sizes and may return partial output for
truncated final tokens. The allocation hint cap is not a total output limit.
ZIP selection uses the first case-insensitive name containing `sps0.loxcc`,
including nested names and suffixes; ZIP member integrity errors still propagate.
Compression emits LZ4 blocks and CRC32 over the exact original bytes; its size
fields retain the existing `u32` casts.

These APIs do not establish that an archive is safe to deploy. Strict
verification/resource limits would require a separately designed explicit
policy, not a silent change to this compatibility codec.

## Not exposed yet

XML parsing/editing, runtime/network/authentication operations, and Python or
Java/Kotlin native bindings are **not** exposed through `lox-core`. Existing CLI
capabilities outside this archive vertical remain binary-private. No cross-head
parity, native package availability or ABI stability is claimed.

Run `cargo test -p lox-core` from the workspace. This stage is validated on
Windows AMD64; other platforms are not newly qualified by it.

## License and provenance

This is an extraction of this project's own archive code, not an import of an
upstream client/config crate. Licensing and attribution are unchanged:
see the repository [LICENSE](../../LICENSE), [LICENSE-AGPL](../../LICENSE-AGPL),
[LICENSE-COMMERCIAL](../../LICENSE-COMMERCIAL) and [NOTICE](../../NOTICE).
The package retains `AGPL-3.0-or-later AND LicenseRef-Commercial` metadata.
Format reference: [Inside-The-Loxone-Miniserver](https://github.com/sarnau/Inside-The-Loxone-Miniserver).
CRC32 discovery and plaintext-field notes originate with eisber/lox-cli contributors.
