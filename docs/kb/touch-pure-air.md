# Touch Pure Air

Source: https://www.loxone.com/enen/kb/touch-pure-air/

---

The Loxone Touch Pure Air features five touch points on a glass surface to control the most important functions of a room. When a button is touched, an audible click confirms the action.

The large centre touch zone is ideal for controlling the lighting, while the corner zones are suitable for controlling music and shading. It is based on the [Loxone switch standard](https://www.loxone.com/enen/smart-home/switch-standard/). The buttons can also be freely used for other applications.

An integrated sensor measures temperature and relative humidity.

Please note that there is a certain delay when measuring humidity due to the housing. The Room Comfort Sensor is better suited for a fast detection of changes in humidity.

The device can be operated with two AAA batteries or with external power supply. An orientation light is also available with external power supply.

**[Datasheet Touch Pure Air White / ](https://pim.loxone.com/datasheet/100798-touch-pure-air-white)****[Anthracite / ](https://pim.loxone.com/datasheet/100799-touch-pure-air-anthracite)****[Gold](https://pim.loxone.com/datasheet/100800-touch-pure-air-gold)**

[**Datasheet Touch Pure Classic Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureAir_100463,100464,100501.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Battery Replacement](#battery_change)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Install the mounting frame at the desired location. Insert the battery or connect the power supply (orange/white terminal):

![100399 install lrg](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100399 install lrg.png)

> **ℹ️ Note:** When powering the Touch Pure Air with 24V (via Flush-Mounted Power Supply, Nano IO Air, ...), ensure the system is a PELV installation with GND connected to PE. Otherwise, the internal transformer may disturb the touch sensitivity, especially with short cables.

---

## Commissioning

In delivery state, pairing mode will be active after inserting the battery or after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.

The available functions differ depending on whether the device is battery operated or connected to an external power supply. This is determined during pairing. The device must therefore be paired in the same supply method in which it is to be operated later.
If the supply method is changed later, the device must be deleted from the configuration and paired again.

---

## Battery Replacement

To replace the batteries, remove the Touch Pure Air from the wall. Two AAA batteries are located in the back. Remove the batteries and insert new ones. The device will restart and the status LED will flash green 3 times. In case the LED does not blink at all or is permanently (faintly) red, the batteries are empty. Note: The LED is off during normal operation.

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| T5 | Combined input for the 5 touch points according to the Loxone Switch Standard. | - | ∞ |
| Temperature | Provides the measured value of the current temperature | ° | ∞ |
| Air humidity | Provides the measured value of the current air humidity | % | ∞ |

---

## Actuators

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Backlight | This device output switches the integrated orientation light. Due to excessive power consumption, this actuator only works with an external 24v power supply. | Digital | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Touch Pure Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
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

When connecting to an external power supply, the installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is protected from water.

---

## Documents

**[Datasheet Touch Pure Air White / ](https://pim.loxone.com/datasheet/100798-touch-pure-air-white)****[Anthracite / ](https://pim.loxone.com/datasheet/100799-touch-pure-air-anthracite)****[Gold](https://pim.loxone.com/datasheet/100800-touch-pure-air-gold)**

[**Datasheet Touch Pure Classic Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureAir_100463,100464,100501.pdf)

---