# Pulse Meter & Storage

Source: https://www.loxone.com/enen/kb/pulse-meter-storage/

---

This meter is used to integrate a storage and records its level, charging and discharging.

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
| Pd | Pulse discharge | For meters with pulse output (S0), connected to digital inputs. | 0/1 |
| Pc | Pulse charge | For meters with pulse output (S0), connected to digital inputs. | 0/1 |
| Fd | Frequency discharge | For meters with pulse output (S0), connected to digital inputs used as frequency counter. | 0...∞ |
| Fc | Frequency charge | For meters with pulse output (S0), connected to digital inputs used as frequency counter. | 0...∞ |
| Slvl | Storage level or state of charge | ∞ |
| R | Reset | Pulse: Meter reading outputs are reset. On: Block is locked. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Pf | Power or flow | Current power/flow calculated using inputs (Pd) and (Fd) is output as a positive value. Current power/flow calculated using inputs (Pc) and (Fc) is output as a negative value. | ∞ |
| Mrd | Meter Reading discharge | 0...∞ |
| Rdd | Reading today discharge | 0...∞ |
| Rldd | Reading yesterday discharge | 0...∞ |
| Rmd | Reading this month discharge | 0...∞ |
| Rlmd | Reading last month discharge | 0...∞ |
| Ryd | Reading this year discharge | 0...∞ |
| Rlyd | Reading last year discharge | 0...∞ |
| Mrc | Meter Reading charge | 0...∞ |
| Rdc | Reading today charge | 0...∞ |
| Rldc | Reading yesterday charge | 0...∞ |
| Rmc | Reading this month charge | 0...∞ |
| Rlmc | Reading last month charge | 0...∞ |
| Ryc | Reading this year charge | 0...∞ |
| Rlyc | Reading last year charge | 0...∞ |
| Slvl | Storage level or state of charge | ∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Npd | Number of pulses per unit | 0...∞ | 1000 |
| Npc | Number of pulses per unit | 0...∞ | 1000 |
| Mrod | Meter reading offset discharge | Value is added to output (Mrc). | ∞ | 0 |
| Mroc | Meter reading offset charge | Value is added to output (Mrd). | ∞ | 0 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Maximum storage level | Maximum storage level used for the user interface | 0...∞ | 100 |
| Report invalid Meter Reading | If checked, you will be notified if invalid Meter Reading values have been detected. For instance, when a physical counter sends unrealistic values due to transmission errors. | - | - |

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