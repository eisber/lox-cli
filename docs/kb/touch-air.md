# Touch Air

Source: https://www.loxone.com/enen/kb/touch-air/

---

For installation in the UK, please refer to the use of the EU back box for mounting [here](#backbox)

The Loxone Touch Air features five touch points to control the most important functions of a room. When a button is touched, an audible click confirms the action.

The large centre touch zone is ideal for controlling the lighting, while the corner zones are suitable for controlling music and shading. It is based on the [Loxone switch standard](https://www.loxone.com/enen/smart-home/switch-standard/). The buttons can also be freely used for other applications.

An integrated sensor measures temperature and relative humidity.

Please note that there is a certain delay when measuring humidity due to the housing. The Room Comfort Sensor is better suited for a fast detection of changes in humidity.

[**Datasheet Touch Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchAir_100155.pdf)

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

![100155 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100155 install.png)

---

## Commissioning

In delivery state, pairing mode will be active after inserting the battery. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after inserting the battery. The pairing button is located on the back of the device.

---

## Battery Replacement

To replace the battery, remove the Touch Air from the wall. The CR2450 lithium battery is located in the back. Remove the battery and insert a new one. The device will restart and the status LED will flash green 3 times. In case the LED does not blink at all or is permanently (faintly) red, the batteries are empty. Note: The LED is off during normal operation.

> **ℹ️ Note:** Please only use the lithium battery provided by Loxone, as these will ensure an exact fit!

![100155 batt](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100155 batt.png)

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| T5 | Combined input for the 5 touch points based on the Loxone switch standard. Maximum duration of a long click: 15sec | - | ∞ |
| Temperature | Provides the measured value of the current temperature | ° | -20...80 |
| Air humidity | Provides the measured value of the current air humidity | % | 0...100 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Touch Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
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

[**Datasheet Touch Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchAir_100155.pdf)

---

## UK Installation

The Loxone Touch is a slimline product, as such it should be mounted on a European style circular back box for correct fitting. If it is mounted on a standard UK back box/pattress box then it will have an improper fitting and will be likely to fall in at the corners or fall off the back box. [Click on the image to purchase them.](https://shop.loxone.com/enuk/circular-dry-lining-box.html)

Additionally for the Touch Air please do not use metal back boxes as these can cause communication issues on the Air signal which may cause excess data package retries. This could lead to higher energy consumption than normal requiring more frequent battery replacements.

[
![European Dry Lining Backbox](https://www.loxone.com/enen/wp-content/uploads/sites/3/2021/04/Orange-circular-back-box-300x300-1.jpg)
](https://shop.loxone.com/enuk/circular-dry-lining-box.html)