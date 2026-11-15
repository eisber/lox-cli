# Experimental generated native archive bindings

This is the first **archive-only** shared-core binding slice, not a general
Loxone SDK or CLI-parity release. `crates/lox-bindings` is one thin UniFFI
facade over `lox_core::configuration::archive`; all codec behavior remains in
the Rust core. Python and the Java-first JVM API load the same native library.
Kotlin uses the Java API, not a separately generated Kotlin implementation.
No head invokes the CLI or runs another protocol stack.

## Actually exposed capabilities

| Capability | Rust core | CLI | Generated Python | Generated Java / Kotlin |
|---|---|---|---|---|
| Compress opaque bytes to LoxCC | `compress_loxcc` | Existing archive compression | `compress_loxcc(bytes) -> bytes` | `LoxArchive.compressLoxcc(byte[]) -> byte[]` |
| Decode raw LoxCC | `decompress_loxcc` | Existing archive extraction | `decompress_loxcc(bytes) -> bytes` | `LoxArchive.decompressLoxcc(byte[]) -> byte[]` |
| Decode raw LoxCC or select ZIP member | `extract_and_decompress` | Existing backup extraction | `extract_and_decompress(bytes) -> bytes` | `LoxArchive.extractAndDecompress(byte[]) -> byte[]` |
| Filesystem/config context selection | No implicit operations | Existing CLI behavior | Unsupported; caller owns I/O | Unsupported; caller owns I/O |
| XML parsing/editing, validation, deployment, sidecars/trust | Not migrated by this slice | Existing CLI behavior unchanged | Unsupported | Unsupported |
| HTTP clock observation and validation | Optional `ClockClient` / `ClockDriver` | `live time` uses shared runtime with explicit legacy compatibility | Unsupported | Unsupported |
| Clock cancellation and runtime close | Caller-owned async client or explicitly owned blocking driver | Per-command owned driver; no new cancellation command | Unsupported/unproved | Unsupported/unproved |
| In-memory control name/room resolution | Typed `control` query/catalog/errors; default features | Existing resolver producer uses core; acquisition/formatting stay in CLI | Unsupported | Unsupported |
| Token auth/crypto, control transport, WebSocket events | Not extracted | Existing binary-private behavior | Unsupported | Unsupported |
| State subscriptions and event-stream backpressure | Not extracted | No new behavior | Unsupported/unproved | Unsupported/unproved |

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

These are stateless synchronous calls returning owned values. They prove
**nothing** about runtime shutdown, cancellation, subscription disposal,
callbacks, resource bounds or event-loop integration. Those require the next
real mocked-runtime slice; language-only tests and generator internals are
not lifecycle proof. Task `lox-vuc6` remains open for that work.

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
Windows paths for its own target/temp (MSVC long-path avoidance), stack size,
two build jobs and `LOX_TELEMETRY=0`. It changes no global PATH, service or VM.
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

The builder therefore applies **lox-archive-java-0.5.2-rustbuffer-free-1**:
copy the descriptor into a confined arena **before** the nested call, then
free the unchanged Rust allocation using that independent descriptor. No
Rust/core code, FFI signature, ABI checksum or error representation changes.
The patch is a maintained, explicit local correction, not an upstream fix.
Its MPL-2.0 specification lives in `crates\lox-bindings\patches`. The patcher
requires the exact generator version, complete `RustBuffer.java` and
`UniffiHelpers.java` input SHA256s, one exact method match, and the complete
corrected output SHA256. Unknown/changed/already-patched output fails closed;
upgrades require a new review, never automatic hash updates or silent fallback.
Guard regressions run before the actual correction. Both JVM jars include
the specification/application receipt; the sources jar includes the
corrected helper. Distributors must use this corrected build, not raw 0.5.2
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
path only: **no runtime close/cancel/subscription/lifecycle credit** follows.

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
