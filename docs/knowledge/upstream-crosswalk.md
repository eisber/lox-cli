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
