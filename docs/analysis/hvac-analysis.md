# HVAC Loxone Configuration Analysis Report

## Executive Summary

Analyzed `/tmp/hvac-fixed.Loxone` - A comprehensive HVAC system with 1,416 blocks across **161 unique block types**.

**Key Findings:**
- **14 Intelligent Room Controllers** managing multi-zone heating/cooling
- **9 Flow Temperature Calculators** (Heatmixer2) controlling radiators, floor heating, and fancoils
- **63 Memory blocks** for state persistence and mode tracking
- **174 Input References** feeding 84+ HVAC sensors
- **9 State machines** monitoring critical systems (DHW, CO sensors, fan speed)
- **116 block types NOT in connector-map.json** - our eval/simulator is incomplete for this domain

---

## 1. Complete Block Type Inventory (161 types, 1,416 instances)

### Top 20 Most Used Block Types:
```
  1. InputRef:              174  (sensor references)
  2. TreeSensor:            158  (device tree sensors)
  3. TreeAsensor:            88  (analog tree sensors)
  4. RightGroup:             87  (UI organization)
  5. CallerVirtualIn:        70  (virtual input callers)
  6. Memory:                 63  (state persistence)
  7. Online:                 51  (connectivity status)
  8. Category:               46  (organization)
  9. Text:                   44  (UI labels)
 10. Place:                  41  (rooms/zones)
 11. TreeDevice:             39  (physical devices)
 12. OutputRef:              35  (actuator references)
 13. TreeActor:              26  (device tree actors)
 14. ModbusASensor:          25  (Modbus analog sensors)
 15. TreeAactor:             25  (analog device actors)
 16. Actor:                  22  (actuator controls)
 17. Mode:                   20  (operational modes)
 18. Page:                   18  (UI pages)
 19. LoxCaption:             17  (UI captions)
 20. LoxAIRsensor:           14  (Lox AIR sensors)
```

### Critical HVAC Blocks:
- **HeatIRoomController2:** 14 blocks (intelligent room control)
- **Heatmixer2:** 9 blocks (flow temperature calculation)
- **State:** 9 blocks (state machines)
- **Minmax:** 9 blocks (min/max logic)
- **AnalogMultiplexer2:** 6 blocks (signal routing)
- **OnDelay:** 7 blocks (timer delays)

### Blocks NOT in connector-map.json (116 unsupported types):

**Category: Sensors & Hardware**
- Lox1wireDevice, Lox1wireAsensor (1-wire devices)
- LoxAIR, LoxAIRsensor, LoxAIRAactor, LoxAIRAsensor, LoxAIRDevice, LoxAIRactor
- ModbusDevice, ModbusSensor, ModbusASensor, ModbusAActor, ModbusActor
- DigitalIn, VoltageIn, DimCurrentIn (analog inputs)

**Category: System/Infrastructure**
- TreeDevice, TreeSensor, TreeAsensor, TreeActor, TreeAactor, LoxTree
- CallerVirtualIn, RightGroup, Place, Page, Mode, Category, Text
- Memory, SysVar, Online, Logger, Notification

**Category: Controllers & Automation**
- HeatIRoomController2, Heatmixer2 (HVAC-specific!)
- LightController2, HVACController, AcControl, HvacAC
- Ventilation, OvertempShutdown, WeatherData

**Category: Time & Calendar**
- Calendar, CalendarEntry, Day2009, Day, Month, Year, Hour, Minute, Second
- Sunrise, Sunset, Daylight, Morningtwilight, Eveningtwilight, SunAltitude, SunAzimuth

**Category: UI/Metadata**
- Document, CategoryCaption, PlaceCaption, UserCaption, MemoryCaption, OutputCaption, etc.

---

## 2. Rooms (Places) - 41 Total

**Multi-zone coverage across 41 physical spaces:**

Attic, Bar, Bathroom, Bedroom, Central, Conference Room, Conservatory, Dining Room, Dressing Room, Fancoils, Garage, Garage ventilation, Garden, Gym, Hall, Ironing Room, Kitchen, Landing, Laundry Room, Living Room, Measurement, Mechanical Room, Mud Room, Not Assigned, Nursery, Outside, Patio, Playroom, Pool, Porch, Powder Room, Sauna, Server Room, Stairwell, Storage Cellar, Study, Typical application 1, Typical application 2, Ventilation 1, Ventilation 2, Washing Area

---

## 3. Intelligent Room Controllers (HeatIRoomController2) - 14 Blocks

### Detailed Configuration:

| # | Name | Heat Comfort | Cool Comfort | HC Setpoint | Temp Δ | Max | Save | Sources |
|---|------|--------------|--------------|-------------|--------|-----|------|---------|
| 1 | Intelligent Room Controller | 22.5°C | 24.5°C | 22°C | 1.5°C | 28°C | 2°C | Heat pump |
| 2 | Intelligent Room Controller | 18°C | 20°C | 22°C | 1.5°C | 28°C | 2°C | Heat pump; AC Central Controller |
| 3 | Intelligent Room Controller | 22.5°C | 24.5°C | 22°C | 1.5°C | 28°C | 2°C | Heat pump |
| 4 | Intelligent Room Controller | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | Heat pump |
| 5 | Room 1 (Radiator) | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | Main heating source |
| 6 | Room 2 (Radiator) | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | Main heating source |
| 7 | Room 3 (Floor heating) | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | Main heating source |
| 8 | Room 4 (Floor heating) | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | Main heating source |
| 9 | Intelligent Room Controller | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | Main heating source |
| 10 | Intelligent Room Controller | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | Main heating source |
| 11 | Intelligent Room Controller | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | Heating and Cooling Controller |
| 12 | Room 1 | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | (none) |
| 13 | Room 2 | 20°C | 22°C | 22°C | 1.5°C | 28°C | 2°C | (none) |
| 14 | Intelligent Room Controller | 22.5°C | 24.5°C | 22°C | 1.5°C | 28°C | 2°C | Master Bedroom |

**Pattern:** All use 1.5°C hysteresis. Multiple configurations for 2-point heat/cool vs heating-only vs cooling-only.

---

## 4. Flow Temperature Calculators (Heatmixer2) - 9 Blocks

### Configuration Details:

| Block | Rooms | Controllers | Mode | Cool | Temp Range | Notes |
|-------|-------|------------|------|------|-----------|-------|
| Flow Temperature Calculator | Living Room, Bedroom, Mud Room, Bathroom | 4 | 1 | 2 | 5-35°C | Primary heating/cooling loop |
| Radiators flow temp. calculator | Typical application 1 | 2 | 1 | 2 | 5-55°C | High-temp radiator circuit |
| Floor heating flow temp. calculator | Typical application 1 | 2 | 1 | 2 | 5-40°C | Low-temp floor heating circuit |
| Flow Temperature Calculator | Typical application 2 | 2 | 1 | 2 | 5-40°C | Secondary zones |
| Flow Temperature Calculator | Fancoils | 1 | 1 | 1 | 5-60°C | Central fancoil unit |
| Room 1 Heating Calculator | Fancoils | 1 | 1 | 1 | 25-70°C | Fancoil room 1 heating |
| Room 2 Cooling Calculator | Fancoils | 1 | 1 | (none) | 5-15°C | Fancoil room 2 cooling |
| Room 2 Heating Calculator | Fancoils | 1 | 1 | 1 | 25-70°C | Fancoil room 2 heating |
| Room 1 Cooling Calculator | Fancoils | 1 | 1 | (none) | 5-15°C | Fancoil room 1 cooling |

**Key Parameters:**
- External sensor default: -5.646°C (weather temperature)
- Min setpoint: 5-25°C
- Max setpoint: 15-70°C
- Proportional control with transconductance = 0.5
- Threshold = 35°C buffer temperature
- 5-second update rate for all

---

## 5. State Machines (State Blocks) - 9 Critical Monitors

These implement finite state automata for critical system monitoring:

1. **DHW tank temperature monitor** - Hot water tank safety
2. **CO sensor 1.1** - Carbon monoxide detection zone 1
3. **CO sensor 1.2** - Carbon monoxide detection zone 1b
4. **CO sensor 2.2** - Carbon monoxide detection zone 2
5. **CO sensor 2.1** - Carbon monoxide detection zone 2
6. **Fan speed limiter** (3 instances) - Speed ramping for noise control
7. **Status** - Overall system health indicator

---

## 6. Memory Blocks for State Persistence (63 Total)

### HVAC Control States (23):
Boost (2x), Circulation pump, Circulation pump cooling, Circulation pump heating, Cooling demand, DHW demand (4x), Fan speed (5x), Fan speed bedroom, Heating demand (2x), Heating flow temp. request, HRV mode, On-demand circulation, Room 1 mode, Room 2 mode, Thermal desinfection of piping, Ventilation demand (2x)

### Comfort/Occupancy States (8):
Arriving at Home, Evacuate Garage, Good Morning, Good night Master Bedroom, Goodnight, Goodnight Bedroom, Leaving House, Leaving House Mud Room

### Safety/Monitoring (10):
1st floor CO critical, 1st floor CO warning, 2nd floor CO critical, 2nd floor CO warning, Presence in building, Room Controller Reset, Temp. sensor error, Temperature setpoint (2x), Value out of the range

### Environmental (13):
Day zone CO2, Dew point Room1, Dew point Room2, Dew Point OK, Humidity OK, Leaving Room Bathroom, Leaving Room Bedroom, Leaving Room Living Room, Leaving Room Mud Room, Night Zone CO2, Outside Absolute Humidity, Qp Cooling room 1, Qp Cooling room 2, Qp Heating room 1, Qp Heating room 2, Required thermal disinfection, Room Absolute Humidity

---

## 7. Logic & Automation Blocks

### InputRef (174 total) - HVAC Sensor Inputs

**Environmental Sensors (30+):**
- Buffer tank floor heating temperature
- CO sensors (1.1, 1.2, 2.1, 2.2)
- CO2 levels (4 zones)
- Cooling buffer temperature (2x)
- Dew point calculations (Room1, Room2)
- Dew Point OK status
- Exhaust air temperature
- Flow temperature (2x)
- Heating buffer temperature (2x)
- Humidity levels (multiple zones)
- Outside air humidity
- Outside temperature
- Pressure sensors
- Relative humidity
- Room temperature sensors
- Supply air humidity
- Supply air temperature

### OutputRef (35 total) - Actuator Outputs

**HVAC Actuators:**
- Basement Fan
- Boiler pump
- Boiler valve (heating/cooling)
- Circulation pumps (heating/cooling)
- Cooling coil control
- DHW heating element
- Fancoil controllers (multiple rooms)
- Floor heating valves
- Heat recovery ventilation (HRV) bypass
- Heating coil control
- Humidifier control
- Outdoor temperature (sensor reference)
- Radiator thermostatic valves
- Ventilation dampers

### Minmax Blocks (9) - Multi-input Aggregation

These combine multiple sensor signals to produce max/min of N inputs:

1. **Cooling temp.** - Maximum cooling demand from all zones
2. **Desired temp.** - Min/max room setpoint averaging
3. **Heating temp.** - Maximum heating demand from all zones
4. **Heating/ventilation demand** (3x) - Multi-source demand aggregation
5. **Max CO2** - Highest CO2 level across building
6. **Max Humidity** - Peak humidity detection
7. **Min Dew point** - Minimum dew point for condensation prevention

**Pattern:** Used to aggregate multi-zone sensor data into system-level control signals.

### OnDelay Blocks (7) - Timer-Based Automation

Implements delays for:
1. **10 min** (2x) - Circulation pump warmup delay
2. **10 s** (2x) - Sensor debounce
3. **10s delay** - Startup hysteresis
4. **push not. after 10s** (2x) - Notification suppression

### AnalogMultiplexer2 (6) - Signal Routing

These select 1 input from N possibilities:
1. **temp. request** (2x) - Select heating vs cooling setpoint
2. **H/C Slope** (3x) - Switch between heating/cooling curves
3. One more (unlabeled)

---

## 8. Novel Block Types We Haven't Seen

### New Hardware Interfaces:
- **Lox1wireDevice** (13 blocks) - 1-wire device gateway
- **Lox1wireAsensor** (13 blocks) - 1-wire analog sensors
- **LoxAIR** (1 block) - Lox AIR quality monitoring system
- **LoxAIRsensor** (14 blocks) - CO2/humidity sensors
- **LoxAIRactor** (1 block) - AIR system actuators
- **LoxAIRAactor** (9 blocks) - AIR analog actuators
- **LoxAIRAsensor** (8 blocks) - AIR analog sensors
- **LoxAIRDevice** (4 blocks) - AIR device management

### New Control Blocks:
- **HeatIRoomController2** (14 blocks) - Intelligent room controller with built-in scheduling
- **Heatmixer2** (9 blocks) - Advanced 2-point flow temperature control
- **HVACController** (4 blocks) - Generic HVAC controller
- **TreeDevice/TreeSensor/TreeAsensor** - Device tree hierarchy (158+88 blocks)

### New Sensors:
- **ModbusASensor** (25 blocks) - Modbus analog sensors
- **ModbusSensor** (3 blocks) - Modbus discrete sensors
- **WeatherData** (11 blocks) - Weather service integration
- **Lox1wireAsensor** (13 blocks) - 1-wire temperature sensors

---

## 9. HVAC-Specific Automation Patterns NOT in Our Eval Set

### Pattern 1: Multi-Zone Adaptive Mixing

**What it is:**
9 Heatmixer2 blocks managing different circuits (radiators 5-55°C, floor heating 5-40°C, fancoils 5-70°C) with proportional control using external weather temperature.

**Homeowner Utterance:**
> "Adjust the heating system so radiators get hotter water in winter and cooler water in summer, with floor heating staying moderate all year round based on the outside temperature."

**Technical Implementation:**
- External temperature sensor input
- Hysteresis control (1.5°C deadband)
- Three separate circuits with different min/max setpoints
- Proportional gain = 0.5 (0.5°C rise per °C above 35°C buffer threshold)
- Transconductance = 0.5 for smooth control curves

---

### Pattern 2: Intelligent Room Control with Multi-Mode Scheduling

**What it is:**
14 HeatIRoomController2 blocks with per-room comfort setpoints, hysteresis, and mode switching. Each room can independently be in heating-only, cooling-only, or heat/cool mode.

**Homeowner Utterance:**
> "Make the living room comfortable at 22.5°C for heating but 24.5°C for cooling. Drop it to 20°C in save mode, but never let it go above 28°C. The bedroom needs 18°C heating but 20°C cooling."

**Technical Implementation:**
- Separate setpoints per room for heat/cool/HC modes
- 1.5°C hysteresis (prevents chattering)
- Per-room mode selection (heating-only vs cooling-only vs auto)
- Temperature difference override (±1.5°C from setpoint)
- Max temperature safety limit (28°C)
- Save mode override (-2°C from comfort)

---

### Pattern 3: Proportional Flow Temperature Control

**What it is:**
Calculates required flow temperature based on:
- Room demand (0-100%)
- External air temperature
- Proportional gain
- Min/max saturation
- Buffer temperature feedback

**Homeowner Utterance:**
> "Heat the radiators less aggressively on mild days and more aggressively on cold days. Use the weather temperature to figure out how hot the water needs to be, with a minimum of 5°C and maximum of 55°C."

**Technical Implementation:**
```
Flow_Temp = (Demand × Gain + External_Temp × Transconductance) + Buffer_Bias
clamped to [Min, Max] at each calculation cycle
Update interval: 5 seconds
```

---

### Pattern 4: DHW Safety with State Machine

**What it is:**
State machine monitoring DHW tank temperature with multi-state thresholds for:
- Low temperature warning
- Critical temperature alarm
- Thermal disinfection trigger (boost heating)
- Circulation pump enable/disable logic

**Homeowner Utterance:**
> "Watch the hot water tank temperature and tell me if it drops below safe levels. Trigger a boost heating cycle if it gets too cold, and switch on the circulation pump only when the tank is hot enough."

**Technical Implementation:**
- 9 state blocks monitoring system safety
- Memory blocks storing state transitions
- OnDelay blocks preventing rapid cycling
- OutputRef blocks controlling circulation pumps and heating elements
- CO sensor integration (4 sensors for 2 zones)

---

### Pattern 5: CO2-Based Ventilation Demand

**What it is:**
Tracks CO2 levels across 4 zones and triggers ventilation based on:
- Zone-specific thresholds
- Day zone vs night zone differentiation
- Memory block state tracking
- Max CO2 aggregation via Minmax block

**Homeowner Utterance:**
> "If the CO2 level in any room gets above 1000 ppm during the day or 700 ppm at night, run the ventilation system to bring in fresh air. Keep track of which zones need ventilation."

**Technical Implementation:**
- InputRef blocks: 4 CO2 sensors
- Memory blocks: Day zone CO2, Night Zone CO2
- Minmax block: Max CO2 across all zones
- Threshold trigger: >1000 (day), >700 (night)
- OutputRef: Ventilation damper control

---

### Pattern 6: Dew Point Condensation Prevention

**What it is:**
Monitors dew point in multiple rooms and prevents condensation by:
- Calculating absolute humidity vs saturation point
- Switching ventilation/dehumidification on demand
- Per-room dew point tracking

**Homeowner Utterance:**
> "Watch the humidity in the living room and bedroom. If it looks like dew might form on windows, turn on ventilation to dry out the air."

**Technical Implementation:**
- InputRef: Dew point sensors (Room1, Room2)
- Memory: Dew point Room1, Dew point Room2, Dew Point OK
- Minmax: Min Dew point (safety threshold)
- Logic: If current_temp < dew_point + 2°C → activate dehumidification
- Debounce: 10s OnDelay to prevent rapid cycling

---

### Pattern 7: Fancoil Room-Specific Heating/Cooling

**What it is:**
9 Heatmixer2 blocks for fancoil systems, allowing independent heat/cool control per room:
- Room 1: Heating (25-70°C) + Cooling (5-15°C)
- Room 2: Heating (25-70°C) + Cooling (5-15°C)

**Homeowner Utterance:**
> "Room 1 with the fancoil can heat up to 70°C when it's cold but only cool down to 5°C in summer. Room 2 has the same thing but they work independently—one room can be heating while the other is cooling."

**Technical Implementation:**
- Separate Heatmixer2 for each room + mode combination
- Independent setpoint control per room
- Proportional mix valve position calculation
- Cross-talk prevention (room 1 heating doesn't affect room 2)
- Feedback from room temperature sensors to each Heatmixer2

---

### Pattern 8: Presence-Based Comfort Mode Switching

**What it is:**
Memory blocks tracking:
- "Arriving at Home" (restore comfort setpoint)
- "Leaving House" (reduce setpoint)
- "Good Morning" (wake-up boost)
- "Goodnight" (sleep mode)
- Per-room "Leaving Room" tracking

**Homeowner Utterance:**
> "When we leave home in the morning, lower the thermostat to save energy. When we come back in the evening, warm it back up 20 minutes before we arrive. If someone is still in the living room at night, keep it comfortable, but if the bedroom is empty, that room can cool down."

**Technical Implementation:**
- 8 Memory blocks tracking presence events
- Geofencing integration (Leaving House trigger)
- CallerVirtualIn detecting button presses
- OnDelay timers for warm-up scheduling
- Per-room occupancy tracking (7 rooms with "Leaving Room" logic)

---

### Pattern 9: Boost/Bypass Control for Emergency Demand

**What it is:**
Quick-boost logic for undersized systems:
- Memory blocks: "Boost", "Boost", "Bypass ON"
- Raises temperature setpoint by fixed amount temporarily
- Overrides normal proportional control

**Homeowner Utterance:**
> "If someone calls for a really hot bath, boost the hot water heating for 15 minutes at maximum power. Then go back to normal once it's hot enough."

**Technical Implementation:**
- PushButton trigger for boost request
- Memory block stores boost state
- OnDelay timer (10 min) for automatic cutoff
- OutputRef: Direct valve/pump control (bypass mixer during boost)
- Temperature sensor feedback to detect completion

---

### Pattern 10: Thermal Disinfection Cycle

**What it is:**
Legionella prevention by periodically heating DHW tank to 60°C:
- Memory: "Thermal desinfection of piping"
- Memory: "Required thermal disinfection"
- Automatic trigger weekly or on-demand
- Scheduled via Calendar block

**Homeowner Utterance:**
> "Once a week, heat the water tank up to 60°C to kill any bacteria, then let it cool back down to normal. Schedule this for late night when nobody needs hot water."

**Technical Implementation:**
- Calendar block scheduling (weekly trigger)
- Memory blocks for state persistence
- Temporary setpoint override (+30°C boost)
- Temperature sensor feedback
- Duration limit (2 hours max) via OnDelay
- Notification when cycle completes

---

## 10. Comparison Summary: What We're Missing

### Blocks in HVAC Config But NOT in connector-map.json (116 types)

**Critical for HVAC eval but missing:**
- ✗ HeatIRoomController2 (14 instances!)
- ✗ Heatmixer2 (9 instances!)
- ✗ WeatherData (11 instances)
- ✗ Ventilation (2 instances)
- ✗ OvertempShutdown (2 instances)
- ✗ HVACController (4 instances)

**Device tree and sensor integration (158+88+39 blocks):**
- ✗ TreeDevice, TreeSensor, TreeAsensor
- ✗ TreeActor, TreeAactor
- ✗ Lox1wireDevice, Lox1wireAsensor
- ✗ LoxAIR, LoxAIRsensor, LoxAIRactor, LoxAIRAsensor, LoxAIRDevice

**Automation infrastructure we don't handle:**
- ✗ Memory (63 instances - state persistence!)
- ✗ CallerVirtualIn (70 instances - event routing)
- ✗ RightGroup (87 instances - UI organization)
- ✗ Place (41 instances - room/zone abstraction)

### Estimated Gap Analysis:

**Total blocks in HVAC config:** 1,416
**Total blocks we can meaningfully evaluate:** ~400 (28%)
**Blocks we're blind to:** ~1,000+ (72%)

**HVAC-critical blocks we cannot evaluate:**
- 14 HeatIRoomController2 (intelligent room control)
- 9 Heatmixer2 (flow temperature calculation)
- 63 Memory (state machine backbone)
- 158 TreeSensor (sensor integration layer)
- 11 WeatherData (weather integration)
- 35 OutputRef (actuator mapping)

---

## 11. Recommended Evaluation Set Additions

For meaningful HVAC automation testing, add evaluation utterances for:

1. **Room Comfort Control**
   - "Set the living room to 22 degrees for heating"
   - "The bedroom should be 20 degrees in heating mode but 24 in cooling"
   - "If it's warmer than 28 degrees, activate cooling in all rooms"

2. **Proportional Mixing**
   - "Adjust the radiator water temperature based on how cold it is outside"
   - "The floor heating should never exceed 40 degrees"
   - "Use a 0.5 gain factor for the proportional controller"

3. **Multi-Zone Control**
   - "Make the fancoils heat up to 70 degrees in winter"
   - "Cool the fancoils down to 5 degrees in summer"
   - "Let the living room and bedroom operate independently"

4. **Environmental Safety**
   - "If the CO2 level gets too high, turn on ventilation"
   - "Alert me if any room's humidity approaches the dew point"
   - "Check the hot water tank temperature and warn me if it's too low"

5. **Presence-Based Automation**
   - "Warm up the house 20 minutes before we arrive home"
   - "Lower the temperature 5 degrees when nobody's home"
   - "If I leave the living room, stop heating it after 30 minutes"

6. **Emergency Control**
   - "Boost the hot water heating to maximum for 15 minutes"
   - "Disable the radiators and use the bypass valve instead"
   - "Trigger a thermal disinfection cycle on the hot water tank"

