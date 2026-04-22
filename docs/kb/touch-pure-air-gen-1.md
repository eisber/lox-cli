# Touch Pure Air Gen. 1

Source: https://www.loxone.com/enen/kb/touch-pure-air-gen-1/

---

The Loxone Touch Pure Air features five touch points on a glass surface to control the most important functions of a room. When a button is touched, an audible click confirms the action.

The large centre touch zone is ideal for controlling the lighting, while the corner zones are suitable for controlling music and shading. It is based on the [Loxone switch standard](https://www.loxone.com/enen/smart-home/switch-standard/). The buttons can also be freely used for other applications.

An integrated sensor measures temperature and relative humidity.

Please note that there is a certain delay when measuring humidity due to the housing. The Room Comfort Sensor is better suited for a fast detection of changes in humidity.

[**Datasheet Touch Pure Air Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureAirGen1_100206.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Battery Replacement](#battery_change)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Install the mounting frame at the desired location. Insert the battery and start the pairing process. Then attach the device by snapping it onto the mounting frame.

![100206 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100206 install.png)

---

## Commissioning

In delivery state, pairing mode will be active after inserting the battery. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after inserting the battery. The pairing button is located on the back of the device.

---

## Battery Replacement

To replace the battery, remove the Touch Air from the wall. The CR2450 lithium battery is located in the back. Remove the battery and insert a new one. The device will restart and the status LED will flash green 3 times. In case the LED does not blink at all or is permanently (faintly) red, the batteries are empty. Note: The LED is off during normal operation.

> **ℹ️ Note:** Please only use the lithium battery provided by Loxone, as these will ensure an exact fit!

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| T5 | Combined input for the 5 touch points according to the Loxone Switch Standard. | - | ∞ |
| Temperature | Provides the measured value of the current temperature | ° | ∞ |
| Air humidity | Provides the measured value of the current air humidity | % | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Touch Pure Air Gen. 1 | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - | - | - |
| Show Button 1 | Show individual button | - | - | - |
| Show Button 2 | Show individual button | - | - | - |
| Show Button 3 | Show individual button | - | - | - |
| Show Button 4 | Show individual button | - | - | - |
| Show Button 5 | Show individual button | - | - | - |
| Transmission cycle | Request or transmit cycle for temperature and humidity (1-120 min, 0 = Off) | min | 0...120 | - |
| Audible acknowledgement | Audible acknowledgement on button press | - | - | - |
| Button Behaviour | Specifies the behavior when a button is pressed. Pulse: Sends a pulse on rising edge OnOff: Sends ON on rising edge and OFF on falling edge, used for long click Automatic: Sends a pulse on rising edge for buttons 1 & 4 (shading) and 3 (lighting). Sends ON on rising edge and OFF on falling edge for buttons 2 & 5 (music) to enable volume up/down via long press | - | - | - |

---

## Safety Instructions

Ensure that the device is protected from water.

---

## Documents

[**Datasheet Touch Pure Air Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureAirGen1_100206.pdf)

---