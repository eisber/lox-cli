# Quick Reference: Missing Wiring Patterns in Eval Set

## Top Patterns Found (NOT in current eval set)

| Pattern | Block Type | Count | Homeowner Utterance | Priority |
|---------|-----------|-------|----------------------|----------|
| **Smart Climate Zone** | HeatIRoomController2, AcControl | 55 | "Set bedroom to 22°C during day, 18°C at night" | 🔴 HIGH |
| **Multi-Input Logic Gates** | And, Or, Comparators | 14 | "Turn on AC only if (temp>26) AND (sun>500W) AND (time 9-6pm)" | 🔴 HIGH |
| **Weather-Responsive Blinds** | EIBJalousie | 54 | "Close pergola if wind>30km/h OR sun intensity>600W/m²" | 🔴 HIGH |
| **Pulse Timing** | OnPulseDelay, PulseGen | 21 | "Send 200ms buzzer pulse when doorbell pressed" | 🟠 MEDIUM |
| **Mode Selector** | State | 14 | "Switch between Manual/Auto/Economy modes, remember on restart" | 🟠 MEDIUM |
| **Energy Accumulator** | Memory | 45 | "Track total solar energy produced today, add hourly values" | 🟠 MEDIUM |
| **Door Access Control** | Doorcontroller, NfcCodeTouch | 23 | "Unlock door with NFC tag after 6pm, show camera on TV" | 🟠 MEDIUM |
| **Cross-Page State Sync** | All types | 85 | "Reflect blind position on all UI pages simultaneously" | 🟡 LOW |
| **Smart Watering** | Irrigation | 9 | "Water garden if soil moisture<30% AND weather forecast dry" | 🟡 LOW |

## Block Type Distribution

**Already Covered:**
- ✅ Memory (threshold/state storage)
- ✅ DayTimer (time-based switching)
- ✅ PushButton (manual triggers)
- ✅ OnPulseDelay (timing - though different from Monoflop)
- ✅ And/Or/Not (basic logic)

**Completely New:**
- ❌ HeatIRoomController2 (intelligent thermostat)
- ❌ AcControl (AC zone management)
- ❌ EIBJalousie (motorized blinds/pergolas)
- ❌ State (multi-state selector)
- ❌ Doorcontroller (video doorbell control)
- ❌ Intercom (video intercom)
- ❌ NfcCodeTouch (NFC access control)
- ❌ Irrigation (smart watering)

## What These Patterns Enable

### 1. Room-Level Automation (not just whole-home)
Climate blocks allow **per-room temperature targets** with occupancy-based boost and window-open detection.

### 2. Complex Decision Trees
The config uses **3+ input And/Or gates** that evaluate conditions simultaneously (not sequential if-then chains).

### 3. Energy Awareness
Memory blocks **accumulate values over time** (daily PV production, cumulative consumption) to enable smart load-shifting.

### 4. Multi-Zone Coordination
Blinds, pergolas, and zones are **synchronized across the building** with weather-responsive triggers.

### 5. Security Integration
Doorbell + NFC + intercom create **access control workflows** with time-based rules and video feeds.

## Implementation Recommendation

**Phase 1 - Core Missing (do these first):**
1. `smart-climate-zone` — Most impactful (55 instances)
2. `multi-condition-logic` — Foundation for complex rules (14 instances)
3. `weather-responsive-blind` — Complete shading automation (54 instances)

**Phase 2 - Extended (add after Phase 1):**
4. `pulse-timing-control`
5. `mode-selector` with persistence
6. `energy-accumulator`

**Phase 3 - Integration (nice to have):**
7. `door-access-control`
8. `intercom-video`
9. `smart-watering`

---

## Data-Driven Facts

- **1,807 total wiring connections** across 2,944 blocks
- **46 unique block types** in wiring (vs ~18 in eval set)
- **85 cross-page connections** binding UI to logic
- **0 feedback loops detected** (data-flow architecture, not event loops)
- **Maximum chain depth: 2 blocks** (shallow computation, stateful storage)

This is a **modern KNX automation system** emphasizing:
- Distributed intelligence (per-room thermostats, zone control)
- Energy optimization (PV + storage + load-shifting)
- Multi-sensory awareness (weather, occupancy, time)
- Cross-system integration (door locks, CCTV, HVAC)
