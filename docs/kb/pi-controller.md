# PI Controller

Source: https://www.loxone.com/enen/kb/pi-controller/

---

Proportional Integral Controller
The I share is responsible for an exact adjustment to this value.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| PV | Process value | Actual value of the controlled variable. | ∞ |
| Auto | Automatic | 0 = Manual 1 = Automatic | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| CO | Controller output | Output of the controller for manipulating the controlled variable. | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| SP | Setpoint | Setpoint of the controlled variable. | - | ∞ | 5 |
| St | Sampling time | At this interval, the controller calculates new values for (CO). | s | 0...∞ | 1 |
| Th | Threshold | Used to suppress small control differences between (PV) and (SP). If the difference is less than the threshold, no new value is calculated for (CO). This helps avoid any unnecessary load on the actuator. | - | ∞ | 1 |
| Kp | Proportional gain | - | ∞ | 2 |
| Ki | Integral gain | - | ∞ | 1 |
| Mv | Manual value | When (Auto) is 0, this value is output at (CO). | - | ∞ | 5 |
| Min | Minimum | Specifies the minimum output value for (CO). | - | ∞ | 0 |
| Max | Maximum | Specifies the maximum output value for (CO). | - | ∞ | 10 |

---