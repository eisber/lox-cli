# HVAC Configuration Analysis - Complete Index

## Quick Navigation

### 📊 Analysis Files Generated

1. **HVAC_ANALYSIS.md** (22 KB) - Complete technical analysis
   - All 161 block types with counts
   - Detailed HeatIRoomController2 configurations (14 blocks)
   - Detailed Heatmixer2 configurations (9 blocks)
   - All 63 Memory blocks categorized
   - All 9 State machines
   - All 9 Minmax blocks
   - All 7 OnDelay blocks
   - 10 specific homeowner utterance patterns
   - Comparison vs connector-map.json

2. **HVAC_KEY_FINDINGS.txt** (7.9 KB) - Executive summary
   - Critical findings at a glance
   - Evaluation coverage gap analysis
   - HVAC-unique patterns not in eval set
   - Block type distribution
   - 6 homeowner utterance examples
   - Recommended priorities

3. **This file** - Navigation and quick reference

---

## Key Numbers

| Metric | Count | Status |
|--------|-------|--------|
| Total Blocks | 1,416 | ✓ |
| Unique Block Types | 161 | ✓ |
| Rooms/Zones | 41 | ✓ |
| Room Controllers (HeatIRoomController2) | 14 | ✗ Not supported |
| Flow Temp Calculators (Heatmixer2) | 9 | ✗ Not supported |
| Memory Blocks | 63 | ✗ Not supported |
| State Machines | 9 | ✗ Not supported |
| Sensor Inputs (InputRef) | 174 | Partial |
| Actuator Outputs (OutputRef) | 35 | Partial |
| Minmax Aggregators | 9 | ✗ Not supported |
| OnDelay Timers | 7 | ✗ Not supported |
| AnalogMultiplexer2 Routers | 6 | ✗ Not supported |

**Overall Coverage: ~28% (400/1,416 blocks)**

---

## Critical Missing Blocks for HVAC Evaluation

### 1. HeatIRoomController2 (14 blocks)
**Purpose:** Intelligent room-level heating/cooling control

**Key Parameters:**
- Per-room comfort setpoint (heat: 18-22.5°C, cool: 20-24.5°C)
- Hysteresis: 1.5°C
- Save mode: -2°C from comfort
- Max temperature safety: 28°C
- Mode support: Heating-only, Cooling-only, Auto heat/cool

**Example utterance:**
> "Set the living room to 22.5°C for heating but 24.5°C for cooling, with a 1.5 degree hysteresis. In save mode, drop it 2 degrees."

### 2. Heatmixer2 (9 blocks)
**Purpose:** Proportional flow temperature calculation

**Key Configuration:**
- External temperature sensor input
- Proportional gain: 0.5
- Transconductance: 0.5
- Different circuits: Radiators (5-55°C), Floor heating (5-40°C), Fancoils (5-70°C)
- 5-second update cycle

**Example utterance:**
> "Use the outside temperature to calculate how hot the radiator water should be. Keep it between 5 and 55 degrees, using weather data to adjust."

### 3. Memory (63 blocks)
**Purpose:** State persistence for HVAC logic

**Categories:**
- HVAC Control (23): Boost, circulation pumps, heating/cooling demand
- Comfort/Occupancy (8): Arriving home, leaving house, good morning
- Safety (10): CO warnings, sensor errors, temperature setpoint
- Environmental (13): CO2 zones, dew points, humidity

**Example utterance:**
> "Remember if someone's arriving home so we can pre-heat the house 20 minutes before they get here."

### 4. State Machines (9 blocks)
**Purpose:** Safety monitoring with state transitions

**Systems Monitored:**
1. DHW tank temperature (low/critical/safe states)
2. CO sensors (4 total - 2 zones × 2 sensors)
3. Fan speed limiters (3 instances)

**Example utterance:**
> "Monitor the hot water tank temperature and trigger a heating boost if it falls below 50 degrees, or alert me if CO levels get dangerous."

---

## 41 Rooms/Zones

**Living Spaces:** Attic, Bar, Bathroom, Bedroom, Conservatory, Conference Room, Dining Room, Dressing Room, Gym, Hall, Kitchen, Living Room, Mud Room, Nursery, Playroom, Porch, Study

**Service Areas:** Garage, Garage ventilation, Ironing Room, Laundry Room, Mechanical Room, Pool, Sauna, Stairwell, Storage Cellar, Washing Area

**Measurement & Control:** Central, Fancoils, Measurement, Outside

**Templates:** Typical application 1, Typical application 2, Ventilation 1, Ventilation 2, Not Assigned (unplaced objects)

---

## 63 Memory Blocks Breakdown

### HVAC Control (23 blocks)
```
Boost (2x)
Circulation pump
Circulation pump cooling
Circulation pump heating
Cooling demand
DHW demand (4x)
Fan speed (5x)
Fan speed bedroom
Heating demand (2x)
Heating flow temp. request
HRV mode
On-demand circulation
Room 1 mode
Room 2 mode
Ventilation demand (2x)
```

### Comfort & Occupancy (8 blocks)
```
Arriving at Home
Evacuate Garage
Good Morning
Good night Master Bedroom
Goodnight
Goodnight Bedroom
Leaving House
Leaving House Mud Room
```

### Safety & Critical (10 blocks)
```
1st floor CO critical
1st floor CO warning
2nd floor CO critical
2nd floor CO warning
Presence in building
Room Controller Reset
Temp. sensor error
Temperature setpoint (2x)
Value out of the range
```

### Environmental (13 blocks)
```
Day zone CO2
Dew point Room1
Dew point Room2
Dew Point OK
Humidity OK
Night Zone CO2
Outside Absolute Humidity
Qp Cooling room 1
Qp Cooling room 2
Qp Heating room 1
Qp Heating room 2
Required thermal disinfection
Room Absolute Humidity
Thermal desinfection of piping
```

---

## Unique HVAC Patterns Discovered

### Pattern 1: Multi-Circuit Temperature Management
- **Radiators:** 5-55°C (high temperature for fast heating)
- **Floor heating:** 5-40°C (low temperature, safety)
- **Fancoils:** 5-70°C (flexible, high heating capacity)
- **Control:** One Heatmixer2 per circuit with independent proportional mixing

### Pattern 2: Room-Specific Comfort with Mode Switching
- Each room can operate in heating-only, cooling-only, or auto mode
- Per-mode comfort setpoint (heat ≠ cool)
- 1.5°C hysteresis prevents rapid cycling
- Save mode override reduces setpoint by 2°C

### Pattern 3: Safety-Critical State Machines
- DHW tank monitoring (prevent legionella)
- CO sensor arrays (2 zones, 2 sensors each)
- Thermal disinfection cycles (weekly 60°C boost)
- Fan speed ramping (noise control)

### Pattern 4: Multi-Zone Sensor Aggregation
- 174 sensor inputs from 40+ devices
- 9 Minmax blocks aggregate to zone/system level
- Separate day/night thresholds (CO2, humidity)
- Memory blocks track state across power cycles

### Pattern 5: Presence-Based Automation
- "Arriving at Home" trigger (pre-heating)
- "Leaving House" trigger (setback)
- Per-room occupancy tracking ("Leaving Room X")
- Good Morning / Goodnight scheduling

### Pattern 6: Environmental Protection
- Dew point calculation (per room)
- Humidity monitoring (condensation prevention)
- Outside air quality integration
- Absolute humidity tracking (room vs supply air)

### Pattern 7: Boost & Bypass Control
- Temporary maximum power for DHW demand
- Mixer bypass during emergency boost
- Automatic reset after timeout
- Manual override capability

---

## Block Types NOT in connector-map.json (116 total)

### Hardware Integration (39 blocks)
```
Lox1wireDevice (13)          1-wire device gateway
Lox1wireAsensor (13)         1-wire analog sensors
LoxAIRsensor (14)            Air quality CO2/humidity sensors
LoxAIRactor (1)              Air quality actuator control
LoxAIRAactor (9)             Air quality analog actuators
LoxAIRAsensor (8)            Air quality analog sensors
LoxAIRDevice (4)             Air quality device manager
LoxAIR (1)                   Air quality integration
ModbusASensor (25)           Modbus analog sensor interface
ModbusSensor (3)             Modbus discrete sensor interface
WeatherData (11)             Weather service integration
HVACController (4)           Generic HVAC control
LightController2 (4)         Lighting control
Ventilation (2)              Ventilation-specific control
```

### System Infrastructure (158 blocks)
```
TreeDevice (39)              Physical device tree nodes
TreeSensor (158)             Device tree sensor representation
TreeAsensor (88)             Device tree analog sensor rep.
TreeActor (26)               Device tree actuator nodes
TreeAactor (25)              Device tree analog actuator rep.
LoxTree (2)                  Device tree management
```

### State Persistence (70 blocks)
```
Memory (63)                  Persistent state storage
SysVar (13)                  System variables
Online (51)                  Connectivity status tracking
CallVirtualIn (70)           Virtual input event routing
```

### UI & Organization (196 blocks)
```
Place (41)                   Room/zone abstraction
Page (18)                    UI page organization
Category (46)                Logical grouping
RightGroup (87)              UI element grouping
Text (44)                    Text labels
Mode (20)                    Operational mode definitions
Various Captions (17+)       UI metadata
```

---

## Homeowner Utterance Examples (By Pattern)

### 1. Proportional Mixing (Heatmixer2)
> "Adjust the heating system so radiators get hotter water in winter and cooler water in summer, with floor heating staying moderate all year round based on the outside temperature."

> "Use the weather forecast to adjust how hot the boiler should run—cooler on mild days, hotter on cold days. Keep the radiator water between 5 and 55 degrees."

### 2. Intelligent Room Control (HeatIRoomController2)
> "Set the living room to 22 degrees for heating but 24 degrees for cooling, with a 1.5 degree swing. Drop it to 20 on save mode but never exceed 28."

> "The bedroom should be cooler—18 degrees when heating, 20 degrees when cooling. Make the hall 20 degrees for both."

### 3. Safety Monitoring (State Machines)
> "Watch the hot water tank temperature and alert me if it drops below safe levels. Trigger a boost heating cycle if needed and switch on the circulation pump when hot."

> "Monitor carbon monoxide levels in both zones and show me warnings if they get dangerous. Cut the ventilation fan speed if CO levels rise."

### 4. Environmental Control (Minmax + Memory)
> "If the CO2 level in any room gets above 1000 ppm during the day or 700 ppm at night, run the ventilation to bring in fresh air."

> "Watch the humidity in the bedroom and living room. If it looks like dew might form on the windows, turn on ventilation to dry out the air."

### 5. Presence-Based Scheduling (Memory + OnDelay)
> "Warm up the house 20 minutes before we arrive home in the evening. When we leave in the morning, lower everything to 18 degrees to save energy."

> "If someone's in the living room at 10 PM, keep it comfortable. After 30 minutes of no movement, drop the temperature by 2 degrees."

### 6. Boost/Emergency Control (OnDelay + OutputRef)
> "If someone calls for a really hot bath, boost the hot water heating to maximum for 15 minutes, then return to normal."

> "During the day when someone's working from home, bump up the office temperature by 2 degrees and increase ventilation."

### 7. Thermal Disinfection (Calendar + State)
> "Once a week, heat the water tank up to 60 degrees to kill any bacteria, then let it cool back down. Schedule this for 2 AM when nobody needs hot water."

### 8. Multi-Zone Fancoil Control (Heatmixer2 × 4)
> "The conference room has a fancoil unit. Let it heat up to 70 degrees in winter but cool down to just 5 degrees in summer. Room 2 with the fancoil can do the same but independently."

### 9. Comfort Mode Defaults (HeatIRoomController2 + Memory)
> "Living room usually stays at 22.5 in winter and 24.5 in summer. Bedrooms are 20 degrees either way. Pool room stays at 28 degrees max."

### 10. Dew Point Prevention (Memory + Minmax)
> "Calculate the dew point in the bathroom and kitchen. If the actual temperature approaches the dew point within 2 degrees, automatically reduce humidity."

---

## Technical Implementation Patterns

### Flow Temperature Calculation
```
Flow_Temp = (RoomDemand × Gain + ExternalTemp × Transconductance) 
            + BufferTemp_Feedback
            
Clamped to [MinSetpoint, MaxSetpoint]
Update interval: 5 seconds
Gain: 0.5 for proportional response
```

### Room Control Loop
```
IF TargetTemp > CurrentTemp + Hysteresis:
    Send HEAT command
ELSE IF TargetTemp < CurrentTemp - Hysteresis:
    Send COOL command
ELSE:
    Hold current state
    
Hysteresis: 1.5°C (prevents cycling)
Save override: -2°C from setpoint
Max safety: 28°C absolute
```

### Dew Point Protection
```
IF RoomTemp < DewPoint + 2°C:
    Enable dehumidification
    Wait 10s debounce (OnDelay)
ELSE:
    Disable dehumidification
```

---

## Files in This Analysis

```
/home/amy/src/lox-cli/HVAC_ANALYSIS.md          Full technical report (22 KB)
/home/amy/src/lox-cli/HVAC_KEY_FINDINGS.txt     Executive summary (7.9 KB)
/home/amy/src/lox-cli/HVAC_INDEX.md             This navigation file
```

---

**Analysis Date:** April 21, 2025
**Source File:** `/tmp/hvac-fixed.Loxone`
**Total Blocks Analyzed:** 1,416
**Unique Types Found:** 161
**Coverage:** 28% of blocks supported in current eval/simulator
