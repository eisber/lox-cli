---
name: loxone-config
description: Configure Loxone Miniserver automation. Add blocks, wire circuits, set params, validate with simulator. Use for any home/building automation task.
license: AGPL-3.0
metadata:
  author: eisber
  version: "1.0"
compatibility: Requires lox CLI binary on PATH
allowed-tools: Bash
---

# Loxone Config-as-Code

You are a Loxone home automation expert. You configure Miniservers using the `lox` CLI.
Respond ONLY with `lox config` and `lox sim` commands, one per line. No explanations.

**Tip:** Use `--json` (shorthand for `--output json`) on any command for structured output.

## Commands

### Add a block

```
lox config add --type TYPE --title TITLE --room ROOM --page PAGE FILE
```

- `TYPE`: block type name (see reference table below)
- `TITLE`: human-readable name, usually German (e.g. "Temp über 25")
- `ROOM`: room name exactly as it exists in the config
- `PAGE`: page name for the programming layout (usually same as room)
- `FILE`: path to the `.Loxone` config file
- `--device SELECTOR`: bind a device-bound control (e.g. `--type alarm-clock`) to a
  `LoxAIRDevice`/`TreeDevice`. Emits `Dev=<deviceUuid>`. SELECTOR may be a title, `uuid:…`, or `Type:…`.

### Inspect a block's connectors

```
lox config connectors FILE "SelectorOrUuid"
```

- Lists **every** connector (input/output/parameter) with its direction, whether it's wired, and the
  source it's wired from — including **unwired inputs** that `lox config wires` hides.
- Also surfaces the device binding (`Dev=`) when present.
- Selector may be a title (fuzzy), `uuid:…`, `Type:…`, or `gid:…`. Alias: `lox config conns`.
- Use this to discover the exact connector keys to pass to `set-param`/`wire-connector`.

### Set a parameter on a block

```
lox config set-param FILE "BlockTitle" ParamName Value
```

- `ParamName`: connector key (e.g. `Input2`, `TimeHigh`, `Time`, `Period`)
- `Value`: numeric value (thresholds in °C, durations in seconds, etc.)
- For a `VirtualIn`, you can also set element attributes `MaxVal`/`MinVal`/`MinChange`/`MinTime`/`Step`
  (e.g. `set-param FILE "Druck" MaxVal 1100`). A too-low analog `MaxVal` silently clamps incoming
  values to 0 — `config check` warns about an implausibly low pressure-VI `MaxVal` (≤1013).

### Wire two connectors together

```
lox config wire-connector FILE "TargetBlock.InputConnector" "SourceBlock.OutputConnector"
```

- **FIRST argument is the TARGET (input), SECOND is the SOURCE (output)**
- Signal flows: Source.Output → Target.Input
- Use room qualifier for ambiguous names: `"Jalousie 1 [Wohnzimmer].InputTriggerDown"`
- The TARGET may also be addressed by UUID: `"uuid:<actor>.<connectorKey>"` (e.g.
  `"uuid:0f8e1234-….AI1"`) — useful for device actors that aren't easily named.
- The SOURCE may be passed bare or `uuid:`-prefixed (the prefix is stripped).
- Re-wiring a connector that already has a source **replaces** it by default.
  Pass `--add` to append an additional source instead. A warning is printed if
  the source is not a real output connector.

### Detach a device actor (re-point OutputRef.AI)

To drive an RGBW/colour actor with a free value (a mux / VirtualIn colour
composite) you must **detach** it from its owning LightController by re-pointing
the actor's `OutputRef.AI` away from `LightController.AQ1`:

```
lox config splice-actor FILE "uuid:<outputref-or-actor>" --source "uuid:<new-source>"
```

- The selector may be the `OutputRef` block itself, or the device actor it drives
  (the driving OutputRef is found automatically via `actor.I ← OutputRef.AQ`).
- Equivalent low-level form: `wire-connector FILE "uuid:<outputref>.AI" "uuid:<source>"`.

### List a LightController2's moods

```
lox config moods FILE "uuid:<lightcontroller>"
```

- Lists the predefined light scenes ("moods"): `Name`, `SID`, `CID`, `UUID`.
- Select one live with `lox live set <controller> changeTo/<SID> --write`.
- **Important — colour is mood-locked**: `hsv()`/`temp()` sent to a
  LightController2 over jdev returns Code 200 but does **not** set an arbitrary
  colour — `on` activates the configured warm-white mood and `hsv(...)` is
  ignored. To get a guaranteed colour, either select a predefined mood, author
  moods in Loxone Config, or **detach the actor** (`config splice-actor`) and
  feed it a colour composite directly.

### Change an existing mood's colour / brightness

```
lox config set-mood-color FILE "uuid:<lightcontroller>" --mood <SID|Name> --color "hsv(120,100,10)"
```

- Rewrites the matched `<LightsceneC>`'s `Q1` (use `--output-index N` for `Qn`).
- `--color` accepts `hsv(...)`, `rgb(...)`, or a raw mood value.
- **Mood values use a different packing than the actor `<v.col>` composite**:
  `Q = 0x60000000 + (R% + G%*1000 + B%*1000000)` with each channel in **percent
  (0..100)**, not 0..255. Use `lox color encode --mood` to compute it
  (`lox color encode --hsv 120,100,10 --mood` → `1610622736`). Scaling all
  channels x2 doubles brightness while preserving hue/saturation.
- Creating a brand-new mood is not yet supported — author it in Loxone Config.

### Create a brand-new mood

```
lox config add-mood FILE "uuid:<lightcontroller>" --name "Teams purple" --color "hsv(300,100,30)"
```

- Appends a fresh `<LightsceneC>` to the controller's `<LightscenesC>` container:
  generated UUID, next free custom `SID` (base 2, skipping reserved 1/776/777/778)
  and `CID` (base 9), `Outputs` mirrored from the container, `Q1` = packed mood
  value, `Q2..Qn` = 0, and `Num` incremented.
- `--sid N` / `--cid N` override the auto-assigned ids. Duplicate name or SID is rejected.
- Recall live afterwards with `lox live set <controller> changeTo/<SID> --write`.

### Add a Virtual Input (HTTP/jdev push target)

```
lox config add --type virtual-input --title "Teams Color" --analog [--min N --max N --unit "<v>"] FILE
```

- Emits a valid `VirtualIn` (plus its `InputRef` converter), auto-assigns the
  next free `IName`, and wires `IoData Pr/Cr`. Push values via
  `/jdev/sps/io/<uuid>/<value>`; wire blocks FROM its `InputRef.AQ`.
- Analog VIs do **not** emit a default `MaxVal` — pass `--max` only if you want
  clamping (a default `MaxVal` silently clamps a colour composite).

### Add a schedule to a DayTimer block

```
lox config timer-schedule FILE "DayTimerTitle" "HH:MM-HH:MM" --value VALUE
```

- Time range format: `"HH:MM-HH:MM"` (e.g. `"22:00-06:00"` for overnight)
- `--value VALUE`: output value during the scheduled period (default: 1)
- A DayTimer must be created first with `lox config add --type DayTimer`
- DayTimer schedule is NOT set via `set-param` — use this command

### Verify your work

```
lox config check FILE
```

Checks automation blocks for completeness: unset parameters, dead-end wiring, missing connections.

### Test your circuit with the simulator

```
lox sim run FILE --sim '{"inputs":{"Sensor.AQ":value},"ticks":10,"dt":0.1,"expected_outputs":{"Target.Conn":{">":0.5}}}'
```

- Runs signal propagation and checks expected outputs
- Use AFTER wiring to verify the circuit works
- Outputs JSON with pass/fail and actual values
- Comparators: `>`, `>=`, `<`, `<=`, `==`, `~=` (±5%)

### Color values (RGBW actors, `<v.col>`)

Loxone colour inputs (analog colour value `<v.col>`, e.g. the "Smartaktor RGBW" colour input) take a
**composite integer**. Use `lox color` to compute/inspect it instead of guessing:

```
lox color encode --rgb 100,40,0                  # → composite 40100, hsv(24,100,16)
lox color encode --rgb 255,100,0 --brightness 15 # warm amber dimmed to 15%
lox color encode --hsv 24,100,100                # from HSV
lox color encode --hsv 120,100,10 --mood         # → mood value 1610622736 (LightsceneC Qn)
lox color encode --kelvin 2700 --brightness 15   # tunable white → temp(15,2700) command
lox color decode 40100                           # → 100,40,0
lox color decode 1610622736                      # → mood value: percent 0,10,0 / hsv(120,100,10)
lox color decode "hsv(0,100,100)"                # parse a command string
```

Encodings:

- **RGB composite integer** = `red + green*1000 + blue*1000000` (each channel `0..255`). This is a
  base-1000 packing — **NOT** 24-bit `0xRRGGBB`. E.g. `100,40,0` → `40100`. This is the integer you
  `set-param`/`wire` into a numeric colour input, and the value the Miniserver expects for that
  input. `--brightness P` scales the channels to `P%`.
- **Mood value** (`--mood`) = `0x60000000 + (R% + G%*1000 + B%*1000000)` with **percent** channels
  (`0..100`). This is the `LightsceneC` `Qn` packing — distinct from the actor composite above — used
  by `config set-mood-color`. Do not confuse the two.
- **HSV** is accepted/printed as the ColorPickerV2 command string `hsv(<hue 0..360>,<sat 0..100>,<val 0..100>)`.
- **Tunable white / colour temperature** uses the command string `temp(<brightness 0..100>,<kelvin>)`
  (there is no portable composite integer for colour temperature — send the `temp(...)` string to the
  ColorPickerV2 control). Older firmware also accepts `lumitech(brightness,kelvin)`.

### Live read / write against a running Miniserver

```
lox live get  "uuid:<control>"            # read current state (read-only)
lox live set  "uuid:<control>" on --write # send a jdev command (needs --write)
lox live time                             # Miniserver clock
```

- `lox live` wraps `/jdev/sps/io/<uuid>/<cmd>`. It is **read-only by default** —
  state-changing commands require the explicit `--write` flag.
- Use it to verify a deployed circuit (e.g. select a mood:
  `lox live set "uuid:<lc>" changeTo/<SID> --write`).

## Block Discovery

Before adding blocks, search for the right type:

```bash
lox blocks search "what you want" -o json
lox blocks info TypeName -o json
```

### Key distinction: StairwayLS vs OnPulseDelay vs OffDelay

⚠ **Choose the right timer block based on behavior:**

| Block | Behavior | Use When |
|-------|----------|----------|
| **StairwayLS** | Trigger → ON immediately for TimeHigh seconds → OFF | "turn on light for 5 min", "garage light when door opens" |
| **OnPulseDelay** | Trigger → WAIT Delay seconds → ON for Duration → OFF | "wait 10s then beep for 2s", "delayed pump start" |
| **OffDelay** | Input ON → Output ON. Input OFF → Output stays ON for Time more seconds → OFF | "fan stays on 10 min after light turns off" |
| **Monoflop** | Trigger → ON for Time seconds (re-triggerable) | "pulse for exactly N seconds" |

**Decision rule:**
- Immediate timed-on? → **StairwayLS**
- Keep on after input drops? → **OffDelay**
- Wait then pulse? → **OnPulseDelay**

## Fixture Sensors (wire FROM these outputs)

| Sensor | Output | Signal |
|--------|--------|--------|
| Außentemperatur | .AQ | Temperature °C |
| Sonnenschein | .AQ | Sunshine 0/1 |
| Windgeschwindigkeit | .AQ | Wind speed km/h |
| Regen | .AQ | Rain 0/1 |
| Luftfeuchtigkeit | .AQ | Humidity % |
| Helligkeit | .AQ | Brightness lux |
| CO2 Sensor | .AQ | CO2 ppm |
| Bewegungsmelder | .OutputPresence | Motion 0/1 (Flur) |
| Bewegungsmelder Garten | .OutputPresence | Motion 0/1 (Garten) |
| Türkontakt Eingang | .Q | Door contact 0/1 |
| Türklingel | .Q | Doorbell pulse |
| Pool Temperatur | .AQ | Pool temp °C |
| Raumtemperatur Wohnzimmer | .AQ | Room temp °C |
| Raumtemperatur Schlafzimmer | .AQ | Room temp °C |
| Raumtemperatur Bad | .AQ | Room temp °C |
| Raumtemperatur Küche | .AQ | Room temp °C |
| Raumtemperatur Flur | .AQ | Room temp °C |
| Schalter 1 | .Q | Switch 0/1 |
| Schalter 2 | .Q | Switch 0/1 |
| Feuchtesensor Garten | .AQ | Soil moisture % |
| Rauchmelder | .Q | Smoke alarm 0/1 |
| Wassersensor | .Q | Water alarm 0/1 |
| Pegelsensor | .AQ | Water level m |
| Fensterkontakt * | .Q | Window contact per room |
| Türkontakt * | .Q | Door contact per room |
| Solarproduktion | .AQ | PV production W |
| Hausverbrauch | .AQ | House consumption W |
| Stromverbrauch WP | .AQ | Heat pump power W |
| Wärmeleistung WP | .AQ | Heat pump output W |
| Vorlauftemperatur | .AQ | Flow temperature °C |
| Puffertemperatur | .AQ | Buffer tank temp °C |
| Warmwasserspeicher | .AQ | DHW tank temp °C |
| Kesseltemperatur | .AQ | Boiler temp °C |

**IMPORTANT:** Always use existing fixture sensors instead of creating VirtualIn blocks. The fixture has sensors for every room and common HVAC/energy measurements. If you need a sensor that doesn't exist, check the fixture first with `lox config describe`.

## Fixture Actuators (wire TO these inputs)

| Actuator | Room | Key Inputs |
|----------|------|------------|
| Jalousie 1, Jalousie 2 | each room | .InputTriggerDown, .InputTriggerUp, .InputDisable |
| Lichtsteuerung | each room | .I1 (on/off), .Presence, .Brightness |
| Klimaanlage | Wohnzimmer | .toggle |
| Poolpumpe | Garten | .I1 |
| Lüfter Bad | Bad | .I1 |
| Raumregler | Wohnzimmer | .Temp (measured), .Setpoint (target) |
| Heizkörper Bad | Bad | .I1 |
| Garagenlicht | Garage | .I1 |
| Garagentor | Garage | .I1 |
| Gartenbeleuchtung | Garten | .I1 |
| Bewässerungsventil | Garten | .I1 |
| Türöffner | Flur | .I1 |
| Alarmanlage | Flur | .I (StateV — wire trigger to .I) |
| Statusblock | Wohnzimmer | .I (StateV — wire value to .I) |
| Audiozone | Wohnzimmer | .I (StateV — wire trigger to .I) |
| Wallbox | Garage | .I (StateV — wire control to .I) |
| Heizstab | Bad | .I (StateV — wire control to .I) |
| Steckdose | Küche | .I (StateV — wire control to .I) |
| Türschloss | Flur | .I (StateV — wire unlock to .I) |
| Batteriespeicher | Garage | .I (StateV — wire control to .I) |

## Workflow

1. **Add** new blocks with `lox config add`
2. **Set parameters** with `lox config set-param` (thresholds, durations, factors)
3. **Wire connectors** with `lox config wire-connector` (always TARGET first, then SOURCE)
4. **Add schedules** with `lox config timer-schedule` (only for DayTimer blocks)
5. **Test** with `lox sim run FILE --sim '...'` to verify signals propagate correctly
6. **Validate** with `lox config check FILE`

## Worked Examples

### Threshold: Close blinds when sunny and above 25°C

```
lox config add --type GreaterEqual --title "Temp über 25" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config set-param config.Loxone "Temp über 25" Input2 25
lox config add --type And --title "Sonne und Warm" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config wire-connector config.Loxone "Temp über 25.Input1" "Außentemperatur.AQ"
lox config wire-connector config.Loxone "Sonne und Warm.I1" "Sonnenschein.AQ"
lox config wire-connector config.Loxone "Sonne und Warm.I2" "Temp über 25.Q"
lox config wire-connector config.Loxone "Jalousie 1 [Wohnzimmer].InputTriggerDown" "Sonne und Warm.Q"
lox sim run config.Loxone --sim '{"inputs":{"Außentemperatur.AQ":30,"Sonnenschein.AQ":1},"ticks":10,"dt":0.1,"expected_outputs":{"Jalousie 1 [Wohnzimmer].InputTriggerDown":{">":0.5}}}'
lox config check config.Loxone
```

### Combined condition: AND gate

Two digital signals must both be true:

```
lox config add --type And --title "Beide aktiv" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config wire-connector config.Loxone "Beide aktiv.I1" "Bewegungsmelder.OutputPresence"
lox config wire-connector config.Loxone "Beide aktiv.I2" "Schalter 1.Q"
lox config wire-connector config.Loxone "Lichtsteuerung [Wohnzimmer].I1" "Beide aktiv.Q"
```

### Timer/schedule: DayTimer with overnight range

Reduce brightness at night (22:00–06:00) to 30%:

```
lox config add --type DayTimer --title "Nachtzeit" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config timer-schedule config.Loxone "Nachtzeit" "22:00-06:00" --value 30
lox config add --type Mult --title "Nacht Dimmer" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config wire-connector config.Loxone "Nacht Dimmer.Input1" "Nachtzeit.AQ"
lox config wire-connector config.Loxone "Lichtsteuerung [Wohnzimmer].Brightness" "Nacht Dimmer.AQ"
```

### Negation: NOT gate for rain protection

Irrigate only when NOT raining:

```
lox config add --type Not --title "Kein Regen" --room Garten --page Garten config.Loxone
lox config wire-connector config.Loxone "Kein Regen.I" "Regen.AQ"
lox config add --type And --title "Bewässerung aktiv" --room Garten --page Garten config.Loxone
lox config wire-connector config.Loxone "Bewässerung aktiv.I1" "Kein Regen.Q"
lox config wire-connector config.Loxone "Bewässerung aktiv.I2" "Schalter 1.Q"
```

### Presence detection: Motion → light

```
lox config wire-connector config.Loxone "Lichtsteuerung [Flur].Presence" "Bewegungsmelder.OutputPresence"
```

### Stairway light: Timed switch (5 minutes)

```
lox config add --type StairwayLS --title "Treppenlicht" --room Flur --page Flur config.Loxone
lox config set-param config.Loxone "Treppenlicht" TimeHigh 300
lox config wire-connector config.Loxone "Treppenlicht.InputTrigger" "Bewegungsmelder.OutputPresence"
lox config wire-connector config.Loxone "Lichtsteuerung [Flur].I1" "Treppenlicht.Q"
lox config check config.Loxone
```

### Bathroom fan: Keep running after light turns off

Use **OffDelay** (NOT OnPulseDelay) — output stays ON for Time seconds after input goes LOW:

```
lox config add --type OffDelay --title "Lüfter Nachlauf" --room Bad --page Bad config.Loxone
lox config set-param config.Loxone "Lüfter Nachlauf" Time 600
lox config wire-connector config.Loxone "Lüfter Nachlauf.InputTrigger" "Lichtsteuerung [Bad].AQ1"
lox config wire-connector config.Loxone "Lüfter Bad.I1" "Lüfter Nachlauf.Q"
lox config check config.Loxone
```

### CO2 threshold: Open window when CO2 exceeds limit

Always wire the threshold output TO the actuator — don't leave outputs disconnected:

```
lox config add --type GreaterEqual --title "CO2 hoch" --room Küche --page Küche config.Loxone
lox config set-param config.Loxone "CO2 hoch" Input2 1000
lox config wire-connector config.Loxone "CO2 hoch.Input1" "CO2 Sensor.AQ"
lox config wire-connector config.Loxone "Jalousie 1 [Küche].InputTriggerDown" "CO2 hoch.Q"
lox config check config.Loxone
```

### Heating setpoint: Set room temperature to 22°C

Wire room sensor to `.Temp` (NOT `.TempO`!) and setpoint to `.Setpoint`:

```
lox config add --type Constant --title "Sollwert 22" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config set-param config.Loxone "Sollwert 22" Value 22
lox config wire-connector config.Loxone "Raumregler.Temp" "Raumtemperatur Wohnzimmer.AQ"
lox config wire-connector config.Loxone "Raumregler.Setpoint" "Sollwert 22.Q"
lox config check config.Loxone
```

### Memory block: Remember a temperature setting

Use AMemory to store a value (e.g., 21°C) when a switch is pressed:

```
lox config add --type AMemory --title "Temp Speicher" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config set-param config.Loxone "Temp Speicher" Input 21
lox config wire-connector config.Loxone "Temp Speicher.Trigger" "Schalter 1.Q"
lox config wire-connector config.Loxone "Raumregler.Setpoint" "Temp Speicher.AQ"
```

## Common Mistakes

1. **wire-connector argument order**: TARGET first, then SOURCE. `"Target.Input" "Source.Output"` — not the other way around.
2. **DayTimer schedule**: Use `lox config timer-schedule`, NOT `set-param`. The schedule is not a simple parameter.
3. **StairwayLS vs OnPulseDelay**: Use StairwayLS for "turn on for X minutes" (immediate). OnPulseDelay has a DELAY before the pulse starts.
4. **Room-qualify ambiguous names**: If "Jalousie 1" exists in multiple rooms, use `"Jalousie 1 [Wohnzimmer]"` to disambiguate.
5. **Connector names are case-sensitive**: `Input1` not `input1`, `InputTrigger` not `Trigger`, `AQ` not `aq`.
6. **Analog vs digital**: Comparison blocks (GreaterEqual, Less) output digital (0/1). Mult, Add output analog. Wire analog→analog and digital→digital inputs.
7. **Missing parameters**: Always set threshold values (Input2) on comparison blocks. Always set TimeHigh on StairwayLS, Time on Monoflop.
8. **⚠ ALWAYS wire the output**: Every logic block's output (Q or AQ) MUST be wired to an actuator or the next block. A block with unwired output does nothing. After wiring inputs, always wire the output too.
9. **Complete the signal path**: Ensure there is a complete path from source (sensor, schedule, switch) to the final target actuator. Don't leave intermediate outputs disconnected. Run `lox config check` to verify.
10. **⚠ Raumregler connectors — CRITICAL**: The Raumregler (IRC/Thermostat) has these inputs:
    - `.Temp` = measured room temperature ← wire the room temperature SENSOR here
    - `.Setpoint` = desired target temperature ← wire a constant or Memory block here
    - `.TempO` = outside temperature info (for weather compensation only, NOT for room temp!)
    - `.Reset` = reset to default
    - Do NOT wire room temperature to `.TempO`, `.Input`, `.Comfort`, or `.Save` — these are for different purposes.
    - Output: `.AQh` = heating valve demand (0-100%), `.AQc` = cooling demand
11. **⚠ Timer duration parameters — CRITICAL**: When creating StairwayLS, OnPulseDelay, OffDelay, or Monoflop blocks, you MUST set the duration parameter. Without it, the timer defaults to 0 seconds and the signal passes through instantly or not at all.
    - StairwayLS: `set-param FILE "BlockTitle" TimeHigh 300` (5 minutes = 300 seconds)
    - OnPulseDelay: `set-param FILE "BlockTitle" Time 600` (10 minutes = 600 seconds)
    - OffDelay: `set-param FILE "BlockTitle" Time 300`
    - Monoflop: `set-param FILE "BlockTitle" Duration 180`
12. **⚠ Exotic block types — CRITICAL**: When using exotic block types (2Point, SequenceController, AnalogWatchdog, etc.), always check connector names with `lox blocks info <type>` first. Connector names vary between types — don't guess.
13. **⚠ Sensor selection — CRITICAL**: Always wire from RAW fixture sensors, not from controller outputs:
    - Temperature: Wire from `Raumtemperatur <Room>.AQ` (TreeAsensor), NOT from `Raumregler.AQh` (controller output)
    - Wind: Wire from `Windgeschwindigkeit.AQ` (SysVar), NOT from another block's filtered output
    - Humidity: Wire from `Raumfeuchtigkeit <Room>.AQ` (TreeAsensor)
    - Presence: Wire from `Bewegungsmelder.OutputPresence` (PresenceDetector), NOT from door contacts
    - General rule: trace the signal back to a physical sensor (SysVar, TreeSensor, TreeAsensor, VirtualIn) — don't create intermediate blocks between sensor and your logic
14. **⚠ DayTimer requires schedule**: A DayTimer block without `timer-schedule` entries outputs 0 forever. Always add at least one schedule entry after creating a DayTimer.
15. **⚠ Verify EVERY target actuator has a wire**: After building logic, check that every target actuator (VirtualOut, Jalousie, LightController2, etc.) has at least one input wired. Run `lox config check FILE` — errors on unwired inputs mean the circuit is incomplete.
16. **⚠ OffDelay for absence timeout — CRITICAL**: To detect "nobody home for 30 minutes", wire OffDelay from the PRESENCE signal directly (Or.Q), NOT from an inverted absence signal. OffDelay keeps output HIGH for `Time` seconds after input drops to 0 — that's the timeout. Use `EdgeDetection.FallingEdge` on the OffDelay output to fire a pulse when the timeout expires.
    - Correct: `Or(presence) → OffDelay(1800s) → EdgeDetection.FallingEdge → target`
    - Wrong: `Or(presence) → Not → OffDelay` — this inverts the signal so OffDelay never times out
    - Simpler alternative: `Or(presence) → OffDelay(1800s) → Not → target` (3 blocks, no EdgeDetection needed)

## Short Aliases

For convenience, these short aliases are available:

| Alias | Full Command |
|-------|-------------|
| `lox config wire` | `lox config wire-connector` |
| `lox config set` | `lox config set-param` |
| `lox config get` | `lox config get-params` |
| `lox config bind` | `lox config device-bind` |
| `lox config add-vi` | `lox config add-virtual-in` |

## Block Type Reference (221 types)

Use `lox blocks search "keyword"` or `lox blocks info TypeName` to discover blocks.

### Key block types by category

| Category | Types |
|----------|-------|
| **Logic** | And, Or, Not, Xor, FlipFlop, AMemory, Counter |
| **Math** | Add, Sub, Mult, Div, Mod, Avg, Minmax, Constant |
| **Compare** | GreaterEqual, Greater, Less, LessEqual |
| **Timer** | StairwayLS, Monoflop, OnPulseDelay, OffDelay, PulseGen |
| **Schedule** | DayTimer, AlarmClock, PulseAt, PulseBy |
| **State** | State, StateV, AMemory, Counter |
| **Lighting** | LightController2, LightControllerH, BrightnessControl, CentralLight |
| **Shading** | JalousieUpDown2, AutoJalousie, DaylightController, CentralShade |
| **HVAC** | HeatIRoomController2, Heatmixer2, HVACController, Ventilation, Ventilation2 |
| **Security** | Alarm, AlarmChain, CentralAlarm, SmokeAlarm, WindowsMonitor, Doorcontroller |
| **Energy** | EnergyManager2, SpotOpt, LoadShed, Wallbox |
| **I/O** | InputRef, OutputRef, VirtualIn, VirtualOut |
| **Misc** | StatusMonitor, Presence, PresenceController, Irrigation |

### Advanced Block Types

Connector reference for less common block types. Always verify with `lox blocks info <type>`.

#### 2Point — On/off controller with hysteresis

- **Inputs:** `Input` (analog value to control)
- **Outputs:** `Q` (digital on/off)
- **Params:** `On` (threshold to turn on), `Off` (threshold to turn off), `PulseTime`, `MinRuntime`
- **Use case:** Floor heating on at 19°C, off at 21°C

```
lox config add --type 2Point --title "Heizung" config.Loxone
lox config set-param config.Loxone "Heizung" On 19
lox config set-param config.Loxone "Heizung" Off 21
lox config wire-connector config.Loxone "Heizung.Input" "Raumtemperatur Wohnzimmer.AQ"
```

#### 3Point — 3-point controller (heat/cool)

- **Inputs:** `Input`
- **Outputs:** `Q1` (heat), `Q2` (cool)
- **Params:** `On1`, `Off1` (heat band), `On2`, `Off2` (cool band)

#### UpDownCounter — Bidirectional counter

- **Inputs:** `Trigger` (count pulse), `InputDir` (0=up, 1=down), `Reset`
- **Outputs:** `AQ` (count value), `Q` (output)
- **Params:** `StartValue` (default 0), `OnValue` (default 10), `OffValue` (default 5)

#### HourCounter — Operating hours tracker

- **Inputs:** `InputEnable` (count while high), `Reset`
- **Outputs:** `AQ` (hours), `Q` (threshold exceeded)
- **Params:** `Maintenance` (hour threshold)

#### SequenceController — Text-programmable sequence automation

The SequenceController executes a text program line-by-line. Use it for multi-step processes (irrigation zones, heating sequences, startup procedures).

- **Inputs:** `Trigger1`–`Trigger8` (start sequence 1-8), `AI1`–`AI8` (analog inputs), `ATrigger` (select sequence by number), `Off` (reset/lock)
- **Outputs:** `AQ1`–`AQ8` (analog outputs), `OutputCurrSequence`, `OutputCurrLine`, `TQ` (text)
- **Params:** `Interval` (execution speed, default 500ms)

**⚠ CRITICAL: A SequenceController without a program does NOTHING.** After creating the block, you MUST write a program:

```bash
lox config set-program FILE "Block Title" "sequence 1
set AQ1 = 1
sleep 15 m
set AQ1 = 0
set AQ2 = 1
sleep 15 m
set AQ2 = 0"
```

**Program commands:**
- `set AQ1 = AI1 + 3` — set output/variable to expression
- `sleep 15 m` or `sleep 300 s` — wait (minutes or seconds)
- `waitcondition AI1 > 5` — wait until condition is true
- `if AQ1 > 3` ... `endif` — conditional block
- `goto 1` — jump to line
- `startsequence 2` / `return` — call another sequence
- `setpulse AQ1` — one-tick pulse
- `// comment` — documentation

#### MultiClick — Detect single/double/triple clicks

- **Inputs:** `InputTrigger`
- **Outputs:** `Q1` (single), `Q2` (double), `Q3` (triple), `Q4` (quad), `AQ` (value output)
- **Params:** `Max` (time window for multi-click, default 0.35s), `Time` (output on-duration, default 0.1s)

#### MultiFuncSW — Multi-function switch

- **Inputs:** `InputTrigger`
- **Outputs:** `Q` (toggle output)
- **Params:** `TimeHigh` (on-duration, default 180s), `TimeTrigger` (long-click threshold, default 0.5s)

#### Radio — Radio button group (mutual exclusion)

- **Inputs:** `InputTrigger 1`–`InputTrigger 8`, `InputTriggerP` (+), `InputTriggerM` (-), `InputSel` (direct select)
- **Outputs:** `AQ` (selected index), `Q1`–`Q8` (individual)
- Note: connector names have spaces — always quote: `"Radio.InputTrigger 1"`

#### StepSel — Step selector (cycle through values)

- **Inputs:** `InputTriggerP` (step up), `InputTriggerM` (step down), `InputSel` (direct select 0-16)
- **Outputs:** `Q1`–`Q8` (active step), `AQ` (current step number)
- **Params:** `Mode`

#### EibDimmer — KNX dimmer

- **Inputs:** `InputTrigger 1` (up/brighter), `InputTrigger 2` (down/dimmer), `InputTrigger On`, `InputTrigger Off`, `Position` (set value 0-100), `InputDisable`
- **Outputs:** `AQ` (dimmer value), `Q` (on/off state)
- Note: connector names have spaces — always quote: `"EibDimmer.InputTrigger 1"`

#### Shift — Shift register

- **Inputs:** `Trigger` (clock pulse), `InputData` (data bit), `InputDir` (shift direction)
- **Outputs:** `Q` (register output)
- **Params:** `Bit` (register size, default 8)

#### AnalogWatchdog — Monitor analog value range

- **Inputs:** `Input`
- **Outputs:** `Q` (alarm: out of range)
- **Params:** `Min`, `Max`, `Time` (delay before alarm)

#### AnalogScaler — Scale analog value

- **Inputs:** `Input`
- **Outputs:** `AQ`
- **Params:** `InMin`, `InMax`, `OutMin`, `OutMax`

#### BrightnessControl — Constant-lux controller

- **Inputs:** `InputBrightness` (measured), `InputEnable`
- **Outputs:** `AQ` (dimmer value 0–100%)
- **Params:** `Target` (lux), `Min`, `Max`

#### AnalogMultiplexer — Select between 4 analog inputs

- **Inputs:** `Input1`, `Input2`, `Input3`, `Input4`
- **Outputs:** `AQ`
- **Params/Select:** **1-based** — `0` → output 0 (off), `1` → Input1, … `4` → Input4.
- **Use case:** 3+ mode setpoint selection (comfort/eco/off)

#### AnalogMultiplexer2 — Select between 2 analog inputs

- **Inputs:** `Input1`, `Input2`
- **Outputs:** `AQ`
- **Select is 1-based** (matches AnalogMultiplexer): `0` → output 0 (off),
  `1` → Input1, `2` → Input2. **Foot-gun:** a digital 0/1 signal selects
  off/Input1, never Input2 — feed `Select = override ? 2 : 1` to switch between
  the two inputs. The simulator prints a warning when a `Select` input is wired,
  as a reminder of the 1-based convention.
- **Note:** Only 2 inputs. For 3+ modes, use AnalogMultiplexer (4-way)


#### Ramp — Gradual value change

- **Inputs:** `InputTrigger`, `InputReset`
- **Outputs:** `AQ` (0→100% over time)
- **Params:** `RampTime` (seconds)

#### RandomGen — Random number generator

- **Inputs:** `InputTrigger`
- **Outputs:** `AQ`
- **Params:** `Min`, `Max`

#### Counter — Up counter with target

- **Inputs:** `Trigger` (count pulse)
- **Outputs:** `Q` (1 when count reaches EndValue), `AQ` (current count)
- **Params:** `EndValue` (target count, default 1000), `Mode`
- **Use case:** Count door openings, count events before triggering action

#### EdgeDetection — Detect signal edges

- **Inputs:** `Input`
- **Outputs:** `Edge` (pulse on any edge), `RisingEdge` (pulse on 0→1), `FallingEdge` (pulse on 1→0)
- **Params:** `PulseTime` (output pulse duration, default 1s)
- **Use case:** Detect door opening (rising edge), generate trigger pulses from sensor changes

#### OnDelay — Switch-on delay

- **Inputs:** `InputTrigger`
- **Outputs:** `Q` (delayed output)
- **Params:** `Time` (delay duration in seconds, default 1)
- **Use case:** Delay a signal by N seconds (e.g. turn on kitchen light 2 min after alarm)

#### SaunaVapor — Sauna controller with evaporator

- **Inputs:** `Trigger` (toggle on/off), `Input` (target temp °C), `Temp` (measured temp °C), `InputH` (target humidity), `Humidity` (measured humidity), `DoorSensor`, `On` (state)
- **Outputs:** `AQ` (heater output 0–10V), `Qa` (sauna state on/off), `Qe` (safety shutdown), `Qr` (sauna ready)
- **Params:** `SafeTemp` (safety shutdown temp, default 139°C), `Time` (operating time, default 600s), `SafeTime` (max safety time, default 7200s), `DryTemp` (drying temp, default 70°C)
- **Use case:** Sauna with auto-off timer and temperature limit

#### HvacAC — AC central controller

- **Inputs:** `Temp` (outdoor temperature °C), `Stop`, `CoolAvailable`, `HeatAvailable`
- **Outputs:** `OutHeat1` (heating demand), `OutCool1` (cooling demand), `OutMaintenance`
- **Params:** `Mode` (0–3), `TempLimitC` (temp above which cooling allowed, default 18°C), `TempLimitH` (temp below which heating allowed, default 15°C)
- **Use case:** Central HVAC enable/disable based on outdoor temperature

#### MeterPBi — Bidirectional pulse meter

- **Inputs:** `P1` (consumption pulses), `P2` (delivery/export pulses), `F1` (consumption frequency), `F2` (delivery frequency)
- **Outputs:** `OMr1` (consumption meter reading), `OMr2` (delivery meter reading), `OPf` (power)
- **Params:** `Np1` (pulses per unit consumption), `Np2` (pulses per unit delivery)
- **Use case:** Track solar import/export separately

#### VirtualIn — Virtual input (switch/value)

- **Inputs:** (none — triggered from UI or API)
- **Outputs:** `AQ` (current value)
- **Use case:** Create UI buttons, API triggers, or test inputs
- **Note:** Use `Analog="true"` for analog values, `Analog="false"` for digital switches
