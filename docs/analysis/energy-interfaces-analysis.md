# Energy & Interfaces Training Config Analysis

**Source:** `/tmp/energy-interfaces.Loxone` (593KB, Loxone Config v16.01)  
**Location:** Kollerschlag, Austria (48.602°N, 13.838°E)  
**Date:** 2025-06-30

## Overview

| Metric | Value |
|--------|-------|
| Total C elements | 974 |
| Unique types | 147 |
| Rooms | 13 |
| Pages | 11 |
| Energy-specific blocks | 46 |

This is a **Loxone training configuration** focused on energy management and interfaces. It demonstrates every energy block type available in Loxone Config, with simulated inputs using VirtualIn/InputRef blocks and comprehensive wiring patterns.

## Pages (Logical Groups)

| # | Page | Purpose |
|---|------|---------|
| 1 | Meters, Energy Flow Monitor | All meter types + EFM dashboard |
| 2 | PS&B Overload Protection | PowerUnit overcurrent scenarios |
| 3 | PS&B Backup Mode | Battery backup mode handling |
| 4 | Load Manager | LoadShed with prioritized appliances |
| 5 | Energy Manager | EnergyManager2 with PV surplus routing |
| 6 | Spot Price Optimizer | 3 SpotOpt instances (general, DHW, battery) |
| 7 | Energy Storage | Battery charge/discharge management |
| 8 | Wallbox NFC | Single wallbox with NFC authentication |
| 9 | Wallbox + Flex | PV-surplus & spot-price wallbox charging |
| 10 | Wallbox Manager | WBEM with 5 wallboxes in 2 groups |
| X | Simulator | Test inputs for all scenarios |

## Rooms

Central, Energy Measurement, PS&B, Load Manager, Test, Energy Manager, Spot Price Optimizer, Energy Storage, Wallbox, Wallbox Manager, Wallbox NFC, Trust, Not Assigned

## Energy Block Types Found

### Meter Blocks (4 variants)

| Type | control_type | Count | Purpose |
|------|-------------|-------|---------|
| **MeterAbsUni** | 520 | 20 | Unidirectional absolute meter (power → energy) |
| **MeterAbsBi** | 521 | 2 | Bidirectional meter (import/export, e.g. grid) |
| **MeterAbsSt** | 522 | 1 | Storage meter (charge/discharge + SOC level) |
| **MeterDig** | 523 | 2 | Digital meter (on/off state × rated power) |

**Meter output naming convention:**
- `OPf` — current power factor/value
- `OMr` / `OMr1` / `OMr2` — meter reading(s)
- `ORd` / `ORw` / `ORm` / `ORy` — current day/week/month/year consumption
- `ORld` / `ORlw` / `ORlm` / `ORly` — last day/week/month/year consumption
- `OSlvl` — storage level (MeterAbsSt only)
- Suffix `1`/`2` on MeterAbsBi/MeterAbsSt = direction 1 (import/charge) and direction 2 (export/discharge)

**Key meter attributes:**
- `Rec` — UUID of the EFM this meter reports to
- `NOEFM=true` — exclude from Energy Flow Monitor
- `Acum=true` — accumulation mode (absolute meter reading vs. power)
- `IName` — internal sub-meter index (0=System, 1=Peripherals, 2=Lights, 3=Audio, 7=PSB total)
- `PwrN` — power output display name
- `Un1`/`Un2` — display units for power/energy

### Energy Flow Monitor (EFM)

| Type | control_type | Count | Purpose |
|------|-------------|-------|---------|
| **EFM** | 524 | 1 | Dashboard: energy flow visualization |

The EFM uses a **tree structure** (`<root>` → `<nd>`) to define the energy flow topology:

```
House (root, type 5)
├── PV (type 4, icon: solar)
├── Grid (type 2, icon: grid)
├── Battery (type 3, icon: battery)
└── Appliance (group)
    ├── Sauna (type 1)
    ├── Notebook/SSA (type 1)
    ├── Boiler (type 1)
    └── Heat pump (type 1)
```

**Node types:** 1=Appliance, 2=Grid, 3=Battery, 4=PV/Production, 5=Root/House

**EFM outputs:**
- `OGpwr`/`OPpwr`/`OSpwr` — Grid/PV/Storage power
- `OEd`/`OPd`/`OCd`/`OScd`/`OCo2d` — energy/production/consumption/self-consumption/CO2 daily
- `OYd`/`OYt` — yield daily/total
- `OSc` — self-consumption ratio

**Parameters:** `Pre` (export price), `Pri` (import price), `CO2` (CO2 factor kg/kWh)

### Energy Manager

| Type | control_type | Count | Purpose |
|------|-------------|-------|---------|
| **EnergyManager2** | 515 | 3 | PV surplus distribution to priority loads |

**Key inputs:**
- `Gpwr` — Grid power (negative = export)
- `Ppwr` — PV production power
- `Spwr` — Storage power
- `Soc` — Battery state of charge
- `IL1`–`IL12` — Priority load status inputs (up to 12 loads)
- `Recalc` — Trigger recalculation (typically 5s pulse)
- `Prio` — Override priority

**Key outputs:**
- `OL1`–`OL12` — Load enable/surplus allocation outputs
- `ONext` — Next load to enable
- `OMinSoc` — Minimum SOC output
- `O` — Total surplus available

**Parameters:** `MinSoc` (default 20%), `MaxSpwr` (default 3 kW)

**Wiring pattern from config:**
```
InputRef "Grid".AQ → EnergyManager2.Gpwr
InputRef "PV".AQ → EnergyManager2.Ppwr
PulseGen "5s recalc".Q → EnergyManager2.Recalc
InputRef "Boiler state".Q → EnergyManager2.IL1
InputRef "Heating element".AQ → EnergyManager2.IL2
EnergyManager2.OL1 → WBEM.Peco (surplus to wallbox manager)
```

### Load Manager (LoadShed)

| Type | control_type | Count | Purpose |
|------|-------------|-------|---------|
| **LoadShed** | 459 | 1 | Overload protection with prioritized shedding |

Uses `<Load>` child elements to define loads with priorities:
```xml
<Load nm="Hairdryer" pwr="1" />       <!-- 1 kW, shed first -->
<Load ioIdx="1" nm="Sauna" pwr="5" /> <!-- 5 kW, shed second -->
<Load ioIdx="2" nm="Boiler" pwr="3" /> <!-- 3 kW, shed third -->
```

**Key inputs:** `cP` (current power), `Sl1`–`Sl12` (load status), `maxP` (limit, default 10kW)  
**Key outputs:** `AQl1`–`AQl12` (shed commands), `aP`/`AvgP` (actual/average power)

### Spot Price Optimizer

| Type | control_type | Count | Purpose |
|------|-------------|-------|---------|
| **SpotOpt** | 526 | 4 | Schedule loads during cheapest electricity hours |

Four instances demonstrate different use cases:
1. **Spot Price Optimizer** — general optimization with hourly price inputs
2. **Spot DHW** — hot water heating at cheapest morning/evening hours
3. **Spot grid battery charging** — charge battery from grid at cheapest overnight hours
4. **Wallbox SPO** — EV charging during cheapest hours

**Key inputs:**
- `Tr` — Trigger recalculation
- `U0`–`U23` — Hourly price inputs (24 hours)
- `R` — Reset

**Key outputs:**
- `Act` — Currently active (cheap hour now)
- `Uc` — Current hour price
- `vHigh`/`High`/`Low`/`vLow` — Price level indicators
- `Max`/`Min`/`Avg` — Price statistics
- `Nv` — Number of cheap hours found

**Parameters:** `Tn` (min cheap hours, default 4), `Ta` (analysis window, default 24h), `Mrt`/`Mvh` (moving thresholds)

### Wallbox

| Type | control_type | Count | Purpose |
|------|-------------|-------|---------|
| **Wallbox** | 525 | 8 | EV charging station control |

The config shows 8 wallbox instances across 3 pages with different configurations:
- **Wallbox** (page 8) — NFC authentication, basic charging
- **Wallbox A/B** (page 9) — Spot price + PV surplus modes
- **Wallbox 1.1/1.2/2.1/2.2/2.3** (page 10) — Fleet managed by WBEM

**Key inputs:**
- `allow` — Enable charging (from NFC, spot price, or energy manager)
- `connected` — Vehicle connected signal
- `energy` — Energy meter reading
- `user` — User ID (from NFC)
- `active` — Charging active signal
- `prio` — Priority level
- `setmode1`–`setmode5` — Mode set triggers
- `valmode1`–`valmode5` — Mode values (e.g., surplus power)
- `pricemode1`–`pricemode6` — Price thresholds per mode
- `loadshed` — Load shedding command
- `off` — Force stop

**Key outputs:**
- `outAllow`/`outConnected`/`outActive` — Status
- `outPower`/`outLimit` — Current power and limit
- `outConsCurr`/`outConsDay`/`outConsWeek`/... — Consumption rollups
- `outSesS`/`outSesE` — Session start/end
- `outMode`/`outPrice`/`outUserId` — Current mode info
- `outLoadshed`/`outLog` — Load shed state and log

**Parameters:** `P1` (min power, typically 4.16 kW = 6A×3), `P2` (max power, typically 11 kW = 16A×3)

### Wallbox Energy Manager (WBEM)

| Type | control_type | Count | Purpose |
|------|-------------|-------|---------|
| **WBEM** | 529 | 1 | Fleet management for multiple wallboxes |

Uses a **tree structure** to define wallbox groups:
```
Root (fuse=0)
├── Group 1 (fuse=22kW)
│   ├── Wallbox 1.1 (Pmax=11, Pmin=4.16)
│   └── Wallbox 1.2 (Pmax=11, Pmin=4.16)
└── Group 2 (fuse=22kW)
    ├── Wallbox 2.1 (Pmax=11, Pmin=4.16)
    ├── Wallbox 2.2 (Pmax=11, Pmin=4.16)
    └── Wallbox 2.3 (Pmax=11, Pmin=4.16)
```

**Key inputs:** `Pmax` (max available power), `Peco` (eco power from EnergyManager2), `Off` (force all off)  
**Parameters:** `PrEco` (eco price €0.10/kWh), `PrPrio` (priority price €0.20/kWh)  
**Key outputs:** `PrHour` (price per hour), `Cp`/`Ap` (connected/active power)

### Power Supply & Backup

| Type | control_type | Count | Purpose |
|------|-------------|-------|---------|
| **PowerUnit** | 514 | 3 | UPS / power supply with sub-metering |

Automatically creates child MeterAbsUni blocks with `IName` categories:
- `IName=0` → System
- `IName=1` → Peripherals
- `IName=2` → Lights
- `IName=3` → Audio
- `IName=7` → Total PSB power

**Outputs:** `OutPower` (total), `Power1`–`Power7` (per category), `OutBackup` (backup mode), `OutSOC`, `OutOverCurrent`, `OutRemaining`

### Fronius Inverter

**Not present** in this config (type 465 exists in schema but no instances here). Documented from `loxone-block-types-full.json`:

**Inputs:** `InEnergyGenerated`/`InPowerGenerated`, `InEnergyConsumed`/`InPowerConsumed`, `InEnergyGrid`/`InPowerGrid`, `InEnergyBat`/`InPowerBat`, `InBatSoC`  
**Outputs:** Daily/total production, grid, consumption, battery stats

## Wiring Patterns

### Pattern 1: Sensor → Meter → EFM Pipeline
```
ModbusASensor → InputRef.AQ → MeterAbsUni.Pf
                                    ↓ (Rec=EFM UUID)
                                   EFM tree node
```

### Pattern 2: Energy Manager Surplus Distribution
```
Grid sensor → InputRef → EnergyManager2.Gpwr
PV sensor   → InputRef → EnergyManager2.Ppwr
PulseGen "5s" ————————→ EnergyManager2.Recalc
Load status → InputRef → EnergyManager2.IL1..IL12
                          EnergyManager2.OL1 → WBEM.Peco
                          EnergyManager2.OL2 → Boiler relay
```

### Pattern 3: Spot Price → Wallbox Chain
```
                  SpotOpt.Act → And.I1
Vehicle detect ——————————————→ And.I2
                                And.Q → Wallbox.allow
Surplus calc ——————————————————————→ Wallbox.valmode1
```

### Pattern 4: Wallbox Manager Fleet
```
VirtualIn "Max power" → WBEM.Pmax
EnergyManager2.OL1 ——→ WBEM.Peco
                         WBEM manages → Wallbox 1.1
                                      → Wallbox 1.2
                                      → Wallbox 2.1–2.3
```

### Pattern 5: Load Shedding
```
Formula "total power" → LoadShed.cP
Load 1 status ————————→ LoadShed.Sl1
Load 2 status ————————→ LoadShed.Sl2
                         LoadShed.AQl1 → Relay "shed load 1"
                         LoadShed.AQl2 → Relay "shed load 2"
```

## Interface Types Used

| Interface | Count | Purpose |
|-----------|-------|---------|
| TreeSensor | 125 | Tree bus digital sensors |
| TreeAsensor | 48 | Tree bus analog sensors |
| TreeActor | 36 | Tree bus digital actors |
| TreeAactor | 21 | Tree bus analog actors |
| ModbusASensor | 9 | Modbus analog inputs (energy meters) |
| ModbusDev | 3 | Modbus device definitions |
| ApiActor | 16 | HTTP API output actors |
| VirtualIn | 36 | Virtual digital inputs |
| InputRef | 113 | Input references (wire sources) |
| OutputRef | 23 | Output references (wire destinations) |
| SysVar | 13 | System variables |
| WeatherData | 11 | Weather data channels |

## Numeric Type ID Reference

All block types in this config use **named types** (string identifiers), not numeric IDs. However, the corresponding numeric `control_type` IDs from the block type registry are:

| Named Type | control_type | Category |
|-----------|-------------|----------|
| MeterAbsUni | 520 | Energy metering |
| MeterAbsBi | 521 | Energy metering |
| MeterAbsSt | 522 | Energy metering |
| MeterDig | 523 | Energy metering |
| EFM | 524 | Energy monitoring |
| EnergyManager | 468 | Energy management |
| EnergyManager2 | 515 | Energy management |
| SpotOpt | 526 | Price optimization |
| LoadShed | 459 | Load management |
| Wallbox | 525 | EV charging |
| WBEM | 529 | EV fleet management |
| PowerUnit | 514 | Power supply |
| Fronius | 465 | Solar inverter |

## Key Takeaways

1. **Meter hierarchy:** MeterAbsUni (simple) → MeterAbsBi (bidirectional) → MeterAbsSt (storage+SOC) → MeterDig (on/off fixed power). Choose based on the measurement need.

2. **EFM is the dashboard:** All meters connect to the EFM via the `Rec` attribute (UUID reference). The EFM's tree structure (`<root>`/`<nd>`) defines the visual energy flow topology.

3. **EnergyManager2 replaces EnergyManager:** The v2 block has dedicated grid/PV/storage inputs instead of generic analog inputs, plus `Recalc` trigger and `MinSoc`/`MaxSpwr` parameters.

4. **Wallbox is highly configurable:** 55 connectors support 5 operating modes with per-mode values and prices, NFC user auth, load shedding integration, and OCPP support.

5. **WBEM uses tree structure:** Similar to EFM, the Wallbox Manager defines groups with fuse limits. Each wallbox node specifies `Pmax`/`Pmin` and links to the Wallbox block via `ctrlUuid`.

6. **Spot price is versatile:** Used for general load scheduling, DHW heating, battery grid charging, and EV charging — each with different trigger sources and analysis windows.

7. **PowerUnit auto-creates sub-meters:** Child MeterAbsUni blocks with `IName` categories are automatically generated, each reading from the corresponding `Power1`–`Power7` outputs.

8. **Load child elements in LoadShed:** The `<Load nm="..." pwr="..." />` elements define the loads with names and rated powers — these are not connectors but configuration data.
