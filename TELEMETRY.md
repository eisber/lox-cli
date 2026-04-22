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
| Block types created | `And`, `LightController2`    | Template / wizard priorities  |
| Config stats        | 12 rooms, 87 blocks, 34 wires | Complexity profiles          |
| Check error types   | `missing_lightscenesC`       | Improve validation rules      |
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

Events are sent to [PostHog](https://posthog.com) (US cloud instance) via
their HTTP capture API. PostHog is an open-source product analytics platform.

- Endpoint: `https://us.i.posthog.com/capture/`
- Transport: HTTPS POST with JSON body
- Timeout: 2 seconds (fire-and-forget — never blocks the CLI)

## PostHog privacy

PostHog's privacy policy: <https://posthog.com/privacy>

We have configured PostHog to:
- **Not** store IP addresses
- **Not** use cookies or browser fingerprinting (this is a CLI tool)
- Retain data for 90 days only

## Technical details

- Session IDs are random UUIDs, rotated daily, stored in
  `~/.lox/.telemetry_session`. They are **not** tied to any user identity.
- Telemetry runs in a background thread and never blocks CLI execution.
- If the POST fails (network error, timeout), it is silently dropped.
- The PostHog API key is a *public* project key (safe to embed in source).

## Source code

The telemetry implementation is in
[`src/telemetry.rs`](src/telemetry.rs) — about 250 lines of Rust. We
encourage you to review it.

## Questions?

Open an issue at <https://github.com/eisber/lox-cli/issues> if you have any
concerns about data collection.
