# EIB Shading

Source: https://www.loxone.com/enen/kb/eib-shading/

---

EIB motor control for the blinds using two button control

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Po | Partial open with push & hold | - | 0/1 |
| Pc | Partial close with push & hold | - | 0/1 |
| Pos | Position | Used to provide feedback of the current shading position to the function block | % | 0...100 |
| DisPc | Disable periphery control | Disables inputs Po, Pc, when On. (e.g Child lock, cleaning) Control via user interface is still possible. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Lo | Long-term operation | -1 = passive 0 = open 1 = close Connect this output to an EIB-actuator (group address for long-term operation of blinds) | ∞ |
| So | Short-term operation | -1 = passive 0 = open 1 = close Connect this output to an EIB-actuator (group address for short-term operation of blinds) | ∞ |
| Pos | Position of shading | ∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Tlc | Time long-click | Pulse duration on inputs (Po) (Pc) to trigger a long-term operation (Lo). | s | 0...∞ | 0,3 |

---