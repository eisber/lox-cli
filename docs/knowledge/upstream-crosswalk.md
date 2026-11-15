# Initial implementation crosswalk

This comparison is pinned to local commit
[`3894515`](https://github.com/eisber/lox-cli/tree/38945159049f34ade4e43bceceb7761a2bef74d1).
It records implementation evidence, not native or firmware qualification.
The reusable archive-core extraction is a separate, later change; see the
[current Rust core scope](../../crates/lox-core/README.md).
The [upstream catalog](upstreams.md) records licenses, attribution, ancestry
limits and the deliberately read-only refresh policy.

| Topic | Classification | Local implementation and upstream finding |
| --- | --- | --- |
| Cryptographic building blocks | Covered | [Local token code](https://github.com/eisber/lox-cli/blob/38945159049f34ade4e43bceceb7761a2bef74d1/src/token.rs#L52-L151) already implements AES-CBC/PKCS7 and HMAC-SHA256, alongside RSA exchange. Compare the pinned [Kotlin crypto implementation](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/LoxoneCrypto.kt#L16-L25). Coverage of primitives is not proof of complete version-specific authentication. |
| Token command/version strategy | Discrepancy, unresolved | [Local acquisition](https://github.com/eisber/lox-cli/blob/38945159049f34ade4e43bceceb7761a2bef74d1/src/token.rs#L289-L301) always requests `gettoken`; [rs-loxone](https://github.com/JoDehli/rs-loxone/blob/2cdafc4f151f68f8814cd784c3a5b4a83cd31a6a/src/auth/commands.rs#L26-L70) selects `getjwt` for firmware >=10.2. Establish explicit version/vector coverage before changing the legacy path; this is not a claim that it fails on every firmware. |
| Reusable runtime and subscriptions | Missing capability at this baseline | [Private binary modules](https://github.com/eisber/lox-cli/blob/38945159049f34ade4e43bceceb7761a2bef74d1/src/main.rs#L1-L22) and [raw WebSocket helpers](https://github.com/eisber/lox-cli/blob/38945159049f34ade4e43bceceb7761a2bef74d1/src/ws.rs#L73-L128) do not expose a public Rust runtime, subscription or disposal contract. Compare [rs-loxone's supervisor](https://github.com/JoDehli/rs-loxone/blob/2cdafc4f151f68f8814cd784c3a5b4a83cd31a6a/src/supervisor/mod.rs#L59-L110). This does not claim that all internal event decoders are absent. |
| Archive extraction | Covered CLI behavior; validation policy remains distinct | [Local CLI characterization](https://github.com/eisber/lox-cli/blob/38945159049f34ade4e43bceceb7761a2bef74d1/tests/config_integration.rs#L1012-L1210) protects raw bytes and emitted CRC, paths, output and baseline handling. [discostu105/lox](https://github.com/discostu105/lox/blob/7c6c952eab17d23f2906f62d5ccb374749dcee44/crates/loxone-config/src/archive.rs#L41-L52) and [smartmacherei](https://github.com/smartmacherei/Loxone-Integration/blob/85bf863edb8032c5a9e93f22fd2061cc2ca2c866/custom_components/loxone/udp_program.py#L34-L40) illustrate differing reader-validation policies. Do not silently conflate permissive extraction, strict verification and deployment trust. |
| Statistics versions and timestamps | Unresolved | The inspected [local statistics CLI](https://github.com/eisber/lox-cli/blob/38945159049f34ade4e43bceceb7761a2bef74d1/src/commands/stats_cmd.rs#L1-L107) uses monthly XML. [Kotlin](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/app/StatisticExtensions.kt#L20-L92) distinguishes V1 XML/local-2009-epoch and V2 binary/Unix-UTC. Full local binary/version/timezone equivalence is not established by this route. |
| Home Assistant entity/platform registration | Irrelevant to the generic core, with reusable lifetime lessons | [PyLoxone's coordinator](https://github.com/JoDehli/PyLoxone/blob/4a368b2937797308cb247190bced2f5e122e32f1/custom_components/loxone/coordinator.py#L13-L24) is application integration, not the standalone API library. Its [unload tests](https://github.com/JoDehli/PyLoxone/blob/4a368b2937797308cb247190bced2f5e122e32f1/tests/test_unload_cleanup.py#L73-L106) still inform cancellation and cleanup expectations. |

These are six mapped findings, not six resolved discrepancies. No code change
or new native qualification is credited by this table. Revisit only relevant
changed upstream evidence and actual remaining gaps.

## Follow-up: global configuration path identity

A metadata-only refresh found a relevant
[upstream correction](https://github.com/discostu105/lox/commit/f131767cf8a187b84d96a45a9df5c7b03966d428):
the working directory and home directory can name the same directory through
different filesystem paths. Comparing those paths lexically can incorrectly
select the global `.lox/config.yaml` as a project-local flat configuration,
bypassing its multi-context interpretation.

Local discovery excludes the global directory by resolved directory identity
as well as its original lexical path. This comparison does not canonicalize the
returned project path or change nearest-project precedence. When a global
directory cannot be resolved, the existing lexical exclusion still applies.
The directory-exclusion helper accepts explicit paths so temporary-directory
regressions do not change the process home or write real user configuration.

This is a focused comparison with the
[pinned upstream source](https://github.com/discostu105/lox/blob/70f33122df14e46bafbde3e124b53d6b1902386b/src/config.rs#L474-L491),
not wholesale code adoption, acceptance of every change at that head, or native
firmware qualification. The catalog's reviewed repository pin is not advanced
by observing this later commit.

## Complete numeric value-state payloads

**Missing public capability at local base
`73621a8a242c8130c569795b68b2d2097737b603` → supported Rust payload branch.**
The default-feature `lox_core::numeric_state::decode_value_state_payload`
operation now decodes complete numeric value-state tables into owned, typed
records/errors. A search of the actual workspace and the pinned Rust source
found no existing private numeric decoder or caller to extract. `src/ws.rs`
provides raw connection/legacy reload transport; the authentication receivers
in `src/token.rs` handle text and skip binary messages. Neither is migrated.
This is a new Rust-only operation, with a public library consumer test, not
an imaginary CLI command, foreign binding or full event runtime.

Wire evidence at the already-reviewed Smarteon/loxone-client-kotlin commit
[`22fc2d9c1ec58065295fde384fb2cab79fa71d31`](https://github.com/Smarteon/loxone-client-kotlin/commit/22fc2d9c1ec58065295fde384fb2cab79fa71d31):

- [`Codec.kt::readValueEvents/readUuid`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/Codec.kt)
  reads UUID16 followed by little-endian f64 in each 24-byte record. UUID
  rendering uses unsigned u32 LE/u16 LE/u16 LE and eight bytes in wire order,
  yielding lowercase native35 (`8-4-4-16`). The `readUuid` comment's shorter
  twelve-digit tail is inconsistent with its implementation and tests; the
  implementation/tests, not that comment, inform this mapping.
- [`event/ValueEvent.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/event/ValueEvent.kt)
  defines the record shape and 24-byte size.
- [`event/ValueEventTest.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonTest/kotlin/event/ValueEventTest.kt)
  explicitly checks the sixteen-digit tail, empty input, complete events,
  and intentionally lenient incomplete-tail/prefix behavior.
- The pinned [BSD-3-Clause license](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/LICENSE)
  and Smarteon Systems attribution remain in the existing catalog. Only
  factual wire-format knowledge is used here, not copied implementation or
  upstream test vectors.

The existing local protocol reference at the same local base independently
confirms the layout: [Event-Table of Value-States](../loxone-miniserver-protocol.txt#L825-L842)
and [The UUID](../loxone-miniserver-protocol.txt#L950-L975).
The new implementation and synthetic test vectors are original.
**Upstream implementation/reference evidence is not native/firmware qualification.**

The strict policy is a deliberate **difference**, not an upstream defect fix:
empty input is valid, but every non-multiple-of-24 payload returns typed
`NumericStatePayloadError::InvalidLength { actual, trailing }` atomically,
including complete-prefix-plus-tail inputs. No lenient local caller is changed.
Records retain order, duplicate UUIDs, raw UUID bytes and every IEEE f64 bit
pattern (including nonfinite values and signed zero), with explicit raw-bit
access. There is no JSON conversion or finite-value rule. Config RFC36 and
other runtime identifier contracts are unchanged.

The precondition is an already-identified, assembled numeric table payload.
Length validation cannot authenticate or identify payload types. Modern
WebSocket headers/framing/reassembly, legacy WSX packets, binary statistics,
text/daytimer/weather, subscriptions, caches, callbacks and backpressure remain
outside this operation. CLI, Python and Java/Kotlin decoding are unsupported;
see the [per-head matrix](../toolkit-archive-bindings.md).
The seven-repository catalog, reviewed/latest-seen pins and ancestry findings
are unchanged; this focused source comparison does not advance any pin.

## Complete text-state payloads

**Missing public capability at local base
`6461952a3345b31d4d156c7cb5e1430642b11521` → supported Rust payload branch.**
The default-feature `lox_core::text_state::decode_text_state_payload` adds typed
owned records and payload-free errors. A pinned Rust search for
`text_state`/`TextState`/`TextEvent` found only an unrelated configuration type,
not a decoder/caller to migrate. This is new source-recovered wire semantics,
not a feature flag, preview, invented CLI operation or extraction of an existing
local parser.

Focused static evidence at the **unchanged reviewed pin**
Smarteon/loxone-client-kotlin
[`22fc2d9c1ec58065295fde384fb2cab79fa71d31`](https://github.com/Smarteon/loxone-client-kotlin/commit/22fc2d9c1ec58065295fde384fb2cab79fa71d31):

- [`Codec.kt::readTextEvents`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/Codec.kt#L150-L181)
  reads state UUID16, icon UUID16, unsigned LE32 length, that many text **bytes**
  and `(4 - length % 4) % 4` padding bytes. Admission requires the full text
  and padding. Padding contents are skipped, not validated as zero.
  `decodeToString()` interprets UTF-8 with replacement by default
  ([Kotlin standard-library contract, since 1.4](https://kotlinlang.org/api/core/kotlin-stdlib/kotlin.text/decode-to-string.html)),
  then `trimEnd(NUL)` removes trailing in-length NULs.
- [`event/TextEvent.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/event/TextEvent.kt)
  establishes the icon field and 36-byte header; the icon is used by Status.
  [`event/TextEventTest.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonTest/kotlin/event/TextEventTest.kt)
  covers empty input/text, multiple events, padding extents, short header/text
  leniency and trailing-NUL removal. It does not characterize malformed UTF-8,
  nonzero padding or unsigned lengths above signed `Int` range.
- The existing local [text-event reference](../loxone-miniserver-protocol.txt#L844-L869),
  blob `8f84fca03c987c5d4447f1179ee8adcaedac19e0`, independently confirms
  UUID/icon/LE32 fields and ignored padding to four-byte boundaries.
- Both identifiers reuse the established numeric `StateUuid` contract and
  `readUuid` evidence above, including the implementation/comment discrepancy.
  No formatter is duplicated or identifier policy changed.

| Mapping | Result |
| --- | --- |
| Covered wire semantics | Complete records, byte lengths, UTF-8 interpretation, all padding extents/content ignored, empty text, state/icon identifiers, order and duplicates. Original Rust consumer tests cover multibyte bytes, nonzero padding and lossless raw/native35 IDs. |
| Deliberate new policy differences | Atomically reject every incomplete input (even after valid records), reject malformed UTF-8 rather than replacing it, preserve embedded/trailing NULs rather than trimming. No partial-prefix fallback and no upstream bugfix credit. |
| Length handling | Original checked conversion, bounds and slice consumption before text allocation; no signed narrowing, unchecked offset addition or arbitrary cap. Upstream unsigned-to-`Int` behavior at very large lengths is not adopted or certified. |
| Missing capabilities | Frame classification/reassembly (including upstream type 3 header handling), runtime/network acquisition, subscriptions/backpressure, CLI and Python/Java/Kotlin heads. |
| Unknown/unqualified | Actual firmware/native Config behavior and universal device encoding guarantees; strict UTF-8 is this API's explicit policy, not native proof. Other event-table formats and timestamps are outside this slice. |

This is original implementation and original synthetic tests, **not copied code
or upstream vectors**. [BSD-3-Clause / Smarteon Systems](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/LICENSE)
provenance remains in the existing catalog. All seven catalog entries,
reviewed/latest-seen pins and ancestry findings are unchanged. Only the relevant
source/model/tests/reference were inspected; no upstream integration or device
was executed. Source evidence is not firmware/native qualification.

## Complete runtime Daytimer-State payloads

**New Rust-only payload capability at base
`5440927e86ac5356f8af8c31aef466994576aad2`.** No runtime decoder/caller
was found locally to extract. This does not restart completed Config DayTimer
editing. The public operation is
`lox_core::daytimer_state::decode_daytimer_state_payload`, not a command or
WebSocket client.

Focused static evidence reuses the unchanged reviewed
[Smarteon/loxone-client-kotlin pin `22fc2d9c1ec58065295fde384fb2cab79fa71d31`](https://github.com/Smarteon/loxone-client-kotlin/commit/22fc2d9c1ec58065295fde384fb2cab79fa71d31):

- The actual runtime caller,
  [`KtorWebsocketLoxoneClient.kt::processBinaryMessage`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/ktor/KtorWebsocketLoxoneClient.kt#L321-L361),
  dispatches `EVENT_DAYTIMER` to `processAndEmitEvents` with
  `Codec::readDaytimerEvents`. The wrapper parses a list and then emits each
  event; the caller logs `IndexOutOfBoundsException`/`IllegalArgumentException`.
  Event buffering may independently drop updates. None of that session,
  logging, error suppression or delivery policy is imported.
- [`Codec.kt::readDaytimerEvents`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/Codec.kt#L184-L219)
  reads UUID16/default LE f64/signed LE32 count, then 24-byte entries containing
  signed mode/from/to/activation and LE f64. Activation becomes `word != 0`.
  Short headers terminate the loop; negative counts or short entries break it,
  returning previously parsed states, not an atomic error.
- **Both models** are in
  [`event/DaytimerEvent.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/event/DaytimerEvent.kt)
  (blob `4ea82d017fdc12a2d182edda19e628b6d824cfdf`); there is no separate
  `DaytimerEntry.kt` at this pin. The entry model's time-formatting convenience
  and documented range are not adopted as validators. Its tests even format
  1440 as `24:00`; no universal scheduling range is established here.
- [`event/DaytimerEventTest.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonTest/kotlin/event/DaytimerEventTest.kt)
  (blob `db395cea3af59e554c69b352d43323b7c1fef3d2`) exercises the actual codec
  with empty input and zero-entry/single-entry/multiple-entry states and
  lenient truncation/negative counts. `CodecTest.kt` has no Daytimer cases.
  Upstream vectors are not copied.
- The existing [protocol reference](../loxone-miniserver-protocol.txt#L870-L901),
  blob `8f84fca03c987c5d4447f1179ee8adcaedac19e0`, agrees on layout and
  minutes. Existing numeric `StateUuid` supplies wire/native35 identity; no UUID
  parser or formatter is duplicated.

| Mapping | Result |
| --- | --- |
| Covered observations | Signed fields, state/entry order and duplicates, empty/zero-entry states, shared UUID identity and default/entry IEEE bits. Raw activation is retained separately from nonzero convenience interpretation. |
| Original local policy | Strict atomic complete-input errors, checked count/extent/bounds before entry allocation, payload-free positions/counts, all IEEE patterns preserved without reconstructing authoritative bits from host floats. This deliberately differs from upstream prefix success. |
| Out of scope | Separate type-4 eight-byte header, message/session/reassembly/subscription ownership, CLI/native heads, Config editing, mode UUID/range/activation validation, schedule normalization or round-trip encoding. |
| Unknown/unqualified | Actual firmware/native behavior, delivery and schedule semantics; 32-bit overflow branch is checked in source but not qualified by a 64-bit run. Payload shape is not proof of message type. |

This is original implementation, concise factual mapping and original synthetic
public-consumer tests, not imported code/tests. The existing
[BSD-3-Clause license](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/LICENSE)
attributes Copyright (c) 2024, Smarteon Systems. Catalog pins/ancestry and all
other upstream scopes are unchanged. Static upstream evidence is not firmware
or native Config qualification; no integration or device was executed.

## Complete Weather-State payloads

**Missing Core decoder at public local base
`137c24f9848b717c4c3b2cc3de84d77264a1875c` → new Rust-only raw observation API.**
`weather_state::decode_weather_state_payload` accepts a complete, unframed,
already-identified table. The published header classifier's `WeatherStates = 7`
is not a decoder. No existing CLI caller is migrated; the HTTP `weatheru.bin`
108-byte records are a different format and are not reused.

Relevant static review remains at Smarteon/loxone-client-kotlin
[`22fc2d9c1ec58065295fde384fb2cab79fa71d31`](https://github.com/Smarteon/loxone-client-kotlin/commit/22fc2d9c1ec58065295fde384fb2cab79fa71d31),
BSD-3-Clause, Copyright (c) 2024, Smarteon Systems:

- [`ktor/KtorWebsocketLoxoneClient.kt:259–360`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/ktor/KtorWebsocketLoxoneClient.kt#L259-L360):
  `ensureSession` installs `processFrame`; the pending application header selects
  `processBinaryMessage`, whose `EVENT_WEATHER` branch calls
  `processAndEmitEvents` with `Codec::readWeatherEvents`. That helper calls the
  parser and `emitEvent` for each returned event; `tryEmit` may drop events.
  These transport/emission policies are not imported.
- [`Codec.kt:269–320`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/Codec.kt#L269-L320):
  `readWeatherEvents` wraps a little-endian `PlatformBuffer`, uses `readUuid`
  (lines 100–122), reads unsigned last-update then **narrows it to signed Int**,
  reads a signed entry count, and checks count-times-68 using `Long` before
  reading five signed ints and six doubles per entry. A short header, negative
  count or short entry body returns the complete prefix rather than an error.
  `readUuid` uses unsigned LE32/LE16/LE16 and eight ordered tail bytes, matching
  the already-published Rust `StateUuid` native35 contract.
- [`event/WeatherEvent.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/event/WeatherEvent.kt)
  (blob `fb18d85bc2cc140c6c5a87ca9125152b5d976f4f`) defines `WeatherEvent`
  (24-byte header) and `WeatherEntry` (68 bytes).
  [`event/LoxoneEvent.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/event/LoxoneEvent.kt)
  (blob `092dba9e0136ca134d787a523b6ddf9ab5197bc0`) supplies the UUID interface.
  Model comments describe a 2009 UTC epoch, units and Weather-Abo availability;
  those descriptions are **not** range/domain/eligibility validation or a
  firmware guarantee, and are not adopted by this raw API.
- [`event/WeatherEventTest.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonTest/kotlin/event/WeatherEventTest.kt)
  (blob `73756c12daafccfbba8151feb3143b5bacf47bbd`) covers empty input, zero,
  single and multiple entries, multiple states, and lenient short/negative
  cases. It does not characterize unsigned-high update values or arbitrary
  IEEE payload identity. None of its source or vectors is copied.
- The existing [protocol reference](../loxone-miniserver-protocol.txt#L900-L952),
  blob `8f84fca03c987c5d4447f1179ee8adcaedac19e0`, independently specifies the
  unsigned update, signed count and five-signed-int/six-double layout.
  Its `barometicPressure` spelling maps to `barometric_pressure_bits`.

| Mapping | Result |
| --- | --- |
| Covered raw layout | UUID16, unsigned update LE32, nonnegative signed count LE32, 68-byte ordered entries, signed field preservation, zero entries and duplicate IDs. |
| Explicit discrepancy | Preserve update as `u32`, unlike upstream's unsigned-to-signed `Int` narrowing. Entry timestamp stays `i32`; no inferred date conversion. |
| Original local policy | Strict whole-input atomic errors rather than prefix success; checked conversion/multiplication and full-input bounds validation before any result allocation. All six IEEE fields retain authoritative raw `u64` bits, not bits reconstructed from a host float. |
| Missing capabilities | Transport/framing/reassembly/dispatcher/subscriptions, CLI and Python/Java/Kotlin heads. Header classification alone does not establish these. |
| Unknown/unqualified | Epoch conversion, units/domain classification, semantic ranges, subscription eligibility, firmware-version policy, HTTP108 equivalence and native/firmware behavior. |

This is original std-only Rust and synthetic tests, not imported source, test
vectors or a new dependency. No upstream integration, real device or native VM
was run. Existing catalog pins, licenses, ancestry and public operation counts
are unchanged. Source evidence and successful parsing are not native qualification.

## Application binary-message headers

**Missing public capability at local base
`3171a0c076f70bcf5909db8236e83407658752b9` → new Rust-only primitive.**
`message_header::decode_message_header` observes exactly one already-separated
eight-byte application header. `MessageHeader::to_wire_bytes` encodes its raw
fields losslessly. This is not a WebSocket wire-frame parser, a transport
reassembler or an event dispatcher. The committed Core had no equivalent;
`src/ws.rs` contains connection/auth and distinct legacy `/wsx` operations,
and there is no committed `src/stream.rs`. No removed module is restored.

Focused static review uses the **unchanged** Smarteon/loxone-client-kotlin pin
[`22fc2d9c1ec58065295fde384fb2cab79fa71d31`](https://github.com/Smarteon/loxone-client-kotlin/commit/22fc2d9c1ec58065295fde384fb2cab79fa71d31),
BSD-3-Clause, Copyright (c) 2024, Smarteon Systems:

| Reviewed file at that pin | Git blob | Raw SHA-256 |
| --- | --- | --- |
| [`Codec.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/Codec.kt#L54-L87) | `a8c1bc45118d6a9cd441ce252ef01826731893a6` | `cba71b6c8998cc30a441df5a2baee82bbe62317f7a6a8d017c81de571e6cee1d` |
| [`message/MessageHeader.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/message/MessageHeader.kt) | `a6c7e2d33af00a12f0583f25dc32929e21c7426e` | `9d042d0a16a2d52be87209826fae8d6f666d7239c75f7b2a5f73119388f597cc` |
| [`ktor/KtorWebsocketLoxoneClient.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/ktor/KtorWebsocketLoxoneClient.kt#L284-L374) | `7d2e6232b0e4aa887f872dc56e2220a71609eae6` | `033e21ccd51291d8de3c1c3f62ae7be7681ab1af07d740dc626fb4e937778b37` |
| [`CodecTest.kt`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonTest/kotlin/CodecTest.kt#L27-L40) | `0fb02e450904599ffa5b9ff6f22cbe71d995639c` | `2faf25682a9133b79621e73fa671a676774540762f94dd6ab83781b40f0ccdf8` |
| [`LICENSE`](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/LICENSE) | `554c419dbb41f2a8f8714b7d226bdb25044dab8b` | `52d33f64eade844ca653d984ae90dc746582192d0eadf9ae0f60960212d15e71` |

`readHeader` requires length eight and magic `0x03`, uses enum ordinal for
the second byte, tests only bit zero of byte three, ignores byte four, and
reads an unsigned little-endian u32 at offset four into a `Long`.
Its byte-zero read precedes its length test, so empty-input exception behavior
is not adopted. Unknown type bytes are not supported by its enum indexing;
signed byte indexing also precludes interpreting high identifiers as unsigned.
`writeHeader` serializes the known ordinal, boolean estimate and size narrowed
to `UInt`; it does not explicitly write the reserved byte. No allocator
initialization behavior or out-of-range `Long` contract is inferred here.
The model's ordinal order is text, file, value, text-state, daytimer,
out-of-service, keepalive, weather (0–7). Tests cover text read/write and
keepalive read, not estimates, unknowns, reserved fields or malformed headers.

The **actual caller** `processFrame` consumes a pending binary header before
interpreting another binary message as a header. It routes keepalive to a
dedicated channel, text to a text-header channel, and all other known kinds to
the next binary message. `processBinaryMessage` passes value/text/daytimer/weather
data to their respective readers; file handling is unimplemented. It does not
enforce declared length, act on the estimated flag or implement reassembly here.
Those source facts do not justify importing its session assumptions.

The existing local [protocol reference](../loxone-miniserver-protocol.txt#L650-L806)
(blob `8f84fca03c987c5d4447f1179ee8adcaedac19e0`) independently specifies
eight bytes: magic, unsigned identifier, info flags, reserved, unsigned LE32
payload byte count. Bit zero denotes estimated size; other bits are reserved.
It says an estimated header precedes an exact header, and type 5 has **no**
following payload. That differs from the caller's queuing of type 5 and its
lack of estimated-sequence handling. This primitive implements neither policy
as an inferred transport state machine.

**Explicit original observation policy:** reject any non-eight-byte input
(including header plus payload), then reject bad magic, with typed atomic
errors. Observe every unsigned type/flags/reserved/size combination unchanged;
`known_type()` classifies only 0–7. Keeping unknown bytes is a lossless
observation choice, **not forward protocol support** or evidence that peers
accept them. Encoding writes every byte explicitly, including reserved data,
with a `u32` size rather than a narrowing conversion. No size-based allocation,
payload-size equality rule, type-specific zero-size restriction, authentication,
sequence validation or automatic payload decoder selection is added.

Original synthetic default-stack consumer tests cover unsigned extremes,
uninterpreted bytes, estimates, malformed/excess input, known classification and
explicit use with the existing numeric/text payload decoders. No upstream
source or test vectors were copied. CLI/Python/Java/Kotlin remain unsupported
for this new operation; there is no user-level migration or native/firmware
qualification credit. Catalog entries, reviewed/latest-seen pins and ancestry
findings stay unchanged; no upstream code, integration or device was executed.
## Offline firmware metadata follow-up

The bounded [firmware metadata recovery](firmware-metadata.md) follows the actual
`discostu105/lox` update caller at a separately pinned drift commit. It adds a
strict, typed, optional Rust-only observation API, not an updater, a CLI migration
or firmware qualification. The catalog's global reviewed/latest-seen pins and
ancestry statements remain unchanged; the focused source manifest records the
exact additional commit/blobs and deliberate differences.
