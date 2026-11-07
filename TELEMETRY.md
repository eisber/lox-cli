# Telemetry

`lox` includes anonymous usage telemetry to help us understand which
features are used and where to focus development effort.

## Default: ON

Telemetry is **enabled by default** to help improve the tool. You can easily opt out at any time.

## What is collected

Only anonymous, aggregated data:

| Data                | Example                      | Purpose                       |
|---------------------|------------------------------|-------------------------------|
| CLI version         | `0.14.0`                     | Track adoption                |
| OS + architecture   | `linux / x86_64`             | Platform prioritisation       |
| Command name        | `config add`, `blind`        | Feature usage                 |
| Command outcome     | Success / error category    | Find unreliable workflows     |
| Event source        | `usage`, `eval`, `diagnostic` | Separate real and test usage |
| Block types created | `And`, `LightController2`    | Template / wizard priorities  |
| Config stats        | 12 rooms, 87 blocks, 34 wires | Complexity profiles          |
| Check counts        | 10 passed, 2 warnings       | Improve validation rules      |
| Scan result         | clean / 2 issues             | Privacy tooling effectiveness |
| Session ID          | Random UUID, rotated daily   | Group events within a day     |
| Timestamp           | Day granularity only         | Usage trends                  |

## What is NEVER collected

- File paths, file contents, config XML
- IP addresses (PostHog is configured not to store them)
- Usernames, hostnames, Miniserver serial numbers
- Block titles, room names, UUIDs
- Command arguments or flag values
- Any PII from `config scan` results

## How to opt in / opt out

### Enable
```bash
lox telemetry enable
```

### Disable
```bash
lox telemetry disable
```

### Environment variable (overrides config)
```bash
export LOX_TELEMETRY=0   # disable
export LOX_TELEMETRY=1   # enable
```

`DO_NOT_TRACK` also disables telemetry when `LOX_TELEMETRY` is not set.
Dry runs and `lox telemetry disable` never send events.

### Tests, evals, and delivery verification

Automated tests and CI evals disable telemetry with `LOX_TELEMETRY=0`;
they are not production usage. To explicitly verify delivery or collect an eval
run, enable telemetry and label it separately:

```bash
LOX_TELEMETRY=1 LOX_TELEMETRY_SOURCE=diagnostic lox config stats example.Loxone
LOX_TELEMETRY=1 LOX_TELEMETRY_SOURCE=eval lox config check example.Loxone
```

`LOX_TELEMETRY_SOURCE` accepts `usage` (default), `test`, `eval`, or
`diagnostic`. An invalid source disables telemetry with a warning.
Usage dashboards should filter to `source = usage`; verification/eval charts
can include the other sources. Older events have no source property.

The maintainer [lox CLI Usage dashboard](https://eu.posthog.com/project/112518/dashboard/1092443)
shows command trends, popular commands, outcomes, platforms/versions, and a
separate diagnostic delivery view. Access requires membership in the PostHog project.

### Check current status
```bash
lox telemetry status
```

### Config file
In `~/.lox/config.yaml`:
```yaml
telemetry: true   # or false
```

## Where data is sent

Events are sent to [PostHog](https://posthog.com) (EU cloud instance) via
their HTTP capture API. PostHog is an open-source product analytics platform.

- Endpoint: `https://eu.i.posthog.com/capture/`
- Transport: HTTPS POST with JSON body and verified public certificate roots
- Timeout: 2 seconds per request; pending requests run concurrently and finish
  before the process exits, including command failures

## PostHog privacy

PostHog's privacy policy: <https://posthog.com/privacy>

We have configured PostHog to:
- **Not** store IP addresses
- **Not** use cookies or browser fingerprinting (this is a CLI tool)
- Retain data for 90 days only

## Technical details

- Session IDs are random UUIDs, rotated daily, stored in
  `~/.lox/.telemetry_session`. They are **not** tied to any user identity.
- Telemetry sends the full canonical command path and a success/error category,
  never argument values or raw error messages. Help/version exits are not tracked.
- `config add` tracks newly created block types (not idempotent no-ops);
  `config stats`, `config check`, and `config scan` send aggregate counts.
- Telemetry runs in background threads and waits for pending deliveries at exit.
  A short command can therefore take up to two extra seconds when offline.
- Network errors, timeouts, and HTTP rejections produce a warning on stderr;
  they never change the command's exit code or JSON output.
- Events explicitly disable GeoIP enrichment and person-profile processing.
- The PostHog API key is a *public* project key (safe to embed in source).

## Source code

The telemetry implementation is in
[`src/telemetry.rs`](src/telemetry.rs) — about 250 lines of Rust. We
encourage you to review it.

## Questions?

Open an issue at <https://github.com/eisber/lox-cli/issues> if you have any
concerns about data collection.
