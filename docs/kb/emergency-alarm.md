# Emergency Alarm

Source: https://www.loxone.com/enen/kb/emergency-alarm/

---

The Emergency Alarm allows individuals in emergency situations to trigger a call for help or an alarm.
For this, compatible devices such as the [Button Air](https://www.loxone.com/help/button-air), the [Wrist Button Air](https://www.loxone.com/help/wrist-button-air), or a Loxone Touch are linked to the block, or the block's inputs and outputs are used for manual programming.

The alarm is triggered when an assigned button is pressed.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Programming example](#baseconf)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tg | Toggle push & hold | If input is active longer than parameter (Ta), an alarm is triggered. If input is active longer than parameter (Tc) while an alarm is active, the alarm is confirmed. The name of the connected sensor is used in the user interface. | 0/1 |
| A | Activate alarm | Triggers the alarm. The name of the connected sensor is used in the user interface. | 0/1 |
| Ca | Confirm alarm | Confirm alarm. The name of the connected sensor is used in the user interface. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| A | Alarm | 0/1 |
| Aon | Pulse on alarm start | 0/1 |
| Aoff | Pulse on alarm end | 0/1 |
| Ca | Cause of alarm | Provides the name or location of the triggering button or the name of the app user. | - |
| Cc | Cause of confirmation | Provides the name or location of the acknowledging button or the name of the app user. | - |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Ta | Push & hold time alarm | For input (Tg). | s | 1...10 | 4 |
| Tc | Push & hold time confirmation | For input (Tg). | s | 1...10 | 2 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Number of Entries | Maximum number of saved messages. | 2...100 | 20 |
| Configuration | Configure devices for use as emergency buttons. Normal buttons (e.g. Touch Tree) trigger an alarm if they are pressed for the duration 'Ta'. A further pulse for the duration 'Tc' acknowledges the alarm. | - | - |

---

## Programming example

A window for assigning supported devices opens when the block is created, or when clicking on Assign objects at the lower edge of the block.
This is where the devices to be used for emergency alarms are selected:

![emergencyalarm deviceselection](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/emergencyalarm-deviceselection.png)

When using a dedicated device like the [Button Air](https://www.loxone.com/help/button-air) or the [Wrist Button Air](https://www.loxone.com/help/wrist-button-air), the Alarm is triggered immediately when pressing it. Parameter Ta is ignored.

When using a Loxone Touch, the center button (I3) is used to trigger the alarm. A long click with the time specified by parameter Ta is required to trigger the alarm.

Other devices such as conventional retractive buttons or switches can also be connected to the function block via digital inputs.

Subsequently, the sending of the call for help or an alarm can be configured via the text outputs of the function block, for example, by a notification or the caller service.