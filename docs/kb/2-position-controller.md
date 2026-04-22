# 2 Position Controller

Source: https://www.loxone.com/enen/kb/2-position-controller/

---

The 2 Position Controller manipulates a controlled variable by switching an output, using a setpoint and a hysteresis.

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
| O | Output | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| SP | Setpoint | Setpoint of the controlled variable. | ∞ | 5 |
| Hys | Hysteresis | Hysteresis of the two position controller (prevents constant switching on/off of the output) | ∞ | 0,5 |
| Inv | Inverted | Inverts the behavior of the controller. | 0/1 | 0 |

---

## Timing Diagram

![2PositionController normal timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/2PositionController_normal-timediag.png)

![2PositionController inverted timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/2PositionController_inverted-timediag.png)