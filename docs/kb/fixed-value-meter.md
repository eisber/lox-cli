# Fixed Value Meter

Source: https://www.loxone.com/enen/kb/fixed-value-meter/

---

The Fixed Value Meter is suitable for consumers or producers with a constant power or flow rate during operation.
In this case, the function block can calculate the meter reading based on the switch-on duration.

In this way, many devices can be recorded with little effort without having to install a physical meter. Particularly suitable are devices that are already controlled, and whose on/off status is known.

Please note that the calculated value does not provide the accuracy of a true measurement, as fluctuations may occur in both the supply and the device performance.

In combination with other meter function blocks, various consumers, producers or storages can be linked via the **[Energy Flow Monitor](https://www.loxone.com/help/energy-flow-monitor)** for an overall view.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| S | State | If 1, the value of parameter (Pf) is set at the output (Pf), and the meter is running. | 0/1 |
| R | Reset | Pulse: Meter reading outputs are reset. On: Block is locked. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Pf | Power or flow | ∞ |
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
| Pf | Power rating or nominal flow | 0...∞ | 1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Report invalid Meter Reading | If checked, you will be notified if invalid Meter Reading values have been detected. For instance, when a physical counter sends unrealistic values due to transmission errors. | - |

---