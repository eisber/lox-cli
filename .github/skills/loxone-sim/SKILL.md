---
name: loxone-sim
description: Test Loxone automation circuits with the SPS simulator. Verify signal propagation, check behavioral correctness. Use after building a circuit to validate it works.
license: AGPL-3.0
metadata:
  author: eisber
  version: "1.0"
allowed-tools: Bash
---

# Loxone SPS Simulator

Test automation circuits by injecting sensor values and checking that expected outputs appear. Use after wiring a circuit to validate correctness.

## Commands

### Run simulation (pass/fail)

```
lox sim run FILE --sim 'JSON_SPEC'
```

Executes the simulation spec, runs signal propagation, and checks expected outputs. Returns JSON:

```json
{
  "pass": true,
  "total": 1,
  "passed": 1,
  "scenarios": [
    {
      "name": "test name",
      "pass": true,
      "assertions": [
        {"output": "Block.Conn", "comparator": ">", "expected": 0.5, "actual": 1.0, "pass": true}
      ]
    }
  ]
}
```

Exit code 0 on pass, non-zero on failure.

### Step-by-step debugging

```
lox sim step FILE --sim 'JSON_SPEC'
```

Shows tick-by-tick signal changes in human-readable format. Use when `sim run` fails to understand which signal didn't propagate.

Output:

```
Tick 0 (t=0.0s):
  Außentemperatur.AQ = 30.0
  Sonnenschein.AQ = 1.0
Tick 1 (t=0.1s):
  Temp über 25.Q = 1.0 (was 0.0)
  Sonne und Warm.Q = 1.0 (was 0.0)
...
✅ Jalousie 1 [Wohnzimmer].InputTriggerDown > 0.5 → actual: 1.0 PASS
```

### Dump graph structure

```
lox sim dump FILE [--sim 'JSON_SPEC']
```

Dumps the full simulation graph: all blocks, connectors, wires, and current signal values. Use to inspect connectivity and debug missing wires.

### Check block/connector counts

```
lox sim check FILE
```

Quick structural check: reports number of blocks and connectors in the config.
The footer categorizes warnings as **simulated** (fully modeled),
**passthrough-unreliable** (device/weather sinks mirrored but not physically
verifiable), or **structural** (graph issues). Pass `--strict` to make any
warning a non-zero exit (useful in CI).

### Block behavior notes

- **AnalogMultiplexer2 `Select` is 1-based**: `0`→off, `1`→Input1, `2`→Input2
  (matching AnalogMultiplexer). A bare digital 0/1 never selects Input2 — feed
  `Select = override ? 2 : 1`. The sim warns when a `Select` input is wired.
- **LightController2** emits a 5th output `OutputReset` — a momentary pulse
  (1.0 for one tick) on the rising edge of its `Reset` input. Assert it with a
  pulse-style check after toggling `Reset`.
- **Formula** ("Formel") evaluates its `Formula="…"` expression over the four
  parameters I1–I4 (XML `Input1`–`Input4`). Supports `+ - * / ^`, `IF(c;a;b)`
  with `== != > >= < <=`, and `PI ABS SQRT LN LOG EXP SIN COS TAN ARCSIN ARCCOS
  ARCTAN SINH COSH TANH RAD DEG SIGN INT MIN MAX` (case-insensitive; trig in
  radians). Outputs `AQ` (result) and `TQ` (1.0 on error, e.g. divide-by-zero).

## Simulation Spec Format

```json
{
  "name": "scenario name (optional)",
  "inputs": {
    "SensorName.ConnectorKey": 1.0,
    "SensorName": 0.5
  },
  "ticks": 10,
  "dt": 0.1,
  "expected_outputs": {
    "BlockName.OutputKey": {">": 0.5}
  }
}
```

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Optional label for the scenario |
| `inputs` | object | Map of sensor/block connector → value to inject |
| `ticks` | integer | Number of simulation ticks to run (typically 10) |
| `dt` | float | Time step in seconds per tick (typically 0.1) |
| `expected_outputs` | object | Map of output connector → comparator assertion |

### Comparators

| Comparator | Meaning | Example |
|------------|---------|---------|
| `>` | Greater than | `{">": 0.5}` — output must be > 0.5 |
| `>=` | Greater than or equal | `{">=": 1.0}` |
| `<` | Less than | `{"<": 0.1}` — output must be < 0.1 |
| `<=` | Less than or equal | `{"<=": 0.0}` |
| `==` | Equal (±1e-9 tolerance) | `{"==": 1.0}` — output must equal 1.0 |
| `~=` | Approximately equal (±5%) | `{"~=": 25.0}` — output within 5% of 25.0 |

Multiple comparators can be combined in one assertion:

```json
{"Block.AQ": {">=": 20.0, "<=": 30.0}}
```

### Sensor Name Resolution

The simulator resolves sensor/block names in this order:

1. **Bare name**: `"Außentemperatur"` — matches the block title directly (uses default output, typically `.AQ` or `.Q`)
2. **Block.Connector**: `"Außentemperatur.AQ"` — explicit connector on the block
3. **Block [Room].Connector**: `"Jalousie 1 [Wohnzimmer].InputTriggerDown"` — room-qualified for ambiguous names

Always use room qualification when a block name exists in multiple rooms (e.g. "Jalousie 1", "Lichtsteuerung").

### Weather, sensor & device-actor blocks

These I/O block types are simulated as **value sources** and **sinks**, so weather/threshold
circuits and device outputs can be tested:

| Block type | Role | How to drive / assert |
|------------|------|-----------------------|
| `WeatherData` | Weather-service source (one quantity per block: wind, temp, sunshine, …) | inject `"<name>.AQ": value` — the reading persists across ticks |
| `GenTSensor` | Generic MQTT sensor source | inject `"<name>.AQ": value` |
| `TreeAsensor` / `LoxAIRAsensor` | Wired/wireless analog sensor | inject `"<name>.AQ": value` |
| `LoxAIRAactor` / `TreeAactor` | Analog device actor (sink) | assert `"<name>.AQ"` — mirrors the delivered input |
| `LoxAIRactor` / `TreeActor` | Digital device actor (sink) | assert `"<name>.Q"` — mirrors the delivered input |
| `GenTActor` | Generic MQTT publish actor (sink) | assert `"<name>.AQ"` — mirrors the `Text` input |

Device actors are terminal in real configs (no output connector); the simulator mirrors the value
delivered to their `I` input onto a synthesized output (`AQ` for analog `A`-actors, `Q` for digital
actors) so you can assert on what the physical device would emit.

### Multiple Scenarios

Pass an array for multiple test scenarios in one run:

```json
[
  {
    "name": "sunny and warm → blinds close",
    "inputs": {"Außentemperatur.AQ": 30, "Sonnenschein.AQ": 1},
    "ticks": 10, "dt": 0.1,
    "expected_outputs": {"Jalousie 1 [Wohnzimmer].InputTriggerDown": {">": 0.5}}
  },
  {
    "name": "cold → blinds stay open",
    "inputs": {"Außentemperatur.AQ": 15, "Sonnenschein.AQ": 1},
    "ticks": 10, "dt": 0.1,
    "expected_outputs": {"Jalousie 1 [Wohnzimmer].InputTriggerDown": {"==": 0}}
  }
]
```

### Using --sim-file for complex specs

For large or multi-scenario specs, write to a file:

```bash
cat > /tmp/test.json << 'EOF'
[
  {"name": "test1", "inputs": {"Sensor.AQ": 30}, "ticks": 10, "dt": 0.1,
   "expected_outputs": {"Output.Q": {">": 0.5}}},
  {"name": "test2", "inputs": {"Sensor.AQ": 10}, "ticks": 10, "dt": 0.1,
   "expected_outputs": {"Output.Q": {"==": 0}}}
]
EOF
lox sim run config.Loxone --sim-file /tmp/test.json
```

## Example: Testing a Threshold Circuit

After building a "close blinds when temp ≥ 25°C" circuit:

```bash
# Positive test: 30°C should trigger
lox sim run config.Loxone --sim '{"name":"hot day","inputs":{"Außentemperatur.AQ":30,"Sonnenschein.AQ":1},"ticks":10,"dt":0.1,"expected_outputs":{"Jalousie 1 [Wohnzimmer].InputTriggerDown":{">":0.5}}}'

# Negative test: 20°C should NOT trigger
lox sim run config.Loxone --sim '{"name":"cool day","inputs":{"Außentemperatur.AQ":20,"Sonnenschein.AQ":1},"ticks":10,"dt":0.1,"expected_outputs":{"Jalousie 1 [Wohnzimmer].InputTriggerDown":{"==":0}}}'

# Boundary test: exactly 25°C should trigger (GreaterEqual)
lox sim run config.Loxone --sim '{"name":"boundary","inputs":{"Außentemperatur.AQ":25,"Sonnenschein.AQ":1},"ticks":10,"dt":0.1,"expected_outputs":{"Jalousie 1 [Wohnzimmer].InputTriggerDown":{">":0.5}}}'
```

## Example: Testing a Timer Circuit

After building a StairwayLS (5-minute light):

```bash
# Trigger: motion should turn light on
lox sim run config.Loxone --sim '{"name":"motion triggers light","inputs":{"Bewegungsmelder.OutputPresence":1},"ticks":10,"dt":0.1,"expected_outputs":{"Treppenlicht.Q":{">":0.5}}}'

# No trigger: no motion should leave light off
lox sim run config.Loxone --sim '{"name":"no motion","inputs":{"Bewegungsmelder.OutputPresence":0},"ticks":10,"dt":0.1,"expected_outputs":{"Treppenlicht.Q":{"==":0}}}'
```

## Example: Testing a Time-of-Day Schedule (simulated clock)

Time/astro source blocks (`Time`, `Hour`, `Minute`, `Sunrise`, `Sunset`, `NightTime`, …) are
driven by a **simulated wall-clock**. Add a `clock` object to the spec — the simulator sets the
clock before the run and advances it by `dt` each tick. This lets you verify day/night gating
*offline*, without waiting for real time to pass.

`clock` fields (all optional): `time` (`"HH:MM"` / `"HH:MM:SS"`), `date` (`"YYYY-MM-DD"`),
`minutes_since_midnight`, `latitude`, `longitude` (lat/lon affect sunrise/sunset; default Vienna).

For a config that dims a light to 30% during the day and 0% at night
(`Hour → GreaterEqual/Less → And → Mult → Brightness`):

```bash
# Daytime: noon → brightness 30
lox sim run config.Loxone --sim '{"name":"noon","clock":{"time":"12:00"},"ticks":5,"dt":1,"expected_outputs":{"Brightness.AQ":{"==":30}}}'

# Night: 23:00 → brightness 0
lox sim run config.Loxone --sim '{"name":"night","clock":{"time":"23:00"},"ticks":5,"dt":1,"expected_outputs":{"Brightness.AQ":{"==":0}}}'
```

Per-step clocks also work in `lox sim step` specs: give each step its own `clock` to sweep across
the day and watch an output flip at the threshold hour.

## Debugging Workflow

1. **Run sim** → if it fails:
2. **Run step** → inspect tick-by-tick to find where signal stops
3. **Run dump** → check wiring graph for missing connections
4. **Fix wiring** with `lox config wire-connector`
5. **Re-run sim** to confirm fix
