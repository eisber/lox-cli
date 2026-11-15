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
- `control`: typed in-memory query preparation, catalog resolution and identifier
  validation. The existing CLI `LoxClient::resolve`/`resolve_with_room` producer
  uses this core, including `lox live get` and write-gated `lox live set`.
- Optional `configuration-xml`: `configuration::rooms::{parse_rooms, RoomInfo,
  RoomInventoryError}` inventories rooms directly from borrowed bytes. The
  existing `lox config rooms` producer uses it through a CLI-owned serialization
  and error adapter.
- The same optional feature exposes `configuration::summary::{parse_config_summary,
  ConfigSummary, ControlEntry, ConfigSummaryError}`. The summary producer consumed
  by the existing `lox config diff` command uses it.
- `configuration::diff::diff_configs` compares those typed summaries. The actual
  `lox config diff` comparison uses it; serialization and human output stay in CLI.
- Optional `statistics-xml`: `statistics::{parse_series, StatSeries,
  StatisticsError}` decodes monthly statistics XML. The existing `lox stats get`
  producer uses it, while HTTP, selection, merging and formatting stay in the CLI.
- Optional `auth-hash`: `authentication` provides the distinct existing APP
  and CONFIG `/wsx` credential/token hash profiles. Both CLI producers use it;
  transport, JSON, endpoint/permission selection and compatibility errors stay
  in the binary.

The library does not read files, select a connection context, print, implicitly
start a runtime or normalize XML. Arbitrary decoded archive bytes are
preserved, including BOM, mixed line endings, duplicate attributes and non-UTF-8
data. Default features are empty; archive-only consumers do not depend on
reqwest/Tokio/serde_json through the library's normal dependencies.

## Optional monthly statistics decoder

```toml
lox-core = { path = "path/to/lox-cli/crates/lox-core", features = ["statistics-xml"] }
```

```rust,ignore
use lox_core::statistics::{parse_series, StatisticsError};

fn monthly(xml: &str) -> Result<(), StatisticsError> {
    let series = parse_series(xml)?;
    for (timestamp, values) in series.rows {
        let _ = (timestamp, values);
    }
    Ok(())
}
```

This is the decoder for the CLI's `/stats/<file>.xml` monthly route, extracted
from this project's existing implementation. It reuses optional quick-xml 0.37,
independently of `configuration-xml`; default features remain empty. There is no
transport, context, filesystem, serde, terminal, clock runtime or binding
dependency in this feature. Input is borrowed `&str` deliberately: the existing
caller uses `LoxClient::get_text` and reqwest's response-text decoding. The core
does not silently replace that policy with a new UTF-8 byte decoder.

`Name`, `StatsGroup`, `Outputs`, `NumOutputs`, `T` and value attributes are
entity-decoded, **not** raw configuration-room attributes. Header matching is
case-sensitive at any nesting depth; a second `Statistics` header is rejected.
Missing name is empty; group is optional, trimmed and parsed as `u32`.
Output labels are split on commas and trimmed. Missing count defaults to the
label count, at least one; explicit count is trimmed and parsed as a
platform-sized `usize`, including zero. Labels are resized to that count
(discarding excess labels); empty labels become `V`, `V2`, etc. No new resource
cap is imposed, so this compatibility parser is not an untrusted-input resource
limit policy.

Rows require a `T` attribute (which may be empty) and one value per output,
read in `V`, `V2`, ... order regardless of attribute order. Values use trimmed
Rust `f64` parsing and must be finite; negative zero is retained. Row order,
duplicate timestamps and duplicate labels are preserved. Timestamps are
entity-decoded strings, otherwise unchanged: no epoch, calendar, timezone or
V1/V2 conversion is inferred. The upstream crosswalk's statistics timestamp
discrepancy remains unresolved, not fixed by this route-specific extraction.

The original reader/depth/attribute validation order is retained: unclosed or
mismatched elements, missing headers, rows before a header, and missing/invalid
row values fail without returning a partial series. Earlier row/header errors
can precede a later EOF/truncation error. This is not a general XML document
validator: BOM/Unicode, unknown elements, text, multiple unrelated root elements,
trailing text, and rows after the header's closing tag retain their existing
acceptance. Attribute lookup stops at the requested key; malformed/duplicate
attributes encountered before that key fail, while unused attributes and
malformed trailing attributes beyond every requested key may remain unread.
Unknown entities in a requested attribute fail; unrequested attributes are not
entity-decoded. This deliberately does not adopt the room reader's permissive
attribute-error handling.

`StatisticsError` carries typed field/timestamp/value context and original
XML/attribute/integer/float error sources. It never prints or logs input. Error
values and series data can contain caller payloads; callers control disclosure.
The CLI maps sources back to its original anyhow chain with no extra wrapper,
retaining `reading <file>` / `parsing <file>` context, error envelope and exit
behavior. All months are decoded and merged before output, so a failed later
month emits no partial table/CSV or success-shaped JSON. HTTP/retry policy,
HTML index, month filenames/ranges, name/group selection, multi-month merging,
JSON duplicate-label rejection, output flags and raw downloads remain CLI-owned.

Run `cargo test -p lox-core --features statistics-xml --test statistics_series`
for typed consumers, and `cargo test -p lox-cli --bin lox --test
statistics_series_cli monthly_` for before/after decoder characterization and
the actual executable over loopback HTTP. This extraction adds no Python,
Java/Kotlin or native statistics API, and qualifies no firmware, real device or
new platform. Archive, clock, control, rooms and the unresolved `lox-n37w`
shared-DOM-loader behavior are unchanged.

## Optional read-only room inventory

```toml
lox-core = { path = "path/to/lox-cli/crates/lox-core", features = ["configuration-xml"] }
```

```rust,ignore
use lox_core::configuration::rooms::{parse_rooms, RoomInventoryError};

fn inventory(bytes: &[u8]) -> Result<(), RoomInventoryError> {
    for room in parse_rooms(bytes)? {
        let _ = (room.uuid, room.name, room.item_count);
    }
    Ok(())
}
```

This feature reuses the CLI's quick-xml 0.37 reader and its original room
algorithm, not a second XML parser or shared-domain validator. It adds only the
optional `quick-xml` normal dependency; default archive/control consumers still
have no XML, HTTP, JSON, terminal or binding dependency. The existing Rust 1.91
manifest minimum is unchanged, not newly certified on that compiler.

`C`/`Type="Place"` matching is exact and case-sensitive, at any nesting depth.
`U` and `Title` must be present and valid UTF-8 but may be empty; values remain
raw, including entity spellings, whitespace and Unicode. First matching
attributes win; attribute iterator errors are ignored as before. Later room
definitions replace earlier titles for the same raw ID. Every start/empty
`IoData` with a readable raw `Pr` contributes once, regardless of its parent
type or position, including devices/sensors and references preceding the room.
Unknown room IDs do not create rooms. These are item counts, **not** strictly
function-block counts. Results sort case-sensitively by name; equal-name order
remains unspecified because the original algorithm uses a hash map.

BOM-prefixed input is accepted by the reader. Bytes are borrowed and never
mutated; no XML is serialized or written. This is not a byte-roundtrip,
entity-decoding, encoding-conversion or document-validity guarantee. For example,
plain text and unclosed elements at EOF can succeed; invalid UTF-8 in a required
attribute can omit that room, while an irrelevant attribute may be ignored.
Reader failures return typed `RoomInventoryError::Xml` with the original
`quick_xml::Error` source. The CLI unwraps that source before its existing error
envelope/category formatting, avoiding an extra error-chain layer.

The actual CLI chain remains lowercase `.zip` rejection → `fs::read` → room
parser → CLI model mapping → existing human/JSON/limit formatting. It does **not**
call `main::load_config_xml` (archive dispatch) or `ConfigEditor::load` (DOM
loading, digit-name rewriting and duplicate handling). In particular,
`lox-n37w`'s quoted-text digit-name rewrite remains unresolved, not adopted or
silently fixed here. The inventory preserves the raw title containing such text.
`--dry-run` remains a read-only dispatch; legacy CSV/no-header/quiet/trace-id
output behavior is unchanged, not normalized to other commands.

Run `cargo test -p lox-core --features configuration-xml --test room_inventory`
for the typed consumer regressions and `cargo test -p lox-cli --bin lox
--test config_integration room_inventory` for the characterized producer.
The separate [native facade](../../docs/toolkit-archive-bindings.md) exposes
the same read-only inventory to Python and the shared Java/Kotlin API.
Editing and deployment validation remain unsupported by these bindings.

## Optional read-only configuration summary

```rust,ignore
use lox_core::configuration::summary::{parse_config_summary, ConfigSummaryError};

fn inspect(bytes: &[u8]) -> Result<(), ConfigSummaryError> {
    let summary = parse_config_summary(bytes)?;
    for (uuid, control) in summary.controls {
        let room_title = summary.rooms.get(&control.room_uuid);
        let _ = (uuid, control.name, control.control_type, room_title);
    }
    Ok(())
}
```

Enable the existing `configuration-xml` feature, as for rooms. This summarizes
version/date, control records keyed by ID with room relationships, room/category
ID-to-title maps, and user titles in document order. It is not a names-only list
or a strict schema model. Control candidates use the original infrastructure
blacklist: unknown types remain included, not certified function blocks.
Controls require readable `U`/`Title` and a nonempty title; empty IDs are accepted.
Rooms/categories require both attributes but accept empty values. Later matching
IDs replace map entries, including resetting a replacement control's room.
Missing/empty control titles do not replace earlier controls. Users require only
a readable title, retaining empty titles and duplicates.

Each start/empty `IoData` assigns its readable raw `Pr` only to a **direct parent**
control candidate. Nested wrappers, excluded structural/device nodes, and orphan
`IoData` do not assign a room. Later readable assignments replace earlier ones;
missing/invalid `Pr` leaves the previous value. This differs deliberately from
room inventory's all-`IoData` item counts. Repeated `ControlList` and
`C Type="Document"` elements supply the last version/date; a missing attribute
resets that field to empty. Hash-map iteration order remains unspecified, not
sorted; no grouping, counts or comparison output is invented by the core.

The original quick-xml reader/configuration and exact raw-attribute helper are
reused. Attribute values are not entity-decoded; first matching readable
attributes, ignored attribute-iterator errors, BOM/Unicode handling and permissive
EOF behavior match the existing producer. Required invalid-UTF-8 values can omit
an entry. Input is borrowed, never rewritten. `ConfigSummaryError::Xml` carries
the original reader error. CLI unwraps it before restoring its existing
`Failed to parse XML` context, preserving the original error chain.

The actual command is `lox config diff FILE1 FILE2`, **not** a new summary command.
Its chain remains `main::load_config_xml` → raw `fs::read` → lowercase `.zip`
archive extraction when applicable → CLI error adapter → core summary →
core summary comparison → unchanged CLI human/JSON/error formatting. Uppercase `.ZIP` continues
to be treated as raw XML. This does not call `ConfigEditor::load`, so
`lox-n37w`'s quoted-text digit-name rewrite is bypassed, neither adopted nor fixed.
Read-only dry-run behavior is unchanged; the command has no summary limit flag.

Run `cargo test -p lox-core --features configuration-xml --test config_summary`
and `cargo test -p lox-cli --bin lox --test config_integration summary_` for typed
and actual-producer regressions. The separate native facade exposes this
read-only summary to Python and the shared Java/Kotlin API. Editing and strict
validation remain unsupported by these bindings.

## Optional structural-summary comparison

```rust,ignore
use lox_core::configuration::{
    diff::{diff_configs, ControlFieldChange},
    summary::{parse_config_summary, ConfigSummaryError},
};

fn compare(old: &[u8], new: &[u8]) -> Result<(), ConfigSummaryError> {
    let diff = diff_configs(&parse_config_summary(old)?, &parse_config_summary(new)?);
    for change in diff.controls_changed {
        if let ControlFieldChange::Room { old, new } = change.field {
            let _ = (change.uuid, change.name, old.uuid, old.name, new.uuid, new.name);
        }
    }
    Ok(())
}
```

`configuration-xml` also enables `configuration::diff::{ConfigDiff, ControlRecord,
ControlChange, ControlFieldChange, RoomReference, NamedEntry, NameChange}`. The
in-memory comparator borrows existing summaries; it performs no file access,
parsing, normalization, printing or serialization. Added/removed control records,
room/category entries and renames retain their raw UUIDs even though the legacy
CLI output omits them. Control changes carry the new control name and typed
old/new name, type or room values. Room references retain both the raw UUID
(including empty strings) and an optional title from the corresponding summary.
No display placeholders or terminal field labels are stored in Core.

This preserves the existing **summary** comparison only: controls, rooms and
categories match by raw, case-sensitive UUID; duplicate titles remain distinct.
Controls compare name, type and room UUID, in that field order per control.
A room title change alone is a room rename, not a control move; different room
UUIDs still count as a move even when their titles match. Users compare by title
set membership, ignoring duplicate counts and ordering. Metadata is retained
but version/date differences alone do not make `has_changes()` true. Map/set
iteration order is unspecified; no new sorting is introduced. XML parameters,
wiring, layout and other fields absent from the summary are not compared.

The CLI wrapper only maps typed results into its existing serialization models
and resolves room display labels (title, raw unknown UUID, or `(none)`). Existing
output fields, totals, errors, loader/archive selection and read-only behavior
are unchanged. Both parsing and the actual comparison of `lox config diff`
now use Core; this is not a new command, full XML semantic diff, editing API,
or Python/Java/Kotlin diff binding. The `lox-n37w` DOM-loader issue remains open
and bypassed by this raw/archive caller.

Run `cargo test -p lox-core --features configuration-xml --test config_diff`
and `cargo test -p lox-cli --bin lox --test config_integration config_diff`
for typed-consumer and actual human/JSON CLI comparison regressions. Existing
`summary_` CLI tests also cover the unchanged raw/archive and error paths.

## Optional authentication hash profiles

```toml
lox-core = { path = "path/to/lox-cli/crates/lox-core", features = ["auth-hash"] }
```

```rust,ignore
use lox_core::authentication::{
    CredentialProfile, HashError, HashKey, WsxPasswordDigest, hash_credentials,
};

fn config_signature(key_hex: &str, user: &str, password: &str, salt: &str)
    -> Result<String, HashError>
{
    let key = HashKey::from_hex(key_hex)?;
    Ok(hash_credentials(
        CredentialProfile::ConfigWsx {
            salt,
            password_digest: WsxPasswordDigest::Sha1,
        },
        user,
        password,
        &key,
    ).to_hex())
}
```

These are **existing local profiles**, not a generic advertised-HMAC scheme or
a newly qualified authentication protocol:

| Operation | Exact existing calculation | Hex casing |
| --- | --- | --- |
| `hash_credentials(AppRawPassword, ...)` | HMAC-SHA256(key, `user:RAWpassword`) | lowercase |
| `hash_credentials(ConfigWsx { .. }, ...)` | HMAC-SHA256(key, `user:UPPER(HASH(password:salt))`) | lowercase |
| `hash_app_token` | HMAC-SHA256(key, token) | lowercase |
| `hash_wsx_autht` | HMAC-SHA256(UTF8-valid decoded JWT key, token) | uppercase |

`WsxPasswordDigest` explicitly selects only the inner SHA1 or SHA256 digest.
The outer HMAC is **always SHA256**, including with SHA1 inside. The CLI keeps
its exact `"SHA256"` comparison and SHA1 fallback for missing, unknown or
differently cased `hashAlg`. Unicode and colons remain verbatim UTF-8 bytes.
The JWT key check accepts valid non-ASCII UTF-8, not just ASCII.

`HashKey::from_hex` strictly rejects malformed hex with a payload-free
`HashError::InvalidHexKey`; `from_bytes` borrows already decoded bytes, including
empty keys. `hash_wsx_autht` rejects invalid UTF-8 with
`HashError::NonUtf8WsxKey`. Errors retain no backend/input sources. Keys, salted
profiles and hash outputs redact Debug; wire output is available only through
explicit `to_hex()`. There is no implicit logging, generic secret framework or
memory-zeroization guarantee; callers must protect input and returned strings.

The CLI deliberately keeps its original encoding adapters: APP credential
hex failures retain their context/source chain, and CONFIG retains hex and
`String::from_utf8` errors before calling the shared hash implementation.
The old APP token helper **alone** still maps invalid hex to the empty key,
silently, for legacy compatibility. Strict core decoding never does so.
These legacy head errors, existing `TokenStore` Debug, request URLs and opt-in
verbose transport logging are not made secret-safe by this extraction. Core
redaction is not a claim that the entire existing authentication stack is safe
to log.

Only optional hmac/sha1/sha2/hex dependencies are enabled by `auth-hash`; default
features remain empty. There is no network, JSON, filesystem, runtime, CLI or
binding dependency in this module. Tests use original synthetic Python stdlib
known answers and mocked CONFIG getkey2/getjwt exchanges, including the
double-encoded challenge, algorithm fallback, permission8 and encoding errors.
No upstream AGPL-only implementation or vector set was imported.

APP gettoken/permission4, CONFIG getjwt/permission8, challenge extraction,
AES/RSA, persistence, refresh and firmware/version/endpoint policy are unchanged.
Token-version parity and Python/JVM authentication hashes remain **unsupported**.
No native-device or cross-firmware authentication qualification is implied.
Run `cargo test -p lox-core --features auth-hash authentication` and
`cargo test -p lox-cli --bin lox auth_hash_` for these regressions.

## In-memory control resolution

```rust
use std::collections::HashMap;
use lox_core::control::{Control, ControlQuery, PreparedControlQuery, ResolutionError};

fn resolve(
    raw: &str,
    room: Option<&str>,
    aliases: &HashMap<String, String>,
    load_catalog: impl FnOnce() -> Vec<Control>,
) -> Result<String, ResolutionError> {
    match ControlQuery::prepare(raw, room, aliases) {
        PreparedControlQuery::Resolved(control) => Ok(control.uuid),
        PreparedControlQuery::Catalog(query) => {
            query.resolve(&load_catalog()).map(|control| control.uuid)
        }
    }
}
```

Preparation performs no I/O: legacy identifier detection wins **before** exact,
case-sensitive raw alias lookup. Both bypass catalog loading; alias values are
returned verbatim, not recursively resolved or validated. Only afterward does a
trailing `]` enable the rightmost `[` qualifier: the name is trimmed, the room is
not, and the inline room overrides the separate room filter, even when empty.
Names and rooms use Unicode `to_lowercase()` substring comparison, not ranking
or Unicode case folding. An exact full name can therefore still be ambiguous.
Empty queries/rooms remain accepted; a missing room is an empty string for matching.

The caller supplies typed `Control` records and their order. The CLI retains JSON
structure mapping, name sorting, cache/context selection and all candidate/error
formatting. Core ambiguity errors carry ordered candidate records, without
printing. Catalog and alias IDs remain opaque; `is_uuid` intentionally retains
the permissive legacy hex/dash detector (not a UUID parser). Separate
`validate_uuid` preserves the command URL-path guard, including its distinct
length/character rules. It is not automatically applied to aliases/catalog IDs.
The CLI still validates `send_cmd`/`get_all` at their original call sites.

This Rust API is available with default features and adds no dependencies.
The separate [generated binding facade](../../docs/toolkit-archive-bindings.md)
exposes typed in-memory control resolution to Python and the shared Java/Kotlin
API, as well as read-only room inventory and configuration summaries. This
does not migrate all CLI commands, `find_control`'s distinct lookup contract,
structure parsing, or another protocol stack, and establishes no stable FFI/ABI.

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

XML parsing beyond room inventory/summary and monthly statistics, editing, runtime operations beyond this HTTP clock read,
token acquisition/WebSocket authentication, subscriptions, and Python or Java/Kotlin native bindings
are **not** exposed through `lox-core`. Existing CLI capabilities outside the
archive, clock, room inventory/summary, monthly statistics, authentication-hash and control-resolution families remain binary-private. No cross-head parity, native
package availability or ABI stability is claimed. The approved generated UniFFI
Python / one Java 22+ FFM API direction is unchanged; this stage adds no bindings.
Archive byte guarantees and private deployment/sidecar policies are unchanged;
shared-loader issue `lox-n37w` remains unresolved and is not broadened here.

Run `cargo test -p lox-core` for offline codecs and in-memory resolution, or
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
