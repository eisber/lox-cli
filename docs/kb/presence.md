# Presence

Source: https://www.loxone.com/enen/kb/presence/

---

The Presence block combines devices and logic that indicate presence in a room. Loxone sensors such as Touch devices, Door and Window Contacts, and Motion and Presence Sensors are supported.

The desired devices for presence detection can be assigned in the block settings. Additional sensors or logic can be connected via the block inputs.

Every button click on a Loxone Touch activates or extends the presence, as well as detected motion and every change of state of window contacts.

The block also supports the **[presence detection via the Loxone App](#presenceapp)**.

Presence overrun times are synchronized for supported devices. After the last presence, the presence output for Presence Sensors remains ON for the duration set in parameter (Pet).

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Programming example](#baseconf)
- [Usage and Functionality](#baseconf)
- [Combined presence output](#BitMask)
- [Presence Detection via the Desktop App](#presenceapp)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Act | Activate | Activates presence on rising edge. On falling edge, the presence extend time (Pet) starts. Continuous ON and each falling edge extends the presence. When using the Desktop App for Presence detection, the App sends a (Act) impulse every (Pet)/2 seconds. | 0/1 |
| Ext | Extend | As long as the input is active, already active presence is extended. On falling edge, the presence extend time (Pet) starts. | 0/1 |
| AE | Activate / Extend | Each change at the input activates or extends the presence. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Pc | Presence combined | Combined presence output. Can be used on all presence and motion inputs of other function blocks. | - | ∞ |
| P | Presence | - | 0/1 |
| Pon | Pulse on presence start | - | 0/1 |
| Poff | Pulse on presence end | - | 0/1 |
| Pd | Current presence duration | Duration of the current presence period. | s | ∞ |
| Warn | Switch-off warning | Warning pulse before end of presence. | - | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Pet | Presence extend time | Starts with the falling edge of input (Act) and input (Ext) and extends presence by set time, this time is doubled for the presence session if a reactivation happens within 30 seconds. Loxone presence sensors adopt this extend time. The device and the block prolong presence for (Pet) seconds after the last presence was detected by the device. | s | 2...∞ | 900 |
| Tw | Switch-off warning time | Time of the switch-off warning before the end of presence. Minimum 2 seconds or 0 to deactivate. | s | 0...∞ | 15 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Configuration | Configuration of the Inputs and Outputs Used. | - | - |
| Number of Entries | Maximum number of saved messages. | 1...50 | 50 |

---

## Programming example

The Presence block is used to combine multiple Presence or Motion Sensors and other devices and to use them together to detect presence.
A window that lists all compatible devices can be found in the settings, or by double-clicking on the block:

![presenceblock deviceselection](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/presenceblock-deviceselection.png)

All devices that are to be used for presence detection in this room are selected here.

When Presence is active, all assigned devices that support presence detection via sound will activate by sound only.

Other objects or additional logic can be used for presence detection via the block inputs.
E.g. pushbuttons, motion detectors, sensors, or other devices that are operated or triggered when a room is occupied.
The presence information is then transferred from the (Pc) output to the (P) input of a function block, e.g. the Lighting Controller:

![presenceblock inputs lightcontroller](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/presenceblock-inputs-lightcontroller.png)

---

## Usage and Functionality

In a quiet setting, it may happen that no presence is detected although a person is in fact present. For example, if you do not move for a long time when reading and no sounds occur. As a result, the lighting is switched off. Within 10 seconds of the end of presence, it is possible to reactivate the presence input using sound alone. You could shout "Hey", for example, and switch the lighting back on. It is not necessary to move during this time.

When a Presence Sensor is linked in the Presence Block, it automatically inherits the parameter Pet (Presence Timeout) setting from the block. Once linked, the overrun time presence for the Presence Sensor can no longer be adjusted independently.

The "P" input from the Presence Sensor will remain active for the full duration of "Pet", even if the Presence Block itself is turned off. If you want new motion to reactivate the Presence Block, it is advised to delete the Presence Sensor from the Presence Block so that the overrun time presence of the Presence Sensor can be adjusted and set to a lower value. Then connect input "P" or input "Mo" of the Presence Sensor to the "Act" input of the Presence Block.

---

## Combined presence output

The combined presence output is similar to the T5 and contains a range of information in the form of a bitmask:

Bit 0 active: Presence active
Bit 1: Motion active
Bit 2: Switch-off warning

Example for analog values:

0: no presence
1: Presence active
2: Motion active
3: Presence + Motion
5: Presence + switch-off warning

This output can be used on all presence and motion sensor inputs of other blocks

---

## Presence Detection via the Desktop App

On a computer running Windows, macOS, or Linux, the [Loxone App](https://www.loxone.com/enen/support/downloads/) enables presence detection.
When you work on your computer, the app detects your presence and transmits it to the block.

To activate the presence detection, first create a Presence block for a room in Loxone Config and save the program in the Miniserver.
Then enable **Presence Detection** in the App's settings menu.
Here you can select which room is to be used for presence detection:

![presence menu comp](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/presence menu comp.png)

Selects the room in which the computer is located.
Each room that is to be used with this function requires a Presence block.
This allows you to change the room being used, e.g. if a notebook is used in different rooms.

You can find and change the room being used by clicking on the App icon in the taskbar or menu bar of your computer:

![presence taskbar](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/presence taskbar.png)