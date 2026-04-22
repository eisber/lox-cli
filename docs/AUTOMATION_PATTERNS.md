# Loxone Automation Patterns

Reference guide for AI agents building automation logic. Each pattern maps a human intent to the specific blocks, wiring, and parameters needed.

## How to Read This

Each pattern shows:
- **Human says**: natural language that triggers this pattern
- **Blocks**: what to create with `lox config add`
- **Wiring**: what to connect with `lox config wire-connector`
- **Parameters**: what to set with `lox config set-param`
- **Example**: complete CLI commands

---

## Pattern 1: Timed Switch (StairwayLS)

**Human says**: "Turn on X for N minutes when button is pressed"

**Use cases**: Radiator timer, bathroom fan, porch light, garage light

**Blocks**:
- `StairwayLS` — timed output: goes high on trigger, auto-off after TimeHigh seconds

**Wiring**:
```
Button.AQ ──→ StairwayLS.InputTrigger
               StairwayLS.Q ──→ Actor.I (light, relay, valve)
```

**Parameters**:
- `TimeHigh`: on-duration in seconds (e.g., 600 = 10 min)
- `TimeRetrigger`: if true, button press during active resets timer

**CLI Example**:
```bash
lox config add --type StairwayLS --title "Badlüfter" --room "Bad" --page "Bad" config.Loxone
lox config set-param config.Loxone "Badlüfter" TimeHigh 600
lox config wire-connector config.Loxone "Badlüfter.InputTrigger" <button-aq-uuid>
lox config wire-connector config.Loxone "uuid:<fan-actor>.I" <stairwayls-q-uuid>
```

**Variant — configurable duration**: Add `Mult` to scale a VirtualIn value:
```
VirtualIn (duration) ──→ Mult.Input1
                          Mult.Input2 = 60 (minutes→seconds)
                          Mult.AQ ──→ StairwayLS.TimeHigh
```

---

## Pattern 2: Threshold Trigger (GreaterEqual / Less)

**Human says**: "When temperature exceeds N degrees, do X" or "When value drops below N, do Y"

**Use cases**: Overheat protection, frost alarm, wind protection, rain blinds

**Blocks**:
- `GreaterEqual` — output Q=1 when Input1 >= Input2
- `Less` — output Q=1 when Input1 < Input2

**Wiring**:
```
Sensor.AQ ──→ GreaterEqual.Input1
               GreaterEqual.Input2 = threshold (as Def param)
               GreaterEqual.Q ──→ target action
```

**CLI Example** (close blinds when wind > 40 km/h):
```bash
lox config add --type GreaterEqual --title "Windschutz" --room "Wohnzimmer" --page "Wohnzimmer" config.Loxone
lox config set-param config.Loxone "Windschutz" Input2 40
lox config wire-connector config.Loxone "Windschutz.Input1" <wind-speed-aq-uuid>
lox config wire-connector config.Loxone "uuid:<jalousie>.InputTriggerDown" <windschutz-q-uuid>
```

---

## Pattern 3: Combined Condition (And / Or)

**Human says**: "When A AND B, do X" or "When either A or B, do X"

**Use cases**: Sunny + hot → shade, motion + dark → light, alarm + night → alert

**Blocks**:
- `And` — Q=1 when all inputs are 1
- `Or` — Q=1 when any input is 1

**Wiring**:
```
Condition A ──→ And.I1
Condition B ──→ And.I2
                And.Q ──→ target action
```

**CLI Example** (piano protection: sunny AND > 20°C):
```bash
lox config add --type GreaterEqual --title "Temp über 20" --page "Wohnzimmer" config.Loxone
lox config set-param config.Loxone "Temp über 20" Input2 20
lox config wire-connector config.Loxone "Temp über 20.Input1" <outdoor-temp-aq>

lox config add --type And --title "Piano Schutz" --page "Wohnzimmer" config.Loxone
lox config wire-connector config.Loxone "Piano Schutz.I1" <sunshine-aq>
lox config wire-connector config.Loxone "Piano Schutz.I2" <greaterequal-q>
lox config wire-connector config.Loxone "uuid:<jalousie-1>.InputTriggerDown" <and-q>
```

---

## Pattern 4: Time Window (GreaterEqual + Less + And/Or)

**Human says**: "Between 10pm and 6am, do X" or "Only during daytime"

**Use cases**: Night mode brightness, scheduled irrigation, quiet hours

**Blocks**:
- `GreaterEqual` — "after start time"
- `Less` — "before end time"
- `And` — both conditions (for daytime: start AND before end)
- `Or` — either condition (for nighttime: after 22:00 OR before 06:00)

**Important**: Use "Minuten seit Mitternacht" (Minutes Past Midnight) SysVar. Values: 0-1439.
- 06:00 = 360, 10:00 = 600, 18:00 = 1080, 22:00 = 1320

**Wiring (daytime 06:00-22:00)**:
```
MinutesPastMidnight ──→ GreaterEqual.Input1 (Input2=360)
MinutesPastMidnight ──→ Less.Input1 (Input2=1320)
GreaterEqual.Q ──→ And.I1
Less.Q ──→ And.I2
And.Q = 1 during daytime
```

**Wiring (nighttime 22:00-06:00)** — use `Or` since it wraps midnight:
```
MinutesPastMidnight ──→ GreaterEqual.Input1 (Input2=1320)  "after 22:00"
MinutesPastMidnight ──→ Less.Input1 (Input2=360)            "before 06:00"
GreaterEqual.Q ──→ Or.I1
Less.Q ──→ Or.I2
Or.Q = 1 during nighttime
```

**Scaling output** — use `Mult` to convert digital (0/1) to analog value:
```
And.Q (0 or 1) ──→ Mult.Input1
                    Mult.Input2 = 30 (target value)
                    Mult.AQ = 0 or 30
```

---

## Pattern 5: Delayed Action (OnPulseDelay / OffDelay)

**Human says**: "Wait N seconds after X, then do Y" or "Keep running for N seconds after trigger stops"

**Use cases**: Bathroom fan after light off, stairwell light extension, dehumidifier

**Blocks**:
- `OnPulseDelay` — delays the rising edge (trigger → wait → output goes high)
- `OffDelay` — delays the falling edge (trigger stops → wait → output goes low)

**Wiring**:
```
Trigger ──→ OffDelay.InputTrigger
             OffDelay.T = delay in seconds
             OffDelay.Q ──→ Actor.I
```

**CLI Example** (bathroom fan runs 10 min after light off):
```bash
lox config add --type OffDelay --title "Lüfter Nachlauf" --room "Bad" --page "Bad" config.Loxone
lox config set-param config.Loxone "Lüfter Nachlauf" T 600
lox config wire-connector config.Loxone "Lüfter Nachlauf.InputTrigger" <light-controller-aq>
lox config wire-connector config.Loxone "uuid:<fan-actor>.I" <offdelay-q>
```

---

## Pattern 6: Automatic Shading (AutoJalousie)

**Human says**: "Automatically lower blinds when sunny, raise when cloudy"

**Use cases**: Sun protection, energy saving, comfort

**Blocks**:
- `AutoJalousie` — built-in sun tracking with wind/rain safety

**Key inputs**:
- `InputTrigger` — manual toggle (button press enables/disables auto mode)
- Sun position, brightness, wind speed come from system weather data

**Key outputs**:
- `OutputUp` — pulse when blinds should open
- `OutputDown` — pulse when blinds should close

**Wiring**:
```
Button ──→ AutoJalousie.InputTrigger (toggle auto mode)
AutoJalousie.OutputDown ──→ Jalousie.InputTriggerDown
AutoJalousie.OutputUp ──→ Jalousie.InputTriggerUp
```

---

## Pattern 7: Presence-Based Lighting

**Human says**: "Turn on light when someone enters, off when they leave"

**Use cases**: Hallway, bathroom, garage, stairwell

**Wiring** (simplest — direct connection):
```
PresenceDetector.Q ──→ LightController2.Move
PresenceDetector.Brightness ──→ LightController2.Brightness
```

The LightController2's `Move` input triggers the "movement" mood. The `MoveTimeout` parameter controls how long the light stays on after last motion (default 3600s = 1h, set lower for hallways: 120s).

**CLI Example**:
```bash
lox config set-param config.Loxone "Lichtsteuerung [Flur]" MoveTimeout 120
lox config wire-connector config.Loxone "Lichtsteuerung [Flur].Move" <presence-q-uuid>
```

---

## Pattern 8: Climate Control Chain

**Human says**: "Control heating/cooling in the room based on temperature"

**Use cases**: Room temperature regulation, AC control, floor heating

**Blocks**:
- `HeatIRoomController2` — intelligent room controller (PID regulation)
- `AcControl` — air conditioning interface
- `PushButton` — manual temperature override toggle

**Wiring**:
```
TempSensor.AQ ──→ HeatIRoomController2.Temp
PushButton.Q ──→ HeatIRoomController2.Reset
HeatIRoomController2.AQh ──→ HeatingValve.I (via OutputRef)
```

For AC:
```
TempSensor.AQ ──→ AcControl.inTempCurr
AcControl.fan ──→ State "Fan-Auswahl".I1
AcControl.mode ──→ State "Mode-Auswahl".I1
AcControl.status ──→ OutputRef → EIBactor (KNX commands)
```

---

## Pattern 9: Scheduled Action (DayTimer)

**Human says**: "Water the garden every day at 6am for 30 minutes"

**Blocks**:
- `DayTimer` — configurable schedule with per-day/per-mode times

**Note**: DayTimer has built-in calendar integration. The schedule is configured via the Loxone app, not via XML parameters.

**Wiring**:
```
DayTimer.Q ──→ And.I1 (combine with conditions)
NOT Rain ──→ And.I2
And.Q ──→ Irrigation.On
```

---

## Pattern 10: Toggle with Memory (PushButton / PushButton2)

**Human says**: "Toggle X on/off with a button press" or "Enable/disable feature with a switch"

**Blocks**:
- `PushButton` — simple toggle (press → on, press again → off)
- `PushButton2` — toggle with separate on/off outputs

**Wiring**:
```
Button.AQ ──→ PushButton.I
PushButton.Q ──→ target (1 when on, 0 when off)
```

---

## Common Sensors & Their Connectors

| Sensor | Output Connector | Type | What It Provides |
|--------|-----------------|------|------------------|
| SysVar "Außentemperatur" | AQ | Analog | Outdoor temp in °C |
| SysVar "Sonnenschein" | AQ | Digital | 1=sunny, 0=cloudy |
| SysVar "Windgeschwindigkeit" | AQ | Analog | Wind speed km/h |
| SysVar "Regen" | AQ | Digital | 1=raining |
| Time "Minuten seit Mitternacht" | Q | Analog | 0-1439 |
| PresenceDetector | Q (motion), Brightness | Mixed | Presence + light level |
| Temperature sensor | AQ | Analog | Room temp °C |

## Common Actuator Inputs

| Actuator | Input Connector | Type | What It Does |
|----------|----------------|------|-------------|
| LightController2 | Move | Digital | Trigger movement mood |
| LightController2 | Sel1-Sel8 | Digital | Select specific mood |
| LightController2 | Reset | Digital | All off |
| LightController2 | InputDisable | Digital | Lock control |
| JalousieUpDown2/EIBJalousie | InputTriggerUp | Digital | Open blinds |
| JalousieUpDown2/EIBJalousie | InputTriggerDown | Digital | Close blinds |
| JalousieUpDown2/EIBJalousie | InputPos | Analog | Set position 0-100% |
| JalousieUpDown2/EIBJalousie | InputDisable | Digital | Lock blinds |
| HeatIRoomController2 | Temp | Analog | Current room temp |
| HeatIRoomController2 | Reset | Digital | Reset to default |

---

## Pattern 11: Signal Negation (Not)

**Human says**: "Do X but NOT when Y" or "Only when it's NOT raining"

**Block**: `Not` — inverts a digital signal (1 becomes 0, 0 becomes 1)

**Wiring**:
```
Rain.AQ (1=raining) --> Not.I
                         Not.Q (1=NOT raining) --> And.I2
```

**CLI Example** (irrigate only when NOT raining):
```bash
lox config add --type Not --title "Kein Regen" --page "Garten" config.Loxone
lox config wire-connector config.Loxone "Kein Regen.I" <rain-aq-uuid>
# Not.Q is now 1 when dry, 0 when raining
```

---

## Pattern 12: Multi-Target / Fan-Out

**Human says**: "Close ALL blinds" or "Turn off ALL lights in the house"

**Approach**: One output can be wired to multiple inputs. Use `lox config controls` without `--room` to find all targets, then wire each one.

**CLI Example** (wind protection for all blinds):
```bash
# 1. Create threshold
lox config add --type GreaterEqual --title "Sturmwarnung" --page "Zentral" config.Loxone
lox config set-param config.Loxone "Sturmwarnung" Input2 50
lox config wire-connector config.Loxone "Sturmwarnung.Input1" <wind-speed-aq>

# 2. Get the threshold Q UUID
STORM_Q=$(lox config control describe config.Loxone "Sturmwarnung" | grep "Q ->" | awk '{print $3}')

# 3. Find ALL blinds across the house (no --room filter)
lox config controls config.Loxone -t JalousieUpDown2
lox config controls config.Loxone -t EIBJalousie

# 4. Wire each blind (same source UUID reused for fan-out)
lox config wire-connector config.Loxone "uuid:<blind-1>.InputTriggerDown" "$STORM_Q"
lox config wire-connector config.Loxone "uuid:<blind-2>.InputTriggerDown" "$STORM_Q"
# ... repeat for each blind
```

**Key**: The same source UUID can be wired to multiple targets. The Miniserver handles fan-out natively.

---

## Pattern 13: Analog Dimming / Brightness Control

**Human says**: "Dim the light to 30%" or "Set brightness based on time of day"

**Approach**: Wire an analog value (0-100) to `LightController2.Brightness`.
Use `Mult` to convert a digital condition (0/1) to a target percentage.

**Wiring (time-based dimming)**:
```
Night condition (0/1) --> Mult.Input1
                          Mult.Input2 = 20 (target %)
                          Mult.AQ --> LightController2.Brightness
```

`LightController2.Brightness` is a master dimming input (0-100%).

**CLI Example** (20% brightness at night):
```bash
# Create nighttime condition (see Pattern 4 for full time window)
# ... GreaterEqual (>=1320) + Less (<360) + Or -> nighttime Q

lox config add --type Mult --title "Nacht Dimmer" --room "Schlafzimmer" --page "Schlafzimmer" config.Loxone
lox config set-param config.Loxone "Nacht Dimmer" Input2 20
lox config wire-connector config.Loxone "Nacht Dimmer.Input1" <nighttime-or-q-uuid>
lox config wire-connector config.Loxone "Lichtsteuerung [Schlafzimmer].Brightness" <mult-aq-uuid>
```

---

## Pattern 14: Override / Manual Disable

**Human says**: "Automatic blinds, but I can override manually" or "Disable automation with a switch"

**Wiring**:
```
PushButton "Override" --> target.InputDisable
```

When `InputDisable` is high, the block ignores all other inputs.
Most controllers have `InputDisable`: LightController2, JalousieUpDown2, HeatIRoomController2.

---

## Workflow Notes

### Always back up first
```bash
cp config.Loxone config.Loxone.bak
```

### Always specify --page and --room when adding blocks
```bash
lox config add --type And --title "My Logic" --room "Wohnzimmer" --page "Wohnzimmer" config.Loxone
```
Omitting `--page` causes blocks to appear outside the visual area in Loxone Config UX.

### Multi-block chains: create ALL blocks first, then wire
Wiring requires connector UUIDs that only exist after creation. For chains A -> B -> C:
1. Create A, B, C
2. Get A.Q, B.Q UUIDs via `lox config control describe`
3. Wire A.Q -> B.I, B.Q -> C.I

### DayTimer limitation
DayTimer schedules cannot be configured via CLI. The schedule is set in the Loxone app after deployment. The CLI can create the block and wire its output, but timing must be configured manually.

### Validate frequently
Run `lox config validate` after adding blocks AND after wiring. Common errors:
- "not found" -> typo in selector, use `uuid:` prefix
- "matches N elements" -> ambiguous name, use bracket syntax `"Name [Room]"`
- "wired TO Output" -> reversed wiring direction
