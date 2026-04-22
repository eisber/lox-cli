---
title: Command Reference
layout: default
nav_order: 3
has_children: true
---

# Command Reference
{: .no_toc }

All commands support these global flags:
{: .fs-5 .fw-300 }

| Flag | Description |
|:-----|:------------|
| `-o`, `--output` | Output format: `table` (default), `json`, `csv` |
| `-q`, `--quiet` | Suppress non-essential output |
| `--no-color` | Disable colored output (also respects `NO_COLOR` env var) |
| `--no-header` | Suppress table headers |
| `-v`, `--verbose` | Verbose logging (`-v` HTTP requests, `-vv` + response bodies) |
| `--dry-run` | Validate and resolve without executing |
| `--non-interactive` | Fail instead of prompting (implied by `-o json`) |
| `--trace-id` | Correlation ID for agent tracing |
| `--ctx <name>` | Use a specific context (overrides active context) |

## Control resolution

Controls can be referenced by **name** (fuzzy matched), **UUID**, or **alias**.

When multiple controls share the same name, disambiguate with:

```bash
# --room flag
lox get "Temperatur" --room "Schlafzimmer"

# Bracket syntax
lox get "Temperatur [OG Schlafzimmer]"
```

Resolution order: alias > exact UUID > bracket room qualifier > `--room` flag > fuzzy substring.

## Dry-run mode

Preview what a command would do without executing:

```bash
lox --dry-run on "Licht Wohnzimmer"
lox --dry-run on "Licht" -o json
```

JSON dry-run output:

```json
{
  "ok": true,
  "dry_run": true,
  "would_execute": {
    "uuid": "1d8af56e-...",
    "command": "on",
    "control": "Licht Wohnzimmer",
    "room": "Wohnzimmer"
  }
}
```
