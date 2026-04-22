# Remote Mini Air

Source: https://www.loxone.com/enen/kb/remote-mini-air/

---

The Remote Mini Air is a compact wireless remote control with 5 buttons based on Loxone Air.

[**Datasheet Remote Mini Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RemoteAir_100140.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Functions and button assignment](#buttons)
- [Inputs, Outputs, Properties](#Sensor)
- [Documents](#Documents)

---

## Mounting

Open the battery compartment on the back. Insert the supplied CR2032 lithium battery and close the battery compartment.

![100140 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100140 install.png)

---

## Commissioning

In delivery state, pairing mode will be active after inserting the battery. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds immediately after inserting the battery.

The status LED is located at the top left corner and shines through the plastic housing. The pairing button is the top button (1).

---

## Functions and button assignment

### Loxone Switch Standard

The Remote Mini Air uses a modified version of the **[Switch Standard](https://www.loxone.com/enen/smart-home/standards-and-recommendations/)**.
Drag the Remote Mini Air from the periphery tree to the programming page to use the combined T5 input on supported devices (lighting, shading, audio).
The functions of the buttons are as follows:

![remotebutton standard](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/remotebutton standard.png)

### Buttons as separate inputs

If the buttons are to be used freely in programming, activate the individual inputs by ticking the checkboxes in the settings of the Remote.
The inputs are assigned as follows:

![remotebutton single](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/remotebutton single.png)

---

## Sensors

| Summary | Description | Value Range |
| --- | --- | --- |
| T5 | Combined input for the 5 touch points according to the Loxone Switch Standard. | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Show Button 1 | Show individual button | - |
| Show Button 2 | Show individual button | - |
| Show Button 3 | Show individual button | - |
| Show Button 4 | Show individual button | - |
| Show Button 5 | Show individual button | - |
| Button Behaviour | Specifies the behavior when a button is pressed. Pulse: Sends a pulse on rising edge OnOff: Sends ON on rising edge and OFF on falling edge, used for long click Automatic: Sends a pulse on rising edge for buttons 1 & 4 (shading) and 3 (lighting). Sends ON on rising edge and OFF on falling edge for buttons 2 & 5 (music) to enable volume up/down via long press | - |
| Display Error Output | If checked, error output will be displayed in 2nd row. | - |

---

## Documents

[**Datasheet Remote Mini Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RemoteAir_100140.pdf)

---