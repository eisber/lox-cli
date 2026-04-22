# HVAC Training Notes — Loxone Block Behavior Reference

Source: Official Loxone HVAC Training Guide (Heating, Ventilation & Air Condition)

This document extracts Loxone-specific HVAC knowledge for agent prompts and
simulator validation. It covers block behavior, formulas, and design patterns
from official training material.

---

## 1. Sensor Fundamentals

### Analog Input Correction
- Correction maps raw voltage to physical range using two calibration points
- "Input Value 1 → Target Value 1" and "Input Value 2 → Target Value 2"
- Example: 0V → -20°C, 10V → +80°C for Loxone Temperature & Humidity sensor

### Validation Strategy (separating failure types)
Two independent error detection mechanisms should be used together:

| Mechanism | Detects | Location |
|-----------|---------|----------|
| **Receive Timeout** (on AI) | Sensor failure / disconnection | Analog Input properties |
| **Value Validator** (external block) | Value out of expected range | Separate block |

**Best practice**: Disable Value Range validation on the analog input itself.
Use external Value Validator block (Tmc=0, Min, Max, D parameters) so each
failure type has its own error output.

### 1-Wire Sensors
- Polling cycle must be 3-4× shorter than Receive Timeout
- **Counterfeit DS18B20 detection**: Clones send value 25 on restart/interference;
  originals send 85 (but rarely restart due to better interference resistance)
- Clone SN pattern: if SN differs from `28-xx-xx-xx-xx-00-00-xx`, likely a clone
- Reference: https://github.com/cpetrich/counterfeit_DS18B20

### Virtual Status vs Direct Sensor Visualization
Always use a Virtual Status block rather than visualizing the sensor directly.
When replacing a sensor (e.g., Air → Tree), statistics stored in the sensor
are lost. Statistics stored in the block survive sensor replacement.

---

## 2. Analog Processing Blocks

### Scaler
Same function as the correction on analog inputs — linear mapping.

### MinMax
Determines lowest and highest value across all analog inputs.
**Use case**: Select highest CO2 from all sensors for ventilation reference.

### MinMax Since Reset
Tracks min/max of a single input since last reset.
**Use case**: Hourly max/min temperature in refrigerated display.

### Analog Memory
On Tr trigger, writes current analog value to V output. Holds until next trigger.

### Analog MinMax Limiter
Clamps values within a range.
**Use case**: Limiting max/min water temperature.

### Average
Averages multiple analog inputs.
**Use case**: Averaging temperatures from multiple Touch sensors in a large room.

### Moving Average
Smooths fluctuating values by removing short-term noise.

### Threshold Switch (AnalogThresholdTrigger)
Responds to exceeding limit values — turns output on or off when measured value
crosses the threshold. Uses hysteresis (separate on/off thresholds).
**Use cases**: DHW tank temperature control, cooling fan activation.

### Status (StatusController)
Creates custom status texts and symbols in the Loxone App. Can use comparison
functions between four inputs to build decision logic.
**Note**: Cannot display colored symbols in system schematic — use emojis
(🔴 🟠 🟢) in status text instead.

---

## 3. Humidity & Dew Point

### Absolute Humidity Formula
```
216.7 * ((I2/100) * (6.1078 * 10^(7.5*I1/(237.3+I1)))) / (I1 + 273.15)
```
- I1 = temperature (°C)
- I2 = relative humidity (%)
- Result: g/m³
- Precision: sufficient for building automation

### Basement Ventilation Principle
**Rule**: Ventilate ONLY when absolute humidity outside < absolute humidity inside.
- Minimum "advantageous" threshold: **1 g/m³ difference**
- Improper ventilation (based on RH alone) can make humidity worse
- Supplement with dew point monitoring: outdoor dew point must be lower than
  basement temperature, otherwise warm outdoor air condenses around the fan

### Dew Point Block (DewPoint)
Calculates temperature at which air is fully saturated with water vapor.
Below the dew point, water condenses.

**Critical application**: Underfloor cooling minimum water temperature.
The minimum cooling water temperature should be set dynamically from the
DewPoint block to prevent condensation in manifolds or pipes.

---

## 4. CO Monitoring — Status Monitor

### Architecture
```
CO Sensors → Status blocks (3-level evaluation) → Status Monitors (per floor) → Central Status Monitor
```

### Three CO levels
| Level | Symbol | Action |
|-------|--------|--------|
| Safe | 🟢 | Normal operation |
| Warning | 🟠 | Activate floor ventilation |
| Danger | 🔴 | Evacuate, full ventilation |

### Central Status Monitor behavior
When assigning Status Monitors to a Central Status Monitor, all assigned monitors
**take over the configuration from the central one** — configure only the central.

### Ventilation request
Memory Flags per floor activate when CO reaches dangerous levels.

---

## 5. Toilet Ventilation Controller (ToiletFan)

| Input/Param | Purpose |
|-------------|---------|
| **P** | Activating P starts the session |
| **Fsd** | Fan start delay — fan starts after this delay |
| **Fpet** | Fan post-run time — fan turns off with this delay after session ends |
| **Tg** | Toggle — activates session OR ends current session immediately |

---

## 6. Intelligent Room Controller (HeatIRoomController2 / IRC)

### Operating Modes
- **Comfort**: Target comfort temperature active
- **Eco**: Reduced temperature (energy saving)
- **Frost Protection**: Minimum temperature to prevent freezing
- **Heat Protection**: Maximum temperature limit

### P vs Mo Input — Critical Behavioral Difference

| Input | Behavior | Default Overrun |
|-------|----------|-----------------|
| **P** (Presence) | If continuously active >30 min → IRC auto-switches Eco→Comfort | 900s (Presence extend time) |
| **Mo** (Motion) | No auto Eco→Comfort switch (unlikely to stay active 30min) | 60s |

**When to use Mo**: In rooms where sensor noise from adjacent rooms could keep
P active long enough to trigger unwanted Comfort mode. Using Mo eliminates this
risk because sustained 1800s motion is very unlikely.

**In both cases**: Extension of an already-active comfort temperature works normally.

### Window Contact Behavior
- IRC adjusts heating behavior when a window is open
- Behavior depends on **outdoor temperature** — may reduce or completely stop heating
- Window contact must have proper naming and room assignment

### Single Comfort Temperature Mode
Uses one comfort temperature with allowed deviation ϑd instead of separate
comfort/eco temperatures for heating and cooling.
**Use case**: Hotel rooms with Touch Pure Flex temperature control.

### Monitor Temperature
Notifies user when IRC cannot reach desired temperature:
- Valve actuator 100% open
- CCo H output active
- Window closed
- Temperature still not rising

### Valve Standstill (Vs)
After Vs days without movement, valves perform a full open/close cycle
to prevent sticking.

### C and E Inputs
- **C**: Rising edge → Comfort mode (with Cet extend time)
- **E**: Rising edge → Eco mode (with EBpet extend time)
- **Use case**: Guard booth — guard present = Comfort (C), on patrol = Eco

### PWM Outputs
For electric radiators (switching sources): H, C, HC outputs switch from
analog 0-100% to PWM when enabled.
- Pwm parameter: 0 = automatic interval based on heating speed
- Heating speed Hs/Cs: time to raise/lower room temp by 1°C (0 = learned value)

### Multiple Sources
Up to 4 heating/cooling sources (independent H + H1, H2, H3).
Source selection rules:
1. First available source in list is always activated
2. All "cheap" sources activate simultaneously
3. "Expensive" sources used one-by-one in priority order
4. First source should always be cheap (expensive first = wrong setup)
5. Rooms with detected presence have higher priority

### IRC Cheap Energy (Ec/Eh inputs)
When cheap energy available (low spot price, PV surplus):
- Modes 1,2,4,5: offset ϑExc applied
- Single comfort temp: overheating/undercooling ϑd - 0.5° allowed
- Standard mode: (ϑcc - ϑch) / 2 allowed

---

## 7. Heating and Cooling Controller (CCo / Heatmixer2)

### Core Function
Evaluates heating/cooling demand from all connected IRCs and controls the
heating/cooling source.

### Heating/Cooling Mode Selection
For each room: `demand = (Target temp - Current temp) × Room size`
Sum of all demands determines heating vs cooling mode.
Evaluation only after Start of Operation Threshold (Sot) exceeded.
Influenced by Otm (outdoor temperature mean, 48h average).

### Presence Priority
Rooms with presence get higher priority. If both heating and cooling demands
exist simultaneously, presence-based demands evaluated first. Equal presence
demands → total demand of all rooms decides.

### Key Parameters

| Parameter | Purpose |
|-----------|---------|
| **Sot** | Start of Operation Threshold — minimum demand before source starts |
| **Otm** | Outdoor temperature mean (48h average) |
| **ϑLimH / ϑLimC** | Temperature limits for heating/cooling availability |
| **MinHr** | Minimum running time (prevents compressor damage, 5-10 min typical) |
| **MaxTp** | If total valve opening < MaxTp and Sot met → H output cycles Don/Doff |
| **Tt2s** | Time delay before H2 output activates (secondary source) |
| **ϑminS2** | If outdoor temp drops to this → H2 activates immediately |
| **ϑminHP** | If outdoor temp < ϑminHP → H/H2 disabled, Ah (additional heating) activates |
| **Mh** | Manual Heating — activates H even without IRC heating request |

### Transfer Cooling/Heating Ban
If CCo prohibits heating/cooling due to outdoor temp limits, this prohibition
can be transferred to the entire IRC block (prevents cooling/heating even from
other sources).

**Practical example — Shading in autumn**:
- 48h average = 14°C → cooling banned (ϑLimC)
- Current = 20°C, sun shining, indoor rises above 24°C
- **Ban OFF**: IRC switches to cooling → automatic shading at ϑsc=23.5°C
- **Ban ON**: IRC stays in heating → shading delayed until ϑsh=27.5°C

### Heating Type
Oil/Gas setting hides ϑminS2, ϑminHP, Vd parameters (irrelevant for Oil/Gas).

### Cascade Control
Multiple primary sources (e.g., 2+ heat pumps) heating one buffer tank.
- Not the same as multiple IRC sources
- For 2 sources: use H and H2 outputs
- For more: custom logic using ITC Aql output
- Cascade depends on outdoor temp, occupied rooms, or time

---

## 8. Flow Temperature Calculator (ITC / FlowTempCalc)

### Heating Curve
Set by Slope (S) and parallel shift (N).
- Slope determines curve shape
- N adjusts indoor temperature once slope is correct

**Tuning timeline**: 6-8 weeks in well-insulated buildings.
Curve must be tested across full outdoor temp range (+15° to -10°C).
May not be fully tuned in one heating season.

**Practical approach**: Set curve initially steeper (overheats house).
Valve actuators limit flow to individual rooms, fine-tuning comfort.
Curve = "main temperature level", valves = fine-tuning.

### Key Parameters

| Parameter | Purpose |
|-----------|---------|
| **S** (Slope) | Heating curve steepness |
| **N** (Offset) | Parallel shift of heating curve |
| **Gain** | Adjusts water temp based on target-actual difference |
| **Str** | Switch-on threshold — if ≥1 valve open > Str%, Qp (pump) activates |
| **Min/Max** | Water temperature limits (critical for underfloor heating floor protection) |

### Outputs
- **AQf**: Actual flow temperature — used for Modbus integration with heat pumps
- **AQt**: Actual target temperature
- **Qp**: Circulation pump output

### Str + Sot interaction
If all IRCs assigned in ITC are also assigned in CCo, the Str parameter is
ignored and the Sot value from CCo is used instead.

### Min/Max for Underfloor
- Max temp must be capped to avoid floor damage (consider floor covering limits + hygiene regs)
- Min cooling temp: set dynamically from DewPoint block to prevent condensation

---

## 9. AC Control Air (AcControl)

- Configurable mode names, airflow names, fan speed names, default presets
- minT/maxT: limit values for app visualization
- Integration with IRC via AC connector → creates AC Central Controller (ACCC)
- Target Offset (O) and hysteresis (H) parameters affect Comfort vs Eco behavior

---

## 10. Fancoil Control

**Note**: Dedicated fancoil block planned for Q1 2026. Current approach uses
existing blocks.

### Control Approach
Same as underfloor heating/radiators (IRC + CCo + ITC) **plus** fan speed control.

### Fan Speed Control
- Relay: off/low/medium/maximum (use Status block to convert analog → relay steps)
- Analog 0-10V: connect directly to H output (or H1…H3)
- Modbus: via integration

### Fan Speed Limiting
- Use Scaler to adjust range based on presence
- In night mode, limit max speed to reduce noise
- Use Analog MinMax Limiter for presence/night mode limiting

### FCU with Fresh Air Support
Fan serves dual purpose: heating/cooling air + ventilation (CO2, humidity).
Use MinMax block to select highest required speed between heating and ventilation.

### 2-Pipe vs 4-Pipe
- **2-Pipe**: Single buffer tank, all rooms same mode. Standard CCo + ITC + Mixing Valve
- **4-Pipe**: Separate heating/cooling tanks, rooms can independently heat/cool.
  Custom logic replaces CCo (CCo can't heat and cool simultaneously).
  Mixing valve Inv input switches between heating/cooling per IRC mode.

---

## 11. Mixing Valve (MixingValve)

| Parameter | Purpose |
|-----------|---------|
| **Td** | Travel Duration — time for valve to move full range |
| **St** | Sampling Time — measurement interval (1-Wire polling must be shorter) |

**Critical**: When valve is off (pump stopped), valve must stay in cold (H)
position. Otherwise, when pump starts, maximum temperature water flows before
valve reacts.

---

## 12. DHW (Domestic Hot Water) Patterns

### Temperature Control
Use Threshold Switch to monitor tank temperature. Below setpoint →
activate Mh on CCo + switch 3-way valve + set target temp via Analog Multiplexer.

### Circulation Pump
- Measure drain time at furthest outlet → set pump cycle
- Measure cooldown time (insulated: 60-90min, uninsulated: 20min) → set interval
- Match to household routine (off in absence, on at door unlock, off at night)

### Anti-Legionella
- Heat tank >60°C for ≥10 minutes, once per week
- Run circulation during disinfection
- Use Up Counter to track days since last 65°C/10min event
- After 10 days → force disinfection
- **Safety**: Thermostatic mixing valve on outlet required (tank can reach >80°C)
- DHW temperature usually 40-50°C

### Thermal Solar Collector (ThermalSolar)
- Controls circulation pump (Sp output) or adjusts speed (Spa analog output)
- Can work with up to 5 tanks
- Useful for cascade tank storage in large projects

---

## 13. Heat Recovery Ventilation

### Room Ventilation Controller (RoomVentilationController / RVCo)

#### CO2 Regulation
Uses **PI controller** on outputs F, Fea, Fsa.
Parameters Ivp/Bvp (presence) and Iva/Bva (absence) set ventilation limits.
Reaction time: ~30s.

#### Humidity Regulation
Uses **two-point controller** (not PI) — switches between Ivp and Bvp.
Reason for ventilation visible on output S.

#### F, Fea, Fsa Outputs
For balanced pressure ventilation, supply and exhaust volumes must match.
In practice, slight overpressure maintained to push dust/odors out through
building leaks.

If motors controlled directly: measure pressure difference or airflow in
supply/exhaust ducts (supply at 30% and exhaust at 40% may be balanced due
to different pressure losses).

For stepped fans: convert F 0-100% to discrete steps using Status block.

#### Key Inputs

| Input | Behavior |
|-------|----------|
| **Sm** (Sleep Mode) | Turns off unit for Smt duration. Connected to Good Night flag |
| **Boost** | F/Fea/Fsa → 100%. For bathroom/kitchen quick ventilation |
| **DWC** | Window open → ventilation off **immediately** (unlike IRC DWC) |

#### Bypass Control (central units without zoning)
Cannot use automatic He output. Custom logic required:
- Compare exhaust air temp with outdoor air temp
- Bypass only when outdoor air ≥2°C cooler than exhaust

### RVCo as IRC Heating/Cooling Source
- He output controls bypass
- SAT (Supply Air Temperature): if supply air temp > room temp and heating
  demand exists → fan speed increases to support heating (same for cooling)
- If supply air temp not connected → outdoor air temp used instead

### Zonal HRV
- Control dampers regulate air per zone
- Modbus registers for zone control (mode + CO2/humidity values)
- Operating modes: Auto, Comfort, Away, Boost
- Boost auto-reverts to previous mode after timeout
- Add notification if user forgets to disable Boost or re-enable HRV

### Decentralized HRV
- Each unit has own RVCo with Touch CO2/humidity sensors
- No bypass in most cases (unit too small)
- Sleep mode important — fan motor directly in room (noisy)

---

## 14. Belimo & MP-Bus

Belimo Tree/Air provides many data points from a single device connection,
replacing multiple IA/AO extensions for wired connections.
Belimo VAV/EPIV devices available as Loxone Library templates.

---

## 15. Key Design Principles (from training summary)

1. **It must be simple** — installers will want to use it
2. **It must work and be reliable** — installers will want to use it again
3. **It must be universal** — applicable to any project
