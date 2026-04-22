# CLI Feature Plan — Prioritized Roadmap

> **Status: IMPLEMENTED** — All Priority 1–4 features and config inspection commands are implemented.

Based on [API research](api-research.md) and current CLI capabilities.

---

## Priority 1 — Quick Wins (data already available, minimal effort)

These features use data already present in `LoxApp3.json` or require only simple GET requests.

### 1.1 `lox ls --favorites`
Filter controls by `isFavorite` flag. Minimal change — add a `--favorites` flag to the existing `ls` command.

### 1.2 `lox ls --cat <category>`
Filter controls by category (Lighting, Shading, etc.). Categories are already in `LoxApp3.json` under `cats`.

### 1.3 `lox categories`
List all categories defined in the structure file. Simple new subcommand.

### 1.4 `lox info <control>` (extended)
Enhance `lox get` or add a dedicated `lox info` command that shows:
- Sub-controls (e.g., individual dimmers in a LightController)
- State UUIDs and their current values
- Control notes (`/jdev/sps/io/{uuid}/controlnotes`)
- Mood list (for LightControllerV2)
- Statistics config (if any)
- `isSecured` / `isFavorite` flags

### 1.5 `lox globals`
Show global states from `LoxApp3.json.globalStates`: operating mode, sunrise/sunset times, wind/rain warnings, etc. Fetch current values via the existing state endpoint.

### 1.6 `lox modes`
List operating modes from `LoxApp3.json.operatingModes`. Show which mode is currently active.

### 1.7 `lox cache check`
Use `GET /jdev/sps/LoxAPPversion3` to check if the cached structure is still current without downloading the full file. Smarter cache validation.

### 1.8 `lox status --diag`
Add diagnostic info to `lox status`: CPU utilization (`lastcpu`), running tasks (`numtasks`), context switches, SD card health (`sdtest`).

### 1.9 `lox status --net`
Network info: IP, MAC, DNS, gateway, DHCP, NTP settings. All simple GET requests to `/jdev/cfg/*`.

---

## Priority 2 — High Value, Low-Medium Effort

### 2.1 `lox history <control>` / `lox stats`
**Highest-value new feature.** Fetch and display historical statistics (energy consumption, temperature trends).
- `lox stats` — list controls that have statistics enabled (from structure file)
- `lox history <control> [--month YYYY-MM] [--day YYYY-MM-DD] [--csv]` — fetch binary statistics data, decode, and display as table or CSV
- Binary format: `Uint32` timestamp (seconds since 2009-01-01) + N × `Float64` values
- Output formats: table (default), `--csv`, `--json`

### 2.2 `lox thermostat <name>`
Control IRoomControllerV2:
- `lox thermostat <name>` — show current temperature, target, mode
- `lox thermostat <name> --temp 22` — set comfort temperature
- `lox thermostat <name> --mode auto|manual|comfort|eco` — set operating mode
- `lox thermostat <name> --override 23 --duration 60` — temporary override

### 2.3 `lox alarm <name> arm|disarm|quit`
Control alarm systems:
- `arm` → `delayedon/1` (arm with motion detection)
- `arm --no-motion` → `delayedon/0`
- `disarm` → `off`
- `quit` / `ack` → `quit` (acknowledge triggered alarm)

### 2.4 `lox lock <control> --reason "..."` / `lox unlock <control>`
Lock/unlock controls (admin feature, firmware v11.3.2.11+). Useful for maintenance scenarios.

### 2.5 `lox token refresh` / `lox token check` / `lox token revoke`
Extend the existing `lox token` subcommand:
- `refresh` — extend token validity without re-authenticating
- `check` — validate token without consuming it
- `revoke` — invalidate a token

### 2.6 `lox update check`
Check for available firmware updates via `update.loxone.com`. Show current vs. available version.

### 2.7 `lox status --bus`
CAN bus statistics: packets sent/received, errors, frame errors, overruns. Useful for diagnosing extension communication issues.

### 2.8 `lox status --lan`
LAN statistics: TX/RX packets, errors, collisions, overflows. Useful for network troubleshooting.

---

## Priority 3 — Medium Effort, Solid Value

### 3.1 `lox weather`
Decode weather data from Miniserver cache (`/data/weatheru.bin`, 108-byte binary chunks) or Loxone Weather Service.
- `lox weather` — current conditions (temperature, humidity, wind, rain)
- `lox weather --forecast` — 7-day hourly forecast

### 3.2 `lox discover`
UDP broadcast on port 7070 to find Miniservers on the local network. Simplifies initial setup — could feed directly into `lox config set`.

### 3.3 `lox dimmer <name> <0-100>`
Dedicated dimmer command with percentage. Could also integrate into `lox send` documentation as a pattern.

### 3.4 `lox gate <name> open|close|stop`
Dedicated gate control. Type-checked against Gate control type, similar to how `lox blind` validates Jalousie type.

### 3.5 `lox color <name> <rgb|hsv>`
Set color on ColorPickerV2 controls. Parse hex RGB (`#FF0000`) or HSV values.

### 3.6 `lox send --secured`
Support for visualization-password-protected controls via `/jdev/sps/ios/{hash}/{uuid}/{cmd}`.

---

## Priority 4 — Lower Priority / Niche

### 4.1 `lox music play|pause|stop|volume [zone]`
Music Server control (port 7091, unofficial API). Transport controls, volume, shuffle, repeat.

### 4.2 `lox reboot`
Reboot the Miniserver. Must require explicit `--confirm` flag or interactive confirmation.

### 4.3 `lox update install`
Trigger firmware update. Requires confirmation and admin rights.

### 4.4 `lox files ls [path]` / `lox files get <path>`
Browse and download files from the Miniserver filesystem via `/dev/fsget/`.

### 4.5 `lox notify <message>`
Send push notification via Loxone cloud. Requires HMAC-SHA1 signing.

### 4.6 `lox task list` / `lox task add`
Time-scheduled command sequences on the Miniserver. Complements the local scene system.

### 4.7 `lox extensions` / `lox devices`
List connected extensions and Air/Tree devices with status information.

### 4.8 `lox time`
Show Miniserver system date/time. Minimal effort but rarely needed.

---

## Implementation Notes

- **Binary parsing** (history, weather): use `byteorder` crate for reading Uint32/Float64 from binary data.
- **UDP discovery**: use `std::net::UdpSocket` with broadcast; no async needed.
- **Token management**: existing `token.rs` WebSocket infrastructure can be reused for refresh/check/revoke.
- **Control-type commands** (thermostat, alarm, gate, dimmer, color): follow the `lox blind` pattern — validate control type from structure file, send the appropriate `/jdev/sps/io/` command, fetch and display result.
- **All new commands** should support `--json` output for scripting consistency.
- **`lox status` flags** (`--diag`, `--net`, `--bus`, `--lan`): extend the existing `status` command rather than creating new top-level commands. Consider `--all` to show everything.
