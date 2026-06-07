---
name: loxone-patterns
description: Reference patterns for Loxone automation. Threshold, AND/OR logic, timers, schedules, presence, irrigation, blinds, HVAC. Use as recipe book.
license: AGPL-3.0
metadata:
  author: eisber
  version: "1.0"
user-invocable: false
---

# Loxone Automation Patterns

Recipe book of common automation patterns. Each pattern includes when to use it, which blocks are needed, copy-paste CLI commands, and a simulator test spec.

Replace `config.Loxone` with your actual config file path.

---

## 1. Threshold

**When**: Trigger an action when a sensor value crosses a limit (temperature, humidity, brightness, wind speed, CO2, etc.)

**Blocks**: `GreaterEqual` (≥) or `Less` (<) or `Greater` (>)

**Circuit**: Sensor.AQ → Comparison.Input1 | set Input2=threshold | Comparison.Q → Actuator

```bash
lox config add --type GreaterEqual --title "Temp über 25" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config set-param config.Loxone "Temp über 25" Input2 25
lox config wire-connector config.Loxone "Temp über 25.Input1" "Außentemperatur.AQ"
lox config wire-connector config.Loxone "Jalousie 1 [Wohnzimmer].InputTriggerDown" "Temp über 25.Q"
```

**Sim test**:

```bash
lox sim run config.Loxone --sim '{"name":"above threshold","inputs":{"Außentemperatur.AQ":30},"ticks":10,"dt":0.1,"expected_outputs":{"Temp über 25.Q":{">":0.5}}}'
lox sim run config.Loxone --sim '{"name":"below threshold","inputs":{"Außentemperatur.AQ":20},"ticks":10,"dt":0.1,"expected_outputs":{"Temp über 25.Q":{"==":0}}}'
```

---

## 2. Combined Condition (AND/OR)

**When**: Multiple conditions must be true simultaneously (AND) or any one suffices (OR).

**Blocks**: `And` or `Or` + comparison blocks as needed

**Circuit**: ConditionA.Q → And.I1, ConditionB.Q → And.I2, And.Q → Actuator

```bash
lox config add --type GreaterEqual --title "Temp über 20" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config set-param config.Loxone "Temp über 20" Input2 20
lox config wire-connector config.Loxone "Temp über 20.Input1" "Außentemperatur.AQ"
lox config add --type And --title "Sonne und Warm" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config wire-connector config.Loxone "Sonne und Warm.I1" "Sonnenschein.AQ"
lox config wire-connector config.Loxone "Sonne und Warm.I2" "Temp über 20.Q"
lox config wire-connector config.Loxone "Jalousie 1 [Wohnzimmer].InputTriggerDown" "Sonne und Warm.Q"
```

**Sim test**:

```bash
lox sim run config.Loxone --sim '{"name":"both true","inputs":{"Außentemperatur.AQ":25,"Sonnenschein.AQ":1},"ticks":10,"dt":0.1,"expected_outputs":{"Sonne und Warm.Q":{">":0.5}}}'
lox sim run config.Loxone --sim '{"name":"only one true","inputs":{"Außentemperatur.AQ":15,"Sonnenschein.AQ":1},"ticks":10,"dt":0.1,"expected_outputs":{"Sonne und Warm.Q":{"==":0}}}'
```

---

## 3. Timed Switch (Stairway Light)

**When**: Turn something on for a fixed duration after a trigger pulse (light on for 5 minutes after motion).

**Blocks**: `StairwayLS`

**Key param**: `TimeHigh` = duration in seconds

**Circuit**: Trigger.Q → StairwayLS.InputTrigger | set TimeHigh=seconds | StairwayLS.Q → Actuator

```bash
lox config add --type StairwayLS --title "Treppenlicht" --room Flur --page Flur config.Loxone
lox config set-param config.Loxone "Treppenlicht" TimeHigh 300
lox config wire-connector config.Loxone "Treppenlicht.InputTrigger" "Bewegungsmelder.OutputPresence"
lox config wire-connector config.Loxone "Lichtsteuerung [Flur].I1" "Treppenlicht.Q"
```

**Sim test**:

```bash
lox sim run config.Loxone --sim '{"name":"motion triggers","inputs":{"Bewegungsmelder.OutputPresence":1},"ticks":10,"dt":0.1,"expected_outputs":{"Treppenlicht.Q":{">":0.5}}}'
lox sim run config.Loxone --sim '{"name":"no motion","inputs":{"Bewegungsmelder.OutputPresence":0},"ticks":10,"dt":0.1,"expected_outputs":{"Treppenlicht.Q":{"==":0}}}'
```

**⚠️ Do NOT use OnPulseDelay for this**: OnPulseDelay has a delay *before* the pulse starts. StairwayLS turns on immediately.

---

## 4. Delayed Action

**When**: Wait before activating something, then run for a duration (e.g. wait 10s, then pump for 30s).

**Blocks**: `OnPulseDelay`

**Key params**: `Delay` = wait time in seconds, `Duration` = pulse length in seconds

**Circuit**: Trigger.Q → OnPulseDelay.InputTrigger | set Delay + Duration | OnPulseDelay.Q → Actuator

```bash
lox config add --type OnPulseDelay --title "Verzögerte Pumpe" --room Garten --page Garten config.Loxone
lox config set-param config.Loxone "Verzögerte Pumpe" Delay 10
lox config set-param config.Loxone "Verzögerte Pumpe" Duration 30
lox config wire-connector config.Loxone "Verzögerte Pumpe.InputTrigger" "Schalter 1.Q"
lox config wire-connector config.Loxone "Poolpumpe.I1" "Verzögerte Pumpe.Q"
```

---

## 5. Schedule (DayTimer)

**When**: Time-based automation — activate during specific hours (night dimming, irrigation windows, quiet hours).

**Blocks**: `DayTimer` + optionally `Mult` for value scaling

**Circuit**: DayTimer.AQ → Mult.Input1 | set Mult.Input2=factor | Mult.AQ → Actuator

```bash
lox config add --type DayTimer --title "Nachtmodus" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config timer-schedule config.Loxone "Nachtmodus" "22:00-06:00" --value 30
lox config add --type Mult --title "Nacht Dimmer" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config wire-connector config.Loxone "Nacht Dimmer.Input1" "Nachtmodus.AQ"
lox config wire-connector config.Loxone "Lichtsteuerung [Wohnzimmer].Brightness" "Nacht Dimmer.AQ"
```

**Note**: Overnight ranges like `"22:00-06:00"` are handled correctly — they wrap across midnight.

---

## 6. Negation (NOT gate)

**When**: Invert a condition — do something when a sensor is NOT active (irrigate when NOT raining, open blinds when NOT sunny).

**Blocks**: `Not`, often combined with `And`

**Circuit**: Sensor.Q → Not.I | Not.Q → And.I1 (combine with other conditions)

```bash
lox config add --type Not --title "Kein Regen" --room Garten --page Garten config.Loxone
lox config wire-connector config.Loxone "Kein Regen.I" "Regen.AQ"
```

**Sim test**:

```bash
lox sim run config.Loxone --sim '{"name":"raining","inputs":{"Regen.AQ":1},"ticks":10,"dt":0.1,"expected_outputs":{"Kein Regen.Q":{"==":0}}}'
lox sim run config.Loxone --sim '{"name":"dry","inputs":{"Regen.AQ":0},"ticks":10,"dt":0.1,"expected_outputs":{"Kein Regen.Q":{">":0.5}}}'
```

---

## 7. Memory / Latch

**When**: Remember a state — set with one button, reset with another (alarm armed/disarmed, mode toggle, irrigation started/stopped).

**Blocks**: `Memory`

**Key inputs**: `S` (set → Q=1), `R` (reset → Q=0)

**Circuit**: SetTrigger → Memory.S, ResetTrigger → Memory.R, Memory.Q → Actuator

```bash
lox config add --type Memory --title "Bewässerung Merker" --room Garten --page Garten config.Loxone
lox config wire-connector config.Loxone "Bewässerung Merker.S" "Schalter 1.Q"
lox config wire-connector config.Loxone "Bewässerung Merker.R" "Regen.AQ"
```

**Sim test**:

```bash
lox sim run config.Loxone --sim '{"name":"set","inputs":{"Schalter 1.Q":1,"Regen.AQ":0},"ticks":10,"dt":0.1,"expected_outputs":{"Bewässerung Merker.Q":{">":0.5}}}'
lox sim run config.Loxone --sim '{"name":"reset by rain","inputs":{"Schalter 1.Q":0,"Regen.AQ":1},"ticks":10,"dt":0.1,"expected_outputs":{"Bewässerung Merker.Q":{"==":0}}}'
```

---

## 8. Frost Protection

**When**: Protect blinds/shutters from freezing by raising them when temperature drops below 0°C.

**Blocks**: `Less`

**Circuit**: Außentemperatur.AQ → Less.Input1 | set Input2=0 | Less.Q → Jalousie.InputTriggerUp

```bash
lox config add --type Less --title "Frostschutz" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config set-param config.Loxone "Frostschutz" Input2 0
lox config wire-connector config.Loxone "Frostschutz.Input1" "Außentemperatur.AQ"
lox config wire-connector config.Loxone "Jalousie 1 [Wohnzimmer].InputTriggerUp" "Frostschutz.Q"
```

**Sim test**:

```bash
lox sim run config.Loxone --sim '{"name":"freezing","inputs":{"Außentemperatur.AQ":-5},"ticks":10,"dt":0.1,"expected_outputs":{"Frostschutz.Q":{">":0.5}}}'
lox sim run config.Loxone --sim '{"name":"warm","inputs":{"Außentemperatur.AQ":10},"ticks":10,"dt":0.1,"expected_outputs":{"Frostschutz.Q":{"==":0}}}'
```

---

## 9. Wind Protection

**When**: Retract all blinds when wind exceeds a safety threshold (e.g. 40 km/h).

**Blocks**: `GreaterEqual`

**Circuit**: Windgeschwindigkeit.AQ → GreaterEqual.Input1 | set Input2=40 | Q → Jalousie.InputTriggerUp

```bash
lox config add --type GreaterEqual --title "Windschutz" --room Wohnzimmer --page Wohnzimmer config.Loxone
lox config set-param config.Loxone "Windschutz" Input2 40
lox config wire-connector config.Loxone "Windschutz.Input1" "Windgeschwindigkeit.AQ"
lox config wire-connector config.Loxone "Jalousie 1 [Wohnzimmer].InputTriggerUp" "Windschutz.Q"
lox config wire-connector config.Loxone "Jalousie 2 [Wohnzimmer].InputTriggerUp" "Windschutz.Q"
```

**Sim test**:

```bash
lox sim run config.Loxone --sim '{"name":"strong wind","inputs":{"Windgeschwindigkeit.AQ":50},"ticks":10,"dt":0.1,"expected_outputs":{"Windschutz.Q":{">":0.5}}}'
lox sim run config.Loxone --sim '{"name":"calm","inputs":{"Windgeschwindigkeit.AQ":10},"ticks":10,"dt":0.1,"expected_outputs":{"Windschutz.Q":{"==":0}}}'
```

---

## 10. Irrigation (Moisture + Rain Guard)

**When**: Water plants when soil moisture is low AND it's not raining.

**Blocks**: `Less` + `Not` + `And`

**Circuit**: Moisture → Less (< threshold) → And.I1, Rain → Not → And.I2, And.Q → Valve

```bash
lox config add --type Less --title "Boden trocken" --room Garten --page Garten config.Loxone
lox config set-param config.Loxone "Boden trocken" Input2 30
lox config wire-connector config.Loxone "Boden trocken.Input1" "Bodenfeuchtigkeit.AQ"
lox config add --type Not --title "Kein Regen" --room Garten --page Garten config.Loxone
lox config wire-connector config.Loxone "Kein Regen.I" "Regen.AQ"
lox config add --type And --title "Bewässerung nötig" --room Garten --page Garten config.Loxone
lox config wire-connector config.Loxone "Bewässerung nötig.I1" "Boden trocken.Q"
lox config wire-connector config.Loxone "Bewässerung nötig.I2" "Kein Regen.Q"
lox config wire-connector config.Loxone "Bewässerungsventil.I1" "Bewässerung nötig.Q"
```

**Sim test**:

```bash
lox sim run config.Loxone --sim '{"name":"dry no rain","inputs":{"Bodenfeuchtigkeit.AQ":20,"Regen.AQ":0},"ticks":10,"dt":0.1,"expected_outputs":{"Bewässerung nötig.Q":{">":0.5}}}'
lox sim run config.Loxone --sim '{"name":"dry but raining","inputs":{"Bodenfeuchtigkeit.AQ":20,"Regen.AQ":1},"ticks":10,"dt":0.1,"expected_outputs":{"Bewässerung nötig.Q":{"==":0}}}'
lox sim run config.Loxone --sim '{"name":"moist","inputs":{"Bodenfeuchtigkeit.AQ":50,"Regen.AQ":0},"ticks":10,"dt":0.1,"expected_outputs":{"Bewässerung nötig.Q":{"==":0}}}'
```

---

## 11. CO2 Ventilation

**When**: Activate ventilation when CO2 exceeds a healthy level (e.g. 1000 ppm).

**Blocks**: `GreaterEqual`

**Circuit**: CO2.AQ → GreaterEqual.Input1 | set Input2=1000 | Q → Fan.I1

```bash
lox config add --type GreaterEqual --title "CO2 zu hoch" --room Bad --page Bad config.Loxone
lox config set-param config.Loxone "CO2 zu hoch" Input2 1000
lox config wire-connector config.Loxone "CO2 zu hoch.Input1" "CO2 Sensor.AQ"
lox config wire-connector config.Loxone "Lüfter Bad.I1" "CO2 zu hoch.Q"
```

**Sim test**:

```bash
lox sim run config.Loxone --sim '{"name":"high co2","inputs":{"CO2 Sensor.AQ":1200},"ticks":10,"dt":0.1,"expected_outputs":{"CO2 zu hoch.Q":{">":0.5}}}'
lox sim run config.Loxone --sim '{"name":"normal co2","inputs":{"CO2 Sensor.AQ":600},"ticks":10,"dt":0.1,"expected_outputs":{"CO2 zu hoch.Q":{"==":0}}}'
```

---

## 12. Humidity-Based Ventilation

**When**: Activate bathroom fan when humidity exceeds 65%.

**Blocks**: `GreaterEqual` + optionally `StairwayLS` for minimum run time

**Circuit**: Humidity → GreaterEqual.Input1 | set Input2=65 | Q → StairwayLS (optional) → Fan

```bash
lox config add --type GreaterEqual --title "Hohe Feuchte" --room Bad --page Bad config.Loxone
lox config set-param config.Loxone "Hohe Feuchte" Input2 65
lox config wire-connector config.Loxone "Hohe Feuchte.Input1" "Luftfeuchtigkeit.AQ"
lox config add --type StairwayLS --title "Lüfter Nachlauf" --room Bad --page Bad config.Loxone
lox config set-param config.Loxone "Lüfter Nachlauf" TimeHigh 600
lox config wire-connector config.Loxone "Lüfter Nachlauf.InputTrigger" "Hohe Feuchte.Q"
lox config wire-connector config.Loxone "Lüfter Bad.I1" "Lüfter Nachlauf.Q"
```

---

## 13. Pool Temperature Guard

**When**: Run pool pump only when water temperature is within a safe/desired range.

**Blocks**: `GreaterEqual` + `Less` + `And`

**Circuit**: Temp ≥ min AND Temp < max → Pump

```bash
lox config add --type GreaterEqual --title "Pool warm genug" --room Garten --page Garten config.Loxone
lox config set-param config.Loxone "Pool warm genug" Input2 18
lox config wire-connector config.Loxone "Pool warm genug.Input1" "Pool Temperatur.AQ"
lox config add --type Less --title "Pool nicht zu heiß" --room Garten --page Garten config.Loxone
lox config set-param config.Loxone "Pool nicht zu heiß" Input2 32
lox config wire-connector config.Loxone "Pool nicht zu heiß.Input1" "Pool Temperatur.AQ"
lox config add --type And --title "Pool Temp OK" --room Garten --page Garten config.Loxone
lox config wire-connector config.Loxone "Pool Temp OK.I1" "Pool warm genug.Q"
lox config wire-connector config.Loxone "Pool Temp OK.I2" "Pool nicht zu heiß.Q"
lox config wire-connector config.Loxone "Poolpumpe.I1" "Pool Temp OK.Q"
```

---

## 14. Touch Nightlight Air — dim/off at night

**When**: Bring a newly-added Touch Nightlight Air (device-bound `AlarmClock`) in line with its
siblings — the display nightlight should dim (or go off) during night hours and return to normal
during the day. This mirrors the factory wiring of the existing devices.

**Blocks**: device-bound `AlarmClock` + the day-gating chain `Time → GreaterEqual/Less → And → Mult`

**Circuit**: a time-of-day source gates a day window (`≥ start AND < end`); the AND result scales a
brightness via `Mult`, whose output drives the AlarmClock's `BrightInact` (display brightness while
inactive). Bind the AlarmClock to the physical device with `--device` so it emits `Dev=`.

```bash
# 1. Create the AlarmClock and bind it to the Touch Nightlight Air device (emits Dev=)
lox config add --type alarm-clock --title "DG Nachtlicht" --device "Touch Nightlight Air" config.Loxone

# 2. Day window: 07:00 ≤ now < 22:00  (uses the simulated Hour-of-day source)
lox config add --type Hour --title "Stunde" --page NachtPage config.Loxone
lox config add --type GreaterEqual --title "Ab 7 Uhr" --page NachtPage config.Loxone
lox config set-param config.Loxone "Ab 7 Uhr" Input2 7
lox config wire-connector config.Loxone "Ab 7 Uhr.Input1" "Stunde.AQ"
lox config add --type Less --title "Vor 22 Uhr" --page NachtPage config.Loxone
lox config set-param config.Loxone "Vor 22 Uhr" Input2 22
lox config wire-connector config.Loxone "Vor 22 Uhr.Input1" "Stunde.AQ"
lox config add --type And --title "Tagfenster" --page NachtPage config.Loxone
lox config wire-connector config.Loxone "Tagfenster.I1" "Ab 7 Uhr.Q"
lox config wire-connector config.Loxone "Tagfenster.I2" "Vor 22 Uhr.Q"

# 3. Scale to a daytime display brightness (e.g. 50%), then drive BrightInact
lox config add --type Mult --title "Display Helligkeit" --page NachtPage config.Loxone
lox config set-param config.Loxone "Display Helligkeit" Input2 50
lox config wire-connector config.Loxone "Display Helligkeit.Input1" "Tagfenster.Q"
lox config wire-connector config.Loxone "DG Nachtlicht.BrightInact" "Display Helligkeit.AQ"
```

**Sim test** (the simulator advances a virtual clock — see the loxone-sim skill):

```bash
# Daytime → display at 50%
lox sim run config.Loxone --sim '{"name":"day","clock":{"time":"12:00"},"ticks":5,"dt":1,"expected_outputs":{"Display Helligkeit.AQ":{"==":50}}}'
# Night → display off
lox sim run config.Loxone --sim '{"name":"night","clock":{"time":"23:00"},"ticks":5,"dt":1,"expected_outputs":{"Display Helligkeit.AQ":{"==":0}}}'
```

**Note**: `--device` accepts any selector (title, `uuid:…`, `Type:…`). It writes `Dev=<deviceUuid>`
on the new block, which is what binds the `AlarmClock` to a specific `LoxAIRDevice`/`TreeDevice`.

---

## Pattern Selection Guide

| I want to... | Use pattern |
|---------------|-------------|
| React to a sensor threshold | #1 Threshold |
| Combine two conditions | #2 Combined (AND/OR) |
| Turn on for X minutes | #3 Timed Switch (StairwayLS) |
| Wait then pulse | #4 Delayed Action (OnPulseDelay) |
| Activate during hours | #5 Schedule (DayTimer) |
| Do the opposite | #6 Negation (NOT) |
| Remember on/off state | #7 Memory/Latch |
| Protect from frost | #8 Frost Protection |
| Protect from wind | #9 Wind Protection |
| Smart irrigation | #10 Irrigation |
| Air quality control | #11 CO2 Ventilation |
| Humidity management | #12 Humidity Ventilation |
| Range-based control | #13 Pool Temperature |
| Device-bound night dimming | #14 Touch Nightlight Air |
