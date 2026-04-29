# Command Reference

Loxone Config-as-Code CLI. All commands operate on local `.Loxone` XML config files — no live Miniserver connection required for editing, simulation, or validation.

Global flags: `-o`/`--output json|csv|table`, `-q`/`--quiet`, `--no-color` (also respects `NO_COLOR` env var), `--no-header`, `-v`/`--verbose` (request logging, `-vv` for request+response bodies), `--dry-run`, `--non-interactive` (fail instead of prompting; implied by `-o json`), `--trace-id <ID>` (correlation ID for agent tracing), `--ctx <name>` (use a specific context)

---

## Setup

```bash
lox setup set --host https://192.168.1.100 --user admin --pass secret
lox setup set --serial YOUR_SERIAL        # enables correct TLS hostname (DynDNS)
lox setup set --verify-ssl                # enable cert verification
lox setup set --no-verify-ssl             # disable (default, self-signed Miniserver certs)
lox setup show                            # show config (password redacted)
lox setup show -o json                    # JSON output
```

All config fields also support env vars: `LOX_HOST`, `LOX_USER`, `LOX_PASS`, `LOX_SERIAL`.

---

## Config Inspection

```bash
lox config describe file.Loxone                   # human-readable config summary by room
lox config describe file.Loxone --room "Küche"    # describe a single room only
lox config describe file.Loxone -o json           # JSON output

lox config rooms file.Loxone                      # list rooms with item counts
lox config controls file.Loxone                   # list controls with type/room/category
lox config controls file.Loxone -t WeatherData    # filter by type
lox config controls file.Loxone -r "Zentral"      # filter by room
lox config users file.Loxone                      # list user accounts
lox config devices file.Loxone                    # list hardware devices (Tree/Air/Network)
lox config devices file.Loxone --ports            # show I/O ports with used/free status

lox config get-params file.Loxone "Kitchen Light" # show all params with current/default values
lox config stats file.Loxone                      # comprehensive config statistics
lox config autopilot-list file.Loxone             # list automation/autopilot rules
lox config mode-list file.Loxone                  # list operating modes
```

---

## Config Editing

### Add blocks

```bash
lox config add --type light --title "Kitchen Light" --room "Kitchen" file.Loxone
lox config add --type switch --title "Main Switch" file.Loxone
lox config add --type timer --title "Night Timer" --page "Timers" file.Loxone
lox config add --type mqtt-sub --title "Temp Feed" --parent "MQTT Gateway" --topic "home/temp" file.Loxone
```

Available `--type` values: `light`, `switch`, `presence`, `alarm-clock`, `memory`, `timer`, `mqtt-sub`, `mqtt-pub`, `calendar`, `autopilot`.

Options: `--room <ROOM>`, `--category <CATEGORY>`, `--page <PAGE>`, `--parent <PARENT>`, `--topic <TOPIC>`, `--save-as <FILE>`.

### Set parameters

```bash
lox config set-param file.Loxone "Kitchen Light" FadingTime 2.0
lox config set-param file.Loxone "Night Timer" Duration 300
```

Arguments: `<FILE> <SELECTOR> <PARAM> <VALUE>`. Selector is a block title or `"uuid:..."`.

### Wire connectors

```bash
lox config wire-connector file.Loxone "Light.I1" <source-uuid>
lox config wire-connector file.Loxone "Both Sensors.I2" <sensor-q-uuid> --save-as out.Loxone
```

Arguments: `<FILE> <TARGET> <SOURCE_UUID>`. Target format: `"BlockTitle.ConnectorKey"`.

### Device binding

```bash
lox config device-bind file.Loxone "Kitchen Light" AQ1 --device "RGBW Dimmer"
```

Arguments: `<FILE> <CONTROL> <OUTPUT_CONN>`. Required: `--device <DEVICE>`.

### Room templates

```bash
lox config template file.Loxone bedroom --room "DG Schlafzimmer"
lox config template file.Loxone bathroom --room "OG Bad" --save-as out.Loxone
```

Templates: `standard`, `bathroom`, `hallway`, `bedroom`, `kitchen`, `outdoor`.

### Timer schedules

```bash
lox config timer-schedule file.Loxone "Night Timer" "20:00-24:00"
lox config timer-schedule file.Loxone "Night Timer" "06:00-08:00" --value 0.5
```

Arguments: `<FILE> <SELECTOR> <RANGE>`. Range format: `HH:MM-HH:MM`. `--value` sets the output during active period (default: 1).

### Virtual inputs

```bash
lox config add-virtual-in file.Loxone "Sensor Feed"
lox config add-virtual-in file.Loxone "Analog Sensor" --analog
lox config add-virtual-in file.Loxone "Custom Input" --parent "VirtualInCaption"
```

### Room, control, and MQTT management

```bash
lox config room <SUBCOMMAND> ...           # manage rooms (add/rename/move)
lox config control <SUBCOMMAND> ...        # manage controls
lox config mqtt <SUBCOMMAND> ...           # manage MQTT plugin configuration
```

### User management

```bash
lox config user-add file.Loxone ...        # add a user account
lox config user-remove file.Loxone ...     # remove a user account
```

### Autopilot and calendar

```bash
lox config autopilot-add file.Loxone ...   # add an autopilot rule
lox config calendar-add file.Loxone ...    # add a calendar/schedule element
```

### Auto-layout

```bash
lox config layout file.Loxone              # ELK-based auto-layout for all pages
lox config layout file.Loxone --page "Lighting"  # layout a specific page
```

### Low-level XML editing

```bash
lox config xml <SUBCOMMAND> ...            # power-user XML operations
```

---

## Config Validation

```bash
lox config validate file.Loxone            # check UUIDs, wiring, orphans
lox config check file.Loxone               # check blocks for unset params, dead-end wiring
lox config check file.Loxone --selector "Kitchen Light"  # check a specific block only
lox config scan file.Loxone                # scan for PII, secrets, and credentials
lox config scan file.Loxone --strict       # exit with error code if PII found (for CI)
lox config report file.Loxone              # anonymised diagnostic report (scan + stats + check)
```

---

## Config Management

```bash
lox config download                        # download latest config ZIP via FTP
lox config download --extract              # download + decompress to .Loxone XML
lox config download --save-as config.zip   # custom output filename
lox config ls                              # list all configs on the Miniserver
lox config extract config.zip              # decompress LoxCC → .Loxone XML
lox config extract config.zip --save-as out.Loxone
lox config compress file.Loxone            # compress .Loxone XML → LoxCC (with CRC32)
lox config upload config.zip --force       # upload to Miniserver (dangerous)
lox config diff old.Loxone new.Loxone      # compare two configs (accepts .zip or .Loxone)
lox config patch ...                       # download, patch, recompress, and upload in one step
lox config push file.Loxone --force        # recompress + upload edited .Loxone XML
lox config push file.Loxone --reboot --force  # ...and reboot Miniserver after upload
lox config push-http ...                   # upload config ZIP via HTTP POST (fsput)
lox config snapshot file.Loxone            # save a snapshot for evaluation (before/after)
```

---

## GitOps (Config Version Tracking)

```bash
lox config init ~/loxone-config            # initialize a git repo for config tracking
lox config pull                            # download, decompress, diff & git-commit
lox config pull --quiet                    # cron-friendly (no output unless error)
lox config log                             # show config change history
lox config log -c 5                        # last 5 entries
lox config restore abc123 --force          # restore config from git history & upload
```

The `pull` workflow: FTP download → LoxCC decompress → semantic diff → git commit with meaningful message. Multi-Miniserver: each serial gets its own subdirectory in the repo.

---

## Simulation

```bash
lox sim check file.Loxone                  # parse config and report block/connector counts

lox sim run file.Loxone --sim '{"inputs":{"Sensor.Q":1},"expect":{"Light.AQ":1}}'
lox sim run file.Loxone --sim-file test.json  # run sim spec(s) from a JSON file

lox sim step file.Loxone --sim '{"inputs":{"Sensor.Q":1}}'
lox sim step file.Loxone --sim-file test.json # step-by-step showing signal changes per tick

lox sim dump file.Loxone                   # dump full graph (blocks, connectors, wires)
lox sim dump file.Loxone --sim '...'       # apply sim spec before dumping
lox sim dump file.Loxone -o json           # JSON output
```

`--sim` accepts inline JSON (single object or array). `--sim-file` loads from a file.

---

## Block Library

```bash
lox blocks search "timed light"            # search by intent or keyword
lox blocks search "blind wind" -o json     # JSON output

lox blocks info StairwayLS                 # full details: inputs, outputs, parameters
lox blocks info And -o json

lox blocks list                            # list all block types
lox blocks list --category logic           # filter by category
```

Categories: `logic`, `math`, `compare`, `timer`, `schedule`, `state`, `lighting`, `shading`, `hvac`, `security`, `energy`, `io`, `button`, `misc`.

---

## Multi-Context Management

Manage multiple Miniserver connections, similar to `kubectl config use-context`:

```bash
lox ctx add home --host https://192.168.1.100 --user admin --pass secret
lox ctx add office --host https://10.0.0.50 --user admin --pass secret --serial SERIAL
lox ctx use home                           # switch active context
lox ctx list                               # list contexts (* = active)
lox ctx current                            # show active context
lox ctx remove office                      # remove a context
lox ctx rename home house                  # rename a context
```

### One-off context override

```bash
lox --ctx office config download           # run against 'office' without switching
```

### Project-local config

```bash
lox ctx init                               # create .lox/ in current directory
lox ctx init --host ... --user ... --pass ...  # with connection details
```

Project-local `.lox/config.yaml` is auto-discovered by walking up from cwd (like `.git`). Secrets are excluded via `.lox/.gitignore`.

### Migration

```bash
lox ctx migrate                            # convert flat config to 'default' context
```

### Config resolution order

1. `LOX_CONFIG` env var (absolute priority)
2. Project-local `.lox/config.yaml` (walk up from cwd)
3. Global `~/.lox/config.yaml` (flat or multi-context)
4. `--ctx` flag overrides context selection within global config

---

## Cache

```bash
lox cache info                             # show cache age and path
lox cache check                            # check if cache is current (without full download)
lox cache refresh                          # force re-fetch
lox cache clear                            # delete local cache
```

---

## Token Auth

More secure than Basic Auth. Token is valid ~20 days.

```bash
lox token fetch                            # fetch & save token (RSA/AES key exchange)
lox token info                             # show token status
lox token check                            # verify token on Miniserver
lox token refresh                          # extend validity
lox token revoke                           # revoke on Miniserver
lox token clear                            # delete local token file
```

---

## Telemetry

```bash
lox telemetry status                       # show current telemetry setting
lox telemetry enable                       # opt in to anonymous usage analytics
lox telemetry disable                      # opt out
```

---

## Shell Completions

```bash
lox completions bash                       # generate bash completions
lox completions zsh                        # generate zsh completions
lox completions fish                       # generate fish completions
lox completions --install                  # install to standard location for your shell
```

---

## Command Schema (AI Agents)

```bash
lox schema                                 # list all commands with metadata
lox schema config                          # schema for a specific command group
lox schema -o json                         # JSON output for programmatic use
```

Returns command structure, arguments, subcommands, and valid parameters. Designed for AI agent discovery.
