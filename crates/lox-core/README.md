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

- Optional `configuration-xml`: `configuration::mqtt::list_topics(&[u8])`
  returns owned `MqttTopic` records with typed `MqttDirection`. The actual
  `lox config mqtt topics` producer and cached CLI editor share this one walker.
  Exact `C`/`GenTSensor` subscriptions and `C`/`GenTActor` publishes are kept in
  depth-first source order, including duplicates, empty strings and raw QoS.
  Later direct `SET` properties win, including a missing `v` resetting to empty.
  This reuses the editor loader and typed `EditError` with its native cause;
  configuration XML now enables the existing editor dependencies. The known
  lox-n37w quoted-text/digit-name behavior is unchanged. File I/O, output DTOs,
  flag policy and terminal rendering remain in the CLI. This is not a broker,
  topic/QoS validator, device/reference-alias discovery or foreign binding.

- Optional `configuration-xml`: `configuration::wiring::connector_wiring(bytes,
  selector)` returns owned `ConnectorWiring` records with `ConnectorDirection`.
  The existing `config control wires` command and cached legacy editor share
  this producer. It observes direct `Co` children in source order, retaining
  duplicate/missing names and raw `U` values. Direction prefixes are case-sensitive;
  connected means a direct `In` child or nonempty `U` other than exactly
  `00000000-0000-0000-0000000000000000` (35 characters). The conventional
  36-character all-zero UUID remains connected under this legacy policy. It does not validate
  UUIDs, endpoints or physical state. The shared legacy selector retains room,
  type, gid, fuzzy title and UUID-mirror policy; typed `EditError` preserves load
  causes and selection failures. The shared loader's quoted-text/digit-name
  limitation (lox-n37w) is unchanged. This feature now includes the existing
  `configuration-editor` dependencies; default Core still has three direct
  dependencies. No mutable DOM is exposed by this API, and it adds no Python/JVM
  binding or native Config qualification.

- Optional `legacy-token-reply`: `legacy_token_reply::{parse, Reply, ReplyError,
  ReplyErrorKind}` is the exact existing token WebSocket LL projection, now used
  by the CLI's plaintext, key-exchange and encrypted receive helpers through
  their shared adapter. Only lowercase `code` is read; strings use Rust `u32`
  parsing, JSON unsigned integers narrow modulo 2^32, and other forms default
  to zero. A missing `LL` fails; a present non-object `LL` projects zero/empty.
  Missing values are empty, string values decode unchanged, and other present
  values use compact serde_json with its enabled object ordering (including
  `preserve_order`). No success/authentication policy is imposed. Reply Debug
  redacts the value; errors expose only kind/location by default. Explicit
  `into_json_error` opts into the original potentially payload-bearing serde
  diagnostic for legacy CLI error-chain compatibility. This is not the runtime
  clock/control-read envelope policy, a new secret owner/zeroization guarantee,
  a whole-command migration or a Python/JVM capability. The feature adds only
  optional serde_json; default Core dependencies remain unchanged.

- `backup_catalog::{parse_backup_name, parse_backup_listing, BackupInfo}`:
  pure owned observations of the existing case-sensitive backup filename and
  permissive FTP LIST profile. Invalid rows are omitted, duplicate rows retained,
  and raw timestamps sorted lexically descending with stable ties. Timestamps
  have at least fourteen ASCII digits, not calendar validation or an exact length.
  The existing CLI `list_backups` producer delegates after unchanged acquisition;
  its DTO, date formatting, output and transport errors remain head-owned.
  This adds no dependency, whole-command qualification or foreign binding:
  Python and JVM are unsupported. It is not FTP transport, path/file-type
  validation, custom-overlay listing or archive decoding.
- Default `configuration::custom_listing::{parse_custom_listing, ListingError}`
  strictly interprets an already successful `/sys/custom` FTP listing into
  sorted owned raw names, retaining duplicates. Every row must have a last
  whitespace token; ASCII-case-insensitive `.loxcc` selects names without UUID
  validation. Only selected raw lowercase `d`/`l` rows or slash-containing names
  fail. Leading whitespace, uppercase type letters and raw spelling retain the
  existing quirks. Errors are atomic, typed, payload-free and carry a zero-based
  row index. The actual CLI `ftp::list_custom_changes` acquisition producer uses
  this operation inside its existing retry closure, preserving static-str versus
  String error payloads and quit timing. FTP failures never become empty lists.
  This is distinct from permissive backup parsing and strict fragment filenames;
  transport, acquisition/deployment policy and Python/JVM support are not added.
- Optional `legacy-token-aes`: `legacy_token_aes::{encrypt, decrypt,
  encrypt_command, DecryptError}` extracts the existing CLI token AES-256-CBC/
  PKCS7 byte codec and deterministic `jdev/sys/enc/` envelope. Keys and IVs are
  explicit borrowed `&[u8; 32]` / `&[u8; 16]`; plaintext/ciphertext are borrowed
  bytes and outputs are owned bytes. The command helper encrypts UTF-8 bytes and
  uses STANDARD padded Base64 without percent encoding or salt. Invalid
  ciphertext length/padding returns a data-free typed error and no partial
  plaintext. CBC is unauthenticated: success proves neither authenticity nor
  UTF-8 validity. There is no retained key owner or zeroization guarantee.
  The feature adds only the existing RustCrypto `aes`/`cbc` and `base64`
  dependencies; the feature-off direct dependency graph remains three.
  CLI response Base64/UTF-8 contexts, token flow, RSA, randomness and `ws.rs`
  remain unchanged. This evidenced legacy profile is **not** modern salted
  session/challenge authentication, full token acquisition or a foreign binding.
  No new native/CLIwhole qualification, MSRV or other-platform claim is made.
- `message_header::{decode_message_header, MessageHeader, MessageType}`: typed,
  lossless observation/encoding of exactly one eight-byte application header.
  Raw unknown identifiers/flags/reserved bytes are retained, not certified.
  This Rust-only primitive is not WebSocket framing, reassembly, payload
  validation, a dispatcher, a migrated CLI command or a foreign binding.
- Optional `firmware-metadata`: `firmware_metadata::UpdateMetadata::{parse, assess}`
  provides typed offline release/Remote Connect observations with explicit numeric
  versions and dates. This is a new Rust-only capability, not a migrated CLI
  updater or install-eligibility validator. See the [contract and pinned source
  comparison](../../docs/knowledge/firmware-metadata.md).
- `numeric_state::decode_value_state_payload`: typed, offline decoding of complete
  numeric value-state table payloads. This is a new Rust-only operation, not a
  migrated CLI producer or a WebSocket transport/subscription API.
- `text_state::decode_text_state_payload`: complete UTF-8 text-state payloads,
  retaining state/icon UUIDs and exact text with strict atomic errors.
- `weather_state::decode_weather_state_payload`: complete unframed Weather-State
  payloads as raw typed observations, with atomic count/length errors.
- `color`: offline typed RGB/HSV/temperature encoding, distinct actor composites
  and LightsceneC percent mood values, and explicit existing-profile
  `decode_legacy` / `parse_mood_legacy` observations. The actual CLI color
  encode/decode producers and raw configuration mood parser use this family.
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
- The same runtime features expose `ControlReadClient::read_control` and
  `ControlReadDriver`, used for the actual `lox live get` HTTP read, not only
  response decoding. Typed observations, validation, errors, cancellation and
  bounded close share the clock's private transport/lifetime implementation.
- `LegacyControlCommand::new` owns and validates a resolved command identifier.
  `LegacyControlCommandClient::send_command` and the explicitly owned
  `LegacyControlCommandDriver` perform the actual `live set --write` HTTP
  command through that same transport, LL projection and bounded worker.
  They deliberately reuse `ControlReadOptions`, `ControlObservation` and
  `ControlReadError` as the existing control-HTTP vocabulary, not a second
  protocol/error framework. The named legacy factory always selects the CLI
  host/header/body compatibility policy. Commands are literal URL suffixes:
  no new segment encoding or command-language validation is imposed.
  **Retries include non-idempotent commands**, HTTP 5xx, body/transport failures
  and malformed JSON after HTTP success; HTTP 4xx is not retried. A failed or
  cancelled operation may already have executed. There is no exactly-once or
  rollback guarantee. Only the CLI owns `--write`, context, resolution/cache,
  output and legacy error rendering. No Python/JVM write export, general route,
  session/authentication or subscription capability is added.
- `control`: typed in-memory query preparation, catalog resolution and identifier
  validation. The existing CLI `LoxClient::resolve`/`resolve_with_room` producer
  uses this core, including `lox live get` and write-gated `lox live set`.
- Optional `configuration-xml`: `configuration::rooms::{parse_rooms, RoomInfo,
  RoomInventoryError}` inventories rooms directly from borrowed bytes. The
  existing `lox config rooms` producer uses it through a CLI-owned serialization
  and error adapter.
- The same feature exposes `configuration::runtime_overlay::{merge_overlays,
  three_way, validate_document, OverlayError}`. The actual CLI acquisition,
  baseline preparation and patch merge producers share one private span parser
  and these byte transactions. Empty overlays bypass parsing; otherwise native35
  or RFC36 UUID filenames require the exact `.LoxCC` suffix and strict fragment
  lengths/CRC. Replacements retain Type and untouched bytes; only a new
  AutopilotRule may enter a unique Page with `SpNo="1"`. Aliases, overlaps and
  ambiguous placement are refused. Three-way comparison retains existing
  semantic tokens and raw chosen shell/child spans, not canonicalized output:
  local equal to base chooses live, otherwise live equal to base or identical
  edits choose local; incompatible edits fail atomically with typed causes.
  `validate_document` is the same existing preparation/UUID check, not an XSD,
  sidecar or deployment validator. Acquisition, baseline/context persistence,
  ZIP repacking and permission/deployment guards remain in the CLI. This adds
  no whole-command, Python/JVM or native Config qualification claim.
- Optional `configuration-editor`: `configuration::editor::set_param` performs
  the existing CLI's connector-only byte transaction with explicit save time,
  typed outcomes/errors and the shared legacy loader/selector/serializer.
  See [the editing contract](#optional-legacy-compatible-configuration-editing).
- The same optional feature exposes `configuration::summary::{parse_config_summary,
  ConfigSummary, ControlEntry, ConfigSummaryError}`. The summary producer consumed
  by the existing `lox config diff` command uses it.
- `configuration::diff::diff_configs` compares those typed summaries. The actual
  `lox config diff` comparison uses it; serialization and human output stay in CLI.
- It also exposes `configuration::controls::{parse_controls, ControlInfo,
  ControlInventoryError}` for offline XML control inventory. The existing
  `lox config controls` producer uses it; CLI serialization, files and display
  limits remain in the binary. This is distinct from the online `control` API.
- `configuration::devices::{parse_devices, DeviceInfo, DeviceBus,
  DeviceInventoryError}` shares the existing default `lox config devices` table
  producer. `--ports`, JSON, `--summary` and `--room` still use the separate
  DOM/editor discovery paths, not this legacy inventory.
- `configuration::users::{parse_users, UserInfo, UserInventoryError}` shares the
  existing `lox config users` raw inventory producer only. Its independent DOM
  direct-permission enrichment, fallback, limits and presentation stay in CLI.
- Optional `statistics-xml`: `statistics::{parse_index, StatEntry, parse_series,
  StatSeries, StatisticsError, merge_months, MergeMonthsError}` parses the HTML
  index and monthly XML, and concatenates shared series with typed output-label
  mismatches. The actual `lox stats` index, series and merge producers use these;
  `resolve_identity` selects a raw UUID over borrowed `StatisticIdentity` views;
  `StatisticResolutionError` carries sorted candidate data. HTTP, group/month
  selection, suggestions and formatting stay in the CLI.
  Merge consumes caller order without date/value validation or sorting, retains
  first-part metadata and all within-part duplicate timestamps, and drops only
  exact opaque timestamps present in earlier parts. Values keep their f64 bits.
  Mismatch diagnostics use the immediately previous part's month (even empty),
  with the first series' name/labels; the CLI preserves its single-message chain.
  This adds no Python/JVM/FFI surface or whole-command migration claim.
- Optional `auth-hash`: `authentication` provides the distinct existing APP
  and CONFIG `/wsx` credential/token hash profiles. Both CLI producers use it;
  transport, JSON, endpoint/permission selection and compatibility errors stay
  in the binary.
- Optional `structure-json`: `control_catalog::{ControlCatalog, ControlFilter,
  CatalogDecodeError}` projects and selects top-level LoxApp3 controls using the
  existing `control::Control`. The CLI `LoxClient::list_controls/list_controls_ext`
  producer uses it; acquisition, cache, context and raw JSON stay in the CLI.
- The same optional `structure-json` feature exposes
  `structure_metadata::{StructureMetadata, Category, GlobalState, OperatingMode,
  MetadataDecodeError}`. `StructureMetadata::from_loxapp3_json(&bytes)` returns
  owned typed slices through `categories()`, `global_states()` and
  `operating_modes()`. The actual three `LoxClient` metadata methods use this
  projection while keeping their original tuple results and acquisition path.
  Any valid JSON root is accepted; absent/wrong-shaped maps and non-string
  entries are omitted (categories require a string `name`). Empty strings and
  raw IDs are retained. Category names and global-state names sort case
  sensitively; modes sort by `raw_id.parse::<i32>().unwrap_or(0)`.
  Stable ties retain serde_json's input-map iteration order across the cached
  `Value` byte bridge. Unlike the catalog's lookup-only `named_entries`
  HashMap helper, this projection traverses maps directly so ties cannot be
  reordered. Syntax/encoding errors reuse the payload-free line/column catalog
  error via `MetadataDecodeError`. This is not a full document/UUID/mode
  validator, runtime state subscription, mode editor or new foreign API.
- Optional `legacy-wsx`: `legacy_wsx::{Credentials, Nonce, encode_handshake}`
  encodes the complete existing 64-byte legacy handshake. Both CLI fast-reload
  transport paths use it through their time/randomness adapter.
- The same optional `structure-json` feature exposes
  `autopilot_catalog::{AutopilotCatalog, AutopilotRule, CatalogDecodeError}`.
  `AutopilotCatalog::from_loxapp3_json(bytes)` projects the runtime `autopilot`
  map into owned name/UUID/changed-state/history-state rows. The actual CLI
  `LoxClient::list_autopilot_rules` delegates through its cached JSON snapshot.
  All map keys remain eligible, including non-object placeholder rows; absent
  or non-string names become `"?"`, state IDs independently default to `""`,
  and explicit empty/raw strings survive unchanged. Case-sensitive name sorting
  is stable, retaining serde_json map iteration order for ties (including when
  a consumer enables `preserve_order`). Only JSON syntax/UTF-8 errors fail, with
  the shared payload-free line/column error.
  `autopilot_catalog::{resolve_rule, RuleResolutionError}` selects a borrowed
  row from an existing rule slice. The shared legacy loose UUID detector chooses
  a raw case-sensitive UUID search (first duplicate wins); otherwise Rust
  lowercase substring matching applies to names only, with no exact-name
  preference, aliases, room qualifiers, trimming or Unicode normalization.
  Zero/one/many matching rows mean not found/success/ambiguity, including duplicate
  identities; ambiguous candidates retain caller order without sorting.
  The existing `LoxClient::find_autopilot_rule` delegates through its cached
  catalog and retains ownership, candidate printing and original error
  messages/categories. There is currently no whole-command caller of that
  method. Acquisition/cache/context, UUID diagnostics and commands remain
  CLI-owned. This is not
  XML Config Autopilot parsing, a rule/action/condition engine, CRUD, a new whole
  CLI migration or a foreign-binding capability.

The library does not read files, select a connection context, print or implicitly
start a runtime. The optional editor canonicalizes XML; archive decoding does not.
Arbitrary decoded archive bytes are
preserved, including BOM, mixed line endings, duplicate attributes and non-UTF-8
data. Default features are empty; archive-only consumers do not depend on
reqwest/Tokio/serde_json through the library's normal dependencies.

## Optional legacy-compatible configuration editing

Enable only `configuration-editor` for the supported, representation-independent
Rust byte API:

```rust
# #[cfg(feature = "configuration-editor")]
# fn example() -> Result<(), lox_core::configuration::editor::EditError> {
use chrono::DateTime;
use lox_core::configuration::editor::set_param;

let input = br#"<C Type="And" U="gate" Title="Gate"><Co K="I1" Def="0"/></C>"#;
let now = DateTime::parse_from_rfc3339("2026-10-12T07:55:33+01:00").unwrap();
let edited = set_param(input, "uuid:gate", "I1", "1", &now)?;
assert_eq!(edited.change.old_value.as_deref(), Some("0"));
assert!(edited.content_changed);
# Ok(())
# }
```

The actual `lox config set-param` handler uses this transaction. Files/context,
connector-direction warnings, output/error rendering, unsupported dry-run policy,
save-as and exclusive atomic staging remain CLI-owned. It still updates the first
matching `Co` only, accepts non-Parameter connectors with the same warning, and
refuses missing connectors. In particular it **does not** acquire the internal
editor helper's five-name attribute whitelist.

`EditError` distinguishes load, serialization, selection and parameter refusal;
underlying load/emitter causes are retained without exposing their types as the
normal public contract. Selection retains title/type/UUID/gid/room behavior,
including de-duplication of UUID mirrors. `ParameterChange` distinguishes an
absent old `Def` from an empty one. The caller supplies `DateTime<FixedOffset>`;
no clock is read. A changed canonical program stamps a root/immediate Document,
using local `Date`, UTC Loxone-epoch `DateS` and previous+1 monotonic advancement.
Canonical no-ops do not stamp, but may still normalize the raw input.

BOM, detected document EOL, literal attribute LF, digit-name restoration,
value-identical duplicate collapse, quote/empty-tag normalization, ordered
children and unrelated subtree behavior are the **existing** loader/serializer's
contract, not a generic byte-identical XML guarantee. Conflicting duplicates
still fail. The known digit-name/quoted-text rewrite limitation `lox-n37w`
remains unchanged, including collateral it can cause; this API neither repairs
nor rejects previously accepted input to hide it. No native Config validation,
permission/inheritance/version semantics, Python/JVM editor API or ABI promise
is added.

The binary's other editor methods temporarily use
`unstable-legacy-cli-editor`, an explicitly **UNSTABLE**, doc-hidden, opt-in
legacy state adapter. Core owns the DOM and preservation metadata once; the
head-owned wrapper delegates load/select/serialize and retains the other legacy
impl modules. The normal editor API has no public mutable DOM. Do not enable
the adapter in consumers or bindings.

Focused proofs: `cargo test -p lox-core --no-default-features --features
configuration-editor --test configuration_editor`, and
`cargo test --test config_integration editor_kernel_set_param_legacy_boundaries`.
The original serializer/stamp tests now live next to their shared Core code.

## Complete numeric value-state payloads

`numeric_state::{decode_value_state_payload, NumericState, StateUuid,
NumericStatePayloadError}` is available with default features and adds no
dependencies. The caller supplies `&[u8]` containing an already-identified,
complete numeric value-state table **payload**, with protocol/transport headers
removed. Each 24-byte record is UUID16 followed by IEEE 754 f64 little-endian8.
Empty input produces an empty vector; any incomplete tail returns typed
`InvalidLength { actual, trailing }` before decoding, with no partial result.

Owned records preserve event order and duplicate IDs. `uuid().wire_bytes()`
retains the original UUID16; `uuid().to_native35()` renders lowercase
`8-4-4-16` using unsigned u32 LE/u16 LE/u16 LE and eight tail bytes in wire order.
It does not change `control::is_uuid`, `validate_uuid`, configuration RFC36
identifiers or opaque runtime IDs. `value()` returns f64 without arithmetic;
`value_bits()` preserves every bit, including negative zero, infinities,
subnormals and signaling/quiet NaN payloads. Record equality is bitwise, not
floating-point numerical equality. There is no finite-only validation.

Length is not a type discriminator: unrelated bytes of a matching length can
decode. This API does not authenticate, identify or assemble modern WebSocket
frames, process the legacy 64-byte WSX handshake, or decode binary statistics
(V1/V2 epochs), text, daytimer or weather. No cache, subscription, callbacks,
backpressure, CLI command or Python/JVM/FFI head is added.

The public consumer proof is
`cargo test -p lox-core --test numeric_state_payload`; it uses original synthetic
bytes, independently expected identifiers/values, ordered duplicate IDs,
IEEE edge patterns and every incomplete tail around multiple record boundaries.
No existing private decoder/caller was found at base `73621a8a242c8130c569795b68b2d2097737b603`;
authentication text receivers and raw/legacy WSX transport are unchanged.
The [source mapping and policy comparison](../../docs/knowledge/upstream-crosswalk.md#complete-numeric-value-state-payloads)
cite pinned implementation/tests and the existing protocol reference. In
particular, strict rejection is an original API choice, not an upstream bugfix
or firmware/native qualification.

## Complete text-state payloads

Default-feature `text_state::{decode_text_state_payload, TextState,
TextStatePayloadError}` is pure std and adds no dependencies. Supply an
already-classified, fully assembled payload with all framing/headers removed.
Each record is state UUID16, icon UUID16, unsigned little-endian u32 text-byte
length, exactly that many UTF-8 bytes, then `(4 - length % 4) % 4` ignored
padding bytes. Padding must exist but need not be zero. Empty input/text is valid.

Owned records preserve order, duplicate identifiers, embedded **and trailing**
in-length NULs, and Unicode bytes without normalization. `uuid()` and
`icon_uuid()` reuse `numeric_state::StateUuid` (raw wire bytes and mixed-endian
native35); neither identifier is restricted to RFC UUID variants.
`text_byte_len()` retains the declared u32 byte count, not a character count;
`text()` borrows the exact valid UTF-8 string.

Any truncated header, text, padding, unrepresentable length or invalid UTF-8
returns a payload-free structured error, never a partial vector. Errors carry
the record's byte offset and counts; UTF-8 `valid_up_to` is relative to that
record's text, with `error_len: None` for an incomplete final sequence.
Checked slice consumption avoids offset/length addition overflow, and text
allocation follows bounds/UTF-8 checks. There are no arbitrary size caps.

Strict whole-input rejection, invalid-UTF-8 rejection and preserving trailing
NULs are **original API choices**: the pinned upstream reader returns complete
prefixes, replaces malformed UTF-8 and trims trailing NULs. They are not credited
as upstream defect fixes. See the [original source/provenance comparison](../../docs/knowledge/upstream-crosswalk.md#complete-text-state-payloads).
`cargo test -p lox-core --test text_state_payload` exercises this public Rust API
with original synthetic records, every padding extent, nonzero padding,
multibyte text, both mixed-endian identifiers, duplicates/NULs and atomic errors.
Existing numeric contracts/tests are unchanged.

This adds no CLI command, Python/JVM operation, frame classifier/reassembler,
network runtime, subscription, backpressure, timestamp interpretation or
firmware/native Config qualification.

## Optional read-only XML user inventory

Enable `configuration-xml` and call
`configuration::users::parse_users(&[u8]) -> Result<Vec<UserInfo>, UserInventoryError>`.
This extracts existing repository semantics, not new native or authentication
facts. Exact `C Type="User"` start/empty elements at any depth are inventoried
in encounter order, with duplicate identities retained and no filtering.

`name`/`uuid` are raw `Title`/`U`; `description` is raw `Desc` with every literal
underscore replaced by a space. Missing/unreadable fields become empty. `nfc`
only means raw `NFCArr` is nonempty (even whitespace or `0`), not a validated
credential. First matching attributes win; invalid UTF-8 defaults independently,
attribute iterator errors are ignored, and entities are not decoded. Input
bytes are borrowed and unchanged. Empty/text-only XML and incomplete EOF can
succeed; reader failures return typed `Xml { source }`, never a partial list.

The CLI unwraps that source before its existing error context, preserving the
full rendered error chain. It still independently loads the DOM for direct
permissions, falling back to no permissions if that load fails. Duplicate UUID
map behavior and raw-SAX versus decoded-DOM UUID differences are unchanged.
This is producer reuse, not a whole-command migration, permission evaluation,
XML validation or repair. The `lox-n37w` DOM quote/digit rewrite defect remains
explicit and untouched. No Python/JVM user API is exposed.

Focused proofs: `cargo test -p lox-core --features configuration-xml --test
user_inventory` and `cargo test -p lox-cli --test user_inventory_cli`.

## Complete runtime Daytimer-State payloads

Default-feature `daytimer_state::{decode_daytimer_state_payload, DaytimerState,
DaytimerEntry, DaytimerStatePayloadError}` observes already-identified, assembled
payload bytes. It adds no dependencies. This is **runtime state**, not Config
DayTimer entry editing, schedule validation or extraction of an existing local
decoder/caller.

Each state is UUID16, default IEEE f64 LE8 and signed entry count LE4, followed
by that many 24-byte entries: signed LE32 mode/from/to/activation word and IEEE
f64 LE8. `uuid()` reuses `numeric_state::StateUuid` unchanged. Defaults and entry
values retain authoritative `u64` bits (`default_value_bits()`/`value_bits()`);
f64 accessors only interpret them. Equality is bitwise, preserving signed zero
and NaN payload identity. All IEEE patterns are accepted.

`mode()`, `from_minutes()`, `to_minutes()` and `activation_word()` retain signed
observations without range checks, sorting or normalization. The convenience
`needs_activation()` means only `activation_word() != 0`; it never replaces
the raw word. No mode UUID lookup, digital/analog schedule inference or
activation validator is provided. States/entries retain order and duplicates;
empty input and zero-entry states are valid.

Negative counts, unrepresentable count/byte extents, incomplete entries/headers
and trailing fragments produce payload-free structured errors with record byte
offsets and counts, never partial success. Count conversion/multiplication and
input bounds are checked before entry allocation. There is no arbitrary cap.
Atomic rejection is an **original local policy**: upstream returns its parsed
prefix on negative counts/truncation. It is not firmware-equivalence evidence.

The caller owns message classification and complete-payload assembly/removal of
headers. The separate eight-byte type-4 application header is **not** accepted
as framing by this API; matching payload shape alone cannot identify bytes.
No session, subscription, CLI command or Python/JVM API is added.
`cargo test -p lox-core --test daytimer_state_payload` runs original synthetic
public API cases. See the [pinned caller/model/test mapping](../../docs/knowledge/upstream-crosswalk.md#complete-runtime-daytimer-state-payloads).

## Complete Weather-State payloads

Default-feature `weather_state::{decode_weather_state_payload, WeatherState,
WeatherEntry, WeatherStatePayloadError}` is pure std with no new dependencies.
Supply an already-identified, fully assembled payload without protocol/transport headers.
Each state contains UUID16, unsigned little-endian `last_update: u32`, signed
little-endian entry count, then 68 bytes per entry: five signed LE32 fields
(`timestamp`, `weather_type`, `wind_direction`, `solar_radiation`,
`relative_humidity`), followed by six LE64 IEEE fields (`temperature`,
`perceived_temperature`, `dew_point`, `precipitation`, `wind_speed`,
`barometric_pressure`).

States preserve order, duplicate identifiers, zero-entry states and entry order.
`uuid()` reuses the published `numeric_state::StateUuid` raw/native35 contract.
`last_update()` preserves all unsigned bits rather than Kotlin's signed narrowing;
entry `timestamp` remains signed. The six public `*_bits: u64` fields are
authoritative wire observations, including all NaNs, infinities, signed zeros
and subnormals; the correspondingly named methods interpret those bits as f64
without arithmetic. Equality is bitwise. No signed-field ranges are enforced.

Empty input is valid. A validation pass checks **the entire input** before
allocating any result: short headers, negative counts, unrepresentable checked
count-times-68 lengths and truncated entries return typed payload-free errors.
Errors identify the state header's byte offset and declared/expected/available
counts (entry byte counts exclude the 24-byte header). No partial result is
returned for malformed suffixes and no arbitrary size cap is imposed.
Strict atomic rejection is an **original local policy**, not upstream
compatibility or firmware qualification.

No timestamp conversion, timezone/epoch interpretation, weather-type enum,
units, semantic range, Weather-Abo eligibility or firmware-version policy is
provided. This is **not** the HTTP `weatheru.bin` 108-byte entry layout. It adds
no transport, framing, dispatcher, subscription, CLI or Python/JVM/FFI surface.
The [pinned source/caller/model/test comparison](../../docs/knowledge/upstream-crosswalk.md#complete-weather-state-payloads)
distinguishes upstream descriptions from this raw observation contract.
`cargo test -p lox-core --test weather_state_payload` exercises the public API
with original synthetic inputs, mixed-endian UUIDs, unsigned/signed extremes,
IEEE payload identity, duplicate states, multiple entries, and every truncation
of a two-entry state after a valid prefix.

## Optional read-only XML device inventory

Enable `configuration-xml` and supply borrowed XML bytes:

```rust,ignore
use lox_core::configuration::devices::{DeviceBus, DeviceInventoryError, parse_devices};

fn inventory(xml: &[u8]) -> Result<(), DeviceInventoryError> {
    let devices = parse_devices(xml)?;
    for device in devices {
        if device.bus == DeviceBus::Tree {
            let _serial: Option<String> = device.serial;
        }
    }
    Ok(())
}
```

`DeviceInfo` owns `name`, typed `bus`, optional `serial`, optional `sub_type:
u32`, `type_label`, optional `mac` and optional `address`. Only exact raw `C`
types `TreeDevice`, `LoxAIRDevice` and `NetworkDevice` are inventoried, including
nested devices and devices on pages. Bus is determined **only** by that type;
extensions, ancestors, rooms, child ports and serials do not infer another bus.
Rows retain encounter order and duplicates, with no filters, sort or identity
grouping. Only Tree retains `Serial`; only Network retains `MAC` and `Addr`.
Air ignores `Serial`. Network always labels itself `Network device`, while
Tree/Air use the inherited subtype table, `Unknown (n)` for other numbers and
`Unknown` for absent/unparseable subtypes. `subtype_label(u32)` exposes that
same table, returning an empty string for unknown numbers.

Attribute names/types are case-sensitive. Values remain raw UTF-8, including
entity spellings and whitespace; the first matching attribute wins even if
its UTF-8 is unreadable. Unreadable/absent titles become empty strings, optional
values become `None`, and explicit empty optional strings stay `Some("")`.
Subtype uses untrimmed Rust `u32` parsing (leading `+`/zeros accepted; whitespace,
negative and overflow values absent). Attribute syntax errors are ignored and
iteration can recover later attributes. BOM/CRLF and input bytes are unchanged.
This is not strict XML admission: text-only input and incomplete EOF can succeed.
Reader errors return typed `DeviceInventoryError::Xml { source }` with the
underlying `quick_xml::Error`, not a partial inventory. No new resource/value
limits, XML repair, archive detection or deployability validation are added.

The actual default CLI table path uses this parser through a serialization/error
adapter. CLI still owns file reads, lowercase `.zip` rejection, Tree/Air/Network
display grouping, one shared `--limit` budget, total count and truncation warning.
The nested legacy JSON formatter is unreachable: **JSON, `--summary` or any
`--room` selects `ConfigEditor::config_devices` instead**, with different identity,
entity-decoding, ancestry, filtering and sorting semantics. `--ports` takes
precedence over all these flags and stays `ConfigEditor::list_device_ports`.
Those DOM paths are not migrated. The known shared-loader quote/digit rewrite
defect `lox-n37w` and all existing admission/value limits remain unchanged.
No new CLI operation, Python/JVM device API, binding or native qualification is
claimed. Run `cargo test -p lox-core --features configuration-xml --test
device_inventory` and `cargo test -p lox-cli --test device_inventory_cli`.

## Optional LoxApp3 control catalog

```toml
lox-core = { path = "path/to/lox-cli/crates/lox-core", features = ["structure-json"] }
```

```rust,ignore
use lox_core::control_catalog::{CatalogDecodeError, ControlCatalog, ControlFilter};

fn favorites(bytes: &[u8]) -> Result<Vec<lox_core::control::Control>, CatalogDecodeError> {
    let catalog = ControlCatalog::from_loxapp3_json(bytes)?;
    Ok(catalog.select(&ControlFilter {
        favorites_only: true,
        ..Default::default()
    }))
}
```

This feature adds only optional serde_json, independently of the clock runtime.
Input is borrowed serialized LoxApp3 JSON; the owned snapshot contains only the
existing seven-field `Control` values, exposed through `controls()` and owned
`select()` results. It does not expose a raw document model or duplicate control
resolution. `ControlQuery` can consume its control slice directly.

All valid JSON root shapes remain accepted. Missing/null/non-object `controls`,
`rooms` and `cats` maps are empty. Every top-level control key is eligible,
including empty/opaque keys and scalar/null control values; no UUID normalization
or recursive subcontrol enumeration occurs. Missing/non-string name and type
become `?`. Missing/non-string room/category references look up the **empty ID**,
not necessarily an unknown room/category. Only string `name` values in those maps
participate, including empty names; unknown IDs yield `None`. Flags accept only
JSON booleans, otherwise false. Invalid JSON/UTF-8 returns typed
`CatalogDecodeError::InvalidJson { line, column }` without partial results or
payload text. There is no stricter schema validation or added resource cap.

Type, room and category filters reuse `control::matches_text`: Unicode lowercase
substring matching, not full case folding or whitespace trimming. Missing names
match as empty strings. The favorites flag is conjunctive with those filters.
Results sort **only** by case-sensitive control name; equal-name order is
unspecified. Filtering leaves the snapshot unchanged.

The CLI deliberately serializes its already-acquired cached `serde_json::Value`
for this narrow adapter, then calls the byte decoder and typed selection. This
adds serialization/reparse allocation per producer call rather than changing the
shared `get_structure`/cache/raw-consumer contract or freezing a public raw tree.
HTTP/context/cache behavior and JSON output remain CLI-owned. Exact identifier
and alias resolution still bypass catalog acquisition entirely. The existing
`live get` name-resolution path exercises this producer; this is **not** a new
end-to-end CLI operation or a complete CLI migration.

Run `cargo test -p lox-core --features structure-json --test control_catalog`
for public Rust behavior and `cargo test -p lox-cli --bin lox --test
control_catalog_cli catalog_` for producer characterization plus actual executable
loopback/bypass checks. The separate [native facade](../../docs/toolkit-archive-bindings.md)
also exposes stateless `select_loxapp3_controls` / Java
`selectLoxapp3Controls`: caller-owned bytes and a typed filter return owned
seven-field controls. Each call parses anew; it retains no catalog handle.
Matching Python wheel and Java 22 FFM JAR/DLL (also called by Kotlin) require
the documented Windows AMD64 artifact tests. This adds no acquisition/cache,
control transport, real-device/firmware qualification or new platform support.
XML control inventory, archive, clock, rooms, summaries, statistics and the
unresolved `lox-n37w` shared-loader behavior are unchanged.

## Optional legacy WSX handshake codec

```toml
lox-core = { path = "path/to/lox-cli/crates/lox-core", features = ["legacy-wsx"] }
```

```rust,ignore
use lox_core::legacy_wsx::{Credentials, Nonce, encode_handshake};

// The caller obtains these inputs; the codec has no clock, RNG or I/O.
let random_bytes = [0x42; 16]; // synthetic example, not a random source
let timestamp_ms = 0x34fde6a4;
let nonce = Nonce::from_random_bytes(timestamp_ms, random_bytes);
let packet: [u8; 64] =
    encode_handshake(&Credentials::new("fixture", "password", "ENU"), &nonce);
assert_eq!(nonce.timestamp(), timestamp_ms);
```

This dependency-free feature reuses the repository's existing timestamp-derived
16-round RC6 compatibility implementation, not a generic cipher API. The client
applies the **decryption** direction before transmission; the inverse recovers
the payload. It preserves the exact nonce layout (timestamp bytes 3, 0, 1, 2 at
positions 3, 4, 9, 14), little-endian type 1/length 32/sequence 0/magic FEEDBEEF
header and `username\0password\0locale\0` payload. Concatenation is byte-based:
embedded NULs are retained, the first 32 bytes are used even when that splits
UTF-8 or drops later separators/fields, and short input is zero-padded.

`Nonce::from_bytes([u8; 16])` reuses a captured nonce without modification;
`encode_handshake` derives the transform timestamp from that nonce, avoiding
mismatched independent key/nonce inputs. Short nonce slices are not part of this
API. All UTF-8 field strings, including empty and oversized ones, are accepted;
legacy truncation is intentional, so no new runtime input-error policy is added.
Debug output for credentials and nonce omits all fields. Returned packet bytes
contain recoverable credentials: there is no zeroization or global logging
redaction guarantee.

The CLI still owns randomness (one 16-byte fill), timestamp/uptime acquisition,
HTTP upgrade/start framing, TLS records, network options, auth hashes, JWT verbs,
permissions and reload policy. Its existing wall-clock helper and unchecked
head-only nonce accessor are unchanged. This is local legacy compatibility,
not modern `/ws/rfc6455` framing, binary state/event support, firmware
qualification or a security endorsement. Default features and normal direct
dependencies remain unchanged.

Run `cargo test -p lox-core --features legacy-wsx --locked --offline` for the
typed consumer, fixed captured vectors, truncation/recovery and fixed-size API
tests; `cargo test -p lox-cli --bin lox --locked --offline legacy_wsx` exercises
the actual head adapter against packets recorded before extraction.

## Offline color codecs

```rust
use lox_core::color::{encode, EncodeInput, EncodedColor, Rgb, ColorError};

fn dim_amber() -> Result<i64, ColorError> {
    let input = EncodeInput::Rgb(Rgb::parse("255,100,0")?);
    match encode(input, Some(50.0))? {
        EncodedColor::Rgb(rgb) => Ok(rgb.actor_composite().value()),
        EncodedColor::Temperature(_) => unreachable!(),
    }
}
```

This default-feature, pure-std family has no new dependencies. `Rgb` byte
values can be constructed numerically with `Rgb::from_channels(r, g, b)`,
which reuses the exact validation and rounding called by `Rgb::parse`.
The thin [native facade](../../docs/toolkit-archive-bindings.md#offline-color-heads)
maps owned typed inputs/results/errors; matching Python/Java/Kotlin color
artifacts are candidate-only until published.
`Rgb` byte
channels pack as `R + G*1000 + B*1000000`; `MoodValue` instead packs rounded
percent channels plus `0x60000000`. These are distinct result types, both with
`i64` storage. RGB floating inputs round to bytes **before** optional brightness
scaling and its second rounding. `Hsv` uses degrees and percent; `encode`
validates H 0..360, S/V and brightness 0..100. Temperature encoding validates
brightness first and kelvin 1000..15000, and produces only a `temp(...)` string.
The CLI retains its old approximate 2700..6500 error hint.

`decode_legacy` and `parse_mood_legacy` deliberately preserve the old,
asymmetric observations: HSV conversion wraps hue and saturates float-to-byte
casts without encode validation, including NaN/infinities. Temperature/lumitech
decode parses floats without range/finite validation; negative zero is retained.
Function names are ASCII-case-insensitive with outer whitespace trimmed, but
whitespace before `(` is not accepted. `format_number` retains the existing
wire spelling (including its near-integer and negative-zero behavior).

Numeric decode checks the mood prefix first, then the bounded actor range
0..255255255. Raw mood parsing instead checks modulo actor channels without that
global range limit, including some negative integers. Moods validate percent
channels modulo 1000 but preserve the complete original `i64`, including ignored
high digits; neither path narrows to `u32`. These compatibility profiles are
not generalized native/device color validity or Lighting mode eligibility.
The raw `config set-mood-color` and `add-mood` contract stays raw; the separate
mode-aware Lighting algorithms are unchanged.

`ColorError` exposes typed parse notation, channel, value and profile context;
legacy float parser errors have no source, as before. CLI selection/flag errors,
legacy flag wording, human/JSON/hex rendering, filesystem/context and exit policy
stay in the binary. Native color does not qualify native Config/device workflows.
Run `cargo test -p lox-core --test color` and
`cargo test -p lox-cli --bin lox --test color_codec_cli color_`.

## Optional read-only control-state runtime

```toml
lox-core = { path = "path/to/lox-cli/crates/lox-core", features = ["runtime"] }
```

```rust,ignore
use lox_core::runtime::{Cancellation, ControlReadClient, ControlReadOptions};

async fn state(host: &str, user: &str, password: &str, resolved_id: &str)
    -> Result<(), Box<dyn std::error::Error>>
{
    let client = ControlReadClient::new(ControlReadOptions::new(host, user, password))?;
    let result = client.read_control(resolved_id, &Cancellation::new()).await;
    let closed = client.close().await;
    let observation = result?;
    closed?;
    let state = observation.validate()?;
    // The caller owns output; an empty state is legitimate.
    let _ = state.value;
    Ok(())
}
```

This performs **one actual GET** to `/jdev/sps/io/{resolved_target}` using the
shared async reqwest transport, HTTP Basic credentials (including empty ones),
configured user agent/TLS/proxy settings, 10-second per-attempt timeout and
200/400/800ms retry delays. It retries transport/body/JSON/5xx failures, never
4xx. Status is evaluated before reading/decoding a successful body. The inherited
redirect check compares hosts only, not ports/schemes; reqwest's cross-origin
credential stripping remains in force. Hosts are concatenated literally with
`/` and the path, preserving trailing/double slashes and prefixes. No new body
size, JSON-depth, redirect-count or overall operation deadline policy is added.

The client uses a caller-owned Tokio executor; clones share admission accounting
and close state. Sticky `Cancellation`, dropping a read future, or shutdown aborts
in-flight HTTP and retry waits. `request_close` marks closed immediately; `close`
cancels and drains within `shutdown_timeout` (default one second). As with clock,
a caller retaining an unpolled pending future can prevent drain and receives a
typed timeout, not a false resource-release claim. The optional
`runtime-blocking` facade owns one current-thread worker, admits one concurrent
operation (`Busy` otherwise), and joins it on successful close; Drop is bounded
best effort. There is no subscription, callback bridge, or event-stream API.
Diagnostic/body observers run synchronously outside state locks and must not
block or panic; blocking observers can prevent timely cancellation/drain.

`ControlObservation` uses the clock's equivalent **LL wire decoder**, never its
nonempty-value validator. It preserves envelope presence/type, text/f64 values,
and both typed code fields. Default response decoding rejects malformed/non-UTF8
JSON; all valid JSON shapes remain observable without claiming valid state.
`validate()` requires an LL object, successful uppercase code (or lowercase only
if uppercase is missing) and text/finite numeric value, **including empty text**.
`legacy_text()` is explicitly permissive: unsupported/missing values become empty;
code precedence is uppercase string → lowercase string → uppercase i64, not the
strict validator's precedence. Lowercase numeric codes are not projected.
Protocol failure codes do not automatically become HTTP errors.

Default options validate HTTP(S) hosts and the existing command-path identifier
contract. The CLI explicitly selects `ControlReadCompatibility::LegacyCli`,
retaining its deferred host errors and **opaque** alias/catalog targets without
new path validation or URL encoding. This explicit profile also preserves the
old blocking client's **separate** timeout budgets: up to `timeout` waiting for
headers, then a fresh `timeout` for the complete JSON/text body. It does not
install reqwest's async total-timeout body wrapper, preserving the original
backend source chain on truncated transfers. Strict control and Clock retain
their existing total request deadlines.

Resolution, cached/raw structure acquisition,
config/context choice, error envelopes/categories, quiet/JSON/human formatting and
template notes remain in the head. It still constructs the original `LoxClient`
before resolution, retaining its original build/warning/error order and catalog
client. The actual state request runs through Core, then explicitly closes before
rendering. `live set`, other HTTP commands and bindings are not migrated.

New options, observations, results and errors redact payloads/credentials in
Debug; ordinary diagnostics contain only event/status/size. Error Display and
automatic source chains are sanitized. `ControlReadError::kind` is typed;
`into_source()` is an **explicit disclosure opt-in** for the CLI's original
reqwest/serde error-source chain, which may contain URLs/credentials/backend
details. An expired legacy phase budget instead supplies a genuine typed
`ControlReadPhaseTimeout` (with `Headers`/`Body` phase), not a fabricated
`reqwest::Error`: reqwest's constructors for its blocking timeout errors are
private. Existing backend errors retain their reqwest/serde downcasts; phase
budget expiry has this distinct downcast. Its Display/Debug contain no URL;
`request_url()` is a separate explicit disclosure for the head's header-error
context. The CLI's typed adapter preserves the old rendered timeout chain and
category without inspecting or rewriting backend error strings.
The CLI uses it to preserve inherited errors/categories, not to promise
global secret removal. `LegacyCliVerbose` separately enables replacement-decoded
response text and a caller-owned raw-body observer, preserving `-vv` behavior
(including malformed-body logging). Neither opt-in is automatic for library
consumers; data buffers/credentials are not promised zeroized.

`control_read_runtime` tests cover production async/owned clients, loopback auth,
paths, redirects, LL policy, sources, retries, cancellation and actual socket
release. `live_control_read_cli` characterizes actual executable behavior before
and after extraction; existing clock/runtime/catalog regressions remain gates.
Python and Java/Kotlin **control-read transport are unsupported**; no native
artifact, firmware/real-device, platform, registry or Config-parity credit is
earned by this Rust/CLI slice.

## Optional statistics index and monthly decoder

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

`statistics::parse_index(&str) -> Vec<StatEntry>` is the shared producer for the
CLI's `/stats/` index (`ls`, `get` and `download`). Its owned records retain
`file`, lowercase native `8-4-4-16` `uuid`, optional `u32` `group`, hyphenated
`month`, stripped `name` and full `label`. The CLI alone maps these fields into
its existing serde/output shape.

The index parser deliberately recognizes exact lowercase `<li>` and
`<a href="` (after optional whitespace), with double-quoted href first.
Other anchor attributes after href and missing closing `</li>` remain accepted.
Malformed entries are omitted, resynchronizing at the next `<li>`; duplicates
and encounter order survive. Groups are untrimmed ASCII decimal without signs,
including zero and `u32::MAX`. Months are exactly six ASCII digits, **not**
calendar-validated (`202613` becomes `2026-13`). No filename validation API is
exposed.

Labels are XML-entity-decoded, except that **any unknown/invalid entity retains
the entire original raw label**, including otherwise valid entities. This is
intentional compatibility, not an improved general HTML/XML parser. Whitespace
and a matching trailing `YYYYMM` are trimmed; a balanced final parenthesized
room suffix and then a matching `(Statistic Group: N)` suffix are stripped
from the name as before. Index parsing is infallible and returns no error for
omitted entries. The monthly decoder's separate typed errors are unchanged.

`resolve_identity(&[StatisticIdentity], query)` returns a borrowed winning UUID,
not a filtered row set. The caller must map it back to **all original rows**
with that UUID, preserving source order and duplicates, even when other names,
groups or months did not match the query. The CLI keeps its separate serialized
`StatEntry` DTO and borrows only its name/group/UUID fields for this operation.
The query alone is Unicode-lowercased: raw stored UUID equality wins, then
lowercased exact name, then lowercased substring. No trimming, UUID detection,
canonical normalization, aliases or room qualifiers are applied. Empty queries
use the same priority, including empty UUIDs/names before substring matching.
Ambiguity counts distinct raw UUIDs, not rows or groups.
`StatisticResolutionError` exposes a case-sensitive sorted/deduplicated name
inventory on no match, or candidate identities sorted/deduplicated by
`(name, Option<u32> group, uuid)` on ambiguity (`None` before `Some`).
The CLI still owns its original fuzzy suggestions, help, ambiguity text and
anyhow context/source chain; it does not display the Core error generically.
Group ambiguity still fetches newest monthly output labels in the head.
This is a producer migration only: no additional whole-CLI, native or binding
workflow qualification is claimed.

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
Index acquisition/serialization, month range validation, group selection,
JSON duplicate-label rejection, output flags and raw downloads remain CLI-owned.

Run `cargo test -p lox-core --features statistics-xml --test statistics_series`
for typed consumers, and `cargo test -p lox-cli --bin lox --test
statistics_series_cli monthly_` for before/after decoder characterization and
the actual executable over loopback HTTP. `cargo test -p lox-cli --bin lox index_`
exercises the original index regressions and before/after characterization
through the real CLI mapping. This extraction adds no Python,
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

## Optional read-only XML control inventory

Enable `configuration-xml`, as for rooms:

```rust,ignore
use lox_core::configuration::controls::{parse_controls, ControlInventoryError};

fn controls(bytes: &[u8]) -> Result<(), ControlInventoryError> {
    for control in parse_controls(bytes, Some("Switch"), Some("kitchen"))? {
        let _ = (control.control_type, control.title, control.uuid,
                 control.room, control.category);
    }
    Ok(())
}
```

This is the existing offline SAX inventory, not the online catalog's UUID,
alias or query resolution. `ControlInfo` has five string fields, with no serde,
CLI, binding, filesystem, context or runtime dependency. The existing private
raw-attribute helper is shared with rooms and summary; no new parser stack or
dependencies are added. Default features remain empty.

The legacy candidate rule is deliberately **not** Summary's blacklist:
any exact `C` tag with a readable `Type` other than exact `Place`/`Category`
can become pending when its type matches the optional ASCII-case-insensitive
**exact** filter and its `U` is readable. Empty type/ID/title values are accepted.
A missing/unreadable title replaces the pending title with no value, preventing
emission. A missing/unreadable ID, absent type or filtered-out type does not
replace a prior pending candidate. Place/category definitions do not clear it.

The next start/empty `IoData` anywhere consumes a complete pending candidate
once, with raw `Pr`/`Cr` defaulting to empty strings. This is not parent-scoped:
end tags do not clear state, a self-closing candidate can use a later sibling's
`IoData`, and nested candidates overwrite pending ones without restoring the
outer candidate. Filtering during tracking can therefore change which candidate
receives an `IoData`. This intentionally differs from Summary's direct-parent
relationship and rooms' all-`IoData` counts; it is not schema validation.

Room/category names resolve after the entire scan. Later readable definitions
replace earlier ones by raw ID, including empty IDs and names. Missing definitions
leave raw IDs unchanged. Room filtering then uses Unicode lowercase substring
matching on resolved names (or raw fallback IDs); an empty room filter matches all.
Results sort stably by case-sensitive type then title; equal keys retain emission
order, including repeated IDs. There is no deduplication.

Attributes remain raw UTF-8, including whitespace/entity spellings; first matching
attributes win and iterator errors are ignored. BOM/Unicode and incomplete EOF
retain the original reader's permissive behavior. Invalid UTF-8 in requested
attributes acts like a missing value, not document rejection. Input bytes are
never modified, repaired or serialized, and success does not validate all XML.
Reader failures return `ControlInventoryError::Xml` with the original quick-xml
source, without partial results. The CLI unwraps it to the original anyhow chain,
preserving human/JSON errors without adding a wrapper.

The actual command remains lowercase `.zip` rejection → `fs::read` → shared
inventory → CLI model adapter → existing count/limit/human/JSON formatting.
It bypasses both archive-dispatch `load_config_xml` and DOM `ConfigEditor::load`,
so `lox-n37w`'s quoted-text digit-name rewrite remains open and is not adopted.
CSV/no-header/quiet/trace-id/dry-run behavior is unchanged, not normalized.
Other callers (`config autopilot-list` and `config mode-list`) retain the same
adapter; this does not claim their whole-command migration.

Run `cargo test -p lox-core --features configuration-xml --test control_inventory`
for typed consumers and `cargo test -p lox-cli --bin lox --test config_integration
control_inventory` for parser/error-adapter and actual CLI regressions.
The separate [native facade](../../docs/toolkit-archive-bindings.md) exposes the
same Core operation as Python `parse_controls(xml, type_filter, room_filter)` and
the shared Java/Kotlin `LoxArchive.parseControls(byte[], String, String)`.
All five strings are retained in a distinct `XmlControlInfo` snapshot. Filters
are required nullable arguments, not promised Python defaults. This adds only
Windows AMD64 native XML control inventory; no firmware qualification, editing,
deployment validation, runtime catalog API or whole-SDK parity is implied.

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
API, as well as read-only XML control/room inventories and configuration summaries. This
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

XML parsing beyond XML control/room inventory, summary/comparison and monthly
statistics, editing, runtime operations beyond this HTTP clock read,
token acquisition/WebSocket authentication and subscriptions are **not** exposed
through `lox-core`. Existing CLI capabilities outside the archive, clock,
XML control/room inventory, summary/comparison, monthly statistics,
authentication-hash and control-resolution families remain binary-private.
The separate generated UniFFI Python / one Java 22+ FFM facade exposes only the
capabilities in its [per-head matrix](../../docs/toolkit-archive-bindings.md).
Numeric and text-state payloads, comparison, statistics, authentication hashes, editing and runtime catalog
acquisition remain unsupported there. No general cross-head parity, registry
package availability or ABI stability is claimed.
Archive byte guarantees and private deployment/sidecar policies are unchanged;
shared-loader issue `lox-n37w` remains unresolved and is not broadened here.

Run `cargo test -p lox-core` for offline codecs and in-memory resolution, or
`cargo test -p lox-core --all-features` for the clock tests too. This stage is
validated on Windows AMD64; other platforms are not newly qualified by it.

## License and provenance

The archive family extracts this project's own code. The numeric and text-state
payload families are original implementations of documented wire facts, with
original synthetic tests; no upstream code or test vectors were imported.
None imports an upstream client/config crate. Licensing and attribution are unchanged:
see the repository [LICENSE](../../LICENSE), [LICENSE-AGPL](../../LICENSE-AGPL),
[LICENSE-COMMERCIAL](../../LICENSE-COMMERCIAL) and [NOTICE](../../NOTICE).
The package retains `AGPL-3.0-or-later AND LicenseRef-Commercial` metadata.
Format reference: [Inside-The-Loxone-Miniserver](https://github.com/sarnau/Inside-The-Loxone-Miniserver).
CRC32 discovery and plaintext-field notes originate with eisber/lox-cli contributors.
