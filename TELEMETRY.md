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
| Command name        | `config`, `sim`              | Top-level feature usage       |
| Block types created | `And`, `LightController2`    | Template / wizard priorities  |
| Config stats        | 12 rooms, 87 blocks, 34 wires | Complexity profiles          |
| Check error types   | `missing_lightscenesC`       | Improve validation rules      |
| Scan result         | clean / 2 issues             | Privacy tooling effectiveness |
| Session ID          | Random UUID, rotated daily   | Group events within a day     |
| Timestamp           | Day granularity only         | Usage trends                  |

## What automatic usage events NEVER collect

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

`LOX_TELEMETRY` takes precedence over other settings. If it is unset, the
presence of `DO_NOT_TRACK` disables telemetry; otherwise the config preference
applies, defaulting to enabled.

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
- Transport: HTTPS POST with JSON body
- Timeout: 2 seconds (automatic usage events are fire-and-forget)

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
- Automatic usage telemetry runs in a background thread and never blocks CLI
  execution. It is not a durable local usage log.
- Automatic events can be lost when the command exits or the POST fails
  (network error, HTTP rejection, timeout); they are not retried.
- Explicit feedback waits for delivery and surfaces errors instead.
- The PostHog API key is a *public* project key (safe to embed in source).

## Source code

The telemetry implementation is in
[`src/telemetry.rs`](src/telemetry.rs) — about 250 lines of Rust. We
encourage you to review it.

## Questions?

Open an issue at <https://github.com/eisber/lox-cli/issues> if you have any
concerns about data collection.
