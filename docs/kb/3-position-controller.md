# 3 Position Controller

Source: https://www.loxone.com/enen/kb/3-position-controller/

---

The 3 Position Controller manipulates a controlled variable by switching two outputs with opposite effect, using two setpoints.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| PV | Process value | Actual value of the controlled variable. | ∞ |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| O1 | Output 1 | 0/1 |
| O2 | Output 2 | 0/1 |

---

## Parameters

| Abbreviation | Summary | Value Range | Default Value |
| --- | --- | --- | --- |
| SP1 | Setpoint 1 | ∞ | 3 |
| SP2 | Setpoint 2 | ∞ | 7 |

---

## Timing Diagram

![3PositionController timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/3PositionController-timediag.png)