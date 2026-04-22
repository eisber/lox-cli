# Touch & Grill Control

Source: https://www.loxone.com/enen/kb/touch-and-grill-control/

---

This block enables you to integrate the functionality of the Touch & Grill Air into your Loxone system.
Core temperatures can be monitored, target temperatures can be determined and timers can be set.
Up to 2 function blocks can be created per device, e.g. for use in the kitchen and garden.
Only one of the function blocks can remote control the Touch & Grill Air at a time. The other is set to inactive.
A block can be activated via the (Afb) input. At the beginning, the first inserted block is active.
Any touches to the device's touch surface are directed to the active function block's T5 input.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Ca | Confirm alarm | 0/1 |
| Afb | Activate function block | Activates control of this function block by its associated Touch & Grill. | 0/1 |
| DisT | Disable touch controls | Serves as protection against unwanted touch- events, e.g. during cleaning or transport. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Ay | Alarm yellow sensor | Active, when alarm is triggered by the yellow sensor. | - | 0/1 |
| Ag | Alarm green sensor | Active, when alarm is triggered by the green sensor. | - | 0/1 |
| At | Alarm timer | Active, when alarm is triggered at the end of the timer. | - | 0/1 |
| ϑcy | Current temperature yellow sensor | ° | -28...300 |
| ϑcg | Current temperature green sensor | ° | -28...300 |
| ϑty | Target temperature yellow sensor | ° | 10...300 |
| ϑtg | Target temperature green sensor | ° | 10...300 |
| Rt | Remaining time | Remaining run time of a running timer. | s | 0...5999 |
| Fb | Function block state | Output is active as long as this function block is controlled by its associated Touch & Grill. | - | 0/1 |
| Atx | Alarm Text | Text of the last triggered alarm. | - | - |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Dm | Maximum alarm duration (s) | After this time, all existing alarms are automatically acknowledged. | s | 1...∞ | 3600 |
| B | Display-Brightness | Display brightness of the associated Touch & Grill Air device. | % | 0...100 | 100 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Number of Entries | Maximum number of saved messages. | 2...100 | 20 |
| Assigned Touch & Grill Air | Touch & Grill Air that is associated with this function block | - | - |

---