---
title: AI Agent Integration
layout: default
parent: Guides
nav_order: 2
---

# AI Agent Integration
{: .no_toc }

`lox` was designed from the ground up for AI agent integration.
{: .fs-5 .fw-300 }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Design principles

Every command follows these conventions for reliable agent interaction:

- **Exit codes**: 0 on success, non-zero on error
- **Structured output**: `-o json` for machine-readable output on every command
- **Fuzzy matching**: agents don't need UUIDs — natural names work
- **Disambiguation**: `--room` flag and `[Room]` bracket syntax resolve ambiguous names
- **Error messages**: readable errors with suggestions for correction
- **Dry-run**: `--dry-run` validates without executing
- **Tracing**: `--trace-id` correlates agent actions across logs
- **Non-interactive**: `--non-interactive` fails instead of prompting (implied by `-o json`)
- **Schema discovery**: `lox schema` lets agents discover commands programmatically

## Tool definition

Give your LLM a shell tool:

```json
{
  "name": "lox",
  "description": "Control Loxone smart home. Use -o json for structured output.",
  "parameters": {
    "command": {
      "type": "string",
      "description": "e.g. 'on Wohnzimmer', 'blind Sudseite pos 50', 'status --energy'"
    }
  }
}
```

The agent calls `lox <command>` as a shell tool and reads stdout.

## Agent workflow

```bash
# 1. Discover available commands
lox schema -o json

# 2. Discover controls in the home
lox ls -o json

# 3. Preview before executing
lox --dry-run on "Licht" -o json

# 4. Execute with tracing
lox --trace-id "run-42" on "Licht"

# 5. Check device health
lox health --problems -o json
```

## Schema discovery

Agents can introspect what commands exist and what parameters they accept:

```bash
lox schema                    # list all commands
lox schema blind              # schema for a specific command
lox schema -o json            # JSON for programmatic use
```

## Error handling

When using `-o json`, errors return structured envelopes:

```json
{
  "ok": false,
  "error": "control_not_found",
  "message": "No control matching 'Nonexistent'"
}
```

Error codes:

| Code | Description |
|:-----|:------------|
| `control_not_found` | No control matches the name |
| `ambiguous_control` | Multiple controls match — use `--room` to disambiguate |
| `config_not_found` | Missing config file |
| `confirmation_required` | Command needs `--yes` flag |
| `unauthorized` | Invalid credentials |
| `forbidden` | Insufficient permissions |
| `not_found` | Resource not found |
| `http_error` | HTTP request failed |
| `connection_error` | Cannot reach Miniserver |
| `error` | Generic error |

## Conditional logic

Use `lox if` for state-based decisions:

```bash
# Returns exit code 0 (true) or 1 (false)
lox if "Temperatur" gt 25 && lox blind "Beschattung" pos 70
lox if "Schalter" eq 1 && lox on "Licht"
```

## Real-time streaming

For continuous monitoring, use WebSocket streaming:

```bash
lox stream -o json                       # NDJSON stream of all changes
lox stream --room "Kitchen" -o json      # filtered by room
lox stream --type LightControllerV2      # filtered by type
```
