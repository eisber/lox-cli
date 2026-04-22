# Loxone Block System Comprehensive Analysis

## 1. BLOCK TYPES & CATEGORIES

### Statistics
- **Total Block Types:** 221 unique types
- **Connector Types:** 190+ with 2,384+ distinct connector configurations
- **Real Configs:** 2,944+ blocks commonly wired together in real installations

### Block Categories

#### Logic Blocks (14 types)
Gates, comparisons, and boolean operators:
- **Binary Logic:** AND (2 inputs), OR (2 inputs), NOT, XOR
- **Flip-Flops:** RS (reset-set), SR (set-reset) - 4 connectors each
- **Comparators:** Equal, Greater, Less, GreaterEqual, LessEqual, NotEqual (2 inputs each)
- **Analog Comparator:** Threshold with output, rising/falling edge detection

**Example: AND Gate**
```
Control Type ID: 331
Connectors:
  - I1 (Input)
  - I2 (Input)
  - Q (Output)
```

#### Math Blocks (16 types)
Arithmetic and value transformations:
- **Basic Ops:** Add (2/4-way), Subtract, Multiply, Divide, Modulo
- **Aggregation:** Average, MinMax (min/max selection)
- **Scaling:** Scaler (linear input mapping), AnalogScaler (2-point mapping)
- **Multiplexing:** AnalogMultiplexer (2/4-way selectors)
- **Limiting:** AMinmax (clamp value between bounds)
- **Time-based:** TimeMinmax (track min/max since reset with timestamp)

**Signature Pattern:** Most have 1-2 inputs, 1-2 outputs, 1-4 parameters

#### Timing/Scheduling Blocks (11 types)
Time-based triggers and delays:
- **Delays:** OnDelay, OffDelay, OnOffDelay (switch + delay time parameter)
- **Pulse Generation:** PulseGen (periodic), PulseAt (at time), PulseBy (duration)
- **Timing:** OnPulseDelay (like monoflop with delay), Monoflop (pulse on input)
- **Scheduling:** DayTimer (9 connectors - schedule with modes/times)
- **Advanced:** Sequencer (state machine with 4-8 steps), RetOnDelay (retentive delay)

**Example: OnDelay**
```
Control Type ID: 361
Connectors:
  - InputTrigger (Input)
  - Q (Output)
  - Time (Parameter - delay duration in seconds)
Default Parameters: Time=300s (5 minutes)
```

#### Input/Output Blocks (13 types)
User interfaces and virtual elements:
- **Binary:** PushButton (momentary), PushButton2 (on/off), Switch
- **Selection:** PushButtonSel (+), PushButton2Sel (+/-), Radio buttons (8x, 16x)
- **Analogue:** Dimmer (14 connectors with fade timing)
- **Info:** State (status block), StateV (virtual status), Text (note), TextGenerator
- **Special:** GlobalStates (system variables), EibDimmer (KNX integration)

#### Metering & Energy Blocks (13 types)
Energy tracking and accumulation:
- **Meters:** MeterAbsUni (13 connectors), MeterAbsBi (bidirectional, 21 conn), MeterAbsSt (storage)
- **Pulse Meters:** MeterPUni (3 conn), MeterPBi (25 conn - bidirectional)
- **Counters:** Counter (up), UpDownCounter (8 conn), HourCounter (runtime)
- **Energy Management:** Energy (21 conn), EnergyManager (7 conn), EnergyManager2 (14 conn)
- **Special:** FixedValueMeter, UtilityMeter

#### Complex Control Blocks (40+ types)
High-level automation controllers:

**Lighting Systems:**
- LightController (V1, V2, H variant) - 24-33 connectors
  - Scene selection, brightness control, dimming curves
  - RGB/temperature support, alarm clock integration
  - FadingTime, SceneMixTime, AlarmPeriod parameters

**Climate & HVAC:**
- HeatIRoomController2 (49 connectors) - intelligent room temp control
- AcControl (26 connectors) - air conditioning with mode/fan selection
- ClimateControllerUS (29 connectors) - HVAC with heating/cooling coordination
- Heatmixer, Heatmixer2, Fancoil variations

**Blinds & Shading:**
- AutoJalousie (31 connectors) - automated shading with weather response
- EIBJalousie (6 connectors) - KNX blind control
- Pergola, RoofWindow, ShadeRoof, Skylight variants
- Multi-zone coordination with wind/sun sensors

**Room Control:**
- IRoomcontrol (44 connectors) - intelligent room controller V1
  - Temperature setpoints for Comfort/Save/Party/DeepSleep modes
  - Window contact, occupancy, HVAC coordination
  - TimeMove, TimeC, TimeS parameters for scheduling

**Door & Access:**
- Doorcontroller (multi-state), Door (simple), GateController
- Access control with trigger, external ID, pulse timing
- NfcCodeTouch, Authentication blocks

**Metering & Load Management:**
- LoadShed (load shedding), PowerUnit (power management)
- EnergyManager, EnergyManager2 with multiple input channels
- Tracker (PV/battery tracking)

**Media & Communication:**
- MediaController, MediaClient (audio/video)
- Intercom, IntercomDevice (voice communication)
- MusicServer (music distribution)
- Wallbox, CarCharger (vehicle charging)

---

## 2. CONNECTOR I/O SIGNATURES

### Connector Types: I, O, P

| IO Type | Meaning | Direction | Example |
|---------|---------|-----------|---------|
| **I** | Input | Incoming signal (wired TO this) | And gate I1, I2 |
| **O** | Output | Outgoing signal (wire FROM this) | And gate Q output |
| **P** | Parameter | Configuration value (default) | AMinmax Min/Max params |

### Naming Conventions

**Inputs:**
- `InputTrigger` - edge-triggered input
- `Input` - level-triggered input  
- `Disable` - disable control signal
- Domain-specific: `Temp`, `Window`, `Comfort`, `Save`

**Outputs:**
- `Q` - main binary output (1 = ON, 0 = OFF)
- `AQ` - analog output (0.0-1.0 or custom range)
- `TQ` - time output (duration/counter)
- Numbered: `Q1`, `Q2`, `AQ1`, `AQ2` (multiple outputs)

**Parameters:**
- `Min`, `Max` - range constraints
- `Step`, `Dir` - increment/direction
- `Time`, `Delay` - duration in seconds
- `Default`, `Def` - initial value
- `Hyst` - hysteresis (threshold dead-band)

### Real Connector Examples

```
Add Block:
  AQ (Output, Analog) = Input1 + Input2
  Input1 (Parameter, default=0)
  Input2 (Parameter, default=0)
  
OnDelay Block:
  InputTrigger (Input)
  Q (Output) = delayed trigger
  Time (Parameter, default=300s)
  
LightController2:
  AQ1, AQ2 (Outputs - brightness 0-100%)
  Scene (Output - selected scene)
  OutputReset, OutputResetAll (Pulse outputs)
  MaxP (Parameter, default=0.35 - max power)
  Step (Parameter, default=2 - increment)
  FadingTime (Parameter, default=1s)
  SceneMixTime (Parameter, default=1s)
  AlarmPeriod, AlarmClockPeriod (Parameters)
```

---

## 3. XML/CONFIG STRUCTURE

### File Format
Loxone configs are **ZIP archives** containing:
- `LoxAPP3.xml` (XML structure - legacy, may be JSON now)
- `LoxAPP3.json` (JSON structure - current format)

### XML Element Structure

```xml
<C Type="LightController" Title="Wohnzimmer Licht" U="uuid-here">
  <!-- Connectors (Co = connector) -->
  <Co K="AQ1" Def="0" />  <!-- K=connector key, Def=default value -->
  <Co K="Scene" Def="0" />
  <Co K="MaxP" Def="0.35" />
  
  <!-- Wiring connections -->
  <InputRef Ref="uuid-of-source" RefCo="source-output-key" />
  <OutputRef Ref="uuid-of-target" RefCo="target-input-key" />
  
  <!-- Children/nested blocks -->
  <C Type="SomeBlockType" ... />
</C>
```

### JSON Structure (LoxAPP3.json)

```json
{
  "controls": {
    "U:12345678-abcd-ef00-...": {
      "name": "Wohnzimmer Licht",
      "type": "LightControllerV2",
      "states": {
        "jLocked": "state-uuid-1",
        "activeMoods": "state-uuid-2",
        "moodList": "state-uuid-3",
        "daylightConfig": "state-uuid-4"
      },
      "params": {
        "MaxP": "0.35",
        "Step": "2",
        "FadingTime": "1"
      },
      "inputs": {
        "Scene": "uuid-of-scene-selector",
        "MaxP": "uuid-of-brightness-limit-param"
      },
      "room": "room-uuid",
      "cat": "category-uuid"
    }
  },
  "rooms": { ... },
  "cats": { ... }
}
```

### Wiring Representation

**In XML:**
```xml
<!-- Control A has an output, Control B receives it -->
<C Type="And" U="and-uuid" Title="Condition">
  <Co K="Q" />  <!-- Output connector -->
</C>

<C Type="LightController" U="light-uuid" Title="Lights">
  <InputRef Ref="and-uuid" RefCo="Q" />  <!-- Wire from AND.Q -->
</C>
```

**Key Attributes:**
- `U` = UUID of the element
- `Title` = Display name
- `Type` = Block type (And, Or, LightController, etc.)
- `K` = Connector key (I1, Q, AQ, etc.)
- `Def` = Default value for parameter
- `Ref` = UUID of source block
- `RefCo` = Source connector key

### Parsing in Code

From `config_edit.rs`:
```rust
pub fn set_param(&mut self, selector: &str, param: &str, value: &str) -> Result<()> {
    // Find element by selector (name, type, uuid, etc.)
    let elem = self.get_element_mut(&path);
    
    // Walk through connectors <Co> elements
    for child in &mut elem.children {
        if let Some(co) = child.as_mut_element()
            && co.name == "Co"
            && co.attributes.get("K").map(|k| k == param).unwrap_or(false)
        {
            // Set the default value
            co.attributes.insert("Def".to_string(), value.to_string());
            return Ok(());
        }
    }
}
```

---

## 4. STATE HANDLING (WebSocket Protocol)

### State Event Types

From `stream.rs`, blocks emit state changes via WebSocket binary protocol:

```rust
pub enum StateEvent {
    /// Numeric state: UUID → f64 value
    ValueState { 
        uuid: String, 
        value: f64 
    },
    
    /// Text state: UUID → string + icon
    TextState {
        uuid: String,
        icon_uuid: String,
        text: String
    },
    
    /// Daytimer schedule entries
    DaytimerState {
        uuid: String,
        default_value: f64,
        entries: Vec<DaytimerEntry>
    },
    
    /// Weather forecast data
    WeatherState {
        uuid: String,
        last_update: u32,
        entries: Vec<WeatherEntry>
    },
    
    /// Keepalive heartbeat
    Keepalive,
    
    /// Miniserver offline
    OutOfService
}
```

### State UUID Mapping

Each control has **states** (outputs that can be streamed):

```json
"controls": {
  "U:control-uuid": {
    "name": "Lichtschalter",
    "type": "PushButton",
    "states": {
      "Q": "U:state-uuid-1",      // Binary output
      "On": "U:state-uuid-2",      // On pulse
      "Off": "U:state-uuid-3"      // Off pulse
    }
  }
}
```

The **StateUuidInfo** maps state UUIDs back to their parent:
```rust
pub struct StateUuidInfo {
    pub control_name: String,      // "Lichtschalter"
    pub control_uuid: String,      // U:control-uuid
    pub state_name: String,        // "Q", "On", "Off"
    pub control_type: String,      // "PushButton"
    pub room: Option<String>,      // Room name
    pub category: Option<String>,  // Category name
    pub unit: Option<String>,      // "%" for brightness, "°C" for temp
}
```

### Binary Message Format

```
MSG_VALUE_STATES (0x02):  UUID → f64
MSG_TEXT_STATES (0x03):   UUID → text + icon
MSG_DAYTIMER_STATES (0x04): UUID → schedule entries
MSG_WEATHER_STATES (0x07):  UUID → weather data
MSG_KEEPALIVE (0x06):       Heartbeat
MSG_OUT_OF_SERVICE (0x05):  Miniserver offline
```

---

## 5. BLOCK PARAMETERS

### Parameter Types

Based on connector-types.json, parameters have:

1. **Name** - Unique identifier (MinP, MaxP, Step, Time, etc.)
2. **Type** - Inferred from context (numeric, boolean, enum)
3. **Default Value** - Initial value (e.g., `default=0.5`)
4. **Range** - Min/Max constraints (e.g., 0-100 for brightness)
5. **Unit** - Implicit (seconds, %, °C, etc.)

### Parameter Categories

| Category | Examples | Range/Default | Purpose |
|----------|----------|---------------|---------|
| **Numeric** | Min, Max, Step, Time, Delay | 0-100, 0-3600s | Value constraints, timing |
| **Threshold** | On, Off, Hysteresis | 0-100 | Trigger points |
| **Duration** | Delay, FadingTime, AlarmPeriod | 0-3600s | Timing control |
| **Count** | MaxRuns, Max, Def | 1-255 | Step/state limits |
| **Enum (Boolean)** | Wrap, NoLast, RGBalt, Inv | Present/absent | Mode flags |

### Real Examples

**AMinmax (Analog Limiter):**
```
Input (I, default=0)
AQ (O, default=0)
Min (P, default=0)        ← Lower bound
Max (P, default=10)       ← Upper bound
```
*Function:* AQ = clamp(Input, Min, Max)

**OnDelay (Switch-On Delay):**
```
InputTrigger (I)
Q (O)
Time (P, default=300)     ← Delay in seconds
```
*Function:* Q = delayed ON pulse after Time seconds

**LightController2:**
```
AQ1, AQ2 (O, default=0)   ← Brightness outputs (0-100%)
Scene (O, default=0)      ← Selected scene number
MaxP (P, default=0.35)    ← Max power (0.35 = 35%)
Step (P, default=2)       ← Increment per button press (%)
Steptime (P, default=0.2) ← Time between steps (s)
Min (P, default=0)        ← Min brightness
Max (P, default=100)      ← Max brightness
FadingTime (P, default=1) ← Fade duration (s)
SceneMixTime (P, default=1) ← Scene transition time (s)
BrightnessLimit (P, default=30) ← Alarm brightness
```

### Parameter Setting in Code

```rust
// From config_edit.rs
pub fn set_param(&mut self, selector: &str, param: &str, value: &str) -> Result<()> {
    let path = self.require_one(selector)?;
    let elem = self.get_element_mut(&path);
    
    for child in &mut elem.children {
        if let Some(co) = child.as_mut_element()
            && co.name == "Co"
            && co.attributes.get("K").map(|k| k == param).unwrap_or(false)
        {
            co.attributes.insert("Def".to_string(), value.to_string());
            return Ok(());
        }
    }
    bail!("Connector '{}' not found on block", param)
}
```

---

## 6. MAIN CONTROL TYPES & COMPLEXITY

### Lighting Control
- **LightController** (V1, V2, H) - 24-33 connectors
- **Scene support:** Multiple moods/scenes with fade/mix timing
- **Parameters:** Brightness limits, fade curves, scene mix time
- **Complexity:** RGB, temperature, alarm clock integration
- **Real Example:** *"Set living room to 50% brightness with 1-second fade when motion detected"*

### Climate & Room Control
- **HeatIRoomController2** (49 connectors) - Most complex
- **Temperature zones:** Comfort/Save/Party/DeepSleep modes
- **Inputs:** Window contact, occupancy, HVAC demand
- **Parameters:** TimeMove (how long to reach target), TimeC (cool time), TimeS (switch time)
- **Complexity:** Multi-zone coordination, weather integration
- **Real Example:** *"Automatically reduce heating to 18°C when window opens, restore to 22°C when closed"*

### Blind/Shading Control
- **AutoJalousie** (31 connectors) - Weather-responsive automation
- **EIBJalousie** (6 connectors) - Simple KNX blind control
- **Inputs:** Wind speed, sun intensity, manual override
- **Parameters:** Wind threshold (>30 km/h), sun threshold (>600 W/m²)
- **Complexity:** Multi-zone synchronization, safety shutdowns
- **Real Example:** *"Close all pergolas when wind exceeds 30 km/h to prevent damage"*

### HVAC Control
- **HVACController** (36 connectors) - Complete heating/cooling system
- **AcControl** (26 connectors) - Air conditioning with modes
- **Features:** Mode selection, fan speed, temperature setpoints
- **Parameters:** Min/max temp (1-40°C), mode defaults
- **Complexity:** Mode coordination, simultaneous heating/cooling prevention
- **Real Example:** *"Select fan speed based on demand: Off/Low/Medium/High"*

### Energy Management
- **Energy/EnergyManager/EnergyManager2** (7-21 connectors)
- **Meter blocks:** Track generation, consumption, feed-in
- **Parameters:** Meter ratios, accumulation rates
- **Complexity:** Multi-channel metering, energy flow direction detection
- **Real Example:** *"Track total solar production and battery storage separately"*

### Access & Security
- **Access** (7 connectors) - Trigger-based access with timing
- **Doorcontroller** - Door state machine (locked/unlocked/jammed)
- **Inputs:** Trigger, ExternalID (from RFID/NFC), Disable
- **Parameters:** TrTime (transition time), PulseTime (pulse duration)
- **Complexity:** Multi-level access, audit logging
- **Real Example:** *"Unlock door for 10 seconds when valid NFC tag detected"*

### State Machines & Selection
- **State block** (3 connectors) - Multi-state selector
- **Sequencer** (8 connectors) - State machine with transitions
- **Mode selection:** Auto/Manual/Economy/Standby
- **Parameters:** Max (number of states), Def (default state)
- **Complexity:** State memory, timeout handling
- **Real Example:** *"Remember selected mode (Auto/Manual/Economy) even after power loss"*

---

## 7. WIRING PATTERNS & COMPLEXITY

### Simple Linear Wiring
```
PushButton → And → LightController
  (trigger)  (logic)  (output)
```

### Feedback Loops
```
LightController → Memory → Logic → LightController
  (brightness)   (state)   (threshold)  (input)
```

### Multi-Input Aggregation
```
Temp1 ⎤
Temp2 ├→ Average → Comparator → Alarm
Temp3 ⎦
```

### Cross-Page Wiring
- Connections between blocks on different UI pages
- Analyzed in config: 85+ cross-page connections in real configs
- Used for: Room-to-room coordination, system-wide state aggregation

### Complexity Patterns
1. **Type Mismatch:** Binary to analog (requires scaling)
2. **Multi-Output Selection:** Multiple outputs from one block
3. **Parameter Feedback:** Output feeds back as parameter
4. **Event Timing:** Multiple delays chained
5. **State Explosion:** Sequencer + memory (2^N states)

---

## 8. EXISTING SIMULATION/EVALUATION CODE

### Status: **NO existing evaluation/simulation engine found**

Searches for "simulate", "eval", "execute", "tick", "step", "process" in code found:
- ✅ **Command execution:** `client.rs` - sends commands to Miniserver
- ✅ **Dry-run mode:** `main.rs` - prints what would execute without running
- ✅ **State streaming:** `stream.rs` - receives state updates via WebSocket
- ❌ **No block evaluation:** No code that executes blocks in sequence
- ❌ **No state propagation:** No code that simulates signal flow through graph
- ❌ **No timing simulation:** No code that advances time and triggers delays

### What EXISTS:
```rust
// From main.rs - execute commands
pub fn execute_command(&self, uuid: &str, command: &str) -> Result<Response> {
    // Sends to real Miniserver, doesn't simulate locally
}

// From stream.rs - receive state events
pub enum StateEvent {
    ValueState { uuid: String, value: f64 },
    // Listens to Miniserver, doesn't generate them
}
```

### What's MISSING (needed for differentiable simulator):
- [ ] Block state representation (struct with current values)
- [ ] Propagation algorithm (topological sort + evaluation)
- [ ] Time advancement (discrete steps or continuous)
- [ ] Connector resolution (from reference to actual block)
- [ ] Type conversion (binary to analog, analog to binary)
- [ ] Parameter application (defaults, overwrites)
- [ ] Gradient computation (for autodiff)

---

## SUMMARY TABLE

| Aspect | Details |
|--------|---------|
| **Total Block Types** | 221 types |
| **Connector Specifications** | 190+ types, 2,384+ configurations |
| **Main Categories** | Logic (14), Math (16), Timing (11), I/O (13), Metering (13), Control (40+) |
| **Most Complex** | HeatIRoomController2 (49 conn), AutoJalousie (31 conn), LightController2 (24 conn) |
| **Parameter Types** | Numeric, Threshold, Duration, Count, Enum/Boolean |
| **State Types** | Numeric (f64), Text, DayTimer schedule, Weather forecast |
| **Wiring Direction** | Unidirectional: Source Output → Target Input |
| **Config Format** | ZIP with LoxAPP3.xml/json (XML elements or JSON structure) |
| **Protocol** | WebSocket binary (StateEvents streamed from Miniserver) |
| **Simulation Support** | None (receive-only, no generation/execution) |

---

## RECOMMENDED DATA STRUCTURES FOR DIFFERENTIABLE SIMULATOR

```rust
pub struct Block {
    uuid: String,
    block_type: String,
    title: String,
    connectors: Vec<Connector>,
    state: BlockState,
}

pub struct Connector {
    key: String,        // "Q", "I1", "MaxP", etc.
    io_type: IOType,    // Input, Output, Parameter
    value: f64,
    default_value: f64,
    wired_from: Option<(String, String)>,  // (block_uuid, connector_key)
}

pub enum IOType {
    Input,
    Output,
    Parameter,
}

pub struct BlockGraph {
    blocks: HashMap<String, Block>,
    execution_order: Vec<String>,  // Topological sort
    time_step: f64,
}

pub trait BlockEvaluator {
    fn evaluate(&self, inputs: &[f64], params: &[f64]) -> Vec<f64>;
}
```

