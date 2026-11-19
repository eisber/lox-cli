# Experimental native archive, configuration inspection, numeric/text/daytimer/weather, clock, control and color bindings

This is a narrow **archive, XML user/device/control/room/summary inspection, monthly statistics XML, LoxApp3 catalog/metadata, numeric/text/daytimer/weather payload decoding, offline color, HTTP clock and pure control selection** binding slice, not a general
Loxone SDK or CLI-parity release. `crates/lox-bindings` is one thin UniFFI
facade over `lox_core::configuration::{archive, controls, devices, rooms, summary, users}`, `control`, `numeric_state`, `text_state`, `daytimer_state`, `weather_state`, `color`, optional `statistics` / `control_catalog` / `structure_metadata` / `autopilot_catalog` and the optional production
`runtime::ClockDriver`, plus default `configuration::custom_listing`; all codec, selection, transport, retry and validation behavior
remains in the Rust core. Python and the Java-first JVM API load the same native library.
Kotlin uses the Java API, not a separately generated Kotlin implementation.
No head invokes the CLI or runs another protocol stack.
The additional offline `color` candidate below remains publication-gated;
all validation, codec math and wire rendering also remain in Core.

## Actually exposed capabilities

| Capability | Rust core | CLI | Generated Python | Generated Java / Kotlin |
|---|---|---|---|---|
| Compress opaque bytes to LoxCC | `compress_loxcc` | Existing archive compression | `compress_loxcc(bytes) -> bytes` | `LoxArchive.compressLoxcc(byte[]) -> byte[]` |
| Decode raw LoxCC | `decompress_loxcc` | Existing archive extraction | `decompress_loxcc(bytes) -> bytes` | `LoxArchive.decompressLoxcc(byte[]) -> byte[]` |
| Decode raw LoxCC or select ZIP member | `extract_and_decompress` | Existing backup extraction | `extract_and_decompress(bytes) -> bytes` | `LoxArchive.extractAndDecompress(byte[]) -> byte[]` |
| Filesystem/config context selection | No implicit operations | Existing CLI behavior | Unsupported; caller owns I/O | Unsupported; caller owns I/O |
| Read-only room inventory from raw XML bytes | Optional `configuration::rooms::parse_rooms` (`configuration-xml`) | Existing shared parser; unchanged | `parse_rooms(bytes) -> list[RoomInfo]` | `LoxArchive.parseRooms(byte[]) -> List<RoomInfo>` |
| Read-only structural summary from raw XML bytes | Optional `configuration::summary::parse_config_summary` (`configuration-xml`) | Existing shared producer for `config diff`; unchanged | `parse_config_summary(bytes) -> ConfigSummary` | `LoxArchive.parseConfigSummary(byte[]) -> ConfigSummary` |
| Read-only control inventory from raw XML bytes | Optional `configuration::controls::parse_controls` (`configuration-xml`); existing filters and pending associations | `config controls` shares inventory; I/O, limits and presentation stay in CLI | `parse_controls(bytes, type_filter, room_filter) -> list[XmlControlInfo]` | `LoxArchive.parseControls(byte[], String, String) -> List<XmlControlInfo>`; nullable filters |
| Read-only selected-element description | Optional `configuration::editor::describe_element` (`configuration-editor`); owned typed properties, connectors and child identities with typed `EditError`; existing loader limit `lox-n37w` retained | Existing `config control describe` and cached editor share one Core walk; raw last-value SET/IoData observations, unordered properties and direct connector/child order retained; DTOs, formatting and file I/O remain head-owned | Unsupported | Unsupported |
| Offline selected SequenceController program text | Published optional `configuration::editor::get_program_text` (`configuration-editor`); direct text in order and typed editor/type failures | Existing `config get-program` and cached editor share the read; file I/O, JSON counts and terminal presentation stay in CLI | Verified at `0adc6a9`: `get_program_text(ProgramTextRequest(bytes=..., selector=...)) -> str`; typed `ProgramTextError` | Verified at `0adc6a9`: `LoxArchive.getProgramText(ProgramTextRequest) -> String`; checked `ProgramTextException`; same Java 22 API in Kotlin |
| Offline selected Lighting mood inventory | Published optional `configuration::editor::list_moods` (`configuration-editor`) at `1746c9b`; stored rows only | Existing `lox config moods` shares the Core traversal; file I/O, DTO and rendering stay in CLI | Verified at `8b36cb2`: `list_moods(MoodInventoryRequest(bytes=..., selector=...)) -> list[MoodInfo]`; typed `MoodInventoryError` | Verified at `8b36cb2`: `LoxArchive.listMoods(MoodInventoryRequest) -> List<MoodInfo>`; checked `MoodInventoryException`; same Java 22 API in Kotlin |
| Offline selected standalone CentralLight stored links | Published optional `configuration::editor::inspect_central_light_links` (`configuration-editor`) at `19d1d2b`; ordered stored link records and typed graph refusals | Existing `lox config central-light inspect` shares the Core matcher; file I/O and presentation stay in CLI | Source-only, unrun `inspect_central_light_links(CentralLightLinksRequest(bytes=..., selector=...)) -> CentralLightLinks`; typed `CentralLightLinksError` | Source-only, unrun `LoxArchive.inspectCentralLightLinks(CentralLightLinksRequest) -> CentralLightLinks`; checked `CentralLightLinksException`; same Java 22 API in Kotlin |
| Offline selected principal direct permissions | Published optional `configuration::editor::principal_permissions` (`configuration-editor`); ordered direct entries and typed failures | Existing `lox config user-perms` uses Core; file I/O, diagnostics and presentation stay in CLI | Verified at `06d8238`: `principal_permissions(PrincipalPermissionsRequest(bytes=..., selector=..., kind=...)) -> PrincipalPermissions`; typed `PrincipalPermissionsError` | Verified at `06d8238`: `LoxArchive.principalPermissions(PrincipalPermissionsRequest) -> PrincipalPermissions`; checked `PrincipalPermissionsException`; same Java 22 API in Kotlin |
| Read-only legacy device inventory from raw XML bytes | Optional typed `configuration::devices::parse_devices` (`configuration-xml`); raw attributes and encounter order | Default `config devices` table producer shares Core; `--ports`, JSON, `--summary` and `--room` remain separate DOM/editor paths | Candidate `parse_devices(bytes) -> list[XmlDeviceInfo]` | Candidate `LoxArchive.parseDevices(byte[]) -> List<XmlDeviceInfo>`; same API in Kotlin |
| Read-only raw user inventory from XML bytes | Optional typed `configuration::users::parse_users` (`configuration-xml`); existing raw fields/defaults and encounter order | `config users` inventory producer only; separate DOM direct-permission enrichment/fallback, limits and formatting stay head-owned; no whole-command or authentication/permission qualification | Candidate `parse_users(bytes) -> list[XmlUserInfo]` | Candidate `LoxArchive.parseUsers(byte[]) -> List<XmlUserInfo>`; SAME Java API in Kotlin |
| Offline MQTT topic catalog | Optional `configuration::mqtt::list_topics` (`configuration-xml`); owned `MqttTopic` / `MqttDirection`, typed `EditError`; known loader limit `lox-n37w` retained | Actual `config mqtt topics` delegates byte input to Core; file I/O, DTO and rendering remain head-owned | Candidate `list_mqtt_topics(bytes) -> list[MqttTopic]` | Candidate `LoxArchive.listMqttTopics(byte[]) -> List<MqttTopic>`; SAME generated Java22 API in Kotlin |
| Legacy-compatible configuration parameter byte transaction | Optional `configuration::editor::set_param` (`configuration-editor`); explicit stamp, typed outcome/errors, opaque normal API | Existing `config set-param` connector-only transaction; file/output/warnings/dry-run refusal/atomic writer remain head-owned; other editor methods use an opt-in unstable state bridge | Unsupported | Unsupported |
| Legacy-compatible raw attribute byte transaction | Optional `configuration::editor::set_attribute` (`configuration-editor`); explicit clock, typed outcome/errors, pre-insertion identity and previous optional value; shared legacy loader/selector/canonical writer | Existing `config xml set-attr` and `config control rename` share one Core mutation; raw names/values retain legacy acceptance without schema validation, reference or permission repair; report-before-save, dry-run refusal and atomic I/O remain head-owned | Unsupported | Unsupported |
| Legacy-compatible connector disconnection byte transaction | Optional `configuration::editor::unwire` (`configuration-editor`); explicit clock, owned typed change/outcome and `UnwireError` wrapping existing editor failures or identifying the missing connector | Existing `config control unwire` and cached editor share one mutation; direct `In`/`Nc` removal and conditional `U` clearing retain legacy second-call behavior; parsing, report-before-save, atomic writer and dry-run policy remain head-owned. `wire` is not migrated | Unsupported | Unsupported |
| Candidate control-to-room byte transaction | Optional `configuration::editor::move_control` (`configuration-editor`); explicit clock, typed outcome/errors and shared canonical loader/selector/stamp/serializer | Existing `config control move` and unchanged rich preview share one Core plan/mutator; type/title precedence, room lookup/error order, exclusions, UUID mirrors and direct `IoData.Pr` only; DTOs/formatting/file I/O/atomic save remain head-owned. Whole-command migration credit remains publication-gated | Unsupported | Unsupported |
| Read-only selected connector wiring catalog | Optional `configuration::wiring::connector_wiring` (`configuration-xml`); owned records and typed direction, shared legacy loader/selector and typed `EditError` | Existing `config control wires` byte producer and cached editor share one loop; raw `U`, direct-child/source-order/duplicate behavior, case-sensitive direction and exact 35-character legacy zero sentinel retained (a conventional 36-character zero UUID is connected); file/DTO/format/error presentation stay head-owned | Unsupported | Unsupported |
| Byte-preserving offline three-way merge | Optional `configuration::runtime_overlay::three_way` (`configuration-xml`) | Existing patch/preparation callers unchanged; no deployment claim | Verified at published main `2603542`: `three_way(base: bytes, local: bytes, live: bytes) -> bytes`; `OverlayError.Refused` | Verified at published main `2603542`: `LoxArchive.threeWay(byte[], byte[], byte[]) -> byte[]`; checked `OverlayException.Refused`; SAME Java22 API in Kotlin |
| Offline application of supplied LoxCC fragments | Optional `configuration::runtime_overlay::merge_overlays` (`configuration-xml`); existing filename-keyed transaction | Existing CLI acquisition, duplicate guard and merge caller unchanged | Source-only, unrun `merge_overlays(base: bytes, fragments: list[LoxccFragment]) -> bytes`; typed `FragmentApplicationError` | Source-only, unrun `LoxArchive.mergeOverlays(byte[], List<LoxccFragment>) -> byte[]`; checked `FragmentApplicationException`; SAME Java22 API in Kotlin |
| Strict successful `/sys/custom` FTP listing interpretation | Default `configuration::custom_listing::parse_custom_listing`; sorted raw owned names with duplicates, atomic typed row-index errors, exact legacy whitespace/suffix/type-letter quirks; not backup or fragment-filename validation | Actual `ftp::list_custom_changes` called by `custom_overlay::acquire` delegates inside the same retry closure; exact static-str/String diagnostics and quit/error timing retained; transport failures are not empty listings; no whole-command count increase | Source-only unvalidated candidate `parse_custom_listing(list[str]) -> list[str]`; typed `CustomListingError` | Source-only unvalidated candidate `LoxArchive.parseCustomListing(List<String>) -> List<String>`; checked `CustomListingException`; SAME generated Java22 API in Kotlin |
| Other XML mutations, semantic validation, deployment, sidecars/trust | No further functional transaction beyond the rows above | Existing CLI behavior unchanged; other legacy editor and trust/deployment behavior remains head-owned | Unsupported | Unsupported |
| Structural configuration summary comparison | Optional typed `configuration::diff` (`configuration-xml`); raw identities and room references retained | `config diff` shares parsing and comparison; human/JSON presentation stays in CLI | Candidate `diff_configs(ConfigSummary, ConfigSummary) -> ConfigDiff` | Candidate `LoxArchive.diffConfigs(ConfigSummary, ConfigSummary) -> ConfigDiff`; same Java API in Kotlin |
| Monthly statistics XML decoding | Optional `statistics::parse_series` (`statistics-xml`); ordered finite rows, typed errors, timestamps remain opaque | `stats get` shares decoding; acquisition, rich diagnostics, group/month selection and formatting stay in CLI | `parse_statistics_series(str) -> StatSeries`; typed `StatRow` and `StatisticsError` | `LoxArchive.parseStatisticsSeries(String) -> StatSeries`; owned `List<StatRow>` / `double[]` and checked `StatisticsException`; Kotlin uses the same Java 22 API |
| Supplied statistics-index identity resolution | Published optional `statistics::resolve_identity` (`statistics-xml`); raw UUID result and typed identity errors | Actual `stats get` already shares Core selection before separate acquisition/group/month/output logic; unchanged | `resolve_statistics_identity(list[StatEntry], str) -> str`; typed `StatisticResolutionError` | `LoxArchive.resolveStatisticsIdentity(List<StatEntry>, String) -> String`; checked `StatisticResolutionException`; Kotlin uses the same Java 22 API |
| Caller-ordered statistics month merge | Published optional `statistics::merge_months` (`statistics-xml`); existing independent Core contract | Existing CLI caller unchanged | Verified at published main `2603542`: `merge_statistics_months(list[StatMonthPart]) -> StatSeries`; typed `StatisticsMergeError.OutputMismatch` | Verified at published main `2603542`: `LoxArchive.mergeStatisticsMonths(List<StatMonthPart>) -> StatSeries`; checked `StatisticsMergeException.OutputMismatch`; Kotlin uses the same Java 22 API |
| Read-only HTML statistics index catalog | Published optional `statistics::parse_index` (`statistics-xml`); infallible existing omissions, raw fields, order and duplicates | Existing index producer unchanged | Candidate `parse_statistics_index(str) -> list[StatEntry]` | Candidate `LoxArchive.parseStatisticsIndex(String) -> List<StatEntry>`; SAME generated Java API in Kotlin |
| Offline firmware metadata and comparison | Published optional `firmware_metadata` (`firmware-metadata`); strict XML, numeric versions, Gregorian dates and explicit-date observations | Existing CLI unchanged; no updater or device contact | Candidate `parse_firmware_version`, `compare_firmware_versions`, `parse_firmware_metadata`, `assess_firmware_metadata`; typed records/enums/errors | Candidate corresponding `LoxArchive` methods; SAME generated Java22 FFM API in Kotlin |
| Offline color encoding/decoding and raw mood codecs | Typed `color` encoders and explicit legacy observers; distinct actor/mood representations | `color encode` / `color decode` and raw mood parser share core; unchanged | Candidate `encode_color`, `decode_color_legacy`, `parse_mood_color_legacy`, `observe_actor_color_legacy`, `color_format_number`; only when published | Candidate same generated Java API, also used by Kotlin; only when published |
| Existing APP / CONFIG WSX authentication hash profiles | Optional `authentication` (`auth-hash`); explicit profiles and redacted new models | Existing token/WSX hash producers share core; transport and version policy stay in CLI | Unsupported | Unsupported |
| Legacy token AES bytes and deterministic command envelope | Optional `legacy_token_aes` (`legacy-token-aes`); explicit borrowed key/IV, AES-256-CBC/PKCS7 and data-free padding errors | Existing token getkey2/gettoken encrypted-command and encrypted-response producers share Core; complete authentication/session policy remains CLI-owned | Unsupported | Unsupported |
| Legacy WSX handshake packet codec | Optional `legacy_wsx` (`legacy-wsx`); typed nonce and complete legacy packet | Existing fast-reload callers share encoding; time, randomness and transport stay in CLI | Unsupported | Unsupported |
| Exact legacy token LL reply interpretation | Optional `legacy_token_reply` (`legacy-token-reply`); typed u32/opaque-string projection and payload-free kind/location errors with explicit original JSON-error handoff | Actual plaintext, key-exchange and encrypted receive producers call Core through their existing shared helper; success/session policy and legacy diagnostic context remain CLI-owned | Unsupported | Unsupported |
| Complete numeric value-state table payload decoding | Default `numeric_state::decode_value_state_payload`; ordered typed records, raw UUID/native35 and exact f64 bits; strict atomic tail rejection | Unsupported; no existing numeric decoder/caller migrated or new command | `decode_value_state_payload(bytes) -> list[NumericState]` | `LoxArchive.decodeValueStatePayload(byte[]) -> List<NumericState>` |
| Complete runtime Daytimer-State payload observations | Default `daytimer_state::decode_daytimer_state_payload`; shared `StateUuid`, signed entry observations, raw activation word and authoritative IEEE default/entry bits; strict atomic errors | Unsupported; no existing decoder/caller migrated or new command | Candidate `decode_daytimer_state_payload(bytes) -> list[DaytimerState]` | Candidate `LoxArchive.decodeDaytimerStatePayload(byte[]) -> List<DaytimerState>`; SAME generated Java22 API in Kotlin |
| Complete Weather-State table payload observations | Default `weather_state::decode_weather_state_payload`; raw unsigned update, signed entry fields, exact IEEE bits, ordered states/entries and atomic count/length errors | Unsupported; no existing decoder/caller migrated or new command; HTTP weather layout is distinct | Candidate `decode_weather_state_payload(bytes) -> list[WeatherState]`; requires matching qualified artifacts | Candidate `LoxArchive.decodeWeatherStatePayload(byte[]) -> List<WeatherState>`; Kotlin uses that SAME Java API |
| Application binary-message header observation/encoding | Default `message_header`; exact eight-byte header, known-type classification and lossless raw fields, unsigned size and estimated bit; no transport, payload validation or dispatch | Unsupported; new Rust-only primitive, no migrated caller | Unsupported | Unsupported |
| Complete text-state table payload decoding | Default `text_state::decode_text_state_payload`; state/icon `StateUuid`, byte count and exact UTF-8 text; strict atomic bounds/UTF-8 errors, ignored four-byte-alignment padding | Unsupported; no existing decoder/caller migrated or new command | Candidate `decode_text_state_payload(bytes) -> list[TextState]`; only with matching published artifacts | Candidate `LoxArchive.decodeTextStatePayload(byte[]) -> List<TextState>`; Kotlin uses the same Java API |
| Ordered HTTP Basic clock observation and validation | Optional `ClockClient` / `ClockDriver`; explicit validation | `live time` uses shared runtime with explicit legacy compatibility | Strict `ClockRuntime` and `AsyncClockClient` | Strict `ClockRuntime` and Java `ClockClient`; same API in Kotlin |
| Current clock cancellation, shutdown and admission | Production sticky cancellation and owned worker close | Per-command owned driver; no new cancellation command | Native handles; bounded asyncio adapter | Native handles; cancellable `CompletableFuture`, explicit `close` |
| Read-only HTTP control-state observation | Optional `ControlReadClient` / `ControlReadDriver`; typed LL results/errors, validation, cancellation and bounded close | Actual `live get` HTTP read uses Core with explicit legacy path/body/error policy; resolution/cache/output remain head-owned | Unsupported | Unsupported |
| In-memory control name/room resolution | Typed `control` query/catalog/errors; default features | Existing resolver uses core; acquisition/formatting stay in CLI | `prepare_control_query`, `resolve_control_query`; typed records/errors | `LoxArchive.prepareControlQuery`, `resolveControlQuery`; same typed Java API in Kotlin |
| LoxApp3 control catalog projection and filtering | Optional `control_catalog` (`structure-json`); typed records/errors | `list_controls` / `list_controls_ext` share projection/filtering; acquisition and formatting stay in CLI | `select_loxapp3_controls(bytes, ControlFilter) -> list[Control]` | `LoxArchive.selectLoxapp3Controls(byte[], ControlFilter) -> List<Control>` |
| Runtime LoxApp3 autopilot rule catalog and selection | Optional `autopilot_catalog` (`structure-json`); unchanged owned four-field catalog plus borrowed `resolve_rule` / typed `RuleResolutionError`, preserving loose raw UUID or name-substring row matching and caller order | Existing cached `list_autopilot_rules` and `find_autopilot_rule` delegate; acquisition, diagnostics and commands remain head-owned. The selector has no current whole-command caller; no whole-command count increase | `list_loxapp3_autopilot_rules(bytes) -> list[AutopilotRule]`; `resolve_autopilot_rule(rules, query) -> AutopilotRule` with typed errors | `LoxArchive.listLoxapp3AutopilotRules(byte[]) -> List<AutopilotRule>`; `resolveAutopilotRule(List<AutopilotRule>, String)` with checked errors; Kotlin uses the same Java 22 API |
| Legacy backup filename / FTP LIST catalog | Default `backup_catalog`; owned raw filename/version/timestamp/size, permissive row omission and stable descending raw timestamp ordering | Existing `list_backups` producer delegates after unchanged FTP acquisition; head DTO/date/output retained; no whole-command count increase | Candidate `parse_backup_name(str, int) -> BackupInfo \| None`, `parse_backup_listing(list[str]) -> list[BackupInfo]` | Candidate `LoxArchive.parseBackupName(String, long) -> BackupInfo` (nullable), `parseBackupListing(List<String>) -> List<BackupInfo>`; SAME generated Java22 API in Kotlin |
| LoxApp3 metadata snapshot | Optional `structure_metadata` (`structure-json`); typed owned projections | Existing three metadata producers share Core; acquisition/presentation unchanged | Candidate `parse_loxapp3_structure_metadata(bytes) -> StructureMetadata` | Candidate `LoxArchive.parseLoxapp3StructureMetadata(byte[]) -> StructureMetadata`; SAME Java API in Kotlin |
| Identifier detection / command-path validation | Distinct `is_uuid` / `validate_uuid` contracts | Existing CLI contracts unchanged | `is_uuid`, `validate_uuid`; typed `IdentifierError` | `LoxArchive.isUuid`, `validateUuid`; checked `IdentifierException` |
| Full token acquisition, control writes and WebSocket events | Not extracted as complete shared operations | Existing binary-private behavior | Unsupported | Unsupported |
| State subscriptions and event-stream backpressure | No shared event stream exists | No new behavior | Unsupported | Unsupported |

Kotlin sees `byte[]` as `ByteArray`. The generated Python surface returns
actual `bytes`, not lists, strings, JSON or a subprocess result. The bridge
uses owned `Vec<u8>` so both heads copy byte buffers. It deliberately does not
export borrowed buffers or zero-copy/lifetime obligations in this slice.

### Statistics identity resolution

`resolve_statistics_identity` reuses the existing owned `StatEntry` record and
delegates directly to the **already shared** Core identity resolver used by
`stats get`. It does not parse an index or series, fetch anything, or introduce a
second selector. It has no dependency on monthly-series decoding. Input `file`,
`label` and `month` are not consulted or validated.

Core lowercases only the query for an initial exact comparison with each raw
stored UUID. If none match, it tries Unicode-lowercased names exactly, then by
substring. It neither trims nor normalizes Unicode, validates UUIDs, interprets
filenames, follows aliases, handles room qualifiers nor falls back to another
catalog. Empty queries follow exactly this same precedence, including empty
stored UUIDs and names. Stored UUIDs are not case-folded.

Success is the owned **raw UUID string**, not an index row, canonical UUID or
group/month choice. As with the CLI, callers then select **all** original rows
with that UUID, retaining encounter order, duplicates and other
names/groups/months. A result does not authorize an arbitrary filename or URL.
Suggestions, CLI diagnostics and group/month selection remain caller-owned.

`StatisticResolutionError.NotFound` owns a case-sensitively sorted, deduplicated
inventory of `names`. `Ambiguous` owns `distinct_uuids` (`u64`, Python integer /
Java `long`) and `candidates: list[StatisticIdentity]`. Candidates contain only
raw `name`, optional `group` (`u32`, Python integer / Java nullable `Integer`
bit pattern) and raw `uuid`; they are sorted/deduplicated by that tuple using
Core's Rust ordering, absent group before present. The ambiguity count is of
distinct **raw UUIDs**, not rows, names, groups or months. No JSON, erased
diagnostic payload or native handle is exposed.

The SDK built from `2ee0bb6` passed the Core/CLI identity regressions and the
Python, Java 22 and Kotlin smokes on Windows AMD64. The HeapWalk check calibrated
actual success, `NotFound` and `Ambiguous` buffers, then verified zero retained
matching buffers after 64 calls per branch per language without requested GC.
See `STATISTICS-IDENTITY-MECHANISM.json` and the matching `ARTIFACTS.json` for
the tested DLL and logs. This does not qualify other platforms, native Config
workflows, cross-version ABI stability or full CLI parity.

### Strict custom-listing native candidate (not yet qualified)

The thin `parse_custom_listing` adapter accepts caller-supplied rows from an
**already successful** `/sys/custom` FTP LIST. Acquisition and transport failures
remain caller-owned: a failed LIST must not be replaced with an empty list.
There is no network, filesystem, CLI subprocess or overlay application in this API.
The existing Core parser and CLI caller are unchanged.

Each row must contain a whitespace-delimited token, even when it would not select
an overlay. Only the last token's ASCII-case-insensitive `.loxcc` suffix selects a
row. Selected rows reject raw lowercase `d`/`l` prefixes and filename `/` or `\`.
Uppercase prefixes and leading whitespace retain Core's legacy behavior.
No UUID, permission, size, Unicode normalization, NUL or additional filename
validation is introduced. This is neither the permissive backup catalog nor the
runtime fragment-filename validator.

Success returns owned raw strings, sorted by Core's Rust string ordering, retaining
case and duplicates; Java's UTF-16 natural ordering is not substituted.
Python receives `list[str]`; Java and Kotlin share `List<String>`. There are no
native handles for callers to close. Both heads retain typed, atomic first-error
variants `Malformed` and `Unsupported`, each with the zero-based `row` (`u64`,
Python integer / Java `long` bit pattern). Their messages and fields contain no
listing contents. The CLI separately retains its existing richer diagnostics.

This source candidate is **unvalidated**: it adds no native qualification,
platform, package, registry, ABI or parity claim. After resource admission, the
existing builder runs the focused Core/adapter tests and installed Python,
compiled Java22 and Kotlin smokes. Those smokes exercise raw names, ordering,
duplicates, legacy quirks, retained owned values/errors, and actual success plus
both error RustBuffers. The test-only HeapWalk extension constructs the exact
12-byte error tag/row prefix on the stack, using the unchanged heap-walk matcher.
Each branch must calibrate live1/free0, then retain zero matching buffers after
64 public calls without requested GC. Only a successful actual run writes
`CUSTOM-LISTING-MECHANISM.json` and includes its logs in `ARTIFACTS.json`.

### Runtime Daytimer-State payload candidate

This thin adapter calls the existing Core decoder and shared `StateUuid`, not
another parser. Callers supply a complete already-identified payload with
framing removed. Empty input and zero-entry states are valid. States and entries
retain encounter order and duplicates. `DaytimerState` owns `uuid_wire_bytes`,
`uuid_native35`, `default_value_bits` and `entries`; `DaytimerEntry` owns signed
`mode`, `from_minutes`, `to_minutes`, `activation_word`, the Core convenience
predicate `needs_activation` (any nonzero word), and `value_bits`.

Both IEEE-u64 fields are authoritative, including NaN payloads, signed zero,
subnormals and infinities; no floating convenience replaces them. Python
integers are unsigned/nonnegative. JVM `long`/Kotlin `Long` carry the identical
64 bits as signed values; use `Long.toUnsignedString` or Kotlin `toULong()` for
unsigned interpretation.

Python `DaytimerStatePayloadError` and Java/Kotlin checked
`DaytimerStatePayloadException` project all four Core variants:
`TruncatedHeader(record_offset, available)`,
`NegativeEntryCount(record_offset, declared)`,
`LengthOverflow(record_offset, declared)` and
`TruncatedEntries(record_offset, declared, available)` (Java camelCase
accessors). Errors reject the complete input, including a malformed tail after
valid states; no partial list escapes. Length overflow is projected/tested in
Rust; a nonnegative i32 count times24 cannot overflow usize on Windows AMD64.
No framing, Config DayTimer XML, HTTP weather, subscriptions, transport, inferred
units, schedule/range validation or CLI migration is included.

`scripts/build-archive-bindings.ps1` includes focused Rust and real installed
Python, compiled Java and Kotlin tests against one matching DLL. The native
tests use the established unchanged HeapWalk primitive: returned success and
typed-error RustBuffers must each calibrate live1/free0, followed by64 generated
calls per branch with zero retained matching buffers and no requested GC.
Supported qualification remains Windows AMD64, CPython3.13 with GIL,
Java22.0.2 and Kotlin2.4.21 using the SAME Java API, UniFFI0.32.1 and third-party
JavaGen0.5.2 plus the existing reviewed correction. Source proof is not shipment;
archive packages, review, composition, gates and publication remain independent.
No ABI stability, registry distribution, other-platform or full-parity claim.

### Offline selected SequenceController program text

The binding delegates to the existing stable `configuration-editor` read. On
supplied configuration bytes, Core runs the shared loader and selector before
requiring exact `SequenceController` block type. Only direct text nodes in the
first direct `Field Name="Configuration"` contribute, in order; nested elements,
CDATA, other spellings and later fields do not. Missing or empty text is a
successful empty string, not an unsupported-block error. The loader's global
digit-name rewrite can affect quoted values and program text (`lox-n37w`);
this is not a byte-for-byte XML read or a fix for that limitation.

Python `ProgramTextError` and Java/Kotlin checked `ProgramTextException` expose
`Load` (original Core parser cause text), `NotFound` (selector and Core's
sorted/deduplicated titles), `Ambiguous` (selector and ordered Core identities)
and `Unsupported` (selected identity). Identities own the title, block type and
UUID, including empty fields; `Other` is a fallback for unexpected editor
errors. No syntax validation, line/sequence counting, mutation, deployment or
file access is added.

The foreign API accepts one `ProgramTextRequest` (XML bytes and selector).
The pinned generators lower both fields as one input buffer with cleanup on
encoding failure. A lone surrogate is an encoding rejection in the host, not a
Core selection error; the harness checks that it leaves no marked XML input
buffer allocated.

At published main `0adc6a9`, all 168 SDK steps passed using fresh installed
artifacts on Windows AMD64 with CPython 3.13 (GIL), Java 22, and Kotlin on
the same Java API. Each language calibrated the actual export's live and
freed RustBuffer for nonempty text and each of the four reachable typed
errors, then retained 64 distinct host results or errors per case without
a matching native buffer or a GC request. Each language also rejected 64
unpaired-surrogate inputs without retaining a marked XML input buffer.
Empty success is a value check only. This does not qualify other platforms
or native Loxone Config workflows.

### Offline selected Lighting mood inventory

The binding projects the published Core reader. Core loads the
supplied XML and resolves the legacy selector before requiring exact
`LightController2`. The result is a list of stored `LightsceneC` records in
depth-first document order, retaining duplicates even under nested containers.
Missing `name` or `uuid` is empty text. `sid` and `cid` are nullable signed
64-bit integers; missing, invalid or overflowing values are absent. `q1` is
the stored DOM string, with missing distinct from empty and no color decoding.
A suitable controller with no records returns an empty list. The legacy loader
and its quoted digit-name rewrite limitation (`lox-n37w`) are unchanged.

The typed failures preserve Core's load cause text, the original selector and
sorted/deduplicated titles for `NotFound`, and ordered title/type/UUID identities
for `Ambiguous`. `NotLightController` retains the selector and exact observed
type (empty if missing); `Other` is a fallback for unexpected editor errors.
This is not raw mood-color observation, full Lighting inspection, membership
or editing, runtime mood activation, deployment, or a native Config workflow.

One `MoodInventoryRequest` places XML bytes before the selector in a single
generated lowering transaction. At published main `8b36cb2`, all 174 SDK steps
passed using fresh installed artifacts on Windows AMD64 with CPython 3.13
(GIL), Java 22, and Kotlin on the same Java API. Each language calibrated the
actual RustBuffer for nonempty results and each of the four reachable typed
errors, then held 64 distinct host values per case without retaining a
matching native buffer or requesting GC. A separate marked XML input verified
that 64 pre-call lone-surrogate rejections per language retained no matching
input buffer. Empty success is a value check only. This does not qualify other
platforms or native Loxone Config workflows.

### Offline selected standalone CentralLight stored links candidate

The source-only binding projects the published Core reader behind `lox config
central-light inspect`. The caller supplies raw configuration XML bytes and a
legacy selector. Core first loads and selects the element, then checks the
standalone owner, document and stored link graph. Each result owns the selected
owner's UUID and title, followed by ordered `linkC` records with a 1-based
position, the reference's original UUID spelling, and the target's stored
title and type (empty if missing). Graph lookup ignores UUID case; `uuid:`
selection does not. No links is a successful empty list. The legacy loader
and its quoted-text/digit-name limitation (`lox-n37w`) are unchanged.

Typed errors preserve the parser cause text, the original selector and
selection diagnostics, and each current Core graph refusal. Invalid `WF`
retains the raw value, the integer parse-cause kind, and its text.
`InconsistentMarker` preserves the optional reported identity. `Other` is
reserved for unexpected editor errors or future Core variants. The read
does not repair `WF`, move or remove links, infer effective links or native
rows, edit XML, perform file I/O, or act on a running system.

One `CentralLightLinksRequest` places bytes before the selector in a single
generated lowering transaction. The new, **unrun** installed-artifact checks
target Windows x64 CPython 3.13 (GIL), Java 22, and Kotlin on the same Java API.
For nonempty success and the nine named payload-bearing error cases, each
language calibrates the actual export's live and freed RustBuffer and holds
64 distinct host objects while checking for no matching native allocation
without requesting GC. Empty success, payload-free refusals and absent
marker identity are value-only checks. A separate marked XML request per
language calibrates the generated input buffer and checks for no retained
matching buffer after 64 strict pre-call lone-surrogate rejections. This
does not observe an allocation during the failing call or transfer earlier
ProgramText evidence. No installed artifact, other platform, native Config
workflow or stable binary ABI is qualified here.

### Offline selected principal direct permissions

The binding projects the same read as `lox config user-perms`.
The caller supplies configuration XML bytes, a selector and a `PrincipalKind`
(`User` or `UserGroup`). Core matches the title exactly, ignoring case, or
compares the value after a lowercase `uuid:` prefix exactly. An omitted title
or UUID is empty text in the selected identity. The shared loader's
quoted-text/digit-name limitation (`lox-n37w`) is unchanged.

Only direct `PRM` children of the first direct `PH` contribute, in order,
including duplicates; `NUM` is ignored. Rows retain nullable `reference`,
`act`, and `tim`, so missing is distinct from empty. Nullable `title` and
`element_type` come from the first `C` with an exact matching `U`. For that
lookup alone, an absent `REF` acts as empty. A selected principal can have
zero rows. Unlike the CLI presentation, the binding does not replace an
absent `REF` with an empty UUID. This is a read of stored entries, not
inherited or effective access, policy interpretation, credential access,
mutation, or file I/O.

`Load` retains the Core editor message and original parser cause text
(`source_message` / `sourceMessage()`). `NotFound` retains the kind, original
selector and sorted/deduplicated titles; `Ambiguous` retains the kind,
original selector and document-ordered UUIDs including duplicates and empty
values. All values are owned; host UTF-8 encoding refusals are not Core
selection errors.

One `PrincipalPermissionsRequest` places bytes before the selector in a
single generated lowering transaction. At published main `06d8238`, all 180
SDK steps passed using fresh installed artifacts on Windows x64 with
CPython 3.13 (GIL), Java 22, and Kotlin on the same Java API. Each language
calibrated the actual RustBuffer for nonempty results and the `Load`,
`NotFound`, and `Ambiguous` errors, then held 64 distinct host values or
errors per case without retaining a matching native buffer or requesting
GC. A separate marked XML request verified that 64 pre-call lone-surrogate
rejections per language retained no matching input buffer. Empty results,
empty diagnostic collections, and absent fields are value checks, not
separate allocation claims. This does not qualify other platforms, native
Config behavior, or a stable binary ABI.

### Offline MQTT topic catalog candidate

`list_mqtt_topics` delegates supplied bytes directly to Core's existing
`configuration::mqtt::list_topics`. It does not read files, select a context,
contact a broker, validate topics/QoS or mutate configuration. Only exact
`C` elements with `GenTSensor` (subscribe) or `GenTActor` (publish) types
contribute records. Depth-first source order and duplicate rows survive.
Each owned record retains `title`, typed `direction`, `topic` and raw string
`qos`; missing values are empty strings, not numeric defaults.

Direct `SET` properties are observed in encounter order: later `mqtt_topic`
and `mqtt_qos` properties overwrite earlier ones, including properties without
`v`. Entities and legacy loader admission remain Core-owned. In particular,
the known quoted digit-name rewrite limitation `lox-n37w` remains visible:
this wrapper is not a loader correction or native Config acceptance claim.

Python returns native records/enums and raises `MqttCatalogError.Load`;
Java/Kotlin return typed records/enums and checked `MqttCatalogException.Load`.
The original Core parser cause text is retained as `source_message` /
`sourceMessage()`. A typed `Other` fallback retains message and optional cause
for future non-load Core errors; no such branch is invented in tests.
The focused ownership tests calibrate actual returned success and load-error
RustBuffers with the existing unchanged HeapWalk primitive, then exercise
64 calls per branch while retaining language-owned results/errors without
requesting garbage collection. Other platforms, registry distribution,
ABI stability and CLI/native Config qualification are not claimed.

### Offline backup catalog candidate

These two thin operations delegate exclusively to published Core. They observe
caller-supplied strings; they never list FTP, touch files or download, restore,
delete or format dates. This is **not** a path/file-type/calendar safety validator.
Names are case-sensitive `sps_<u32>_<timestamp>.zip`; Rust's accepted leading `+`
and zero padding survive. Timestamps retain every ASCII digit (at least fourteen),
without calendar validation or an upper length cap. Records own the exact
filename, numeric version, raw timestamp and byte size.

Listing rows need at least four whitespace-separated fields. The last field is
the name; field index four is the optional unsigned size, defaulting to zero for
absent, invalid or overflowing values. Directory/link/unknown types and duplicates
are not rejected. Sorting is stable descending **raw string** timestamp order,
not numeric/calendar order. No human-size or Qt/date presentation is introduced.

Python integers preserve u32/u64 values. Java `int`/`long` preserve unsigned bits:
use `Integer.toUnsignedLong` / `Long.toUnsignedString`, or Kotlin `toUInt()` /
`toULong()`, including both MAX values. Missing name observations are Python
`None` and Java/Kotlin null; empty listings return empty lists. Neither operation
has a domain error branch. Ownership qualification measures the actual returned
`Some` RustBuffer with the existing unchanged heap probe (live one, exact free
zero, 64 generated calls retained zero without requested GC in all three heads).
The deliberately long valid timestamp exercises Core's uncapped string contract.
Tiny `None` and listing buffers inherit established generated ownership; their
functional roundtrips are tested, not misrepresented as large-buffer calibration.
Other platforms, registry distribution and native Config qualification remain
unsupported/unclaimed.

### Offline firmware metadata candidate

The four thin operations delegate to published `FirmwareVersion::from_str`,
`UpdateMetadata::parse`, numeric ordering and `UpdateMetadata::assess`; there is
no second version, XML, calendar or comparison implementation in the heads.
Owned release metadata retains optional entity-decoded `config_download` (an
opaque Config installer location, **not** a firmware URL). Remote Connect
metadata retains its minimum version and canonical `YYYY-MM-DD` deadline.
`FirmwareVersion` exposes normalized text and all original numeric components:
Python has unsigned integer values; Java `int[]` retains u32 bit patterns,
recovered with `Integer.toUnsignedLong`, or Kotlin `toUInt()`. Optional records,
strings and enums use Python `None` and Java/Kotlin nullable values.

Versions compare lexicographically without zero padding (`1.2 < 1.2.0`).
Assessment requires caller-supplied `as_of`, accepts an optional program version,
and observes only Core's newer/equal/older/unavailable and major/minor predicates.
`warning_due` is below-minimum AND on/before deadline. False after the deadline
does **not** establish Remote Connect usability. No clock, trust/signature check,
download, eligibility grant, update action, runtime or network/device effect
exists in this family.

Typed `FirmwareMetadataError` / checked Java `FirmwareMetadataException` retains
all seven Core variants, original invalid version/date values, Core Display text,
and XML/attribute source text. Invalid dates, duplicate known elements and missing
attributes remain actual refusal paths, not empty success results. Kotlin uses
the same generated Java API, with no second generator.

The existing native build script adds installed-Python and compiled-Java/Kotlin
tests of actual semantics and ownership. Both actual returned success and
InvalidVersion error RustBuffers are calibrated using only the unchanged generic
128-byte-prefix/minimum64000 HeapWalk matcher (live1, explicit-free0), followed by
64 public calls per branch/language requiring retained0 without requested GC.
The error uses a realistically large invalid-version field and its real Core
Display echo, never an allocation proxy or a new matcher. The same final DLL,
wheel and Java API must also pass every previously published native family and
Clock lifecycle/error/close checks. Current published counts remain CLI13/native18;
this is an unreviewed, unpublished prospective nineteenth family, not a count
increase. Other platforms and Rust MSRV1.91 remain unqualified; actual builds use
Rust1.99.1 on WindowsAMD64 with CPython3.13GIL, JDK22.0.2+9 and Kotlin2.4.21.
Native Config qualifications remain two historical, zero new.

### Monthly statistics XML series

`parse_statistics_series` calls the already-shared
`lox_core::statistics::parse_series` once and projects its tuple rows into owned
`StatRow { timestamp, values }` records. `StatSeries` owns `name`, optional `u32`
`group`, ordered `outputs` and ordered `rows`. Python receives strings, optional
integers, lists, and floats; Java receives records, nullable `Integer`,
`List<String>`, `List<StatRow>` and `double[]` (`DoubleArray` in Kotlin). Kotlin calls that SAME Java
API. As with other unsigned fields, Java `Integer` / Kotlin `Int` carry the
group's raw32 bits; use `Integer.toUnsignedLong` / `toUInt()` for interpretation.

The Core parser alone entity-decodes attributes, trims/defaults/resizes labels,
orders `V`, `V2`, ... values, and validates count/group/value text. It retains
opaque timestamps (including whitespace/empty text), duplicate timestamps and
labels, and encounter order. Unknown elements/text are ignored; rows following
a closed header remain accepted. Zero outputs are valid; absent counts default
to at least one. Attribute validation remains lazy, not a new strict XML
schema. `NumOutputs` retains the Core `usize` parsing/platform bound without a
new resource cap. The adapter neither omits malformed rows nor returns a
partial series: any Core error rejects the entire input.

Values are Core's decimal-text-to-`f64` results, not raw binary wire observations.
Focused Source cases characterize negative zero, minimum positive/negative
subnormals, maximum finite binary64 and signed zero after decimal underflow.
Python floats and Java/Kotlin doubles are checked by their raw finite-value
bits. Core rejects NaN, infinities and decimal overflow to infinity; this is
deliberately different from the numeric/daytimer/weather binary payload
families. There is no NaN-payload preservation claim and no invented units,
epoch, timezone, datetime conversion or firmware guarantee.

Python `StatisticsError` / Java checked `StatisticsException` retain all Core
variants: `Xml`, `MalformedAttribute`, `AttributeValue`,
`UnexpectedClosingElement`, `TruncatedXml`, `MultipleStatistics`,
`MissingStatistics`, `RowBeforeStatistics`, `InvalidStatsGroup`,
`InvalidNumOutputs`, `MissingTimestamp`, `MissingValue`, `InvalidValue` and
`NonFiniteValue`. Relevant variants retain `attribute`, `timestamp`, raw
`value`, and original parser/numeric cause text in `source_message` (Java
`sourceMessage()`). These diagnostics can contain input text. The existing CLI
HTTP-text caller and its rich cause/context handling are unchanged. This parser
does not use the configuration DOM loader; known loader issue `lox-n37w`
remains unchanged and is not repaired or bypassed here.

The existing builder additively includes focused Core/binding tests, installed
Python, compiled Java22 and Kotlin SAME-API cases, and the unchanged HeapWalk
primitive. Success and large `InvalidValue` error RustBuffers must each
calibrate live1/free0, then64 generated calls per branch per language must
retain zero matching buffers without requested GC, while earlier host values
and exceptions remain usable. All new logs, the Kotlin argument file and
`STATISTICS-SERIES-MECHANISM.json` are in `ARTIFACTS.json`. Every existing
family and Clock lifecycle remains independent and retained.

The SDK built from `2ee0bb6` passed these proofs on Windows AMD64, and the fresh
Core and bindings archives passed their consumer checks. This function does not
perform identity selection, HTML index parsing, month merge, statistics
acquisition or binary statistics decoding. It adds no CLI command and does not
change the supported platform boundaries.

### Caller-ordered statistics month merge

`merge_statistics_months` consumes supplied `StatMonthPart { month, series }`
records, reusing the owned `StatSeries` and `StatRow` DTOs. It calls
`lox_core::statistics::merge_months` once, without XML parsing, acquisition,
identity/group/month selection, or changes to the existing CLI caller.

Empty input yields an empty series. Core keeps the first part's metadata and
rows and checks each later part's output labels for exact, ordered equality.
Rows are filtered only against timestamps from earlier parts; repeated
timestamps within the same part survive. Caller order, raw month and timestamp
text, row widths and values are not interpreted or normalized. The typed
`OutputMismatch` contains `name`, `previous_month`, `month`, `previous_outputs`
and `outputs`. The previous month is the immediately preceding part, even if
it has no rows; the name and previous output labels come from the first part.

The Rust boundary moves the `f64` values unchanged; unlike XML parsing, this
operation does not reject nonfinite values. Source tests require the generated
Python API and the same Java 22/Kotlin API (owned `double[]` / `DoubleArray`)
to round-trip raw witnesses for signed zero, a subnormal, both infinities and
noncanonical quiet NaNs. Foreign bit preservation was verified on the matching
generated/native artifacts at published main `2603542`.

This slice adds five checks to the SDK builder. It must show live1/free0 on
actual large success and checked-error RustBuffers using the unchanged
128-byte-prefix/minimum-64000 HeapWalk matcher, then zero matching native
buffers with 64 distinct host objects retained per branch per language without
requested GC. The builder includes the qualification logs in `ARTIFACTS.json`
and emits the mechanism receipt with the tested DLL hash only after all three
language tests pass. With the verified three-way and month operations at that
published main, counts are CLI 16, Python 29 and Java/Kotlin 29, with two
historical and zero new native Config qualifications.

### Read-only HTML statistics index candidate

One String-input/list-output export delegates directly to the published infallible
Core parser. Its owned `StatEntry` record retains `file`, `uuid`, optional `group`,
`month`, `name` and `label`. Python represents `group` as `int | None`; generated
Java uses nullable boxed `Integer` with the original unsigned-u32 bit pattern:
`Integer.toUnsignedLong(group)` or Kotlin `group.toUInt()` recovers the numeric
value, including `4294967295`. This is not a signed narrowing contract.

Core alone recognizes case-sensitive `<li>` / `<a href="`, omits malformed rows
and resynchronizes at the next `<li>`. Extra anchor attributes and missing closing
`li` are accepted. Lowercase native35 filenames, optional ASCII decimal-u32 groups
and six ASCII month digits are preserved without calendar validation. Entity
failure preserves the **entire original** raw label, not a partially decoded
prefix; existing month/room/group suffix stripping remains unchanged. Raw Unicode,
combining characters, NUL and line breaks are retained wherever Core permits them.
There is honestly no declared domain error branch; unexpected internal FFI status
uses the existing generated conventions. Monthly XML errors are not bypassed.

The focused installed Python and compiled Java22/Kotlin tests check values and
ownership using the same final DLL. A legitimate long opaque label/name exercises
the actual returned RustBuffer: unchanged 128-byte-prefix/minimum64000 matcher,
positive-live1, explicit-free0, then64 public calls per language with retained0
without requested GC. Calibration uses each generated **String** converter.
No parser/policy/sort copy, allocation proxy, C probe or generator change is added.
This publication-gated catalog is not monthly-series decoding, identity selection,
merging, binary statistics, network acquisition or epoch/calendar policy, and
does not qualify Config/firmware/UI parity or promise stable ABI. Platform scope
remains WindowsAMD64 CPython3.13GIL, JDK22.0.2/Kotlin2.4 and
UniFFI0.32.1/Javagen0.5.2. The required combined bundle is18 families plus Clock;
CLI10/native17 published and two historical/zero new Config counts are unchanged
until publication.

### Read-only LoxApp3 Autopilot catalog candidate

One thin list-returning operation delegates directly to the published
`AutopilotCatalog::from_loxapp3_json` and consumes `into_rules`. Owned
`AutopilotRule` records contain `name`, `uuid`, `state_changed` and
`state_history` (`stateChanged()` / `stateHistory()` in Java).
Every map key contributes a row, including non-object entries. Missing or
non-string names become `"?"`; explicit empty names remain empty. Changed and
history strings default independently to empty. All IDs and text remain opaque,
including empty, unusual UUID spellings, NUL, combining characters and line breaks.
Core's stable case-sensitive name sort and enabled serde map tie order are
preserved without another parser, validator, normalization or sort.

Valid JSON with a missing/non-object Autopilot map yields an empty list.
Malformed JSON fails atomically through the SAME payload-free
`CatalogDecodeError.InvalidJson(line, column)` / checked Java
`CatalogDecodeException.InvalidJson`; Kotlin consumes that Java API.
These are host-owned values, independent of input storage, not borrowed handles.
The catalog does not select. The source-only supplied-row selector below does not
add an alias/filter API, rule editing, conditions/actions,
state acquisition, authentication/permissions, subscription or Config UI parity.

The focused Python, Java22 and Kotlin consumers exercise actual exports,
typed values/errors, ownership, permissive shapes, raw/default fields and stable
ties. The unchanged C probe checks the actual returned success RustBuffer with
the established 128-byte prefix/minimum64000 match, and the actual 20-byte
InvalidJson at line258/column65540 with the existing catalog-error matcher.
Both branches require live1, explicit-free0, then64 public calls per language
retaining0 without requested GC. No helper allocation substitutes for this export.
Qualification is limited to WindowsAMD64 CPython3.13GIL, JDK22.0.2 and Kotlin2.4,
UniFFI0.32.1/Javagen0.5.2; no stable ABI, MSRV or other-platform claim.
The actual combined same-DLL Python/Java/Kotlin bundle passed all sixteen prior
native families plus this one (seventeen total), and the existing Clock lifecycle
and resource checks. This is one new native family, not broader CLI parity.
Native Config remains two historical qualifications and zero new qualifications;
the ten whole-CLI migration count is unchanged. Publication remains review-gated.

### Supplied typed Autopilot rule selector

`resolve_autopilot_rule(rules, query)` /
`LoxArchive.resolveAutopilotRule(List<AutopilotRule>, String)` reuses the
existing four-field `AutopilotRule` record. It converts supplied rows to Core,
calls `autopilot_catalog::resolve_rule`, and detaches the selected row or error
candidates. It does not parse another JSON snapshot, call the catalog operation,
acquire state, or reuse the different control-resolution algorithm.

Core alone chooses the legacy loose UUID branch: a string longer than twenty
bytes containing at least one hyphen and only ASCII hex digits or hyphens is
searched as an exact raw,
case-sensitive UUID; the first identical UUID wins. Other queries match names
using Rust lowercase substring comparison, with no exact-name preference,
trimming, alias/room interpretation, Unicode normalization or deduplication.
An empty name query matches every row. Ambiguity retains all matching rows in
caller order, including identical IDs and duplicate rows. Every successful row
and candidate retains all four opaque fields, including NUL and line breaks.

Native `RuleResolutionError.UuidNotFound`, `.NotFound` and `.Ambiguous` map to
checked Java `RuleResolutionException` variants, also used by Kotlin. All carry
the exact supplied `query`; `.Ambiguous` additionally owns `candidates`.
The Rust facade preserves Core's `Display` messages. The generated exceptions
carry the typed variants and owned fields, not Rust's display text or a
debug/JSON payload. Returned values are detached host-owned records, with no
new native object handle or disposal protocol.

Rust and Python/Java 22/Kotlin tests cover raw first-UUID wins,
case-sensitive UUID failure, loose hyphen IDs, unranked/duplicate ambiguity,
empty input/query, Unicode lowercase-only behavior, opaque fields and ownership.
The existing builder retains all 141 steps from the published statistics
baseline and adds five selector steps (146 total). Its artifact manifest adds
the three consumer logs and `AUTOPILOT-RULE-MECHANISM.json`. The unchanged C
HeapWalk primitive witnesses the actual export's success, UUID-not-found,
name-not-found and ambiguous RustBuffers. The SDK built from `de1bc5e` passed
live1/free0 calibration and zero matching retained buffers after 64 public
calls per branch in Python, Java 22 and Kotlin, without requested GC. See
`AUTOPILOT-RULE-MECHANISM.json` and the matching `ARTIFACTS.json`. The fresh
archives and their consumer checks passed as well. This selector adds no CLI
command, native Config workflow or new platform qualification.

### Read-only LoxApp3 metadata snapshot candidate

This publication-gated family adds one parse/projection operation over the
already published Core `StructureMetadata::from_loxapp3_json`. Its plain owned
`StructureMetadata` record contains collections of `Category(uuid, name)`,
`GlobalState(name, uuid)` and `OperatingMode(raw_id, name)` records
(`globalStates`, `operatingModes`, `rawId` in Java). There are no opaque handles,
three competing parse APIs, full-document model or additional dependencies.

Core alone parses, omits and sorts. Any valid JSON root is accepted: absent or
non-object maps contribute no entries. Categories require string `name` values;
global states and modes require string values. Empty/raw identifiers and strings,
NUL, combining characters, non-ASCII, CRLF and XML-looking text remain unchanged.
Categories use case-sensitive name order, global states raw state-name order;
mode ordering uses Rust `i32` parsing with failures as zero, retaining raw IDs.
Stable ties retain Core's input-map iteration order, which depends on the Core
serde_json feature set. Bindings do not validate UUIDs, ranges or mode syntax,
normalize strings or sort again.

Malformed JSON and UTF-8 use the existing Python
`CatalogDecodeError.InvalidJson(line, column)` / checked Java
`CatalogDecodeException.InvalidJson` with exact Core positions and no input
payload. The same Java API is consumed by Kotlin. Collections and strings are
host-owned and independent of input storage; no credential erasure or immutable
string disposal guarantee is implied.

`structure_metadata_smoke.py`, compiled `StructureMetadataSmoke.java` and
`.kt` cover typed values, permissive shapes, raw/empty/Unicode fields, mode
fallback/tie ordering, existing typed errors and input independence. The actual
new metadata export is calibrated through its returned success/error RustBuffers:
the unchanged generic C heap probe reads an actual 128-byte success prefix with
minimum 64000 bytes; the unchanged catalog-error primitive matches the actual
20-byte InvalidJson error at line258/column65540. Each branch requires live1,
explicit-free0 and 64 public calls per language retaining0 without requested GC.
Existing native regression families and Clock lifecycle remain in the builder.
This adds no network, authentication, permissions, state values, mode editing,
subscriptions, UI parity, ABI/MSRV/platform or native Config qualification.

### Read-only user inventory candidate

This publication-gated family adds only owned `XmlUserInfo` records with
`name: String`, `uuid: String`, `nfc: bool`, and `description: String`.
Only exact raw `C` elements with first `Type="User"` at any depth count.
Encounter order and duplicate identities remain unchanged. First matching
attributes retain raw UTF-8/entity spellings; absent or unreadable attributes
default independently. Missing `Title`, `U`, and `Desc` become empty strings.
Only literal underscores in the description become spaces; names and identifiers
are untouched. NFC is only whether raw `NFCArr` is nonempty (even `"0"`, whitespace
or an entity spelling): no contents are returned or interpreted.

Empty/text-only input and incomplete EOF can succeed. Actual XML reader errors
are atomic, returning no partial list through the existing Python
`ConfigurationError.Xml` / checked Java `ConfigurationException.Xml`;
`source_message` / `sourceMessage()` retains the original Core reader message.
There is no entity decoding, UUID validation, normalization, stricter firmware
parser, credential/password access, permissions/authentication evaluation,
user CRUD, DOM loading or session handle. The unrelated shared DOM bug
`lox-n37w` remains explicit.

`users_smoke.py`, `UsersSmoke.java` and `UsersSmoke.kt` exercise actual exports,
typed values/errors, raw/default/UTF-8/order semantics and owned snapshots.
The existing unchanged C heap probe guards actual large success and XML-error
buffers with an exact 128-byte prefix and minimum 64000-byte allocation:
live one, explicit release zero, then 64 public calls retaining zero without
requesting GC. Kotlin calls the SAME generated Java API and DLL. This does not
add platform, ABI, MSRV, registry, native Config or full-parity qualification.

### Read-only device inventory candidate

This thin device head is publication-gated; older packages do not expose it.
`XmlDeviceInfo` owns `name`, typed `XmlDeviceBus` (`Tree`, `Air`, `Network`),
optional `serial`, optional `sub_type` (`subType()`), `type_label`
(`typeLabel()`), optional `mac`, and optional `address`. Core's unsigned
32-bit subtype is widened losslessly to `u64`: Python receives an integer,
Java/Kotlin a nullable `Long` in **0..4294967295**, never a narrowed signed
32-bit value. Missing and empty optional strings remain distinct.

Only exact raw `C` types `TreeDevice`, `LoxAIRDevice`, `NetworkDevice` count.
Bus depends on that type alone. Only Tree retains `Serial`; only Network
retains `MAC`/`Addr`, and its label always remains `Network device` regardless
of subtype. Missing names become empty. Labels and untrimmed `u32` parsing
come from Core, including unknown subtype labels. Raw UTF-8/entity spellings,
first matching attributes, ignored malformed attributes, absent invalid UTF-8
values, encounter order and duplicates are unchanged. Empty/text-only input
and incomplete EOF can succeed; reader errors return no partial list and
raise Python `ConfigurationError.Xml` / Java checked
`ConfigurationException.Xml` with original `source_message`/`sourceMessage()`.
These SAX contracts are not the separate strict firmware parser or DOM APIs;
`lox-n37w` remains explicit. There is no port, grouped identity, room, filter,
enrichment, authentication or permission API.

`devices_smoke.py`, `DevicesSmoke.java` and `DevicesSmoke.kt` exercise actual
native exports and host-owned values. Their success/error allocation proofs
reuse the unchanged reviewed Windows heap probe: a real large serialized
device result/error is counted live once, explicitly freed and counted zero,
then 64 public calls retain zero matching buffers without requesting GC.
Long mismatched XML tag names make real source-error buffers large enough
for that existing exact-prefix/64000-byte guard; no small-allocation proxy,
new production counters, heap-scan weakening or recovery claim is involved.
The scope remains Windows AMD64, CPython 3.13 GIL and Java 22 FFM with Kotlin
using that same Java API; no ABI, MSRV or other-platform qualification.

### Typed structural summary comparison candidate

`diff_configs(old, new)` / `LoxArchive.diffConfigs(old, new)` compares supplied
typed `ConfigSummary` snapshots through the existing Core comparator. It is
infallible for valid typed snapshots: there is no invented domain error,
XML parsing, JSON, CLI invocation, or language-side comparison. Host type
errors and existing `parse_config_summary` XML errors remain separate.
This additional family is unpublished until matching artifacts are published.

`ConfigDiff` owns `version_old`, `version_new`, `date_old`, `date_new`,
`controls_added/removed: list[ConfigControlRecord]`,
`controls_changed: list[ConfigControlChange]`, `rooms_added/removed`,
`categories_added/removed: list[ConfigNamedEntry]`, `rooms_renamed`,
`categories_renamed: list[ConfigNameChange]`, `users_added/removed`, and
`has_changes`. Java record accessors use camelCase. `has_changes` is computed
by Core's `has_changes()` before conversion, not a second predicate:
metadata-only changes are carried through but return false.

`ConfigControlRecord` contains raw `uuid` and a `ConfigControlEntry`;
`ConfigControlChange` contains raw `uuid`, the **new** summary's `name`,
and `ConfigControlFieldChange`. Python variants are `.NAME(old, new)`,
`.TYPE(old, new)`, `.ROOM(old, new)`; Java/Kotlin use nested records
`.Name`, `.Type`, `.Room`. Java's generated accessor for a field named
`new` is `_new()`, including in `ConfigNameChange`. Room payloads are
`ConfigRoomReference(uuid, name)` with a nullable per-side title. Missing
titles stay distinct from empty titles. `ConfigNamedEntry` owns `uuid/name`;
`ConfigNameChange` owns `uuid/old/new`. Python record constructors are
keyword-only; enum payload constructors also accept positional arguments.

Identity is the exact raw UUID, never a title. Each control's changed fields
retain Core's name/type/room order. Renaming a room without changing its UUID
does not move controls. User names compare as sets, ignoring duplicate counts
and input order. Map/set enumeration order is **unspecified**; no production
sorting is added. Reverse input conversion moves all snapshot fields without
normalization, defaults, or another parser. Returned records and collections
are owned and remain independent of subsequent caller input mutation.

`config_diff_smoke.py`, `ConfigDiffSmoke.java`, and `ConfigDiffSmoke.kt`
exercise every structural family and sum-type variant, metadata, user sets,
raw identities, Unicode, room-title optionality, and input independence.
`NativeConfigDiffProbe` and the Python test reuse the unchanged reviewed
128-byte-prefix/minimum-64000-byte Windows probe on the **actual exported**
result. A long opaque caller-supplied version string makes that real result
large enough: live=1, explicit free=0, then 64 public calls retain zero matching
buffers without requesting GC. No artificial error, allocator proxy, new
generator correction, or production counter is needed. The existing native
families and Clock lifecycle remain separately tested on the same final DLL.
This is structural-summary comparison, not full XML/semantic Config UI diff,
credentials/permissions, editing/deployment, ABI freezing, or platform parity.

### Errors and compatibility boundaries

Python raises `ArchiveError.HeaderTooShort`, `.InvalidMagic`,
`.ZeroBackReference`, `.BackReferenceOutOfBounds`, `.InvalidZip`, `.ZipEntry`,
`.ReadZipEntry`, or `.MissingConfigEntry`. Java throws the checked
`net.eisber.lox.archive.ArchiveException` with the same nested subclasses.
Kotlin catches those same Java exceptions. Named fields retain lengths,
positions, ZIP member index/name and `source_message` (Java `sourceMessage()`).
For example, short input exposes Python `error.actual == 3` or Java
`error.actual() == 3L`. Underlying Rust error objects cannot cross the ABI;
their source text is retained, not fabricated as a native-language cause.
Generated exception messages describe fields, not the core's Display format;
branch on variants/fields, not message text. `Other` is only a fallback for
future non-exhaustive core variants, not the normal error representation.

Lengths/positions cross as `u64` (Java boxed `Long`); magic is `u32`
(Java boxed `Integer`, use `Integer.toUnsignedLong` for unsigned formatting).
Values are typed, but no stable cross-version ABI or independently swappable
native library is promised. Keep the generated sources/jar/wheel and DLL from
the **same build**; generated contract/checksum checks remain enabled.

The adapter preserves raw bytes: UTF-8 BOM, CRLF, entities, duplicate and
digit-named attributes, order, NUL and non-UTF8 data are not interpreted.
This is **not** XML validation or a deployment trust decision. Core
permissiveness is unchanged: LoxCC header CRC/declared lengths are not
verified; truncated final tokens can yield partial output; the allocation
hint is not an output/memory cap. Compression retains inherited `u32` size
casts. ZIP selection is the first case-insensitive name containing
`sps0.loxcc`, including nested names and suffixes. The APIs add no untrusted-
input hardening, root/unique-member rule, filesystem sidecar or atomic-write
policy. Do not equate successful decoding with safe deployability.

Archive calls are stateless and still prove **nothing** about runtime lifecycle.
The separate clock proofs below cover the current HTTP operation only, never
imaginary subscriptions or full runtime parity. Task `lox-vuc6` remains open
until coherent prerequisite integration and independent review are complete.

## Offline three-way projection

`three_way` was verified on the matching generated/native artifacts at
published main `2603542`. There is no `validate_document` export. All validation,
semantic comparison, placement and selected-byte preservation stay in Core.
Inputs and output are owned byte buffers, not strings, XML trees or handles.
There is no acquisition, I/O, network, schema, sidecar, baseline, permission,
deployment or safe-push guarantee, and no new whole-command migration credit.

Core chooses raw shell/child spans independently: local equal to base chooses
live; otherwise live equal to base or local equal to live chooses local.
The output need not keep local's BOM/line endings when the live shell wins.
Formatting/attribute order equivalence is not normalization of literal
attribute newlines. Incompatible edits, deletion/modification, placement and
final duplicate identities are refused. This private byte-span parser does not
use the DOM/editor loader, so `lox-n37w` is not a generic limitation here.

`OverlayError.Refused.frames` (Java/Kotlin checked
`OverlayException.Refused.frames()`) is an owned, ordered outer-to-inner list of
`OverlayErrorFrame`. It follows actual Core `Error::source()` links between
overlay domain errors, without recursion in the foreign model or truncation.
For independent additions of UUID `c` under different parents, the final
consistency refusal retains exactly:

1. `MERGED_CONSISTENCY` / Java `MergedConsistency`
2. `DUPLICATE_UUID(id="c")` / Java `DuplicateUuid.id()`

The domain frames cover all current `three_way` error paths:
`DocumentUtf8`, `Utf8`, `StringUtf8`, `Xml`, `Attribute`, `DuplicateUuid`,
`MissingControlType`, `UnexpectedClosingTag`, `Dtd`, `OutsideRoot`,
`IncompleteRoot`, `ShellConflict`, `ControlConflict`, `Placement` and
`MergedConsistency`. Python data-enum variant names are upper snake case;
Java uses nested record names as listed. UUID spelling/normalization is Core's,
not reimplemented in the facade.

The native-source boundary is explicit: `Xml` and `Attribute` retain distinct
frame types, typed `OverlayXmlContext.DOCUMENT`/`BARE`, and the native cause's
`Display` text in `source_message` / `sourceMessage()`, never Debug. They do
not pretend native parser errors are additional overlay-domain frames.
UTF-8 frames retain byte `valid_up_to`/`validUpTo()` and optional byte
`error_len`/`errorLen()`; `StringUtf8` also keeps the original invalid bytes.
Native `usize` values widen to `u64` (Java boxed `Long`) without truncation.
An absent error length means an incomplete UTF-8 sequence, not zero.

New non-exhaustive Core variants, and existing overlay-application-only
variants outside this single export's contract, cause the payload-free
`OverlayError.UnsupportedCoreVariant` /
`OverlayException.UnsupportedCoreVariant` refusal. They are never flattened
into XML, arbitrary text, JSON or success. No stable cross-version ABI is
promised; regenerate all matching native/generated artifacts together.

This slice adds six checks to the SDK builder: Core runtime-overlay focus,
Python smoke, Java compile/smoke and Kotlin compile/smoke. With the five
month-merge checks, the integrated builder retains all 146 published steps
and has 157 total. Existing `rust-focused` includes the facade tests. All six logs
and `RUNTIME-OVERLAY-MECHANISM.json` enter the existing artifact manifest.
The new callers include the two-frame witness, byte ownership and typed native
boundaries. HeapWalk sources calibrate actual returned success/error RustBuffers
with the unchanged 128-byte-prefix/minimum-64000 matcher (live1/free0), then
retain host outputs/errors over 64 calls per branch without requesting GC.
The three-way and month-merge checks passed on the matching artifacts at
published main `2603542`; that receipt does not cover fragment application.

## Source-only offline LoxCC fragment application

This preflight is **unrun**. `merge_overlays` accepts the exact base bytes and
caller-supplied `LoxccFragment { name, data }` entries, with compressed bytes in
`data`. The Python record takes keywords (`name=`, `data=`); Java/Kotlin use
the same generated record with `byte[]` / `ByteArray`. The sequence is not
an application order. It retains duplicate entries until the adapter rejects
the lowest exact repeated filename as `DuplicateFilename`, as in the CLI's
sorted listing. A generated map could collapse repeated strings on lift.
Core receives a `BTreeMap` of exact unique keys, applies them in filename order,
and itself refuses normalized UUID aliases.

Empty input passes the base bytes through exactly, including malformed XML or
UTF-8. Nonempty input delegates once to Core. Standalone fragment checks,
length and CRC validation, replacement of selected spans, unique
`AutopilotRule` placement on `Page` with `SpNo="1"`, and atomic refusal are
Core behavior. No I/O, acquisition, schema or deployment check is performed.

`FragmentApplicationError.Refused` / checked
`FragmentApplicationException.Refused` preserves ordered domain frames. The
common parser frames reuse the published `OverlayErrorFrame` inside `Overlay`;
the published three-way error type is unchanged. For example, a short LoxCC
header produces `FragmentRead(name)`, `Archive`, then
`ArchiveHeaderTooShort(actual)`. Invalid magic and both back-reference errors
retain their native numeric fields. ZIP errors are not reachable through this
raw codec call. `UuidConsistency(name)` retains the ordered inner Core frame,
such as `DuplicateUuid(id)`. Unsupported causes reject the entire projection
rather than returning an incomplete chain.

The five additive SDK steps retain all 157 published steps. The new proof
sources cover owned large nonempty success and six named refusals (four codec
causes, integrity failure and the UUID consistency chain). They use the
unchanged HeapWalk primitive: the established 128-byte/minimum-64000 success
match, and the published MQTT short-buffer pattern with a prefix that includes
bytes of the actual filename and a minimum equal to the actual error length. The steps
require live1/free0, then no matching native allocations with 64 distinct
host results or errors retained, without requested GC. Small empty bypass
and other short or payload-free branches have no separate native allocation
attribution claim. Test sources, log paths and an after-success mechanism
receipt are not qualification; compiler, generation, SDK and package
execution for this operation remain unrun.

## Offline color heads

This additional candidate family is available **only when published** in matching
wheel/JAR/DLL artifacts, not by mixing it with older native packages. It is
offline color data, not a new Lighting mode, native Config qualification,
transport API or whole SDK. Existing raw `set-mood-color`, eligibility, statistics,
timezone, authentication and protocol behavior are unchanged.

The public calls are synchronous owned-value operations. Python uses
`encode_color(input, brightness)`; brightness is a **required argument** accepting
`None` (there is no generated default). `None` delegates to Core: no extra RGB/HSV
scaling, or temperature brightness 100. Java/Kotlin call
`LoxArchive.encodeColor(ColorEncodeInput, Double)` with `null` for that option.
Inputs are `ColorEncodeInput.RGB(red, green, blue)`, `.HSV(ColorHsv(...))`,
`.TEMPERATURE(kelvin)` in Python; generated Java nested records are `.Rgb`,
`.Hsv`, `.Temperature`. RGB input channels are floating-point, validated then
rounded by the same Core factory as text parsing. HSV is degrees/percent.
Generated Python records are keyword-only, e.g.
`ColorHsv(hue=120.0, saturation=100.0, value=50.0)`; enum payload constructors
also accept positional arguments as shown below. Java uses record constructors.

```python
import lox_archive as lox

result = lox.encode_color(lox.ColorEncodeInput.RGB(255.0, 100.0, 0.0), 50.0)
assert isinstance(result, lox.ColorEncoded.RGB)
actor_integer = result.actor.value
mood_integer = result.mood.value  # Different profile, never interchangeable.
temperature = lox.encode_color(lox.ColorEncodeInput.TEMPERATURE(2700), None)
assert temperature.command == "temp(100,2700)"
```

`ColorEncoded.RGB` / Java `.Rgb` retains `rgb`, `actor`, `mood`, `hsv`,
`hsv_command` (`hsvCommand()`). Temperature retains `kelvin`, `brightness`,
`command`; no integer temperature encoding exists. All derived values/command
rendering come directly from Core, including RGB parse-round then brightness-round,
hue wrapping, mood percent projection and number spelling.

`ColorRgb` contains signed 32-bit `red/green/blue` outputs (0..255);
`ColorPercentRgb` is a distinct 32-bit percent-channel record (0..100).
`ColorHsv` has three doubles `hue/saturation/value`. `ColorActor` retains
`value: i64` and `rgb`; `ColorMood` retains `value: i64`, `percent`,
`approximate_rgb` (`approximateRgb()`). Java receives `Integer`, `Long` and
`Double`, not signed bytes or unsigned-32 packed colors. Kelvin input is signed
`i64`: checked `u32` representation conversion precedes Core validation.
Out-of-representation values raise the separate `KelvinRepresentation(value)`;
representable values still use Core `Kelvin(value)` with 1000..15000 limits.
There is no truncation of channels, kelvin or raw mood high digits.

Python `decode_color_legacy(spec)` / Java `decodeColorLegacy(String)` returns
`ColorDecoded.HSV(hsv, rgb)`, `.TEMPERATURE(brightness, kelvin)`, `.RGB(actor)`,
or `.MOOD(mood)` (Java `.Hsv/.Temperature/.Rgb/.Mood`). These are **observations**:
HSV and temperature floats may be out of range, NaN, infinities or negative zero.
They remain actual host doubles, never JSON null or normalized strings.
`parse_mood_color_legacy(spec)` / `parseMoodColorLegacy(String)` returns
`ColorMood`; `observe_actor_color_legacy(value)` /
`observeActorColorLegacy(Long)` returns `ColorActor`. Raw actor modulo validation
admits some negative/high values rejected by bounded numeric decode. Mood prefix
classification and percent validation preserve the entire original signed i64,
including ignored high digits. Strict encoding does not adopt these permissive
legacy profiles. `color_format_number(value)` / `colorFormatNumber(Double)`
explicitly calls Core's existing command-number formatter.

Python raises `ColorError` nested variants; Java throws checked `ColorException`
with the same nested subclasses, also used by Kotlin. Exhaustive field mappings:
`Arity(value, notation, expected)`, `Number(value, notation)`,
`RgbChannel(channel, value)`, `HsvRange(value: ColorHsv)`, `Brightness(value)`,
`Kelvin(value)`, `ActorChannel(value, channel, channel_value)`,
`MoodChannel(value, channel, channel_value)`, `InvalidValue(value, profile)`,
`ActorRange(value)`, plus the representation error above. Channel, notation and
integer-profile fields are typed enums, not parsed diagnostic strings.
Error float fields preserve nonfinite host values too. Messages are facade
diagnostics, not a promise of identical Core `Display` spelling.

Installed Python and compiled Java/Kotlin tests use the same binding-crate
`tests/fixtures/color_profiles.tsv` and exercise typed branches/error
fields, profile differences, full i64s and nonfinite/negative-zero ownership.
Every new success/error return path has positive live/released RustBuffer
calibration and 64 public calls per language with zero retained matching
buffers, without requesting GC. This is serialized result/error-buffer
ownership, not deterministic per-call status-arena disposal:
caller-thread `Arena.ofAuto` remains a GC fallback.
Existing generator checksum/fail-closed correction, archive/clock/control and
room/summary tests remain required. Qualification is Windows AMD64, Python 3.13
GIL, JDK 22 and Kotlin 2.4 only; no new MSRV, ABI, platform, registry or native
Config credit. Historical native UI qualifications remain two; new credit zero.

## Payload-only Weather-State heads

`decode_weather_state_payload(bytes)` and Java
`LoxArchive.decodeWeatherStatePayload(byte[])` (also used directly by Kotlin)
are thin owned projections of the existing default Core decoder. The caller
identifies and assembles the complete table and removes transport headers.
This is **not** HTTP `weatheru.bin`, a framer, a subscription or a CLI wrapper.
No Core/CLI caller is migrated and no native configuration capability is added.

Each `WeatherState` contains `uuid_wire_bytes` (16 original bytes),
`uuid_native35` (shared Core `StateUuid` formatting), `last_update` (`u32`),
`entry_count` (`i32`) and ordered owned `entries`. Each `WeatherEntry` contains
five signed `i32` observations: `timestamp`, `weather_type`, `wind_direction`,
`solar_radiation`, `relative_humidity`; and six authoritative `u64` IEEE fields:
`temperature_bits`, `perceived_temperature_bits`, `dew_point_bits`,
`precipitation_bits`, `wind_speed_bits`, `barometric_pressure_bits`.
NaN payloads, signed zero, infinities, duplicate IDs/entries and encounter order
are preserved. No units, epoch, timezone, finiteness or semantic-range validation
is introduced; no floating-point convenience value replaces the authoritative bits.

Python receives unsigned `int` values for `u32` and `u64`. Java `int` / Kotlin
`Int` carries the raw update bits: use `Integer.toUnsignedLong(value)` or
`value.toUInt()` for its unsigned magnitude. Java `long` / Kotlin `Long` carries
each raw IEEE bit pattern; use `Long.toUnsignedString(value)`,
`Long.toUnsignedString(value, 16)`, or Kotlin `toULong()` for unsigned presentation. Signed fields
remain signed. Host records have no native handle or close obligation.

Errors map Core's actual four variants and all fields exactly:
`TruncatedHeader(record_offset, available)`,
`NegativeEntryCount(record_offset, declared)`,
`LengthOverflow(record_offset, declared)`,
`TruncatedEntries(record_offset, declared, expected, available)`.
Offsets/lengths widen from `usize` to `u64`; declarations remain signed `i32`.
The overflow variant is preserved even where unreachable on Windows AMD64.
Core checks the entire input before producing results: a malformed suffix rejects
the whole table; empty input is valid. Errors contain no identifiers or payload.

`weather_state_smoke.py`, compiled `WeatherStateSmoke.java` and `.kt` exercise
the same native DLL and generated Java API, including typed failures and
deterministic RustBuffer release with the unchanged Windows HeapWalk primitive
(live 1, freed 0; 64 calls per success/error branch, retained 0, no requested GC).
Qualification is per matching build: Windows AMD64, CPython 3.13 GIL,
Java 22.0.2 and Kotlin 2.4.21 using UniFFI 0.32.1 / Java generator 0.5.2.
Source proof alone is not package/review/publication credit, ABI stability,
registry availability, other-platform support or full parity.

## Payload-only numeric state heads

The synchronous adapter calls the existing default, pure-std Core decoder;
it contains no second parser. The **caller** identifies and assembles the numeric
value-state table and strips all headers. Matching length cannot identify type.
Each 24-byte record contains 16 UUID bytes (u32/u16/u16 little-endian fields,
then eight bytes in wire order), followed by little-endian IEEE 754 f64 bits.
Empty input is valid. Order and duplicate identifiers are retained. A length not
divisible by 24 rejects **the entire payload**, even with a complete prefix.
This strict Core API policy intentionally differs from the pinned upstream
Kotlin prefix reader, which ignores an incomplete tail. It is not an upstream
bugfix or a firmware/native behavior claim; Core provenance remains unchanged.

Python calls `decode_value_state_payload(payload)`; Java and Kotlin call
`LoxArchive.decodeValueStatePayload(payload)`. The owned `NumericState` record
has Python fields `uuid_wire_bytes: bytes`, `uuid_native35: str`, `value: float`
and `value_bits: int`. Java has `uuidWireBytes(): byte[]`,
`uuidNative35(): String`, `value(): double`, `valueBits(): long`;
Kotlin uses those same Java accessors and `ByteArray`/`Double`/`Long`.
For wire bytes `00 01 ... 0f`, native35 is
`03020100-0504-0706-08090a0b0c0d0e0f`, not RFC36. No UUID validation,
normalization or identifier policy in another API changes.

**`value_bits` / `valueBits()` is the authoritative exact representation.**
It retains all 64 bits, including NaN sign/payload/signaling, signed zero,
subnormals and bit 63. Python's generated u64 converter returns a nonnegative
integer in `[0, 2**64 - 1]`. Java's generated u64 converter returns a signed
primitive `long` with the **same bit pattern**, not the same unsigned numeric value:
use `Long.toUnsignedString(bits)` or Kotlin `bits.toULong()` for unsigned
interpretation. `value` is a convenient host floating interpretation; do not
reconstruct authoritative bits from a float because host ABIs/operations may
quiet or canonicalize NaNs. Floating equality cannot distinguish these cases.
Host record equality is likewise not a bit-identity contract. Returned bytes
and records are owned snapshots, with no native handle/close obligation.
Generated snapshots are mutable; caller edits can make their fields inconsistent.

Malformed input raises Python `NumericStatePayloadError.InvalidLength` or
checked Java `NumericStatePayloadException.InvalidLength` (the same Java
exception in Kotlin), with `actual` and `trailing` lengths widened from Core
`usize` to u64 under a compile-time guard. No payload or prefix result is exposed.
There is no framing, authentication, network, callback, subscription,
statistics/text/weather decoding or CLI migration in this numeric operation.

`numeric_state_smoke.py` and compiled `NumericStateSmoke.java` / `.kt`
exercise the installed wheel and same-DLL JAR: empty input, known wire/native35
UUIDs, duplicates/order, all malformed lengths through 72, failure then success,
host-owned values and exact normal/subnormal/negative-zero/infinity/NaN/high-bit
patterns. The existing Windows HeapWalk probe calibrates a live/freed large
success RustBuffer and the actual 20-byte two-u64 error buffer, then checks
64 public calls per path/language with no requested GC. This proves those
serialized buffers are released, not global zero-allocation behavior or
deterministic disposal of automatic caller-thread status arenas. Existing
generator corrections and all previous family proofs remain unchanged.

## Payload-only text state heads

This additional native family delegates directly to the **already published**
`lox_core::text_state::decode_text_state_payload`; no parser, UUID implementation,
transport, framing, reassembly, type detection or subscription is added to a head.
The caller supplies an identified, assembled payload with headers removed.
Core and CLI behavior are unchanged. This candidate requires independent review
and serialized publication before it is a published package capability.

Python calls `decode_text_state_payload(bytes) -> list[TextState]`; Java and
Kotlin call `LoxArchive.decodeTextStatePayload(byte[]) -> List<TextState>`.
Each flat owned record contains:

| Python field | Java accessor (also Kotlin) | Meaning |
|---|---|---|
| `uuid_wire_bytes: bytes` | `uuidWireBytes(): byte[]` | All 16 state UUID wire bytes |
| `uuid_native35: str` | `uuidNative35(): String` | Core's mixed-endian native35 spelling |
| `icon_uuid_wire_bytes: bytes` | `iconUuidWireBytes(): byte[]` | All 16 icon UUID wire bytes |
| `icon_uuid_native35: str` | `iconUuidNative35(): String` | Same shared UUID contract |
| `text_byte_len: int` | `textByteLen(): int` | Declared unsigned u32 **UTF-8 byte** count; Java uses `Integer.toUnsignedLong`, Kotlin `toUInt()` if needed |
| `text: str` | `text(): String` | Exact valid UTF-8 decoded text |

For `00 01 ... 0f`, native35 is `03020100-0504-0706-08090a0b0c0d0e0f`,
not RFC36. Empty payload and empty text are valid. Order, duplicates, decomposed
Unicode, non-BMP characters, and **every in-length embedded/trailing NUL** survive.
No Unicode normalization, NUL trimming or lossy replacement occurs. A record is
36 header bytes (state UUID16, icon UUID16, u32 little-endian byte length), text,
then `(4 - length % 4) % 4` ignored padding bytes; nonzero padding is valid,
but missing padding is not. Any invalid suffix rejects the **whole payload**,
never returning a valid prefix. Strict atomicity, strict UTF-8 and NUL retention
are original Core policies, not upstream bug fixes or firmware qualifications.

Python raises `TextStatePayloadError` variants; Java throws checked
`TextStatePayloadException` nested subclasses, also caught by Kotlin:
`TruncatedHeader(record_offset, available)`,
`LengthOverflow(record_offset, declared)`,
`TruncatedText(record_offset, declared, available)`,
`TruncatedPadding(record_offset, expected, available)`,
`InvalidUtf8(record_offset, valid_up_to, error_len)`.
Java fields use camelCase accessors. Positions/counts from Core `usize` widen
to u64 under the existing compile-time platform guard; `declared` stays u32.
`error_len` is optional (`None` / nullable `Long`); absence means an incomplete
final UTF-8 sequence. `valid_up_to` is relative to that record's text, while
`record_offset` is a payload byte offset. Errors expose no payload, text or
partial records. Branch on typed variants/fields, not diagnostic spelling.
On the qualified 64-bit platform a u32 declaration cannot cause `LengthOverflow`;
its exhaustive mapping is Rust-tested, not claimed as a reachable native path.

The snapshots do not retain native handles or require `close`. Host records
and Java byte arrays are caller-owned, not immutable consistency guarantees.
Compare UUID bytes and exact UTF-8 bytes explicitly; host container/record
equality is not an authoritative byte/bit-identity test.

The native builder additively retains all eleven prior families and Clock
lifecycle proofs. New installed-wheel and compiled JAR/Kotlin tests check empty
and multi-record input, duplicates, nonzero padding, both complete UUIDs,
byte-versus-character length, non-BMP/decomposed Unicode, embedded/trailing NUL,
all reachable typed failures after a valid prefix, and recovery. Each language
calibrates one live/freed large success RustBuffer and every reachable error
serialization (including both UTF-8 option cases), then runs 64 public calls
per path without requesting GC and requires zero retained matching buffers.
The stack-built test-only error signatures avoid measurement-created matches.
This is path-specific serialized-buffer ownership, **not global zero allocation**
or deterministic caller-thread automatic-arena disposal. Existing snapshot-free
and owned Clock worker Arena-close generator corrections remain unchanged.

Build with the existing `scripts/build-archive-bindings.ps1` command below and
a **fresh** output directory; its manifest now includes the three text proof logs.
Qualification remains Windows AMD64, Python 3.13 GIL, JDK22, Kotlin2.4 and the
same native DLL only. No registry upload, MSRV/other-platform/stable-ABI claim,
CLI migration, native Config or firmware credit is added (two historical
native Config qualifications, zero new).

## Read-only configuration inspection heads

These are three thin, synchronous calls to the existing typed Core byte parsers,
not the shared DOM loader (`lox-n37w`, still open). The binding crate enables
Core's existing `configuration-xml` feature; default Core consumers, Core source,
CLI behavior and parser implementations are unchanged. Caller code owns I/O and
explicit archive decoding. None detects ZIP/LoxCC, repairs XML, normalizes,
edits, serializes, validates deployability, resolves names or interprets versions.

```python
import lox_archive as lox

xml = b'<C Type="Place" U="r" Title="Kitchen"><IoData Pr="r"/></C>'
room = lox.parse_rooms(xml)[0]
assert (room.uuid, room.name, room.item_count) == ("r", "Kitchen", 1)
summary = lox.parse_config_summary(xml)
assert summary.rooms == {"r": "Kitchen"}
```

Java/Kotlin call `LoxArchive.parseRooms(byte[])` and
`LoxArchive.parseConfigSummary(byte[])` on the same generated Java API.
`RoomInfo` has `uuid`, `name`, `item_count` (Java `itemCount()`).
The native count is an unsigned `u64`, widened from Core `usize` under a
compile-time width guard. Python receives an arbitrary-precision `int`;
Java receives a boxed `Long` carrying unsigned bits (use unsigned helpers if
needed, not signed overflow assumptions). No count narrows to a 32-bit integer.

`ConfigSummary` has **all** Core fields: `version`, `date`,
`controls: dict[str, ConfigControlEntry]`, `rooms: dict[str, str]`,
`categories: dict[str, str]`, and `users: list[str]`. Java uses typed
`Map<String, ConfigControlEntry>`, `Map<String, String>`, `List<String>` and
records, not JSON. `ConfigControlEntry` has `name`, `control_type`, `room_uuid`
(Java `controlType()` / `roomUuid()`). All values are host-owned snapshots,
not retained native handles, and remain usable after input replacement and
subsequent calls. Maps have **unspecified iteration order**.

The contracts are deliberately **distinct**:

* Room inventory recognizes exact-case `C Type="Place"` start/empty elements with
  readable ID/title, accepting empty strings. Last duplicate UUID title wins.
  Counts include **every** matching raw `IoData Pr` anywhere, even before room
  definitions, sensors/devices and blacklisted summary elements; unknown IDs
  produce no room. Results retain Core's case-sensitive name sort; ties are
  unordered.
* Summary retains last duplicate map keys, ordered users including duplicate
  and empty names, and last version/date (missing attributes reset to empty).
  Control candidates require readable ID/nonempty title and use Core's exact
  blacklist, not a facade-maintained whitelist: unknown/empty/case-different
  types can qualify. A control's `room_uuid` comes only from its **direct**
  `IoData` children (last readable `Pr`), not inventory counts or nested/sibling
  metadata. Empty/missing room metadata remains an empty string.
* XML control inventory tracks eligible pending `C` candidates and consumes one
  on the next start/empty `IoData` **anywhere**, even across end tags or siblings.
  Nested eligible candidates overwrite without restoring parents. Missing or
  filtered-out type/ID preserves the previous candidate; a missing title on an
  admitted candidate invalidates emission. `Place`/`Category` definitions do not
  clear it. Type filters are ASCII-case-insensitive **exact** matches during
  tracking, so filtering can change which candidate consumes metadata.
  Room filtering is Unicode-lowercase substring matching **after** late
  definition lookup and raw ID fallback. Later definitions win, including empty
  IDs/names; absent names leave raw fallback. Duplicate control IDs remain.
  Results sort stably by case-sensitive type/title, preserving equal-key emission
  order. This is neither summary's direct-parent rule nor rooms' all-`IoData` count.
* Attributes in all three are raw readable UTF-8: entity spellings are **not**
  decoded. BOM/Unicode, ignored invalid attributes, plaintext and permissive
  EOF remain Core behavior. Success is not strict document validation.
  Input bytes are not mutated, but this is **not** a round-trip XML preservation
  or deployment-trust claim.

`XmlControlInfo` has exactly five strings: `control_type`, `title`, `uuid`,
`room`, `category` (Java `controlType()` and otherwise same-name accessors).
It is distinct from runtime seven-field `Control` and summary three-field
`ConfigControlEntry`. Returned lists/records are host-owned snapshots with no
native handle or close obligation. The generated Python signature requires all
three arguments: `parse_controls(xml, None, None)` is unfiltered; optional filters
have no implicit Python defaults. Java/Kotlin pass `null` for absent filters.

XML reader failures become Python `ConfigurationError.Xml(source_message)` /
checked Java `ConfigurationException.Xml` (same exception in Kotlin).
The exhaustive mappings preserve the underlying reader's text and meaningful
XML category without error-string parsing, invented positions or added
validation. No document is logged or attached as an error payload; reader
diagnostics may contain offending tag names, so callers should still treat
source text as untrusted. There is no strict-validation/fallback mode.

`config_inspection_smoke.py`, compiled `ConfigInspectionSmoke.java` and `.kt`
test the installed wheel and same-DLL JAR against one synthetic BOM/Unicode/raw
entity fixture, every result field, exact blacklist/unknown types, duplicate
keys, users/order, distinct inventory/direct-parent rules, permissive inputs,
invalid attributes and typed XML failures. The new Windows heap probe calibrates
actual serialized success **and** error RustBuffers for each new function.
After 64 public calls per path/language, none may remain live, without requesting
GC. This covers nested record/maps/arrays releases; existing archive and clock
calibrations and lifecycle proofs remain mandatory and unchanged. Caller-thread
automatic status arenas are still GC fallbacks, **not** deterministic per-call
or client-close disposal.

The installed-wheel `xml_controls_smoke.py`, compiled `XmlControlsSmoke.java`
and `.kt` read the same crate-local `xml_controls.xml` bytes. They cover all five
raw fields, stable duplicates, filter admission changing association, late
definitions/fallback, Unicode/BOM, permissive EOF, typed reader failure without
partial success, and host ownership. Success/error RustBuffers are each calibrated
live then freed; 64 public calls per path in each language must leave zero matching
buffers without requesting GC. This is Windows AMD64-only artifact evidence, not
an ABI, free-threaded Python, MSRV, global resource or other-platform qualification.
The existing canonical `config_inspection.xml`, source-package checks, archive
error-buffer and owned clock-thread status-arena proofs remain unchanged.

## Stateless LoxApp3 catalog heads

The facade enables Core's existing optional `structure-json` feature and calls
`ControlCatalog::from_loxapp3_json` then `select`, converting through the **same**
seven-field native `Control` used by the query API below. There is no language
JSON parser, raw JSON result, HTTP/cache/protocol stack, CLI wrapper or retained
native catalog handle. **Every call parses and allocates anew**; this API does
not reuse a snapshot across calls. Callers own acquisition and I/O.

```python
import lox_archive as lox

data = b'{"controls":{"opaque":{"name":"Light","type":"Switch","isFavorite":true}}}'
filters = lox.ControlFilter(typ=None, room=None, category=None, favorites_only=True)
controls = lox.select_loxapp3_controls(data, filters)
assert controls[0].uuid == "opaque"
```

All four Python record constructor arguments are **required keyword-only**
arguments; both function arguments are required too. Rust's `Default` does
**not** create Python defaults. For no filtering pass
`ControlFilter(typ=None, room=None, category=None, favorites_only=False)`.
Java and Kotlin use `new ControlFilter(null, null, null, false)` /
`ControlFilter(null, null, null, false)` and `LoxArchive.selectLoxapp3Controls`.
The generated Java record accessors are `typ()`, `room()`, `category()` and
`favoritesOnly()`. Results and filters are host-owned typed values; results
remain usable after replacing input bytes or making further native calls.

The Core compatibility contract is unchanged:

* Every valid JSON root is accepted. Missing/null/non-object top-level maps
  contribute nothing. Every key in the top-level `controls` object qualifies,
  even empty/opaque IDs and scalar/null/array values. Subcontrols are ignored.
  No UUID validation, trimming, schema checks or stricter resource cap is added.
* Missing/non-string name and type become `"?"`. Missing/non-string room/category
  references look up the **empty ID**. Only string map names qualify, including
  empty names. Unknown references return `None` / Java `null`, distinct from an
  existing empty name `""`. Only JSON booleans set the favorite/security flags.
* Optional type/room/category filters are conjunctive Unicode-lowercase substring
  matches, **not** full case folding or trimming; missing names match as empty
  strings. `favorites_only` is also conjunctive. Results sort **only** by
  case-sensitive name; equal-name order is unspecified.
* Invalid JSON or UTF-8 raises Python `CatalogDecodeError.InvalidJson` or checked
  Java `CatalogDecodeException.InvalidJson` (same exception in Kotlin), carrying
  actual Core `line` and `column`, not payload text or parsed error strings.
  Positions widen from `usize` to `u64` under a compile-time guard. Python gets
  integers; Java gets boxed `Long` unsigned bits. No partial result is returned.

`catalog_smoke.py`, compiled `CatalogSmoke.java` and `.kt` read the same
crate-local `tests/fixtures/control_catalog.json` as Rust tests. They cover all
fields, permissive shapes/default semantics, filters, exact error positions and
host ownership. Reusing the existing Windows HeapWalk probe, each language
calibrates a live then freed large success RustBuffer and the actual **20-byte**
payload-free error RustBuffer. A stack-built by-value error prefix avoids the
measurement creating matching heap storage. Each path then requires **64 public
calls, zero retained matching buffers, no requested GC**. This measures result
and error RustBuffers, not all allocations or deterministic caller-thread
`Arena.ofAuto` status disposal: that remains a GC fallback.

The catalog and numeric payload heads retain distinct Core-backed contracts.
XML control inventory is already published separately. Authentication, statistics,
color, control transport, device/native Config qualification and other platforms
remain unsupported by these heads unless published separately. Existing archive,
clock, control-resolution and XML inspection tests and generator corrections remain
mandatory. This experimental source is not itself a registry release or ABI claim.

## Pure control selection heads

The control entries above describe this source's experimental API, available in
matching wheel/JAR/native artifacts **once published**, not an assertion that
the candidate artifacts are a public release or that every CLI operation has parity.
The package and Java namespace retain their existing `lox_archive` /
`net.eisber.lox.archive` names for compatibility.

`prepare_control_query(query, room_filter, aliases)` takes a string, optional
room string and string-to-string dictionary. It calls shared
`ControlQuery::prepare` **before catalog acquisition**. The typed result is
Python `PreparedControlQuery.RESOLVED(control=ResolvedControl(uuid=...))` or
`.CATALOG(query=..., room_filter=...)`. Only the latter needs caller-supplied
catalog data. `resolve_control_query(prepared, controls)` returns a typed
`ResolvedControl`; already resolved plans need only an empty list.
Java/Kotlin use the same generated `PreparedControlQuery.Resolved` and
`.Catalog` Java records, via `LoxArchive.prepareControlQuery` /
`resolveControlQuery`.

The Catalog variant owns the **original request**, not a separately parsed or
normalized query. Completion reborrows that snapshot through the core parser
with no aliases; resolved alias values are never parsed again. Prepared values
are reusable owned data, not native object handles, and need no shutdown or
explicit disposal. Caller-constructed Catalog variants are also interpreted
by the core. There is no catalog download, cache/context selection, sorting,
HTTP command or language-specific resolver.

```python
import lox_archive as lox

prepared = lox.prepare_control_query("Light [Kitchen]", None, {})
controls = []
if isinstance(prepared, lox.PreparedControlQuery.CATALOG):
    controls = [
        lox.Control(name="Light", uuid="opaque-id", typ="Switch", room="Kitchen",
                    cat=None, is_favorite=True, is_secured=False)
    ]  # Supply the catalog from the caller's own source.
selected = lox.resolve_control_query(prepared, controls)
assert selected.uuid == "opaque-id"
```

`Control` has exactly the core's seven typed fields: `name`, `uuid`, `typ`,
optional `room`, optional `cat`, `is_favorite`, `is_secured` (Java camelCase
accessors). No arbitrary JSON is accepted or returned. Python raises
`ControlResolutionError.NotFound(query)` or `.Ambiguous(query, candidates)`;
Java throws checked `ControlResolutionException.NotFound` /
`.Ambiguous`, and Kotlin catches those same Java types. Candidates are an
ordered `list[Control]` / `List<Control>` retaining all seven fields, including
null/empty distinctions, duplicates, favorites and security flags. Those flags
are data, **not selection filters or authorization**. Identifier failures are
typed `IdentifierError` / checked `IdentifierException` variants
`InvalidLength`, `InvalidCharacters`, `NonHex`. Exhaustive adapters preserve
all current shared-core variants; no error-message parsing is used.

Compatibility is deliberately unusual and is not tightened by these bindings:

* Identifier recognition runs **before** exact, case-sensitive aliases of the
  raw input. Recognition means more than 20 **UTF-8 bytes**, at least one dash,
  and only ASCII hex/dashes, with **no maximum**: 21 or 51 dashes are recognized.
  Alias values are returned verbatim, may be empty/unsafe path values, and are
  neither recursively resolved nor validated.
* `validate_uuid` is a separate command-path check, **not** a canonical UUID
  parser: 1–50 UTF-8 bytes, then reject `..`, `/`, `\`, `%`, then require ASCII
  hex/dashes. Short `"a"` and `"-"` pass. 51 dashes are recognized by `is_uuid`
  but fail validation. Length errors take precedence over character errors.
* Only a query ending in `]` uses its **rightmost** `[` as an inline room
  qualifier. Only that form's name is trimmed. The literal room is not trimmed
  and overrides the room flag even when empty. Unqualified names retain spaces.
* Name/room matching uses Unicode lowercase substring comparison, **not case
  folding**, normalization or ranking. There is **no exact-name preference**.
  Missing rooms match the empty string; empty queries/rooms are valid substring
  filters. Ambiguity and candidate order are retained without deduplication.

The actual installed-wheel, compiled Java and Kotlin `control_smoke` callers
exercise typed success, zero-catalog identifier/alias paths, ordered full-record
ambiguity, original-query NotFound, room/Unicode/empty/byte-length edges and
typed validation failures. These pure functions introduce no runtime lifetime
claim; all existing archive error-buffer and clock resource proofs remain required.

## Production-backed clock heads

The native facade exposes `ClockRuntime(host, user, password, settings)`,
`read_clock(ClockCancellation) -> ClockReading`, `shutdown`, `is_closed`, and
`active_operations` (Java camelCase equivalents). Construction explicitly
creates the production driver's worker; import does not start a connection
runtime. `default_clock_settings()` / `LoxArchive.defaultClockSettings()`
obtains production defaults. Settings contain no credentials; constructor
arguments are explicit and never selected from home/context/cache files.

`ClockReading.date/time` are shared-core validated **text**, not calendar,
timezone or synchronized-clock values. The facade calls the production
operation and `ClockObservation.validate()`, not a copied decoder. Both date
and time are observed before validation of their fields. Missing/invalid
envelopes, protocol codes and values raise typed `ClockRuntimeError.Validation`
(Java `ClockRuntimeException.Validation`) with endpoint/issue. HTTP,
transport-kind, strict JSON decode, requested/shutdown cancellation, Busy,
Closed, driver-stop and shutdown-timeout errors retain distinct native types.
No arbitrary JSON or success-shaped fallback is exposed.

The native default stays **StrictJson**. The CLI-only legacy text/lossy decode
and `-vv` raw-body observer are neither exposed nor enabled by these heads.
There are no foreign callbacks/subscriptions or implicit diagnostics. Errors
carry sanitized categories/context, never URLs, credentials or response bodies.
Returned clock text is caller-owned data; do not treat it as trusted calendar
syntax. Caller Python/JVM strings and Rust/HTTP copies are **not promised
zeroized**, and the adapter cannot erase caller-owned secret memory.

### Host async dispatch and explicit disposal

`from lox_archive_async import AsyncClockClient` adds a stdlib-only asyncio
adapter without changing existing `lox_archive` imports/archive APIs. Create it
inside its owning event loop and use `async with` or `await client.aclose()`.
One read reservation and one close reservation bound an owned
`ThreadPoolExecutor(max_workers=2)`; concurrent reads fail Busy, not an
unbounded work queue. Offload threads only wait on generated blocking FFI;
the **same production Rust worker** owns all HTTP/retry/scheduling. No second
Rust runtime, nested caller-thread `block_on`, language HTTP stack or protocol
implementation exists.

Cancelling the Python read task explicitly signals the production Rust
cancellation handle. The underlying wait is shielded and admission is not
released until it returns. `aclose` invokes shared Rust shutdown on its separate
bounded close lane, drains the read, joins its own idle workers and releases its
native reference. Cancelling an `aclose` **awaiter** does not abandon the already
requested cleanup; await `aclose` again to observe completion/errors.

Java's `net.eisber.lox.archive.ClockClient` is a thin convenience over the
generated `ClockRuntime`. `readClock()` returns `CompletableFuture<ClockReading>`;
its cancellation explicitly signals Rust (interrupting/abandoning a Java wait
alone is insufficient). One owned wait thread and a one-entry bounded handoff
queue are guarded by one read reservation. Kotlin uses this **same Java API**,
not another native binding or coroutine protocol stack.

Java `close()` first invokes shared Rust shutdown, then stops/joins the head
executor and explicitly destroys the native handle. It is idempotent. Shared
`ShutdownTimeout` propagates honestly and leaves the closed handle available
for a retry. If caller-owned synchronous future continuations prevent head
drain, `ClockDispatchTimeout` reports that separate limit rather than claiming
disposal; release those continuations and retry close. Do not block a wait
thread's continuation waiting for another operation on that same client.
Use nonblocking/appropriately dispatched continuations. Interrupting close may
also require a retry to finish head cleanup. Java host drain milliseconds must
fit a nonnegative signed `long`.

Each owned wait dispatch owns a confined generated call-status arena, closed
before the dispatch returns (and before the executor can report termination).
Successful `close()` joins that executor and observes deterministic release
of its 32-byte status, without requiring GC. Nested scoped calls restore the
enclosing status, and exceptional dispatch exits also close the arena.
Generated calls on other caller
threads use automatic arenas: those statuses are reclaimed only after their
thread-local references become unreachable and GC processes them, not at each
client close. Returned FFM descriptor views still use the unchanged slab
allocator; closing the status arena does not invalidate returned values.

Reads after close fail Closed. Generated object destruction/Cleaners and
Python GC are **fallbacks, not the shutdown contract**. Direct native-handle
users must call `shutdown()` before releasing the generated object; simply
closing a generated Java wrapper is not a checked shared-core shutdown call.
Core runtime teardown waits at most 100 ms for platform blocking/DNS tasks;
it cannot forcibly terminate OS DNS or arbitrary blocking user code.

### What the clock artifact tests prove

`clock_smoke.py`, compiled `ClockSmoke.java` and `ClockSmoke.kt` share one
loopback-only Basic-auth fixture. They check repeated typed reads on the same
client, numeric-to-text validation, malformed/invalid-UTF8 JSON, missing values,
rejected protocol status and HTTP errors. Host task/future cancellation is
tested while date/time bodies are blocked and after a retryable response;
socket EOF and production active-operation drain must occur **before close**,
with native attempt/retry settings longer than the fixture deadline. A second
read while blocked is rejected Busy.

Closing blocked reads produces native shutdown cancellation, socket EOF,
after-close refusal and idempotent cleanup. Python also cancels a close
awaiter and observes independent completion/EOF before requesting close again.
Java deliberately blocks a caller continuation, observes a typed bounded head
drain timeout after native shutdown, releases it and successfully retries close.
These are actual production API/socket/resource proofs, not RSS/thread-count
thresholds or language-only lifecycle assertions. Forced native-core timeout,
real devices, TLS, DNS termination, calendar meaning, event subscriptions and
other targets remain unqualified. Existing archive/error and both 64-error
allocation regressions remain mandatory.

Both Java and Kotlin additionally create/read/explicitly close 16 clients on
16 owned wait lanes. A gated normal response captures each live 32-byte status
on its actual lane and marks it only after that dispatch's last FFI call.
The calibrated Windows heap probe searches busy allocations for that unique
32-byte marker immediately after close, before requesting GC; all 16 must be
released and all captured scopes closed. No freed bytes are read. The original
runtime-head jar retains all 16 and fails this same check.
An address-only `HeapValidate` check is insufficient here: another allocation
may already have reused the freed address. Expected marker bytes are passed
by value and assembled on the C stack, so the matcher cannot count its own
heap-allocated search prefix.

The runtime composition is private/reversible until both prerequisite source
changes are actually published and parent-serialized reconciliation receives
fresh coherent review/gates. A local artifact proof is not a public release or
authority to publish the composed prerequisite subtree.

## Reproduce the local artifact proof

Verified platform: **Windows AMD64 only**, CPython **3.13.17 GIL-enabled**,
Temurin **22.0.2+9**, Kotlin compiler **2.4.21**, Rust **1.99.1**.
Other platforms/architectures, Python versions/free-threaded builds, newer
JDKs, and declared Rust MSRV 1.91 are **unverified**. The wheel's
`py3-none-win_amd64` tag is not proof of testing every Python interpreter.

Pinned binding/runtime: Mozilla **UniFFI 0.32.1** in `Cargo.lock`.
Pinned third-party Java generator: **uniffi-bindgen-java 0.5.2**, MPL-2.0,
using UniFFI 0.32.1. Pinned wheel builder: **maturin 1.16.0**. Use trusted,
checksum-verified private tools; no global installation is needed.
Cargo dependencies come only from the normal registry. If dependencies are
missing after a manifest change, run `cargo fetch --locked` normally first;
do not disable TLS. Subsequent build/validation runs are locked and offline.

From a clean checkout, with Rust's bin directory on the **process** PATH:

```powershell
$env:PATH = "$HOME\.cargo\bin;$env:PATH"
cargo fetch --locked
pwsh -NoProfile -File scripts\build-archive-bindings.ps1 `
  -Maturin 'C:\private-tools\maturin.exe' `
  -JavaGenerator 'C:\private-tools\uniffi-bindgen-java.exe' `
  -JavaHome 'C:\private-tools\jdk-22.0.2+9' `
  -KotlinCompiler 'C:\private-tools\kotlinc\bin\kotlinc.bat' `
  -Python 'C:\Python313\python.exe' `
  -OutputDirectory 'target\archive-proof-1'
```

Use a **fresh output directory** on each attempt; the script refuses reuse
so failed logs survive. An optional `-CargoTargetDirectory` may reuse only
your own target cache. The script sets process-local `TEMP`/`TMP` and short
Windows paths for its own target/temp (MSVC long-path avoidance), two build jobs
and `LOX_TELEMETRY=0`. It explicitly unsets `RUST_MIN_STACK` for every actual
native artifact/caller proof. Separate full-workspace Windows CLI test-harness
gates may require the established 16 MiB Rust test-thread stack; that is not a
native Python/JVM runtime claim. It changes no global PATH, service or VM.
PowerShell 7+, Windows build tools and the listed compilers are prerequisites.
The allocation regression also compiles a small **test-only** C helper using
existing MSVC x64 tools (`vswhere`, or pass `-VcVars ...\vcvars64.bat`).
On disk-constrained hosts put both output and the owned Cargo target on a
roomy drive; the script never cleans or relocates another owner's files.

The script runs focused Rust tests, builds the pinned helper generator,
invokes maturin to build a native wheel, creates a private venv, and installs
that wheel with `--no-index --no-deps`. `python -I` exercises the installed
package. The local helper is copied as `uniffi-bindgen.exe` into the build-only
PATH: this avoids maturin 1.16.0's workspace-bin feature selection of the root
CLI package. It does not patch a generator or alter its generated source.

Next the script generates Java directly from the **installed wheel's DLL**,
compiles with `javac --release 22`, and packages the API and generated sources
as jars. It copies that exact DLL for JVM use and runs compiled Java and
Kotlin regression callers. No Maven/Gradle/JNA/JNI/PyO3 dependency or CLI
wrapper is involved. Java 22 FFM needs explicit native access:

```powershell
java --enable-native-access=ALL-UNNAMED `
  '-Djava.library.path=C:\artifact\jvm\native' `
  -cp 'C:\artifact\jvm\lox-archive-0.1.0.jar;C:\application\classes' MyApplication
```

Distribute the API jar and its matching `jvm\native\lox_archive.dll` together;
the jar does not auto-download/extract native libraries. The Python wheel
bundles the DLL. Kotlin depends on the same Java jar; no Kotlin runtime is
required by the API jar itself (only the Kotlin smoke-test jar includes one).
Package names are experimental local names, not a registry ownership claim.
No PyPI/Maven/crates.io publication is performed by these commands.

`tests\bindings\archive_smoke.py`, `ArchiveSmoke.java`, and `ArchiveSmoke.kt`
check the same opaque 329-byte fixture, exact cross-language compressed
output, empty/large buffers, raw/ZIP decoding, permissive-header behavior
and typed header/magic/back-reference/ZIP/source-context failures. Synthetic
fixtures contain no device data or credentials. `commands.json`, per-step
logs and `ARTIFACTS.json` retain commands, versions, source index tree and
SHA256s, including generated sources and the installed DLL/module. Build
target/temp/cache trees are excluded from artifact inventories.

### Required local Java generator correction

**Unpatched uniffi-bindgen-java 0.5.2 output leaks native error buffers.**
The generated error descriptor aliases thread-local reusable call status.
`RustBuffer.free` makes a nested native call that zeroes that same status
before native free reads the descriptor. Exception fields can look correct
while the allocation remains live.

The builder therefore applies **lox-archive-java-0.5.2-rustbuffer-free-owned-status-2**:
copy the descriptor into a confined arena **before** the nested call, then
free the unchanged Rust allocation using that independent descriptor. No
Rust/core code, FFI signature, ABI checksum or error representation changes.
The patch is a maintained, explicit local correction, not an upstream fix.
Its MPL-2.0 specification lives in `crates\lox-bindings\patches`. The patcher
requires the exact generator version, complete `RustBuffer.java` and
`UniffiHelpers.java` input SHA256s, exact reviewed fragment counts, and both
complete corrected output SHA256s. The helper also replaces permanent global
call-status storage with the owned/automatic lifetimes described above.
Unknown/changed/already-patched output fails closed;
upgrades require a new review, never automatic hash updates or silent fallback.
Guard regressions run before the actual correction. Both JVM jars include
the specification/application receipt; the sources jar includes the
corrected helpers. Distributors must use this corrected build, not raw 0.5.2
output. All other generated source remains untouched.

`ArchiveAllocationSmoke.java` and `.kt` perform **64 actual public native
domain-error calls per language**. A Windows process-heap instrument first
finds one real 65,536-byte serialized error allocation and observes its
release using an independent descriptor, calibrating its byte-prefix matcher.
It then counts live matching allocations after the public calls: corrected
output must retain **zero**. Frozen uncorrected output retains **64** for
each language and fails the same regression. A separate aliased 1 MiB
descriptor test must also release its allocation; the negative control frees
its saved descriptor only if still live, avoiding a double free.

The test-only C helper keeps `HeapLock`/`HeapWalk`/`HeapUnlock` entirely native
(no JVM transitions while holding the process heap), checks enumeration
errors and uses `HeapValidate` on exact pointers. Large ZIP member names force
the measured error buffers outside the small-allocation heap path. This is
allocation accounting on the verified Windows AMD64 host, **not an RSS
threshold**, a mocked allocator or a portable-memory qualification. Probe
classes/DLL and Kotlin support jars are test artifacts, not part of the API
jar/native runtime pair. These checks prove this archive error-buffer release
path only: runtime close/status-allocation evidence comes from the separate
clock checks, and **no subscription or native-device qualification** follows.

Before any commit/publication, stage the intended source and run all four
repository gates on that exact tree:

```powershell
cargo fmt --all --check
cargo clippy --workspace --locked --offline -- -D warnings
cargo build --workspace --release --locked --offline
cargo test --workspace --locked --offline
```

The wheel and jars carry project notices, the unchanged project license
texts and MPL-2.0 notices for generated helper code. See
`crates\lox-bindings\THIRD_PARTY_NOTICES`; the generated Java sources jar
and Python wheel contain the generated source. Distributors must also
provide matching Corresponding Source and dependency notices. Local
artifact success is neither a public SDK release nor native/firmware
qualification, and it does not establish full language-head parity.
