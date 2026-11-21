# Upstream evidence and maintenance

Reviewed **2026-11-14**. These are original summaries and links to seven upstreams,
not vendored code. **Upstream implementation evidence is not firmware/native
qualification.** Source, test and documentation inspection did not execute upstream
integrations or contact devices. The [initial local crosswalk](upstream-crosswalk.md)
classifies six findings against an explicit local commit; it does not claim that a
discrepancy has been fixed or upstream code adopted. Later own-code archive extraction
is documented in the [Rust core](../../crates/lox-core/README.md).

The [registry](upstreams.json) is authoritative for full immutable `reviewed_sha`
pins, separately recorded `latest_seen_sha`, branch refs, check timestamps, license
digests and pinned source/test/doc/license/compare links. Initial observations match
all seven reviewed pins; an observation is never approval of a newer commit.

## Catalog

| Repository / reviewed pin | Scope and original attribution | Pinned evidence | License observation |
|---|---|---|---|
| [mr-manuel/Loxone_api_documentation](https://github.com/mr-manuel/Loxone_api_documentation) / [`fab40ec`](https://github.com/mr-manuel/Loxone_api_documentation/commit/fab40ec5c8d25e2dd9a92967b963b32d52e026b4) | Unofficial API reference, not a library. Manuel / mr-manuel. | [Source](https://github.com/mr-manuel/Loxone_api_documentation/blob/fab40ec5c8d25e2dd9a92967b963b32d52e026b4/miniserver-api.yaml#L19-L50), [docs](https://github.com/mr-manuel/Loxone_api_documentation/blob/fab40ec5c8d25e2dd9a92967b963b32d52e026b4/README.md#L1-L7); no tests found in pinned tree. | No license file/SPDX identified; **not public domain**. |
| [Smarteon/loxone-client-kotlin](https://github.com/Smarteon/loxone-client-kotlin) / [`22fc2d9`](https://github.com/Smarteon/loxone-client-kotlin/commit/22fc2d9c1ec58065295fde384fb2cab79fa71d31) | Multiplatform coroutine/Ktor runtime. Smarteon Systems. | [Source](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonMain/kotlin/ktor/KtorWebsocketLoxoneClient.kt#L166-L204), [tests](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/src/commonTest/kotlin/event/ValueEventTest.kt#L11-L45), [docs](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/README.md#L1-L34). | [BSD-3-Clause](https://github.com/Smarteon/loxone-client-kotlin/blob/22fc2d9c1ec58065295fde384fb2cab79fa71d31/LICENSE). |
| [smartmacherei/Loxone-Integration](https://github.com/smartmacherei/Loxone-Integration) / [`85bf863`](https://github.com/smartmacherei/Loxone-Integration/commit/85bf863edb8032c5a9e93f22fd2061cc2ca2c866) | Home Assistant integration, embedded API, topology/program handling. smartmacherei and PyLoxone authors. | [Source](https://github.com/smartmacherei/Loxone-Integration/blob/85bf863edb8032c5a9e93f22fd2061cc2ca2c866/custom_components/loxone/udp_program.py#L34-L40), [tests](https://github.com/smartmacherei/Loxone-Integration/blob/85bf863edb8032c5a9e93f22fd2061cc2ca2c866/tests/test_gateway_client.py#L46-L59), [NOTICE](https://github.com/smartmacherei/Loxone-Integration/blob/85bf863edb8032c5a9e93f22fd2061cc2ca2c866/NOTICE#L1-L16). | [Apache-2.0](https://github.com/smartmacherei/Loxone-Integration/blob/85bf863edb8032c5a9e93f22fd2061cc2ca2c866/LICENSE); retain declared attribution. |
| [JoDehli/PyLoxone](https://github.com/JoDehli/PyLoxone) / [`4a368b2`](https://github.com/JoDehli/PyLoxone/commit/4a368b2937797308cb247190bced2f5e122e32f1) | Home Assistant integration consuming `pyloxone-api==0.4.1`, not that standalone library. PyLoxone authors; JoDehli codeowner. | [Source](https://github.com/JoDehli/PyLoxone/blob/4a368b2937797308cb247190bced2f5e122e32f1/custom_components/loxone/coordinator.py#L13-L24), [tests](https://github.com/JoDehli/PyLoxone/blob/4a368b2937797308cb247190bced2f5e122e32f1/tests/test_unload_cleanup.py#L73-L106), [docs](https://github.com/JoDehli/PyLoxone/blob/4a368b2937797308cb247190bced2f5e122e32f1/README.md#L13-L16). | [Apache-2.0](https://github.com/JoDehli/PyLoxone/blob/4a368b2937797308cb247190bced2f5e122e32f1/LICENSE). |
| [JoDehli/pyloxone-api](https://github.com/JoDehli/pyloxone-api) / [`acee572`](https://github.com/JoDehli/pyloxone-api/commit/acee5727914dacbeb3de179438b3172f8659dabc) | Standalone async Python runtime; Python >=3.13. Joachim Dehli. | [Source](https://github.com/JoDehli/pyloxone-api/blob/acee5727914dacbeb3de179438b3172f8659dabc/src/pyloxone_api/message.py#L272-L308), [tests](https://github.com/JoDehli/pyloxone-api/blob/acee5727914dacbeb3de179438b3172f8659dabc/tests/test_message_parsing.py#L59-L74), [docs](https://github.com/JoDehli/pyloxone-api/blob/acee5727914dacbeb3de179438b3172f8659dabc/README.md#L9-L15). | [AGPL-3.0-only](https://github.com/JoDehli/pyloxone-api/blob/acee5727914dacbeb3de179438b3172f8659dabc/LICENSE). |
| [JoDehli/rs-loxone](https://github.com/JoDehli/rs-loxone) / [`2cdafc4`](https://github.com/JoDehli/rs-loxone/commit/2cdafc4f151f68f8814cd784c3a5b4a83cd31a6a) | Async Rust runtime/optional CLI, Tokio supervisor; MSRV 1.95. JoDehli. No Python/JVM FFI identified. | [Source](https://github.com/JoDehli/rs-loxone/blob/2cdafc4f151f68f8814cd784c3a5b4a83cd31a6a/src/lib.rs#L1-L17), [tests](https://github.com/JoDehli/rs-loxone/blob/2cdafc4f151f68f8814cd784c3a5b4a83cd31a6a/tests/auth_hash_vectors.rs#L13-L63), [docs](https://github.com/JoDehli/rs-loxone/blob/2cdafc4f151f68f8814cd784c3a5b4a83cd31a6a/README.md#L1-L12). | [AGPL-3.0-only](https://github.com/JoDehli/rs-loxone/blob/2cdafc4f151f68f8814cd784c3a5b4a83cd31a6a/LICENSE); candidate facts, not reuse approval. |
| [discostu105/lox](https://github.com/discostu105/lox) / [`7c6c952`](https://github.com/discostu105/lox/commit/7c6c952eab17d23f2906f62d5ccb374749dcee44) | Rust CLI plus `loxone-client`/`loxone-config` crates; MSRV 1.91; blocking HTTP/async WebSocket. Christoph Neumuller / discostu105. | [Source](https://github.com/discostu105/lox/blob/7c6c952eab17d23f2906f62d5ccb374749dcee44/crates/loxone-client/src/lib.rs#L3-L19), [tests](https://github.com/discostu105/lox/blob/7c6c952eab17d23f2906f62d5ccb374749dcee44/tests/stream_reconnect.rs#L227-L251), [docs](https://github.com/discostu105/lox/blob/7c6c952eab17d23f2906f62d5ccb374749dcee44/crates/loxone-client/README.md#L1-L19). | [Apache-2.0](https://github.com/discostu105/lox/blob/7c6c952eab17d23f2906f62d5ccb374749dcee44/LICENSE); candidate facts, not relicensing approval. |

License digests in the registry identify the reviewed raw license bytes. This index
does not replace any upstream license, NOTICE or attribution obligation.

## Contract and ancestry leads

The registry's `comparison_leads` contains the exact pinned links for each row.
The local classifications below refer to the crosswalk's pinned `3894515` baseline,
not an automatically refreshed assertion about every later toolkit version.
**No discrepancy is marked fixed.**

| Topic | Upstream evidence | Local Rust mapping |
|---|---|---|
| Authentication | rs-loxone selects `getjwt` at firmware >=10.2; Kotlin marks token acquisition encrypted; discostu105/lox sends `gettoken` after key exchange. | Primitives covered; token command/version discrepancy open |
| Events | UUID + little-endian f64 value records; pyloxone-api has empty daytimer/weather handlers, while discostu105/lox includes parsers. | Unresolved |
| Lifecycle | rs-loxone reconnect supervisor, 30-second keepalive and 90-second silence settings. | Public runtime/subscription/disposal capability missing at the comparison baseline |
| Archives | discostu105/lox describes reader size/CRC without verification; smartmacherei validates before editing. Neither substitutes for local characterization. | CLI byte/CRC-output behavior covered; reader/deployment policies remain distinct |
| Statistics | Kotlin separates V1 XML/local 2009 epoch from V2 binary/Unix UTC. | Unresolved |

| Relationship | Evidence and limits |
|---|---|
| smartmacherei / PyLoxone | Pinned [NOTICE](https://github.com/smartmacherei/Loxone-Integration/blob/85bf863edb8032c5a9e93f22fd2061cc2ca2c866/NOTICE#L1-L10) and identical nonempty `lights/dimmer.py` blob (`96a0eb3e000d09afc696bde799bbe17ce8052bb2`). This demonstrates attribution/source overlap, not shared commit ancestry. |
| eisber/lox-cli / discostu105/lox | Prior research reports `fork=false`, absent parent/source, and fully paginated pinned histories (499 / 929 commits) with no shared SHA reported. Shared current blobs were `.cargo/config.toml`, `docs/Gemfile`, `docs/guides/index.md`; no shared Rust-source blob identified in the reviewed selection. **Raw history API receipts were not persisted:** this summary does not prove absence of copied/adapted code. Exact compared pins are in the registry. |
| Other relationships | Unresolved. Similar protocols, common ownership and GitHub fork metadata do not establish independent ancestry. The three JoDehli scopes above must remain distinct. |

## Cheap refresh, deliberate adoption

Run from the repository root with Python's standard library only:

```text
python scripts/check-upstreams.py
python scripts/check-upstreams.py --timeout 5
python -m unittest discover -s scripts -p test_check_upstreams.py
```

Default execution prints JSON and **does not modify files**, update latest observations
in the registry, or advance reviewed pins. Redirect stdout to an explicitly chosen
report file if needed. `GH_TOKEN`, then `GITHUB_TOKEN`, may be supplied in the process
environment; tokens, exception details, headers and API bodies are not logged.

The checker allows only validated `owner/name`, `refs/heads/...` branch refs and
full 40-hex commit fields. It performs one GitHub API ref GET per entry, without
redirects, retries, arbitrary catalog URL fetching, shell commands or code imports.
Requests use a 10-second timeout (configurable 1..30), a 64 KiB response limit and
at most 32 entries. Timeout bounds socket operations, not a complete remote-service
availability guarantee; the weekly job also has a five-minute total limit.

Output records new `latest_seen_sha`, a pinned compare URL and `needs_review` against
the **reviewed**, not previously observed, pin. Changed refs are successful metadata
observations (`needs_review`, exit 0); network/API/invalid-response errors are explicit
(`error`, exit 1), with unknown observations left null. A ref returning to its reviewed
pin is unchanged; no fast-forward/ancestry claim is made by this metadata-only check.

[Weekly workflow](../../.github/workflows/upstreams.yml): Mondays 06:23 UTC, or
manual dispatch. Read-only repository permissions, checkout without persisted
credentials, JSON artifact (14 days) and job summary only. No commit, push, PR,
issue, device contact, upstream execution or source download; tracking stays in
Beads. It runs only repository-owned checker code, not integrations from upstreams.
The schedule becomes active only after integration/publication on the default branch.

Before adopting any change, inspect relevant source/tests/docs and license/provenance
diffs, validate the intended local contract, and explicitly review a new immutable pin.
Weekly metadata polling plus before-adoption review is the maintenance policy:
neither a full seven-repository reread on every continuation nor automatic code import.
