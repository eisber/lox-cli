# Offline firmware update metadata

## Evidence and actual upstream chain

This is implementation evidence, **not an authoritative firmware schema**.
The [focused source manifest](firmware-metadata-source.json) pins full commit and
blob identities, Apache-2.0 attribution and license. It links the existing
[catalog](upstreams.json) ancestry limitations without advancing any catalog
reviewed/latest-seen pin.

At `discostu105/lox` commit
`70f33122df14e46bafbde3e124b53d6b1902386b`,
[`main.rs:974`](https://github.com/discostu105/lox/blob/70f33122df14e46bafbde3e124b53d6b1902386b/src/main.rs#L974)
dispatches to
[`system.rs:811-1136`](https://github.com/discostu105/lox/blob/70f33122df14e46bafbde3e124b53d6b1902386b/src/commands/system.rs#L811-L1136).
`cmd_update(Check)` obtains current version, resolves serial from configuration,
project or MAC, obtains response text, calls `UpdateCheck::parse`, then
`json` or `text`/`notes`. Missing serial/network failure yields current-only
output rather than metadata. No such acquisition or ambient project lookup is
part of the new core API.

Recovered behavior:

- The observed root is `Miniserversoftware`. Firmware release metadata comes
  from `Release` attributes `Version` and optional `Path`; `Path` identifies a
  **Loxone Config installer**, not a firmware binary. Beta/test offerings are
  not selected. Root/declaration version values are not release versions.
- `RemoteConnect` independently supplies `MinVersion` and `Deadline`.
  The caller ignores `AutoUpdateAvailable`, `Why`, certificates, signatures,
  checksums and sizes. These fields do **not** establish install eligibility.
- `element_attr` finds the first literal `<Release ` / `<RemoteConnect ` anywhere
  in the string, not an actual parent-scoped XML element. The actual
  [`util::xml_attr`](https://github.com/discostu105/lox/blob/70f33122df14e46bafbde3e124b53d6b1902386b/src/util.rs#L6-L39)
  scans double-quoted substrings, does not entity-decode, and is not a validator.
  Upstream tests put the intended release before a nested web-interface release;
  that ordering is not proof of structural selection.
- `version_key` splits on dots and parses `u32`, silently substituting zero on
  invalid/overflow components. Numeric vectors compare lexicographically,
  without padding: `26.10.0` is greater than `26.10`. Program notes compare only
  the first two components, and only against a newer offered release.
- The optional program version producer loads the ambient project's base file,
  reads at most its first 4096 bytes, scans `ConfigVersion`, requires exactly eight
  ASCII digits and converts four two-digit groups into dotted numeric components.
  Missing/unreadable/invalid project data silently omits this note input. Our API
  instead requires the caller to supply an already validated optional version.
- Remote warnings require current below minimum **and today on/before deadline**;
  after the deadline the warning disappears. Absence of that warning therefore
  does not establish usability. The JSON observation contains minimum/deadline
  independent of warnings.
- Missing release becomes `available=current` in upstream JSON, while text says
  the answer could not be read. A release older than current is not an update.
  Both Remote Connect attributes must exist for its upstream tuple.
- `Install` is separate, not gated by `UpdateCheck`: it requires `--yes`, supports
  dry-run/no-wait, and otherwise polls for a *different* version and PLC Running.
  Old firmware/offline/booting do not complete the wait. Completion precedes the
  timeout check; timeout is strictly `elapsed > timeout`, with a stuck hint after
  ten minutes. Those static tests/house observations are not our device proof.

Local baseline `4ec9df1e79918bb20b661b7b760243a40d42e371` has no update
command, XML update metadata operation or complete producer to migrate.
Existing generic HTTP reads, configuration version strings and authentication
version policies are different operations. We add no invented CLI command.

## New original Rust contract

```toml
lox-core = { path = "crates/lox-core", features = ["firmware-metadata"] }
```

```rust,ignore
use lox_core::firmware_metadata::{
    CalendarDate, FirmwareMetadataError, FirmwareVersion, UpdateMetadata,
};

fn inspect(xml: &str) -> Result<(), FirmwareMetadataError> {
    let metadata = UpdateMetadata::parse(xml)?;
    let current: FirmwareVersion = "26.9.4.2".parse()?;
    let as_of: CalendarDate = "2028-02-28".parse()?;
    let assessment = metadata.assess(&current, None, as_of);
    // The caller owns storage, presentation and any subsequent decision.
    let _ = assessment;
    Ok(())
}
```

This independently usable, headless **Rust-only** capability uses the existing
optional quick-xml dependency, adding no default dependencies. No CLI, Python,
JVM, transport, updater, context, clock, files, printing, process exits or JSON
universal API is added. Existing CLI syntax and producers are untouched.

Deliberate original API policies, not upstream fixes or firmware assertions:

- Parse one UTF-8 XML document rooted at exact unprefixed `Miniserversoftware`.
  Select only its direct `Release`/`RemoteConnect` children. Ignore unknown
  elements (including nested release decoys) but check attributes and tag
  structure throughout. Reject extra/missing/unclosed roots, mismatched ends,
  outside-root nonwhitespace text/CDATA and DTDs. This is not a general XML/XSD
  validator; namespace semantics and external entities are unsupported (names are
  compared literally and default namespace declarations are not interpreted).
- Reject duplicate known elements and malformed/duplicate attributes, including
  missing XML whitespace between attributes and raw `<` inside quoted values.
  Single/double quotes, XML whitespace (space, tab, CR, LF) and entity-escaped
  values remain supported, including on ignored elements.
  A present release requires `Version`; a present Remote Connect element requires
  both attributes. Missing release remains explicitly `ReleaseStatus::Unavailable`,
  never a fabricated current-version offer. `Path` is optional, entity-decoded,
  otherwise opaque (including empty); never fetched or URL/trust validated.
- Version components must be nonempty ASCII decimal `u32`, with one or more
  components. Leading zeroes normalize. Invalid input returns typed
  `InvalidVersion`, never a zero fallback. Comparison deliberately retains the
  recovered numeric-vector ordering, not semver/prerelease or padded ordering.
- Dates must be valid Gregorian `YYYY-MM-DD` in years 0001..9999.
  The caller supplies `as_of`, without timezone conversion or ambient clock.
  Invalid dates return typed `InvalidDate`, unlike upstream string comparison.
- Owned typed metadata and assessment distinguish newer/equal/older/unavailable.
  Optional program comparison reports the recovered major/minor predicate only
  for a newer offered release, **not** verified Config compatibility.
  Remote assessment always retains below-minimum and before/on/past status;
  `warning_due` exactly exposes the upstream predicate for valid dates, including
  its false-after-deadline behavior. It never grants install permission.
- Structured XML/attribute/schema errors return no partial metadata. Inputs are
  caller-owned, with no added size/depth cap; callers must bound untrusted input.

`cargo test -p lox-core --locked --offline --features firmware-metadata
--test firmware_metadata` exercises the public operation with original synthetic
documents: declaration/root/nested decoys, numeric ordering, exact date boundary,
missing/older/equal offers, program comparison and typed rejection cases.
No upstream fixtures/vectors were copied and no upstream code was executed.

Unresolved: authoritative schema/version arity, hardware-specific eligibility,
signature/certificate verification, firmware image selection, post-deadline
service behavior and actual install behavior. These must not be inferred from
this observation API. No native qualification credit is claimed.
