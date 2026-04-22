---
title: Scenes
layout: default
parent: Guides
nav_order: 1
---

# Scenes
{: .no_toc }

Multi-step automation sequences defined as YAML files.
{: .fs-5 .fw-300 }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Overview

Scenes let you define a sequence of commands that run in order. They are stored as YAML files in `~/.lox/scenes/`.

## Creating a scene

Create a new scene file:

```bash
lox scene new abend
```

This creates `~/.lox/scenes/abend.yaml`. Edit it to define your steps:

```yaml
# ~/.lox/scenes/abend.yaml
name: Abend
steps:
  - control: "Lichtsteuerung Wohnzimmer"
    cmd: "on"
  - control: "Beschattung Sudseite"
    cmd: "pos 70"
  - control: "LED Kuche"
    cmd: "off"
    delay_ms: 500
```

## Scene format

| Field | Required | Description |
|:------|:---------|:------------|
| `name` | Yes | Display name for the scene |
| `steps` | Yes | List of command steps |
| `steps[].control` | Yes | Control name (fuzzy matched) |
| `steps[].cmd` | Yes | Command to send |
| `steps[].delay_ms` | No | Delay in milliseconds before this step |

## Running scenes

```bash
lox run abend                    # run the scene
lox run abend --dry-run          # preview without executing
```

The `--dry-run` flag resolves all control names and shows what would execute, without actually sending commands.

## Managing scenes

```bash
lox scene ls                     # list all scenes
lox scene show abend             # print YAML definition
```

## Tips

- Controls are resolved using the same fuzzy matching as regular commands
- Use `--room` bracket syntax in control names if needed: `"Temperatur [Schlafzimmer]"`
- Add `delay_ms` between steps when the Miniserver needs time to process
- Scene files can be version-controlled alongside your other dotfiles
