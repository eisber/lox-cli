# AAL Smart Alarm

Source: https://www.loxone.com/enen/kb/aal-smart-alarm/

---

This function block enables intelligent detection of emergency situations. If an individual is in need of help (e.g. after falling), help can be contacted quickly. In addition, a manual alarm is available, e.g. via an emergency button or other alarm logic. Loxone products such as the Motion Sensor Tree can easily be selected via a configuration dialog (double-click on the block).

Intelligent detection of emergency situations:
A timer is started at the falling edge of the motion detection. If no further motion is detected within this time in this or another room, the alarm will be triggered. The timer duration depends on the room type and can be adjusted.
The input (Dis) disables the intelligent detection. It will automatically restart as soon as movement is detected again.

IMPORTANT: To ensure reliable detection, motion sensors must be assigned to each room and the input (Dis) must be used.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| A | Activate alarm | Triggers the alarm. Connect an emergency button or other alarm logic here. The name of the connected sensor is used in the user interface. | 0/1 |
| Mvt | Movement transit room | 0/1 |
| Mvc | Movement common room | 0/1 |
| Mvb | Movement bedroom | 0/1 |
| Ca | Confirm alarm | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |
| Dis | Disable | Intelligent detection of emergency situations is disabled when 1. Can be used when the home is left. A manual alarm through input (A) is still possible. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| A1-2 | Alarm level 1-2 | Level 1 is activated immediately upon alarm triggering. Level 2 is activated after the delay set on parameter D. | 0/1 |
| Ca | Cause of alarm | - |
| Ta | Time and date of alarm | - |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Tt | Time transit room | If no further movement is detected within this time after the last movement in a transit room, the alarm is triggered. | min | 1...∞ | 15 |
| Tc | Time common room | If no further movement is detected within this time after the last movement in a common room, the alarm is triggered. | min | 1...∞ | 60 |
| Tb | Time bedroom | If no further movement is detected within this time after the last movement in a bedroom, the alarm is triggered. | min | 1...∞ | 420 |
| D | Alarm level 2 delay | s | 0...∞ | 60 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Number of Entries | Maximum number of saved messages. | 2...100 | 20 |
| Configuration | Configure devices here for automatic use as detectors in the block (e.g. Presence Sensor Tree). Sensors without an assigned room, or with a room of the types Other or Central are ignored in the intelligent detection of emergency situations and marked grey in the dialogue. | - | - |

---