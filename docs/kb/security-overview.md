# Burglar Alarm Central

Source: https://www.loxone.com/enen/kb/security-overview/

---

With this function block, all Burglar Alarm blocks can be controlled together.
Double-click on the block to open the dialogue for selecting the linked blocks

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)
- [Basic Programming](#basic)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tg | Toggle with presence detection | Toggles between arm / disarm. Presence sensors are used to trigger an alarm. | 0/1 |
| Tgnp | Toggle without presence detection | Toggles between arm / disarm. Presence sensors are not used to trigger an alarm. | 0/1 |
| A | Arm with presence detection | Arms the alarm system. Presence sensors are used to trigger an alarm. | 0/1 |
| Anp | Arm without presence detection | Arms the alarm system. Presence sensors are not used to trigger an alarm. | 0/1 |
| Ad | Arm delayed with presence detection | Arms the alarm system with a delay (Ard). Presence sensors are used to trigger an alarm. | 0/1 |
| Adnp | Arm delayed without presence detection | Arms the alarm system with a delay (Ard). Presence sensors are not used to trigger an alarm. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |
| Ca | Confirm alarm | Confirms the current alarm and resets all alarm outputs. The alarm system remains armed. | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Tg), (Tgnp), (A), (Anp), (Ad), (Adnp) when On. (e.g Child lock, cleaning) Control via user interface is still possible. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| API | API Connector | Intelligent API based connector. API Commands | - |
| Na | Active Armed Alarms | Number of active armed alarms | ∞ |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Selection | All selected Alarm function blocks can be controlled together. | - |

---

## Basic Programming

Double-clicking on the block opens the following window.

![CentralAlarm Menu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/CentralAlarm_Menu.jpg)

Central commands are not blocked by an active (DisPc) input at the respective function block. If a function block is used in the central block, this is indicated by the central symbol on the respective block.
The functions that can be used on the central block depend on the linked blocks and are set via their parameters. If a function block does not support a function, it cannot be controlled.