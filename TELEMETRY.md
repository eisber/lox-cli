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
| Command duration    | `duration_ms: 125.4`        | Find slow operations          |
| Exception stack     | Code function names only    | Diagnose failures and panics   |
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
- Raw exception/panic messages, source file paths, and code context
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
shows command trends, durations, exceptions, popular commands, outcomes,
platforms/versions, countries, and a separate diagnostic delivery view.
See the [maintainer guide](#maintainer-guide-access-learn-and-fix) for access,
feedback lookup, exception triage, and delivery troubleshooting.

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
Runtime failures and panics do emit the sanitized exception events described below.
Agents should fix input/setup errors first, then report reproducible bugs with
sanitized details.

## Command timings and exception traces

Command events include `duration_ms`: monotonic wall time for command execution,
measured before telemetry upload/flush. It is not a precise wall-clock timestamp.

Failed commands also emit native PostHog `$exception` events. These contain the
canonical command name, error category, handled/unhandled status, and a bounded
stack of code function symbols. Returned errors use anyhow's error-origin
backtrace; panics capture a stack before unwinding and preserve normal panic output.
Arguments, raw error/panic messages, filesystem paths, memory addresses, and
source code context are not sent. Build hashes are removed from function names.
This intentionally provides less context than a full unsanitized crash dump.

The CLI enables local anyhow backtrace capture early when no
`RUST_LIB_BACKTRACE` override is set. An explicit `RUST_LIB_BACKTRACE=0` can
disable error-origin stacks; error categories are still sent. Uploads and panic
reporting respect the normal telemetry opt-out and dry-run rules. Panic events
use the same bounded delivery and exclude failures inside the telemetry sender
itself to prevent recursive crash reporting.

## Maintainer guide: access, learn, and fix

### Access and locations

Sign in to **PostHog EU** at `https://eu.posthog.com` and select project
**112518**, currently named **Default project**. Ask a project administrator for
membership and permission to view the dashboard, events, and Error Tracking.
These are authenticated project links, not public dashboard shares. If a link
is denied, check the selected project and request access rather than copying
another maintainer's credentials.

| Location | Use it for |
|----------|------------|
| [lox CLI Usage dashboard](https://eu.posthog.com/project/112518/dashboard/1092443) | Start here: adoption, outcomes, latency, exceptions, platforms, countries, diagnostic delivery |
| [Median / p95 duration](https://eu.posthog.com/project/112518/insights/0oq9Jy2G) | Spot latency changes, split by source |
| [Slowest commands](https://eu.posthog.com/project/112518/insights/ZVLcXMdN) | Find command-level performance work |
| [Exceptions by command and source](https://eu.posthog.com/project/112518/insights/FFxSS16r) | Identify failing workflows without counting diagnostics as usage |
| [Error Tracking](https://eu.posthog.com/project/112518/error-tracking) | Open native issues, inspect occurrences and function stacks, verify fixes |
| [SQL workspace](https://eu.posthog.com/project/112518/sql) | Retrieve fallback `issue_report` feedback and inspect stored event properties |
| [GitHub reports](https://github.com/eisber/lox-cli/issues) | Read reports delivered successfully by `lox report` |

Agents can use the authorized PostHog MCP connection at
`https://mcp.posthog.com/mcp`, selecting the same EU project. Authorization belongs
in the local MCP client or credential store, **not** in repository files.
Dashboard/project IDs and these links are non-secret; personal API tokens,
OAuth credentials, and session cookies must not be copied into docs, reports,
queries, or commits. The capture endpoint is for ingestion, not dashboard login.

### Find report / feedback content

Start with the receipt from [`lox report`](COMMANDS.md#issue-reporting):

- `channel = github`: follow `url`. The public issue body contains `Report ID`.
  Successful GitHub delivery does **not** also send an `issue_report` event.
- `channel = telemetry`: open the SQL workspace and find event `issue_report`
  with `properties.report_id` equal to the receipt's `report_id`. Read
  `properties.title` and `properties.body`; these are explicitly submitted text,
  not sanitized automatic exception fields. `url` is null and `github_error`
  in the receipt explains why fallback was used.
- A nonzero submission result means no delivery was confirmed. Check existing
  reports before retrying, to avoid duplicates. A `command` event named
  `report` alone does not prove that the feedback body was delivered.

For example, replace the placeholder with the receipt ID and adjust the date
range if needed:

```sql
SELECT timestamp, properties.report_id AS report_id,
       properties.title AS title, properties.body AS body,
       properties.cli_version AS cli_version,
       properties.os AS os, properties.arch AS arch,
       properties.source AS source
FROM events
WHERE event = 'issue_report'
  AND timestamp >= now() - INTERVAL 7 DAY
  AND properties.report_id = 'REPLACE_WITH_REPORT_ID'
LIMIT 20
```

For an intake review without a receipt, remove the report-ID predicate and
order by `timestamp DESC` before `LIMIT`. A fallback capture acknowledgement
is not an automatically created GitHub issue, Beads task, or Error Tracking
issue; maintainers must review it and track actionable work in `bd`.
Inspect supplied text before quoting it into a public issue or regression fixture.
`lox config report FILE` is a separate configuration-diagnostics command, not
the feedback inbox. No raw logs or configuration files are attached automatically.

### Learn from signals and close the loop

1. **Review usage and failures together.** Start with the dashboard over complete
   UTC days, normally the last 7 or 30 days. Use `source = usage` for real usage;
   examine `diagnostic`, `test`, and `eval` separately. Compare command failures
   with total executions, version, OS, and architecture before prioritizing.
   Pair median/p95 changes with sample volume; a few fast diagnostic commands
   do not establish production performance.
2. **Inspect an actual exception.** Open Error Tracking, narrow to CLI events
   using `command`, `cli_version`, and `source`, then inspect an occurrence's
   application function frames and error category. `RustError` is a returned
   error (`handled = true`); `RustPanic` is a panic (`handled = false`), which
   does not by itself prove the whole process exited. Panics in worker threads
   can be caught by their caller. A `$exception` is not a full log, distributed
   trace, Session Replay, or automatically submitted feedback report.
3. **Reproduce before diagnosing.** Combine the function stack with sanitized
   feedback reproduction steps and expected/actual behavior. Ordinary input,
   authentication, and connection failures are not automatically CLI bugs.
   Reports and exception occurrences have no automatic report-ID join; matching
   command/version/platform/day is supporting evidence, not proof of identity.
   Request a minimal sanitized reproduction when needed instead of enabling
   raw-message or configuration uploads.
4. **Track and fix.** Deduplicate findings, record actionable work in `bd`, and
   include the report ID or GitHub URL, native Error Tracking issue URL, affected
   command/version/platform, reproduction, and expected behavior. Add a regression
   test that fails before the fix. Keep sensitive feedback out of public tracking.
5. **Verify the outcome.** Run the reproduction against the fixed build. Keep
   normal tests opted out; use `source = diagnostic` for an intentional live check.
   Verify stored command outcome/duration and, for an intentional failure, native
   issue linkage plus useful sanitized frames. Record the fix commit/version,
   close the Beads task and resolve the native issue when the fix is verified,
   then watch `source = usage` for recurrence. Resolve deliberate diagnostic
   issues after checking them; do not suppress a fingerprint merely to hide a test.

Daily anonymous IDs rotate and timestamps are rounded to UTC midnight.
PostHog's "users" count is therefore **not** a count of known people or persistent
installations; do not infer an exact execution sequence or hour-level incident
timeline. Repeated failures from one loop can inflate occurrence counts.
Older events may lack source, duration, or exception fields; missing data is not
zero latency or evidence of no failures. The dashboard description records known
baseline contamination and must be preserved when editing charts.

### If charts or traces look empty

Check `lox telemetry status`, config and environment overrides, the CLI build,
the selected EU project, date range, and source filters. Help/version, argument
parsing failures, dry runs, opted-out commands, and normal tests/evals are not
expected to populate runtime telemetry. A report delivered via GitHub is not
expected in the fallback feedback query.

For a harmless delivery probe, run in a **temporary PowerShell session**:

```powershell
$env:LOX_TELEMETRY = '1'
$env:LOX_TELEMETRY_SOURCE = 'diagnostic'
lox telemetry status
```

Refresh the diagnostic delivery view and inspect the stored `command` event,
including `duration_ms`. Telemetry warnings on stderr indicate sender, TLS,
timeout, or HTTP rejection problems; ordinary telemetry failures do not change
the command's exit code. Allow for ingestion delay and refresh again before
concluding delivery failed; property discovery can lag freshly stored events.
Do not test delivery by publishing a dummy GitHub report.

For a deliberate exception probe, validate a known-nonexistent dummy file in
that same diagnostic session. Check the nonzero exit, `$exception`, native issue,
and sanitized application frames, then resolve the diagnostic issue.
If the exception has no stack, check `RUST_LIB_BACKTRACE` and symbol availability;
`RUST_LIB_BACKTRACE=0` disables error-origin capture. Categories without frames
are possible and are not grounds for uploading raw errors or filesystem paths.

## PostHog privacy

PostHog's privacy policy: <https://posthog.com/privacy>

We have configured PostHog to:
- **Not** store IP addresses
- **Not** use cookies or browser fingerprinting (this is a CLI tool)
- Retain data for 90 days only

## Technical details

- Session IDs are random UUIDs, rotated daily, stored in
  `~/.lox/.telemetry_session`. They are **not** tied to any user identity.
- Usage telemetry sends the full canonical command path, execution duration, and a success/error category,
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

Delivery, privacy metadata, duration/exception payloads, and regression tests:
[`src/telemetry.rs`](src/telemetry.rs). Command dispatch and outcome capture:
[`src/main.rs`](src/main.rs). Report routing and receipts:
[`src/commands/report_cmd.rs`](src/commands/report_cmd.rs). Error envelopes and
reporting guidance: [`src/errors.rs`](src/errors.rs).

## Questions?

Open an issue at <https://github.com/eisber/lox-cli/issues> if you have any
concerns about data collection.
