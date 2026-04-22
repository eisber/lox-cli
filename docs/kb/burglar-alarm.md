# Burglar Alarm

Source: https://www.loxone.com/enen/kb/burglar-alarm/

---

The burglar alarm block can be used to easily set up an alarm system with the existing sensors (motion detectors, window contacts, etc.).

The alarm can be armed immediately or after a delay either via the block inputs or directly through the app.
If an Alarm is triggered, various alarm levels are started in a time-delayed manner.
An alarm can create alerts by flashing lights, playing an alarm sound/loud music, making phone calls, or raising the blinds and opening curtains.

Double click on the block to bring up the configuration menu, this allows selection of which sensors should trigger an alarm and which function blocks (lighting controllers etc) should be used as an alert.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Output on WDot](#AQT)
- [Presence Sensor Functionality](#BWM)
- [Arming the Alarm: Without Delay vs. With Delay](#Arming Alarm)
- [Disarm vs Acknowledge Alarm](#Disarm vs Acknowledge Alarm)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| P | Presence | Alarm input for presence detection. The name of the connected sensor is used in the user interface. | 0/1 |
| Gb | Glass breakage | Alarm input for broken glass detection. The name of the connected sensor is used in the user interface. | 0/1 |
| Wc | Window contacts | Alarm input for window contacts (0 = closed, 1 = open). The name of the connected sensor is used in the user interface. | 0/1 |
| Dc | Door contacts | Alarm input for door contacts (0 = closed, 1 = open). The name of the connected sensor is used in the user interface. | 0/1 |
| Ot | Other | Alarm input for additional sensors and detectors. The name of the connected sensor is used in the user interface. | 0/1 |
| Tg | Toggle with presence detection | Toggles between arm / disarm. Presence sensors are used to trigger an alarm. The name of the connected sensor is used in the user interface. | 0/1 |
| Tgnp | Toggle without presence detection | Toggles between arm / disarm. Presence sensors are not used to trigger an alarm. | 0/1 |
| A | Arm with presence detection | Arms the alarm system. Presence sensors are used to trigger an alarm. The name of the connected sensor is used in the user interface. | 0/1 |
| Anp | Arm without presence detection | Arms the alarm system. Presence sensors are not used to trigger an alarm. The name of the connected sensor is used in the user interface. | 0/1 |
| Ad | Arm delayed with presence detection | Arms the alarm system with a delay (Ard). Presence sensors are used to trigger an alarm. The name of the connected sensor is used in the user interface. | 0/1 |
| Adnp | Arm delayed without presence detection | Arms the alarm system with a delay (Ard). Presence sensors are not used to trigger an alarm. The name of the connected sensor is used in the user interface. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |
| Ca | Confirm alarm | Confirms the current alarm and resets all alarm outputs. The alarm system remains armed. The name of the connected sensor is used in the user interface. | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Tg), (Tgnp), (A), (Anp), (Ad), (Adnp) when On. (e.g Child lock, cleaning) Control via user interface is still possible. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| S | Status | 0 = Disarmed 1 = Armed with motion sensors 2 = Armed without motion sensors | - | 0...2 |
| Sa | Silent alarm | - | 0/1 |
| Aa | Audible alarm | - | 0/1 |
| Va | Visual alarm | - | 0/1 |
| Ia | Internal alarm | - | 0/1 |
| Ea | External alarm | - | 0/1 |
| Ra | Remote alarm | - | 0/1 |
| N | Number of active sensors | - | ∞ |
| At | Alarm test | Only used if parameter Atm = 1. | - | 0/1 |
| Rtad | Remaining time arming delay | s | 0...∞ |
| Ca | Cause of alarm | Reports cause of the last alarm. | - | - |
| Ta | Time and date of alarm | Reports date and time of the last alarm. | - | - |
| WDs | Window / door state | On when any windows or doors are open. If parameter (Aoc) is 1: Any open window / door is ignored and inputs (Wc & Dc) is set to 0 while arming. | - | 0/1 |
| WDot | Text output for open windows / doors | Outputs the names of windows and doors that were open while arming. (Output can be connected to a TTS input.) | - | - |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour – When activating/deactivating an alarm system (max. every 10s) The data is saved on the SD card. | - | 0/1 | 0 |
| Ard | Arming delay | s | 0...∞ | 600 |
| Sad | Silent alarm delay | s | 0...∞ | 0 |
| Aad | Audible alarm delay | s | 0...∞ | 20 |
| Vad | Visual alarm delay | s | 0...∞ | 40 |
| Iad | Internal alarm delay | s | 0...∞ | 90 |
| Ead | External alarm delay | s | 0...∞ | 150 |
| Rad | Remote alarm delay | s | 0...∞ | 300 |
| Eip | Extension of alarm input pulses | Defines the minimum duration of how long alarm input pulses remain active. Used for calculating the number of active sensors at output (N). 0 = Each pulse increases the value at output (N), only as long as it is active. | s | 0...∞ | 0 |
| Spt | Second presence sensor time window | Defines the time window until a second presence sensor must be triggered in order for the alarm to be activated. If only one presence sensor is used, this parameter has no effect and the alarm is triggered immediately. 0 = Function is not used | s | 0...∞ | 900 |
| Atm | Alarm test mode | 1 = Only output (At) is triggered when alarm is active. | - | 0/1 | 0 |
| MaxA | Maximum alarm duration | The alarm is reset to a silent alarm at the end of set duration. 0 = No duration limit The Maxium alarm duration should be longer than the longest alarm delay, otherwise, certain alarm levels will never be activated! | s | 0...∞ | 900 |
| Sac | Silent alarm confirmation | 1 = The silent alarm is confirmed when the maximum alarm duration (MaxA) is reached. | - | 0/1 | 0 |
| Aoc | Arm open contact | 0 = Alarm triggers when a window or door is open while arming. 1 = Alarm only triggers when the open/closed state of a door or window changes when armed. | - | 0/1 | 0 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Suppressed outputs | Suppression of all none connected outputs Aa - Ra. The silent alarm can't be suppressed! | - | - | - |
| Delay after start | Delay of the activation and arming of the alarm following a restart of the Miniserver in seconds. A value below 10 may lead to false alarms on a reboot! Only has an effect if remanence is enabled and the alarm is armed before restart! | s | 0...3600 | - |
| Number of Entries | Maximum number of saved messages. | - | 2...100 | - |
| Configuration | Configuration of the available inputs and outputs. | - | - | - |

---

## Output on WDot

In case a window or door is still open when the alarm system is armed (delayed or normal), output (WDs) is activated and a message is output at (WDot). For example, "Attention, kitchen window is still open"! If several windows / doors are open, the output reads: "Attention! Several windows or doors are still open!".

---

## Presence Sensor Functionality

When the alarm system is armed, **Tree Presence Sensors** connected to the alarm system are automatically set to the shortest overrun duration.

For **battery-operated Presence Sensor Air**, the overrun time can only be adjusted during the next status update from the sensor. These sensors do not support immediate changes to the overrun time.

If the alarm is armed while the Presence Sensor still detects presence, the active motion will immediately trigger the alarm. Therefore, to prevent false alarms, it is crucial to arm the system with a delay.

---

## Arming the Alarm: Without Delay vs. With Delay

When the alarm system is armed, the overrun time for Presence Sensors (Tree) is automatically set to the shortest duration. However, the alarm may be triggered immediately if the Presence Sensor's input is still active. The input will deactivate after 3 seconds of no motion detection. This delay might be too long for the burglar alarm system, potentially triggering a false alarm.

**Arming Without Delay**

Arming the system without a delay is appropriate only when you are confident that no one is inside the building. For instance, if you leave home and forget to set the system with a delay, you can activate it immediately once the house is confirmed empty, ensuring that no movement will trigger the alarm.

**Arming With Delay**

If there is a chance that motion could still be detected, it is advisable to set the system to activate after a delay. The delay duration can be modified to fit your requirements, providing sufficient time to exit without inadvertently setting off the alarm.

---

## Disarm vs Acknowledge Alarm

When the alarm is triggered, there are 2 different options in the App:
- **Disarm:** Disarms the alarm. To trigger the alarm again, it must be re-armed first.
- **Acknowledge Alarm:** Acknowledges the alarm without disarming it. The alarm remains armed, so any connected sensor can trigger it immediately again.

![Alarm DisarmAcknowledgeAlarm](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Alarm_DisarmAcknowledgeAlarm.png)