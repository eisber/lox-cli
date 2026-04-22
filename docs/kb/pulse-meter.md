# Pulse Meter

Source: https://www.loxone.com/enen/kb/pulse-meter/

---

This meter is suitable for recording consumption or delivery of various utilities such as electricity, gas, water, heat, etc.

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
| P | Pulse | For meters with pulse output (S0), connected to digital inputs. | 0/1 |
| F | Frequency | For meters with pulse output (S0), connected to digital inputs used as frequency counter. | 0...∞ |
| R | Reset | Pulse: Meter reading outputs are reset. On: Block is locked. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Pf | Power or flow | Power or flow rate Calculated from pulse to pulse time and parameter Np. Pulse interval  10 seconds: Update every 10 seconds, calculation of an average value. | 0...∞ |
| Mr | Meter Reading | 0...∞ |
| Rd | Reading today | 0...∞ |
| Rld | Reading yesterday | 0...∞ |
| Rm | Reading this month | 0...∞ |
| Rlm | Reading last month | 0...∞ |
| Ry | Reading this year | 0...∞ |
| Rly | Reading last year | 0...∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Np | Number of pulses per unit | 0...∞ | 1000 |
| Mro | Meter reading offset | Value is added to output (Mr). | ∞ | 0 |

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