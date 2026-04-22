# Fire and Water Alarm

Source: https://www.loxone.com/enen/kb/fire-water-alarm/

---

This function block will alert you of a fire or water leakage.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Application](#usage)
- [Disarm vs Acknowledge Alarm](#Disarm vs Acknowledge Alarm)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Ca | Confirm alarm | 0/1 |
| Cs | Confirm alarm signals | Deativates outputs Pas and Mas. Outputs Pa and Ma remain active. Assigned Audio Players or Lighting Controllers are also switched off. | 0/1 |
| S | Smoke detector | Alarm input for external smoke detector. The name of the connected sensor is used in the user interface. | 0/1 |
| W | Water detector | Alarm input for external water detector. The name of the connected sensor is used in the user interface. | 0/1 |
| F | AFCI | Input for Arc-fault circuit interrupter. The name of the connected sensor is used in the user interface. | 0/1 |
| T | Temperature input | Connection for external temperature sensors. The name of the connected sensor is used in the user interface. | ∞ |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Pa | Pre-alarm | 0/1 |
| Ma | Main alarm | 0/1 |
| Pas | Pre-alarm signal | 0/1 |
| Mas | Main alarm signal | 0/1 |
| N | Number of active sensors | ∞ |
| At | Alarm test | 0/1 |
| Ca | Cause of alarm | Reports cause of the last alarm. | - |
| Ta | Time and date of alarm | Reports date and time of the last alarm. | - |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour – When activating/deactivating an alarm system (max. every 10s) The data is saved on the SD card. | - | 0/1 | 0 |
| Mad | Main alarm delay | Main alarm delay after activating the pre-alarm. | s | 0...∞ | 120 |
| Maxϑ | Maximum temperature | If input (T) is greater than or equal to this value, the alarm will be activated. | ° | ∞ | 43 |
| MaxA | Maximum alarm duration | The main alarm is automatically confirmed at the end of set time. If (Pac) = 1, the pre-alarm is also automatically confirmed. 0 = No duration limit | s | 0...∞ | 300 |
| Pac | Pre-alarm confirmation | 1 = Pre-alarm is confirmed automatically when (MaxA) is reached. 0 = Pre-alarm stays active when (MaxA) is reached. | - | 0/1 | 0 |
| Sm | Service mode | Enables the Service mode and suppresses the alarm, only (At) is activated in case of an alarm. 0 = Service mode off, 1 = Service mode permanently on, >1 = Service mode on for this time. | s | 0...∞ | 0 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Number of Entries | Maximum number of saved messages. | 2...100 | - |
| Configuration | Configuration of the available inputs and outputs. | - | - |

---

## Application

Loxone products, such as the Smoke Detector Air and Water Sensor Air, can simply be selected in the configuration dialog (double-click on block).
Function blocks such as audio players, lighting controllers, shading blocks or devices can be selected for audible and visual alarms (double-click on block).
Other third-party products can be connected to the dedicated inputs

---

## Disarm vs Acknowledge Alarm

When the alarm is triggered, there are 2 different options in the App:
- **Disarm:** Disarms the alarm. To trigger the alarm again, it must be re-armed first.
- **Acknowledge Alarm:** Acknowledges the alarm without disarming it. The alarm remains armed, so any connected sensor can trigger it immediately again.

![Alarm DisarmAcknowledgeAlarm](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Alarm_DisarmAcknowledgeAlarm.png)