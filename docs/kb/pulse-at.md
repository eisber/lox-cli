# Pulse at

Source: https://www.loxone.com/enen/kb/pulse-at/

---

The Pulse at function block provides a pulse with adjustable duration.

A specific time or date can be selected in the block, as well as events such as sunrise, dusk, or hour pulse, month pulse, etc.

The time setting can be changed via the user interface.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Basic Programming](#Basic)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O | Output | Activates the output at the defined time, for the duration set in parameter (Don). | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| Don | On-duration of output (O) | s | 0...∞ | 1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Pulse | Selection of when the pulse is to be output. | - |
| Time | Time at which the pulse is created Format hh:mm:ss | - |
| One Off Pulse | One off pulse on a specific date Only available for certain time functions (e.g. Pulse at Sunset). If unticked, the pulse is created every day or every time the selected time function is activated. | - |

---

## Basic Programming

After adding the block, a time function can be selected in its properties.
When setting a specific time, it is entered in the format hh:mm:ss.
The pulse is executed daily, also when time functions such as sunrise are selected.

![pulseat selection](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/pulseat-selection.png)

Time functions, describing an interval other than daily (e.g. Minute pulse), provide the pulse at the interval of their name.

If the option one-time pulse is selected, then a date must also be set. The pulse will then be executed only once on the set date.
This option is not available for all time functions.

In the following example, the function block is used to send a daily pulse to the Automatic Shading block to open the blinds in the morning:

![pulseat blockiosbasic](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/pulseat-blockiosbasic.png)