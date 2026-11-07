# Telemetry

`lox` includes anonymous usage telemetry to help us understand which
features are used and where to focus development effort. Explicit issue
feedback is a separate event type described below.

## Default: ON

Telemetry is **enabled by default** to help improve the tool. You can easily opt out at any time.

## What is collected

Automatic usage events contain only anonymous, aggregated data:

| Data                | Example                      | Purpose                       |
|---------------------|------------------------------|-------------------------------|
| CLI version         | `0.14.0`                     | Track adoption                |
| OS + architecture   | `linux / x86_64`             | Platform prioritisation       |
| Country             | Country code and name       | Regional usage trends         |
| Command name        | `config add`, `sim run`      | Feature usage                 |
| Command outcome     | Success / error category    | Find unreliable workflows     |
| Event source        | `usage`, `eval`, `diagnostic` | Separate real and test usage |
| Block types created | `And`, `LightController2`    | Template / wizard priorities  |
| Config stats        | 12 rooms, 87 blocks, 34 wires | Complexity profiles          |
| Check counts        | 10 passed, 2 warnings       | Improve validation rules      |
| Scan result         | clean / 2 issues             | Privacy tooling effectiveness |
| Session ID          | Random UUID, rotated daily   | Group events within a day     |
| Timestamp           | Day granularity only         | Usage trends                  |

## What automatic usage events NEVER collect

- File paths, file contents, config XML
- IP addresses (PostHog is configured not to store them)
- City, region, postal code, timezone, latitude, and longitude
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

`LOX_TELEMETRY` takes precedence over other settings. If it is unset, the
presence of `DO_NOT_TRACK` disables telemetry; otherwise the config preference
applies, defaulting to enabled.
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
shows command trends, popular commands, outcomes, platforms/versions, countries, and a
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

## Explicit issue feedback

`lox report --title "..." --body "..." --json` first attempts to
create a public issue in `eisber/lox-cli` using `gh`. When that fails and
telemetry is enabled, it sends a PostHog `issue_report` event containing:

- The **explicitly supplied title and body**, which are not anonymous aggregates
- A random report ID and the standard anonymous version/platform/session fields

Only send sanitized descriptions, reproduction steps, and expected/actual
behavior. Never include credentials, private configuration, personal data, or
unsanitized logs. There are no automatic log/config attachments. Use
`--body-file FILE` or `--body-file -` to provide the body from a file or stdin.
`--dry-run` previews the report without submission.

Unlike usage telemetry, feedback delivery is synchronous and checks the HTTP
response. A receipt identifies the channel and report ID; unsuccessful delivery
returns a nonzero error. Opting out of telemetry disables this fallback, but
does not prevent explicitly requested GitHub issue creation.

Argument errors, runtime errors, and Rust panics expose reporting guidance, but
never automatically submit an issue report or its raw error/crash contents.
Agents should fix input/setup errors first, then report reproducible bugs with
sanitized details.

## PostHog privacy

PostHog's privacy policy: <https://posthog.com/privacy>

We have configured PostHog to:
- **Not** store IP addresses
- **Not** use cookies or browser fingerprinting (this is a CLI tool)
- Retain data for 90 days only

## Technical details

- Session IDs are random UUIDs, rotated daily, stored in
  `~/.lox/.telemetry_session`. They are **not** tied to any user identity.
- Usage telemetry sends the full canonical command path and a success/error category,
  never argument values or raw error messages. Help/version exits are not tracked.
- `config add` tracks newly created block types (not idempotent no-ops);
  `config stats`, `config check`, and `config scan` send aggregate counts.
- Telemetry runs in background threads and waits for pending deliveries at exit.
  A short command can therefore take up to two extra seconds when offline.
- Network errors, timeouts, and HTTP rejections produce a warning on stderr;
  they never change the command's exit code or JSON output.
- Events opt into country-only enrichment with `geoip_country_only: true`.
  The scoped PostHog transformation derives country code/name from the request IP,
  removes the IP and finer GeoIP fields (including nested person properties),
  and runs before the standard GeoIP transformation. Its versioned source is
  [`scripts/posthog-country-only.hog`](scripts/posthog-country-only.hog).
  Its filter is `geoip_country_only = true`, with execution order `0` before the
  standard GeoIP step. If the scoped transformation is unavailable, the CLI's
  full-GeoIP disable flag still prevents detailed location enrichment.
- Full GeoIP enrichment remains disabled with `$geoip_disable: true`, and
  person-profile processing remains disabled. No additional geolocation service
  is called by the CLI. VPN/proxy/CI traffic reflects the exit country's location.
- There is no durable local usage log or retry queue.
- Explicit issue feedback waits for delivery and returns an error on failure,
  rather than only warning.
- The PostHog API key is a *public* project key (safe to embed in source).

## Source code

The telemetry implementation is in
[`src/telemetry.rs`](src/telemetry.rs) — about 250 lines of Rust. We
encourage you to review it.

## Questions?

Open an issue at <https://github.com/eisber/lox-cli/issues> if you have any
concerns about data collection.
