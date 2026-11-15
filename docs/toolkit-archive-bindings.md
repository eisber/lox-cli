# Experimental generated native archive, clock and control bindings

This is a narrow **archive, HTTP clock and pure control selection** binding slice, not a general
Loxone SDK or CLI-parity release. `crates/lox-bindings` is one thin UniFFI
facade over `lox_core::configuration::archive`, `control` and the optional production
`runtime::ClockDriver`; all codec, selection, transport, retry and validation behavior
remains in the Rust core. Python and the Java-first JVM API load the same native library.
Kotlin uses the Java API, not a separately generated Kotlin implementation.
No head invokes the CLI or runs another protocol stack.

## Actually exposed capabilities

| Capability | Rust core | CLI | Generated Python | Generated Java / Kotlin |
|---|---|---|---|---|
| Compress opaque bytes to LoxCC | `compress_loxcc` | Existing archive compression | `compress_loxcc(bytes) -> bytes` | `LoxArchive.compressLoxcc(byte[]) -> byte[]` |
| Decode raw LoxCC | `decompress_loxcc` | Existing archive extraction | `decompress_loxcc(bytes) -> bytes` | `LoxArchive.decompressLoxcc(byte[]) -> byte[]` |
| Decode raw LoxCC or select ZIP member | `extract_and_decompress` | Existing backup extraction | `extract_and_decompress(bytes) -> bytes` | `LoxArchive.extractAndDecompress(byte[]) -> byte[]` |
| Filesystem/config context selection | No implicit operations | Existing CLI behavior | Unsupported; caller owns I/O | Unsupported; caller owns I/O |
| XML parsing/editing, validation, deployment, sidecars/trust | Optional read-only `configuration::rooms` and `configuration::summary` (`configuration-xml`); remaining capabilities not migrated by this slice | Existing CLI behavior unchanged | Unsupported | Unsupported |
| Monthly statistics XML decoding | Optional `statistics::parse_series` (`statistics-xml`); timestamps remain opaque | `stats get` shares decoding; acquisition, selection, merging and formatting stay in CLI | Unsupported | Unsupported |
| Ordered HTTP Basic clock observation and validation | Optional `ClockClient` / `ClockDriver`; explicit validation | `live time` uses shared runtime with explicit legacy compatibility | Strict `ClockRuntime` and `AsyncClockClient` | Strict `ClockRuntime` and Java `ClockClient`; same API in Kotlin |
| Current clock cancellation, shutdown and admission | Production sticky cancellation and owned worker close | Per-command owned driver; no new cancellation command | Native handles; bounded asyncio adapter | Native handles; cancellable `CompletableFuture`, explicit `close` |
| In-memory control name/room resolution | Typed `control` query/catalog/errors; default features | Existing resolver uses core; acquisition/formatting stay in CLI | `prepare_control_query`, `resolve_control_query`; typed records/errors | `LoxArchive.prepareControlQuery`, `resolveControlQuery`; same typed Java API in Kotlin |
| Identifier detection / command-path validation | Distinct `is_uuid` / `validate_uuid` contracts | Existing CLI contracts unchanged | `is_uuid`, `validate_uuid`; typed `IdentifierError` | `LoxArchive.isUuid`, `validateUuid`; checked `IdentifierException` |
| Token auth/crypto, control transport, WebSocket events | Not extracted | Existing binary-private behavior | Unsupported | Unsupported |
| State subscriptions and event-stream backpressure | No shared event stream exists | No new behavior | Unsupported | Unsupported |

Kotlin sees `byte[]` as `ByteArray`. The generated Python surface returns
actual `bytes`, not lists, strings, JSON or a subprocess result. The bridge
uses owned `Vec<u8>` so both heads copy byte buffers. It deliberately does not
export borrowed buffers or zero-copy/lifetime obligations in this slice.

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
