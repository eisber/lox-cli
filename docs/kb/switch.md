# Switch

Source: https://www.loxone.com/enen/kb/switch/

---

The Switch function block is used to toggle a digital output.
It can be controlled by a pushbutton or other logic, and via the user interface.

![switch config visu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/switch-config-visu.png)

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)
- [Presence Simulation](#PresenceSimulation)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tg | Toggle | Switches output (O) on / off. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |
| On | Switches output (O) on. | 0/1 |
| DisPc | Disable periphery control | Disables all inputs when On. (e.g Child lock, cleaning) Control via user interface is still possible. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O | Output | 0/1 |
| Off | Pulse when output (O) is switched off. | 0/1 |
| On | Pulse when output (O) is switched on. | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |

---

## Timing Diagram

![switch timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/switch-timediag.png)

---

## Presence Simulation

This function block has a presence simulation.
Activate and define the presence simulation in the properties window:

![PresenceSimulation Switch](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PresenceSimulation_Switch.png)