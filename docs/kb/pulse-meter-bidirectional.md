# Pulse Meter Bidirectional

Source: https://www.loxone.com/enen/kb/pulse-meter-bidirectional/

---

This bidirectional meter is suitable for recording consumption and delivery of various utilities such as electricity, gas, water, heat, etc.

The values are retrieved from a physical utility meter with pulse output (S0).

In combination with other meter function blocks, various consumers, producers or storages can be linked via the **[Energy Flow Monitor](https://www.loxone.com/help/energy-flow-monitor)** for an overall view.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Meter reading and units](#MeterCalcUnits)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Pc | Pulse consumption | For meters with pulse output (S0), connected to digital inputs. | 0/1 |
| Pd | Pulse delivery | For meters with pulse output (S0), connected to digital inputs. | 0/1 |
| Fc | Frequency consumption | For meters with pulse output (S0), connected to digital inputs used as frequency counter. | 0...∞ |
| Fd | Frequency delivery | For meters with pulse output (S0), connected to digital inputs used as frequency counter. | 0...∞ |
| R | Reset | Pulse: Meter reading outputs are reset. On: Block is locked. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Pf | Power or flow | Current power/flow calculated using inputs (Pc) and (Fc) is output as a positive value. Current power/flow calculated using inputs (Pd) and (Fd) is output as a negative value. | ∞ |
| Mrc | Meter Reading consumption | 0...∞ |
| Rdc | Reading today consumption | 0...∞ |
| Rldc | Reading yesterday consumption | 0...∞ |
| Rmc | Reading this month consumption | 0...∞ |
| Rlmc | Reading last month consumption | 0...∞ |
| Ryc | Reading this year consumption | 0...∞ |
| Rlyc | Reading last year consumption | 0...∞ |
| Mrd | Meter Reading delivery | 0...∞ |
| Rdd | Reading today delivery | 0...∞ |
| Rldd | Reading yesterday delivery | 0...∞ |
| Rmd | Reading this month delivery | 0...∞ |
| Rlmd | Reading last month delivery | 0...∞ |
| Ryd | Reading this year delivery | 0...∞ |
| Rlyd | Reading last year delivery | 0...∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Npc | Number of pulses per unit | 0...∞ | 1000 |
| Npd | Number of pulses per unit | 0...∞ | 1000 |
| Mroc | Meter reading offset consumption | Value is added to output (Mrc). | ∞ | 0 |
| Mrod | Meter reading offset delivery | Value is added to output (Mrd). | ∞ | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Report invalid Meter Reading | If checked, you will be notified if invalid Meter Reading values have been detected. For instance, when a physical counter sends unrealistic values due to transmission errors. | - |

---

## Meter reading and units

The physical meter provides a certain number of pulses per unit of quantity, this must be set at parameter (Np). The block uses this to form the meter reading (Mr), calculates the power or flow rate (Pf), and records the statistics.

The units for Pf/Mr must be hour-related and must match in magnitude so that Pf is calculated correctly.
Examples:
Pf=kW, Mr=kWh
Pf=W, Mr=Wh
Pf=l/h, Mr=l
Pf=m³/h, Mr=m³
Pf=kg/h, Mr=kg