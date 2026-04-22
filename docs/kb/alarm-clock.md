# Alarm Clock

Source: https://www.loxone.com/enen/kb/alarm-clock/

---

Creates a pulse on output (Aon) at a preset time.
A snooze function is also integrated.
In addition to individual days of the week, you can define your own alarm times for the operating modes "Public Holiday", "Vacation", and "School Holidays/Day off".
For the operating modes either corresponding operating times must be created or the output of the respective operating mode must be activated via a program logic.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Set alarm time](#TimeConfig)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Ca | Confirm alarm | - | 0/1 |
| S | Start snooze timer | Pulse sets output (Buzzer) to 0 and starts snooze timer for the duration of parameter (Sd). Another pulse during an active snooze timer restarts the timer. | - | 0/1 |
| DisA | Disable alarm entries | Disables all alarm entries. | - | 0/1 |
| Tg | Toggle | Enables / disables the alarm entry 'Default alarm'. A pulse during an active alarm ends the alarm. | - | 0/1 |
| Set | Set alarm time for default alarm | Defined by minutes past midnight (E.g. 360 min = 06:00am) | min | 0...1439 |
| Rtd | Reset to default | Resets parameters and settings of the block to the default values as specified in the block preset. | - | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Aon | Pulse on alarm start | - | 0/1 |
| Aoff | Pulse on alarm end | - | 0/1 |
| Buzzer | Buzzer | ON when alarm sound is active. OFF during snooze time. | - | 0/1 |
| A | Alarm | ON when alarm is active. | - | 0/1 |
| Pa | Pre-alarm | Outputs a pulse before the alarm. Start time set via parameter (Pat). | - | 0/1 |
| Rst | Remaining snooze time | s | 0...∞ |
| Da | Default alarm state | On, when alarm entry 'Default alarm' is enabled. | - | 0/1 |
| Tna | Time of next alarm | Outputs date and time of the next alarm | - | - |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| MaxA | Maximum alarm duration | Ends alarm after set time. If setting "Confirmation required" is activated, the snooze timer starts after the alarm duration has expired. | s | 0...∞ | 120 |
| Pat | Pre-alarm time | Time that the pre-alarm is started before the alarm. | s | 0...∞ | 180 |
| Sd | Snooze duration | Snooze duration until the alarm sound is restarted. | s | 60...1800 | 300 |
| Bri | Touch Nightlight display brightness inactive | This parameter is only visible in certain configurations. | % | 0...100 | 0 |
| Bra | Touch Nightlight display brightness active | This parameter is only visible in certain configurations. | % | 0...100 | 50 |
| As | Alarm sound (Touch Nightlight) | 1: Deep 4-way beep 2: High 4-way beep 3: Deep 2-way beep 4: High 2-fold beep 5: Siren This parameter is only visible in certain configurations. | - | 1...5 | 1 |
| Asv | Alarm sound volume (Touch Nightlight) | This parameter is only visible in certain configurations. | % | 5...100 | 100 |
| Asf | Alarm sound fade-in (Touch Nightlight) | 1 = Alarm sound fades in for the duration of 60 seconds. 0 = No fade-in This parameter is only visible in certain configurations. | - | 0/1 | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Alarm settings | Edit Alarms | - |
| Acknowledgement required | After the alarm the snooze timer starts automatically until it is acknowledged | - |

---

## Set alarm time

With the green cross or eraser, alarm times can be added and removed. The alarm time can be defined directly in the settings menu, but also in the user interface.

![AlarmClock TimeConfig](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AlarmClock_TimeConfig.jpg)