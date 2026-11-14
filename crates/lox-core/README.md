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
- Optional `runtime` feature: async `ClockClient::read_clock`, typed observations,
  explicit validation, cancellation, HTTP errors and bounded close.
- Optional `runtime-blocking` feature: `ClockDriver`, the explicitly owned
  blocking facade used by `lox live time`.

The library does not read files, select a connection context, print, implicitly
start a runtime or parse/normalize XML. Arbitrary decoded archive bytes are
preserved, including BOM, mixed line endings, duplicate attributes and non-UTF-8
data. Default features are empty; archive-only consumers do not depend on
reqwest/Tokio/serde_json through the library's normal dependencies.

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

## Optional clock runtime

```toml
lox-core = { path = "path/to/lox-cli/crates/lox-core", features = ["runtime"] }
# For explicitly owned blocking use instead: features = ["runtime-blocking"]
```

```rust
use lox_core::runtime::{Cancellation, ClockClient, ClockError, ClockOptions, ClockObservation};

async fn observe(options: ClockOptions) -> Result<ClockObservation, ClockError> {
    let client = ClockClient::new(options)?;
    let result = client.read_clock(&Cancellation::new()).await;
    let closed = client.close().await;
    let observation = result?;
    closed?;
    Ok(observation)
}
```

`ClockOptions::new(host, user, password)` takes explicit settings; no global
configuration, token file, structure cache or device discovery is consulted.
HTTP Basic credentials (including empty credentials) are accepted. Defaults match
the CLI: 10 seconds per attempt, four attempts with 200/400/800ms retry delays,
and self-signed TLS acceptance (`verify_tls = false`). Opt into certificate
verification as appropriate. System proxies are honored by default; isolated
loopback callers can explicitly set `use_system_proxy = false`. Redirect checking
deliberately retains the legacy host-only restriction, not a new scheme/port
policy. HTTP 4xx responses are not retried; other non-success statuses, transport
and JSON failures are. Date is requested before time, without rereading date when
time fails.

The return type is a **wire observation**, not an assertion of a valid clock.
`ClockField` records the LL envelope state, text/number/missing/unsupported value,
and uppercase/lowercase protocol codes without exposing arbitrary JSON.
`validate()` requires LL objects, code 200 (uppercase preferred, lowercase used
only when uppercase is absent), and nonempty string or finite numeric values.
It returns typed `ClockValidationError` with endpoint and issue. It does **not**
validate calendar syntax, date/time ranges, timezone, atomicity across the two
requests, or firmware meaning.

`legacy_text()` is a deliberately permissive conversion: ignore LL codes, keep
strings and f64-to-string numbers, and map missing/unsupported values to empty
strings. `lox live time` explicitly uses this path to retain its existing
syntax, stdout, error envelope/categories, exit status, context/TLS/auth selection
and request retries. Success-shaped defaults are not the new observation API's
meaning. Other `live` operations retain their original binary-private decoder.

### Executors, cancellation and close

- `ClockClient` starts no thread/runtime. Its futures need a Tokio executor with
  I/O and time enabled. Dropping a read future aborts the request and unregisters
  it. Clones share close state. A sticky `Cancellation` can be signaled before or
  during a request or retry delay; in-flight shutdown reports
  `Cancelled(Shutdown)`, explicit cancellation reports `Cancelled(Requested)`,
  and reads started after close report `Closed`.
- `request_close()` synchronously rejects new work and cancels in-flight reads.
  Async `close()` also waits up to `shutdown_timeout` (default one second) for
  registered operations to drain. A caller retaining an unpolled read future can
  prevent drain: `ShutdownTimeout { active }` reports that limit honestly. Poll
  or drop such futures, then retry close. No work is secretly spawned to poll
  caller-owned futures.
- `ClockDriver::new(options)` explicitly starts a dedicated current-thread Tokio
  worker. `read_clock` blocks the caller, admitting one operation at a time;
  concurrent calls report `Busy` rather than waiting uncancellably in a queue.
  Async Rust users should prefer
  `ClockClient`. Blocking calls and disposal are safe under an existing executor:
  no nested `block_on`, and runtime destruction occurs only on the owned thread.
- Driver `close()` is idempotent and cancels reads, drops queued requests, and
  waits at most `shutdown_timeout` for worker completion. Success joins the
  worker. A timeout leaves the driver closed and can be retried. Drop is bounded
  best effort, not a substitute for checking close. Runtime teardown bounds its
  wait for platform blocking/DNS tasks to 100ms; this cannot promise forcibly
  terminating an OS DNS call or a blocking user diagnostic observer.
- Diagnostics are opt-in typed metadata, never implicit printing. Observers
  execute synchronously outside state locks and must not block or panic. URLs,
  credentials, raw response bodies and backend error chains are excluded.
  CLI `-v` retains request/status messages. Explicit CLI `-vv` separately selects
  `ClockResponsePolicy::LegacyCliVerbose`: reqwest response-text replacement
  decoding followed by the same shared JSON/field decoder, plus the CLI-owned
  existing raw-body/truncation logger. This opt-in can reveal response contents;
  it is not enabled for ordinary library/foreign callers. Default `StrictJson`
  still validates raw JSON bytes strictly. The separate legacy body observer
  receives borrowed decoded text synchronously outside state locks and must not
  block or panic; ordinary metadata/errors remain payload-free. Body metadata
  counts raw bytes in strict mode and decoded text bytes in legacy mode.
  Connection options, policy/callback Debug and text payload Debug are redacted;
  strings are not promised zeroized.
- Subscriptions, event streams, callback delivery bridges, and subscription
  backpressure are **unsupported**, not simulated by this stateless operation.

Validation uses production Rust APIs and loopback HTTP only, including stalled
socket disconnects after timeout/cancel/close, caller-retained future timeout,
owned-worker joining under an existing executor, and the unchanged 22-case CLI
characterization. No real device, firmware, native/package or VM readiness
qualification is implied.

## Not exposed yet

XML parsing/editing, runtime operations beyond this HTTP clock read, token/crypto/
WebSocket authentication, subscriptions, and Python or Java/Kotlin native bindings
are **not** exposed through `lox-core`. Existing CLI capabilities outside the
archive and clock verticals remain binary-private. No cross-head parity, native
package availability or ABI stability is claimed. The approved generated UniFFI
Python / one Java 22+ FFM API direction is unchanged; this stage adds no bindings.
Archive byte guarantees and private deployment/sidecar policies are unchanged;
shared-loader issue `lox-n37w` remains unresolved and is not broadened here.

Run `cargo test -p lox-core` for offline codecs, or
`cargo test -p lox-core --all-features` for the clock tests too. This stage is
validated on Windows AMD64; other platforms are not newly qualified by it.

## License and provenance

This is an extraction of this project's own archive code, not an import of an
upstream client/config crate. Licensing and attribution are unchanged:
see the repository [LICENSE](../../LICENSE), [LICENSE-AGPL](../../LICENSE-AGPL),
[LICENSE-COMMERCIAL](../../LICENSE-COMMERCIAL) and [NOTICE](../../NOTICE).
The package retains `AGPL-3.0-or-later AND LicenseRef-Commercial` metadata.
Format reference: [Inside-The-Loxone-Miniserver](https://github.com/sarnau/Inside-The-Loxone-Miniserver).
CRC32 discovery and plaintext-field notes originate with eisber/lox-cli contributors.
