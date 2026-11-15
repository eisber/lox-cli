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
