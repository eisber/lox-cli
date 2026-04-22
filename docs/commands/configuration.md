---
title: Configuration
layout: default
parent: Command Reference
nav_order: 4
---

# Configuration Commands
{: .no_toc }

## Table of contents
{: .no_toc .text-delta }

1. TOC
{:toc}

---

## Setup

```bash
lox setup set --host https://192.168.1.100 --user admin --pass secret
lox setup set --serial YOUR_SERIAL         # enables correct TLS hostname
lox setup set --verify-ssl                 # enable cert verification
lox setup set --no-verify-ssl              # disable (default, self-signed)
lox setup show                             # show config (password redacted)
```

All fields support environment variables: `LOX_HOST`, `LOX_USER`, `LOX_PASS`, `LOX_SERIAL`.

---

## Aliases

```bash
lox alias add wz "1d8af56e-036e-e9ad-ffffed57184a04d2"
lox alias remove wz
lox alias ls
```

Then use directly: `lox on wz`, `lox get wz`

---

## Cache

The structure cache (`~/.lox/cache/structure.json`) stores the Miniserver's LoxApp3.json with a 24-hour TTL.

```bash
lox cache info               # show cache age and path
lox cache check              # check if cache is current
lox cache refresh            # force re-fetch
lox cache clear              # delete local cache
```

---

## Token auth

More secure than Basic Auth. Tokens are valid ~20 days.

```bash
lox token fetch              # fetch and save new token
lox token info               # show token status
lox token check              # verify token on Miniserver
lox token refresh            # extend validity
lox token revoke             # revoke token on Miniserver
lox token clear              # delete local token file
```

The token auth flow uses RSA+AES key exchange via WebSocket. Once acquired, the token is automatically used for all HTTP requests.

---

## Scenes

```bash
lox scene ls                 # list all scenes
lox scene show abend         # print YAML definition
lox scene new abend          # create empty scene file
```

Scene files are stored in `~/.lox/scenes/*.yaml`. See the [Scenes guide](/lox-cli/guides/scenes) for details.

---

## Loxone Config management

Download, inspect, and manage Loxone Config files:

```bash
lox config download                       # download latest config ZIP via FTP
lox config download --extract             # download + decompress to .Loxone XML
lox config download --save-as config.zip  # custom output filename
lox config ls                             # list available configs
lox config extract config.zip             # decompress LoxCC to .Loxone XML
lox config extract config.zip --save-as out.Loxone
lox config upload config.zip --force      # upload to Miniserver
lox config users file.Loxone              # list user accounts
lox config devices file.Loxone            # list hardware devices
lox config devices file.Loxone --ports    # show all I/O ports with used/free status
lox config diff old.Loxone new.Loxone     # compare two configs
```

### Describe & inspect

```bash
lox config describe file.Loxone                 # full config summary by room
lox config describe file.Loxone --room "Küche"  # single room
lox config rooms file.Loxone                    # list rooms with item counts
lox config controls file.Loxone                 # list controls with type/room
lox config controls file.Loxone -t WeatherData  # filter by type
lox config validate file.Loxone                 # check UUIDs, wiring, orphans
```

### Add blocks & templates

```bash
lox config add --type LightController2 --title "Kitchen Light" \
  --room "Kitchen" --category "Beleuchtung" file.Loxone
lox config template file.Loxone bedroom --room "DG Schlafzimmer"
```

Available templates: `standard`, `bedroom`, `bathroom`, `hallway`, `kitchen`, `outdoor`, `office`.
German aliases work too: `schlafzimmer`, `badezimmer`, `flur`, `küche`, `büro`.

Each template creates appropriate controls (LightController2, JalousieUpDown2, Thermostat, etc.) with sensible parameter defaults.

### Parameters

```bash
lox config get-params file.Loxone "Kitchen Light"            # show all params
lox config set-param file.Loxone "Kitchen Light" FadingTime 2.0
```

### Wiring & device binding

```bash
lox config add-virtual-in file.Loxone --title "Sensor Feed"  # create VirtualIn
lox config wire-connector file.Loxone "Light.I1" <source-uuid>  # wire source → target
lox config device-bind file.Loxone "Kitchen Light" AQ1 \
  --device "RGBW Dimmer"                                      # wire control → actor
```

The `device-bind` command creates an OutputRef chain: Control.AQ → OutputRef.AI → OutputRef.AQ → Actor.I. It auto-detects `LinkRefType` based on the device type (Actor=55, TreeAactor=175, etc.).

### Auto-layout & deploy

```bash
lox config layout file.Loxone                   # ELK-based auto-layout
lox config push file.Loxone --reboot --force    # upload + fast SPS reload (~4s)
lox config patch --replace OLD NEW --reboot --force  # download, patch, upload
```

### Git-based config versioning

Track configuration changes in a git repository:

```bash
lox config init ~/loxone-config           # initialize git repo
lox config pull                           # download, diff, and git-commit
lox config pull --quiet                   # cron-friendly
lox config log                            # show change history
lox config log -n 5                       # last 5 entries
lox config restore abc123 --force         # restore from git history
```

See the [Config Versioning guide](/lox-cli/guides/config-versioning) for the full workflow.

---

## Context management

Manage multiple Miniserver connections, similar to `kubectl config use-context`:

```bash
lox ctx add home --host https://192.168.1.100 --user admin --pass secret
lox ctx add office --host https://10.0.0.50 --user admin --pass secret
lox ctx use home                # switch active context
lox ctx home                    # shortcut for `lox ctx use home`
lox ctx list                    # list all contexts (* = active)
lox ctx current                 # show active context
lox ctx remove office           # remove a context
lox ctx rename home house       # rename a context
```

Use `--ctx <name>` on any command for a one-off override:
```bash
lox --ctx office status         # run against 'office' without switching
```

### Project-local config

```bash
lox ctx init                    # create .lox/ in current directory
lox ctx init --host https://192.168.1.100 --user admin --pass secret  # with connection details
```

Project-local `.lox/config.yaml` is auto-discovered by walking up from cwd (like `.git`). Each context gets its own cache, token, and scenes directory. Secrets are excluded via `.lox/.gitignore`.

### Migration from flat config

```bash
lox ctx migrate                 # convert existing config to 'default' context
```

Existing flat `~/.lox/config.yaml` files continue to work unchanged.

### Multi-context config format

```yaml
active_context: home
contexts:
  home:
    host: https://192.168.1.100
    user: admin
    pass: secret
  office:
    host: https://10.0.0.50
    user: admin
    pass: secret
```

### Config resolution order

1. `LOX_CONFIG` env var (absolute priority)
2. Project-local `.lox/config.yaml` (walk up from cwd)
3. Global `~/.lox/config.yaml` (flat or multi-context)
4. `--ctx` flag overrides context selection within global config

---

## Shell completions

```bash
lox completions bash                      # generate bash completions
lox completions zsh                       # generate zsh completions
lox completions fish                      # generate fish completions
lox completions powershell                # generate PowerShell completions
lox completions --install                 # auto-detect and install
```

---

## Command schema

For AI agent integration — discover available commands programmatically:

```bash
lox schema                                # list all commands with metadata
lox schema blind                          # schema for a specific command
lox schema -o json                        # JSON for programmatic use
```
