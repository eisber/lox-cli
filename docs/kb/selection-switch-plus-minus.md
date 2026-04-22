# Selection Switch +/-

Source: https://www.loxone.com/enen/kb/selection-switch-plus-minus/

---

A pulse on input (+) / (-) increases / decreases the value on output (O) by the step size (Sts) in between (Vmin) and (Vmax).
A long-click on (+) / (-) increases / decreases the value on output (O) every (Rr) seconds by step size (Sts).

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| + | Value+ | Increases value on output (O) by step size (Sts). | 0/1 |
| - | Value- | Decreases value on output (O) by step size (Sts). | 0/1 |
| V | Set value | Sets a specific value on output (O). | ∞ |
| Off | Off | Pulse: Output (O) is reset to the default value (Vdef). On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |
| DisPc | Disable periphery control | Disables all inputs when On. (e.g Child lock, cleaning) Control via user interface is still possible. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O | Output | ∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| Vmin | Minimum value | - | ∞ | 1 |
| Vmax | Maximum value | - | ∞ | 10 |
| Sts | Step size | - | ∞ | 1 |
| Rr | Repetition rate | Long-click on (+) / (-) increases / decreases the value on output (O) every (Rr) seconds. | s | 0...∞ | 0,2 |
| Vdef | Default value | Default value when input (Off) is triggered. | - | ∞ | 1 |

---

## Timing Diagram

![SelectionSwitch pm timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/SelectionSwitch_pm-timediag.png)