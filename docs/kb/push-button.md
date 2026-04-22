# Push Button

Source: https://www.loxone.com/enen/kb/push-button/

---

On/Off push-button

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tr | Trigger | Switches output (O) on for the duration set in parameter (Don). Another pulse extends the on-duration. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Tr), (Off) when On. (e.g Child lock, cleaning) Control via user interface is still possible. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O | Output | On for the duration set in parameter (Don). | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| Don | On-duration of output (O) | 0 = Output (O) remains active as long as the button is pressed. | s | 0...∞ | 0,3 |

---

## Timing Diagram

![PushButton timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PushButton-timediag.png)