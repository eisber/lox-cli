# Meter

Source: https://www.loxone.com/enen/kb/meter/

---

This meter is suitable for recording consumption or delivery of various utilities such as electricity, gas, water, heat, etc.

The values are retrieved from a physical utility meter.

In combination with other meter function blocks, various consumers, producers or storages can be linked via the **[Energy Flow Monitor](https://www.loxone.com/help/energy-flow-monitor)** for an overall view.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Meter reading and units](#MeterCalcUnits)
- [Detection of Invalid Meter Readings](#MeterInvalidReadings)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Pf | Power or flow | If only this input is used, the meter reading is also calculated from it. Otherwise it is only used for the output (Pf) and the user interface. | 0...∞ |
| Mr | Meter reading | Input for meters that send the meter reading directly as an analog value. For meters that only send partial quantities (e.g. Smart Socket), relative metering must be activated in the settings of the block. | 0...∞ |
| R | Reset | Pulse: Meter reading outputs are reset. On: Block is locked. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Pf | Power or flow | 0...∞ |
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
| Mro | Meter reading offset | Value is added to output (Mr). | ∞ | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Relative metering | Active: The physical meter only sends partial quantities at intervals (relative), the block adds them up and uses them to form the meter reading. Not active: The physical meter provides its own total meter reading (absolute), the function block only maps it. | - |
| Report invalid Meter Reading | If checked, you will be notified if invalid Meter Reading values have been detected. For instance, when a physical counter sends unrealistic values due to transmission errors. | - |

---

## Meter reading and units

In general, a physical meter records the quantity and transfers the value to the function block at a meter reading input. The function block maps the meter reading and records the statistics for it.
The units for Pf/Mr can be selected independently of each other, as the value at input Pf is only used for output Pf and for display in the user interface.

If the input Pf is used on its own, the block takes over the function of the meter.
In this case, the units for Pf/Mr must be hour-related and must match in magnitude so that the meter reading is calculated correctly.
Examples:
Pf=kW, Mr=kWh
Pf=W, Mr=Wh
Pf=l/h, Mr=l
Pf=m³/h, Mr=m³
Pf=kg/h, Mr=kg

When only the Mr input is connected, power is calculated based on the meter value. The power is estimated using the difference between the last two meter readings and their time interval. If the same or a smaller meter value is received, power is set to zero (Please note that this only works if the same value causes an additional edge!). The power output remains unchanged until a new meter value is received.

---

## Detection of Invalid Meter Readings

The meter block detects invalid meter readings at the Mr input(s), which can occur due to faulty data transmission from a physical absolute value meter, and ignores them.

In general, only increasing or constant meter readings are allowed; decreasing or jumping to 0 readings are discarded.

Unrealistically high increases in meter readings are discarded if:
- The change in value is greater than the total previous consumption and greater than 10000.
- If less than 8 hours have passed since the last value change.