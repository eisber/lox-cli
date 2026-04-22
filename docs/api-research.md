# Loxone REST API Research — New Opportunities for the CLI

> **Status: REFERENCE** — Research document. See [plan.md](plan.md) for implementation status of proposed features.

## Already Implemented (Status Quo)

| Endpoint | Used in `lox` |
|----------|--------------|
| `GET /data/LoxApp3.json` | Structure cache (controls, rooms, etc.) |
| `GET /jdev/sps/io/{uuid}/{cmd}` | `lox send`, `lox on`, `lox off`, `lox pulse`, `lox blind`, `lox mood` |
| `GET /dev/sps/io/{uuid}/all` | `lox get`, `lox watch`, `lox ls --values` |
| `GET /dev/sps/io/{name}/state` | Input state (daemon) |
| `GET /dev/cfg/version` | `lox status` |
| `GET /dev/sys/heap` | `lox status` (memory) |
| `GET /dev/sps/state` | `lox status` (PLC state) |
| `GET /dev/sys/check` | `lox status` (connections) |
| `GET /data/status` | `lox status` (name, IP, online) |
| `GET /dev/fsget/log/def.log` | `lox log` |
| `WSS /ws/rfc6455` | Daemon (WebSocket) |

---

## NEW Endpoints — Interesting for the CLI

### 1. Statistics / History (High Value)

| Endpoint | Description |
|----------|------------|
| `GET /stats/statistics.json` | Overview of all stored statistics (which controls have history) |
| `GET /stats/statistics.json/{controlUUID}` | Filtered per control (since firmware 6.1.10.16) |
| `GET /jdev/sps/getstatsdate` | Date of the statistics.json file |
| `GET /binstatisticdata/{controlUUID}/{YYYYMM}` | Binary statistics data per month |
| `GET /binstatisticdata/{controlUUID}/{YYYYMMDD}` | Binary statistics data per day |

**Binary format:** `ts` (Uint32, seconds since 2009-01-01 in Miniserver local time) + N x Float64 values. Number of values per entry is defined in `LoxApp3.json` under `control.statistic.outputs[]`.

**CLI ideas:**
- `lox history <control> [--month YYYY-MM] [--day YYYY-MM-DD]` — Fetch statistics and display as table/CSV
- `lox stats` — List all controls with statistics enabled
- Especially useful for energy consumption, temperature trends, etc.

---

### 2. Weather (High Value)

| Endpoint | Description |
|----------|------------|
| `GET /data/weatheru.bin` | Cached weather data on the Miniserver (binary format, max 200 entries, 108 bytes each) |
| `weather.loxone.com:6066/forecast/?user=loxone_{serial}&coord={lon},{lat}&asl={elevation}&format=1` | Loxone Weather Service (7 days, hourly) |

**Binary format weatheru.bin:** 108-byte chunks with timestamp, temperature, humidity, wind, rain, etc.

**CLI ideas:**
- `lox weather` — Show current weather data (from Miniserver cache or cloud)
- `lox weather --forecast` — 7-day forecast

---

### 3. Control Extensions

| Endpoint | Description |
|----------|------------|
| `GET /jdev/sps/io/{uuid}/controlnotes` | Notes/help text for a control (plaintext) |
| `GET /jdev/sps/io/{uuid}/lockcontrol/{0,1}/{reason}` | Lock a control with reason (admin, since v11.3.2.11) |
| `GET /jdev/sps/io/{uuid}/unlockcontrol` | Unlock a control (admin) |
| `GET /jdev/sps/io/{uuid}/securedDetails` | Encrypted details (intercom, camera URLs) |
| `GET /jdev/sps/ios/{hash}/{uuid}/{cmd}` | Secured command (visualization password) |
| `GET /jdev/sps/io/{name}/I1/on` | Set specific inputs on function blocks |
| `GET /jdev/sps/io/{name}/R` | Reset light controller |

**CLI ideas:**
- `lox lock <control> --reason "maintenance"` / `lox unlock <control>` — Lock/unlock controls
- `lox get` could also display `controlnotes`
- `lox send --secured` — Operate password-protected controls

---

### 4. System & Diagnostics

| Endpoint | Description |
|----------|------------|
| `GET /jdev/cfg/api` | API info: MAC address, config version, httpsStatus, hasEventSlots |
| `GET /jdev/cfg/apiKey` | API key for hashing + httpsStatus |
| `GET /jdev/cfg/timezoneoffset` | Timezone offset of the Miniserver |
| `GET /jdev/cfg/mac` | MAC address |
| `GET /jdev/cfg/ip` | IP address |
| `GET /jdev/cfg/mask` | Network mask |
| `GET /jdev/cfg/gateway` | Default gateway |
| `GET /jdev/cfg/dns1` / `dns2` | DNS servers |
| `GET /jdev/cfg/dhcp` | DHCP setting |
| `GET /jdev/cfg/ntp` | NTP server |
| `GET /jdev/sys/date` | System date |
| `GET /jdev/sys/time` | System time |
| `GET /jdev/sys/lastcpu` | CPU utilization |
| `GET /jdev/sys/numtasks` | Number of running tasks |
| `GET /jdev/sys/ints` | System interrupts |
| `GET /jdev/sys/comints` | Communication interrupts |
| `GET /jdev/sys/contextswitches` | Context switch counter |
| `GET /jdev/sys/sdtest` | SD card health/memory test |
| `GET /jdev/sys/reboot` | Reboot the Miniserver (requires system rights) |
| `GET /jdev/sps/LoxAPPversion3` | Check structure file version without full download |
| `GET /jdev/sps/status` | PLC frequency/status |
| `GET /jdev/sps/changes` | Poll PLC changes (polling alternative) |
| `GET /jdev/sys/checktoken/{hash}/{user}` | Validate token without using it |
| `GET /jdev/sys/killtoken/{hash}/{user}` | Revoke a token |
| `GET /jdev/sys/refreshtoken/{hash}/{user}` | Refresh a token |
| `GET /jdev/sys/getcertificate` | Get the Miniserver's TLS certificate |

**CLI ideas:**
- `lox status --diag` — CPU, tasks, interrupts, context switches, SD card
- `lox status --net` — Network configuration (IP/MAC/DNS/DHCP/NTP)
- `lox reboot` — Reboot the Miniserver (with confirmation!)
- `lox time` — Show Miniserver system time
- `lox cache check` — Check if cache is current without full download
- `lox token check` / `lox token revoke` / `lox token refresh`

---

### 4b. Network & Bus Statistics

| Endpoint | Description |
|----------|------------|
| `GET /jdev/lan/txp` | Transmitted packets |
| `GET /jdev/lan/txe` | Transmission errors |
| `GET /jdev/lan/txc` | Collisions |
| `GET /jdev/lan/rxp` | Received packets |
| `GET /jdev/lan/rxo` | Receive overflow |
| `GET /jdev/lan/eof` | EOF errors |
| `GET /jdev/bus/packetssent` | CAN bus transmitted packets |
| `GET /jdev/bus/packetsreceived` | CAN bus received packets |
| `GET /jdev/bus/receiveerrors` | CAN bus receive errors |
| `GET /jdev/bus/frameerrors` | CAN bus frame errors |
| `GET /jdev/bus/overruns` | CAN bus buffer overruns |
| `GET /jdev/sys/ExtStatistics/{serial}` | Extension statistics |

**CLI ideas:**
- `lox status --bus` — CAN bus statistics (detect errors!)
- `lox status --lan` — LAN packet statistics

---

### 5. Control-Type-Specific Commands (via `/jdev/sps/io/{uuid}/`)

Already implemented: `on`, `off`, `pulse`, `PulseUp`, `PulseDown`, `FullUp`, `FullDown`, `AutomaticDown`, `manualPosition/{pct}`, `plus`, `minus`, `setMood/{id}`

**New commands for dedicated CLI subcommands:**

| Control Type | Command | Description |
|-------------|---------|------------|
| **Dimmer** | `{0-100}` | Set dimmer percentage |
| **IRoomControllerV2** | `setComfortTemperature/{value}` | Set comfort temperature |
| **IRoomControllerV2** | `override/{value}/{duration}` | Temperature override (degrees/duration) |
| **IRoomControllerV2** | `setOperatingMode/{0-4}` | 0=auto, 1=manual, etc. |
| **Gate** | `open` / `close` / `stop` | Control gate |
| **Alarm** | `delayedon/{0,1}` | Arm with delay (0=no motion, 1=with motion) |
| **Alarm** | `off` / `quit` | Disarm / acknowledge alarm |
| **Alarm** | `dismv/{0,1}` | Enable/disable motion sensor |
| **Sauna** | `{0-6}` | Mode: 0=off, 1=Finnish, 2=humidity, etc. |
| **ColorPickerV2** | `{color}` | Set color (RGB/HSV) |
| **Ventilation** | `{mode}` | Set ventilation mode |

**CLI ideas:**
- `lox thermostat <name> [--temp 22] [--mode auto]` — Control room thermostat
- `lox gate <name> open/close/stop` — Control gate
- `lox alarm <name> arm/disarm/quit` — Control alarm system
- `lox dimmer <name> <0-100>` — Set dimmer level
- `lox color <name> <rgb>` — Set color

---

### 6. Structure File — Unused Data

`LoxApp3.json` contains much more than is currently used:

| Field | Description |
|-------|------------|
| `cats` | Categories (e.g. "Lighting", "Shading") |
| `controls[].details` | Detail data per control (e.g. mood lists for LightController) |
| `controls[].states` | State UUIDs for sub-states (e.g. `activeMoods`, `colorList`) |
| `controls[].subControls` | Sub-controls for complex types (e.g. individual dimmers in LightController) |
| `controls[].statistic` | Statistics configuration (frequency, outputs) |
| `controls[].isFavorite` | Favorite flag |
| `controls[].isSecured` | Requires visualization password |
| `globalStates` | Global states (operating mode, sunrise/sunset, wind warning, etc.) |
| `operatingModes` | Operating mode definitions |
| `msInfo` | Miniserver metadata (serial number, location, language) |
| `weatherServer` | Weather configuration (coordinates, elevation) |
| `autopilot` | Autopilot rules |
| `caller` | Caller service configuration |
| `messageCenter` | Message center |
| `times` | Timer configuration |

**CLI ideas:**
- `lox ls --cat <category>` — Filter controls by category
- `lox ls --favorites` — Show only favorites
- `lox categories` — List all categories
- `lox info <control>` — Detailed view with sub-controls, states, moods, notes
- `lox globals` — Show global states (operating mode, sunrise, etc.)
- `lox modes` — List/set operating modes

---

### 7. Task Recorder

| Endpoint | Description |
|----------|------------|
| Task system via `/jdev/sps/io/` | Time-scheduled command sequences |

Tasks = time-scheduled commands that the Miniserver executes itself. Each task consists of one or more commands with their own timestamp. Tasks must be kept up-to-date via polling (no state updates).

**CLI ideas:**
- `lox task list` — Show scheduled tasks
- `lox task add` — Create a new time-scheduled command
- Complements the existing scene system

---

### 8. Extensions & Device Management

| Endpoint | Description |
|----------|------------|
| Extension commands | Commands for Loxone Extensions (requires full access) |
| Air/Tree device commands | Commands for Air/Tree devices by serial number or name |

**CLI ideas:**
- `lox extensions` — List all connected extensions
- `lox devices` — List Air/Tree devices with status

---

### 9. Discovery & Network

| Endpoint | Description |
|----------|------------|
| `UDP:7070` (broadcast 0x00) | Local Miniserver discovery |
| `dns.loxonecloud.com/?getip&snr={serial}&json=true` | Public IP via Loxone Cloud DNS |
| `/upnp.xml` | UPnP device description |

**CLI ideas:**
- `lox discover` — Scan local network for Miniservers (UDP broadcast)
- `lox config set` autodetect — Find Miniserver automatically

---

### 10. Notifications & Services

| Endpoint | Description |
|----------|------------|
| `mail.loxonecloud.com/sendmail/{serial}` | Email via Loxone Mailer Service |
| `push.loxonecloud.com/v1/push` | Push notifications (HMAC-SHA1 signed) |
| `caller.loxone.com/cgi-bin/loxlive/call.pl` | Text-to-speech call service |

**CLI ideas:**
- `lox notify <message>` — Send push notification
- Integration in automations (action: notify)

---

### 11. Music Server (Port 7091, unofficial)

| Endpoint | Description |
|----------|------------|
| `GET /zone/{zone}/play` | Play/pause/stop transport control |
| `GET /zone/{zone}/volume/{0-100}` | Set volume |
| `GET /zone/{zone}/repeat/{0,1,2}` | Repeat mode (off/one/all) |
| `GET /zone/{zone}/shuffle/{0,1}` | Shuffle on/off |

**Note:** Not officially documented, from community reverse engineering.

**CLI ideas:**
- `lox music play/pause/stop [zone]`
- `lox music volume <zone> <level>`

---

### 12. Firmware Updates

| Endpoint | Description |
|----------|------------|
| `GET update.loxone.com/updatecheck.xml?serial={}&version={}` | Check for available updates |
| `GET /jdev/sys/autoupdate` | Trigger auto-update |
| `GET /jdev/sys/updatetolatestrelease` | Update to latest version |

**CLI ideas:**
- `lox update check` — Check if new firmware is available
- `lox update install` — Trigger update (with confirmation)

---

### 13. Filesystem

| Endpoint | Description |
|----------|------------|
| `GET /dev/fsget/<path>` | Read files from Miniserver filesystem |
| FTP (port 21) | Read/write files (requires FTP rights) |

**CLI ideas:**
- `lox files ls [path]` — Browse Miniserver filesystem
- `lox files get <path>` — Download file

---

## Prioritized Recommendations

### High Value, Low Effort (Quick Wins)

| Feature | Effort | Benefit |
|---------|--------|---------|
| `lox categories` + `lox ls --cat` | Low (data already in LoxApp3.json) | Better navigation |
| `lox ls --favorites` | Minimal (just `isFavorite` flag) | Quick overview |
| `lox globals` (operating mode, sunrise) | Low (data already in LoxApp3.json) | Useful info |
| `lox cache check` (LoxAPPversion3) | Minimal | Smarter cache validation |
| Extended `lox info` (sub-controls, notes) | Low | Deeper insights |
| `lox status --diag` (CPU, tasks, SD) | Low (simple GET requests) | Diagnostic info |
| `lox status --net` (IP/MAC/DNS/NTP + LAN stats) | Low | Network overview |

### High Value, Medium Effort

| Feature | Effort | Benefit |
|---------|--------|---------|
| `lox history` / `lox stats` | Medium (parse binary format) | Very high — energy data, temperatures |
| `lox weather` | Medium (binary 108B chunks) | High — weather forecast |
| `lox thermostat` (IRoomController) | Low-Medium | Direct heating control |
| `lox alarm arm/disarm` | Low-Medium | Alarm control |
| `lox discover` | Medium (UDP broadcast) | Simplify setup |
| `lox lock` / `lox unlock` | Low-Medium | Admin functionality |
| `lox update check` | Low | Firmware management |
| `lox status --bus` (CAN bus statistics) | Low | Extension error diagnosis |

### Lower Priority / Niche

| Feature | Effort | Benefit |
|---------|--------|---------|
| `lox music` (unofficial) | Medium | Music server control |
| `lox gate open/close` | Minimal | Gate control |
| `lox dimmer` | Minimal | Dedicated dimmer command |
| `lox reboot` | Minimal | Rarely needed |
| `lox notify` (push/mail) | Medium (HMAC) | Niche |
| `lox files` (filesystem) | Low | Niche |

---

## Sources

- [Loxone Web Services Documentation](https://www.loxone.com/enen/kb/web-services/)
- [Loxone API Documentation](https://www.loxone.com/enen/kb/api/)
- [Communicating with the Miniserver (PDF, 2025)](https://www.loxone.com/wp-content/uploads/datasheets/CommunicatingWithMiniserver.pdf)
- [Structure File Documentation (PDF)](https://www.loxone.com/wp-content/uploads/datasheets/StructureFile.pdf)
- [Inside-The-Loxone-Miniserver (Reverse Engineering)](https://github.com/sarnau/Inside-The-Loxone-Miniserver)
- [Statistics Download Script (sarnau)](https://gist.github.com/sarnau/e859f2d7beae882476ce6b78a8ab59f1)
- [LoxWiki REST Webservice](https://loxwiki.atlassian.net/wiki/spaces/LOX/pages/1517355410/REST+Webservice)
- [LoxWiki API Collection](https://loxwiki.atlassian.net/wiki/spaces/LOX/pages/1542816653)
- [XciD/loxone-ws (Go Library)](https://github.com/XciD/loxone-ws)
- [ioBroker.loxone Adapter (Control Type Reference)](https://github.com/UncleSamSwiss/ioBroker.loxone)
- [loxberry-music-server-gateway (Audio Endpoints)](https://github.com/mjesun/loxberry-music-server-gateway)
- [netdata-be/loxone (Monitoring, /jdev/sys/ /jdev/bus/ /jdev/lan/ Endpoints)](https://github.com/netdata-be/loxone)
