# Switch (On/Off)

Source: https://www.loxone.com/enen/kb/selection-switch-onoff/

---

Pulse on (Son) / (Soff) switches the digital output (O) accordingly.
By using the input (Off) the output is deactivated.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Son | Switch on | Switches output (O) on. | 0/1 |
| Soff | Switch off | Switches output (O) off. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |
| DisPc | Disable periphery control | Disables all inputs when On. (e.g Child lock, cleaning) Control via user interface is still possible. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O | Output | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |

---

## Timing Diagram

![SwitchOnOff timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/SwitchOnOff-timediag.png)