# Missing Eval Cases: Homeowner Language Examples

Generated from analysis of `/tmp/koen-sanitized.Loxone`

---

## 1. SMART CLIMATE ZONE (HeatIRoomController2, AcControl)
**Block Count: 55 | Priority: 🔴 HIGH**

### What it represents:
Room-level intelligent heating/cooling with occupancy awareness, window detection, and fan speed control.

### Utterances:

#### Temperature Management
- "Set living room to 22°C during the day, 18°C at night"
- "Automatically boost bathroom heating to 24°C when door opens"
- "If bedroom temperature exceeds 23°C, turn on AC cooling"
- "Reduce heating setpoint by 3 degrees when window is open"
- "Maintain kitchen at 21°C, adjust if occupancy detected"

#### Fan Control
- "Run HVAC fan at low speed if temperature drifts 1 degree"
- "Switch to high fan speed when room exceeds 24°C"
- "Keep fan off during sleep mode, enable on motion detection"
- "Ramp up cooling fan gradually as temperature rises"

#### Multi-Room Coordination
- "Coordinate heating across master bedroom and child's bedroom"
- "If living room reaches 25°C, also cool kitchen"
- "Balance temperature between connected zones automatically"

#### Energy Optimization
- "Heat living room only between 6am-11pm, maintain bedroom heat 24/7"
- "Switch to economy mode (19°C target) when away"
- "Preheat bedroom to 22°C 30 minutes before wake time"

---

## 2. MULTI-INPUT LOGIC GATES (And, Or, Comparators)
**Block Count: 14 | Priority: 🔴 HIGH**

### What it represents:
Complex conditional logic combining 3+ independent conditions with AND/OR operators.

### Utterances:

#### Security Rules
- "Turn on security lights ONLY when (motion detected) AND (it's dark) AND (door is unlocked)"
- "Trigger alarm if (motion at garage) OR (door forced open) OR (window shattered)"
- "Arm security mode if (away from home) AND (all windows closed) AND (no presence detected)"

#### Lighting Rules
- "Activate patio lights if (sunset + 30min) AND (motion detected) AND (NOT in sleep mode)"
- "Dim lights if (brightness > 500 lux) OR (occupancy = none)"
- "Keep entryway lights on from (sunset) until (7am)"

#### Climate Rules
- "Start AC if (temperature > 26°C) AND (solar intensity > 500 W/m²) AND (time between 9am-6pm)"
- "Heat basement if (underground room) AND (moisture > 60%) AND (temperature < 16°C)"
- "Ventilate kitchen if (humidity > 70%) OR (cooking detected) OR (odor sensor triggered)"

#### Presence-Based Automation
- "Turn on lights if (presence detected) AND (brightness < 200 lux) OR (manual override active)"
- "Enable guest mode if (doorbell pressed) AND (time < 10pm) AND (manual toggle ON)"
- "Disable automation if (mode = manual) OR (occupancy = away) OR (maintenance window active)"

#### Energy Management
- "Charge battery if (solar production > 3kW) AND (grid demand < 1kW) AND (battery < 80%)"
- "Enable demand response if (grid price high) OR (solar = peak) OR (battery full)"

---

## 3. WEATHER-RESPONSIVE BLIND CONTROL (EIBJalousie)
**Block Count: 54 | Priority: 🔴 HIGH**

### What it represents:
Motorized blind/pergola automation responding to wind, solar intensity, and time-based schedules.

### Utterances:

#### Wind Protection
- "Retract all pergola awnings if wind speed exceeds 30 km/h"
- "Lower outdoor blinds immediately if wind gust > 40 km/h detected"
- "Resume normal blind position once wind drops below 20 km/h"

#### Solar Glare Management
- "Close outdoor blinds when sun intensity exceeds 600 W/m²"
- "Keep living room blinds closed between 9am-4pm during summer"
- "Open blinds to let morning sun in after 7am"
- "Close blinds if temperature rises above 25°C and blinds aren't already closed"

#### Multi-Zone Synchronization
- "Synchronize pergola 1 and 2 — if one retracts, retract both"
- "Keep all south-facing blinds in sync"
- "Master blind controls all kitchen blinds when automated mode active"

#### Manual Override
- "Allow manual control to override weather-based automation"
- "Resume weather rules 2 hours after last manual adjustment"
- "Lock blinds in place during strong wind, unlock after conditions improve"

#### Time-Based Scheduling
- "Close all blinds at sunset, open at 8am"
- "Keep patio blinds open on weekends, closed on weekdays until 6pm"
- "Raise blinds during night to prevent break-in appearance"

#### Privacy & Security
- "Close living room blinds between 6pm-8am for privacy"
- "If motion detected outside, close blinds automatically"

---

## 4. PULSE TIMING CONTROL (OnPulseDelay, PulseGen)
**Block Count: 21 | Priority: 🟠 MEDIUM**

### What it represents:
Generate brief electrical pulses or timed on/off signals, different from sustained states.

### Utterances:

#### Doorbell & Buzzer
- "Send a 500ms buzzer pulse when doorbell button pressed"
- "Generate double-pulse (100ms on, 100ms off, 100ms on) for alarm"
- "Create 1-second pulse to unlock electric strike on button press"

#### Stairwell Lighting
- "Stairwell lights: on for 5 minutes, then auto-off"
- "Motion-triggered stairwell light with 3-minute auto-shutdown"
- "Keep lights on while motion detected, fade out over 2 minutes after motion stops"

#### Garden Automation
- "Generate pulse every sunrise to trigger garden lights off"
- "Create 30-minute irrigation pulse when soil moisture drops below 40%"
- "Send sprinkler pulse (10 seconds on, 5 seconds off, repeat 3x) for morning watering"

#### Appliance Control
- "Send momentary pulse to washer start button (200ms)"
- "Trigger pool pump with 2-hour on, 4-hour off pulse pattern"
- "Generate periodic refresh pulse to keep devices awake"

---

## 5. MODE SELECTOR WITH PERSISTENCE (State)
**Block Count: 14 | Priority: 🟠 MEDIUM**

### What it represents:
Multi-state selection (Manual/Auto/Economy, Off/Low/Medium/High, etc.) that remembers choice after power cycle.

### Utterances:

#### HVAC Modes
- "Switch between Manual/Auto/Economy heating modes, remember selection on restart"
- "Select fan speed: Off → Low → Medium → High, persist across power loss"
- "Remember last selected temperature preference when system powers back on"

#### System Modes
- "Enable 3 modes: Home/Away/Sleep with automatic time-based switching"
- "Select security mode: Armed/Disarmed/Guest with state persistence"
- "Store last occupancy mode (Present/Away/Vacation) in non-volatile memory"

#### Lighting Modes
- "Switch between Day/Evening/Night lighting presets, remember setting"
- "Select color temperature: Warm/Neutral/Cool and keep preference"

#### Energy Modes
- "Toggle between Standard/Economy/Peak-Shaving operation, retain mode on reboot"
- "Select backup power priority: Critical/Essential/Full-Load per mode"

---

## 6. ENERGY ACCUMULATOR (Memory with tracking)
**Block Count: 45 | Priority: 🟠 MEDIUM**

### What it represents:
Store and accumulate energy/consumption values over time (daily totals, cumulative usage, etc.).

### Utterances:

#### Solar Energy Tracking
- "Store total solar power generated today in memory, update every hour"
- "Track cumulative kWh produced since year start, accumulate daily values"
- "Show running total of PV production vs. consumption for the day"

#### Grid Interaction
- "Track grid feed-in (excess solar sent to grid) as running total"
- "Accumulate negative power (grid import) per billing period"
- "Alert if feed-in exceeds 5kW sustained (trigger battery charging mode)"

#### Consumption Tracking
- "Record total household energy consumption per day in persistent memory"
- "Track consumption by room: kitchen/heating/EV separately"
- "Calculate daily cost based on kWh used and time-of-use rates"

#### Battery Management
- "Store current battery charge percentage, track charge/discharge cycles"
- "Remember battery state from last shutdown, resume operation"

---

## 7. DOOR ACCESS CONTROL (Doorcontroller, NfcCodeTouch, Intercom)
**Block Count: 23 | Priority: 🟠 MEDIUM**

### What it represents:
Video doorbell, NFC access control, and intercom integration with time-based and state-based rules.

### Utterances:

#### Video Doorbell Integration
- "When doorbell rings, show live camera feed on TV and living room tablet"
- "If motion at garage entrance, trigger recording and send alert to phone"
- "Unlock door remotely from phone after identifying visitor on camera"

#### NFC/Keypad Access
- "Unlock door with NFC tag, but only after 6pm and before 10am"
- "Grant temporary access: unlock door for 2 hours after receiving text code"
- "Restrict garage access: NFC unlock allowed only on weekdays 7am-9pm"

#### Time-Based Access
- "Unlock front door at 6am for staff, lock again at 9pm"
- "Allow guest access (door unlock) from 2pm-6pm, auto-lock after"

#### Monitoring & Alerts
- "If door opened outside normal hours, send immediate alert and video clip"
- "Log all access attempts (NFC, keypad, unlock) with timestamp and video"
- "Alert if door remains unlocked after 30 minutes post-arrival"

#### Emergency Override
- "Lock down all doors if security alarm triggered"
- "Unlock all doors and disable lock override if fire alarm activated"

---

## 8. CROSS-PAGE STATE SYNCHRONIZATION (UI binding)
**Block Count: 85 | Priority: 🟡 LOW**

### What it represents:
Keep automation state (blind position, mode, temperature) synchronized across desktop/mobile UI.

### Utterances:

#### UI Consistency
- "Reflect blind position on all control pages (desktop, mobile, wall panel) simultaneously"
- "Show live HVAC mode selection on both desktop and mobile UI"
- "Update temperature setpoint on all thermostats when changed on any page"

#### Status Feedback
- "When override is activated on mobile, update desktop page immediately"
- "Display battery percentage on main dashboard and settings page in sync"

---

## 9. SMART WATERING (Irrigation)
**Block Count: 9 | Priority: 🟡 LOW**

### What it represents:
Automated garden/lawn irrigation with soil moisture and weather awareness.

### Utterances:

#### Weather-Aware Watering
- "Water garden if soil moisture < 30% AND weather forecast shows no rain"
- "Skip watering if rain is predicted in next 6 hours"
- "Reduce watering duration by 50% on cloudy days"

#### Scheduling
- "Water lawn every morning at 6am for 15 minutes"
- "Reduce summer watering to every other day, increase to daily in heat wave"
- "Water flower beds 3x per week, vegetable garden 5x per week"

#### Efficiency
- "Automatically adjust watering based on recent rainfall"
- "Alert if soil never dries out (overwatering/leak detection)"

---

## Summary Statistics

| Category | Count | Utterances | Avg Utterances/Pattern |
|----------|-------|-----------|----------------------|
| Climate Control | 55 | 15 | 4.2 |
| Multi-Input Logic | 14 | 15 | 2.9 |
| Blind Control | 54 | 16 | 5.4 |
| Pulse Timing | 21 | 12 | 2.6 |
| Mode Selector | 14 | 10 | 2.5 |
| Energy Tracking | 45 | 14 | 3.1 |
| Door Access | 23 | 16 | 3.7 |
| Cross-Page Sync | 85 | 4 | 1.3 |
| Smart Watering | 9 | 6 | 1.5 |
| **TOTAL** | **320** | **108** | **3.1** |

---

## Implementation Priority

Based on frequency + homeowner value:

**IMMEDIATE (Phase 1):**
1. **Smart Climate Zone** — 55 blocks, high complexity, high homeowner demand
2. **Weather-Responsive Blind** — 54 blocks, complete new capability
3. **Multi-Input Logic** — 14 blocks, foundation for advanced rules

**SHORT TERM (Phase 2):**
4. **Energy Accumulator** — 45 blocks, enables smart load-shifting
5. **Door Access** — 23 blocks, security integration
6. **Pulse Timing** — 21 blocks, completes timing library

**LATER (Phase 3):**
7. **Mode Selector** — 14 blocks, nice to have with persistence
8. **Cross-Page Sync** — 85 connections but lower value
9. **Smart Watering** — 9 blocks, niche feature

