# LED Ceiling Light RGBW Air

Source: https://www.loxone.com/enen/kb/led-ceiling-light-rgbw-air/

---

The Loxone LED Ceiling Light RGBW Air is a surface mounted fixture with warm white and coloured light. It features an integrated motion and brightness sensor. It is connected to the Miniserver via the Loxone Air wireless technology.

[**Datasheet LED Ceiling Light RGBW Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDCeilingLightAir_100286,100287,100500.pdf)

[**Datasheet LED Ceiling Light RGBW Air US**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDCeilingLightAirUS_100635,100636.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Switch off the supply voltage before installation.

Mount the base plate to the ceiling. The lamp will be held to the base plate by 3 magnets.

Connect the mains voltage lines (L, N, PE).

![100286 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100286 install.png)

Then place the lamp on the base plate. Make sure that the magnets are positioned correctly and that no wires are jammed.

The additional safety rope is a security feature and must not be removed!

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for two minutes, then pairing mode is activated for 30 minutes.

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Motion | Input is active when motion is detected | - | 0/1 |
| Brightness | Provides the measured value of the current brightness. Value is not updated when the light is on | Lx | ∞ |

---

## Actuators

| Summary | Description | Value Range |
| --- | --- | --- |
| Smart Actuator | The minimum fading time is 1s. | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status LED Ceiling Light RGBW Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Actuator Type | Use device with Standard Actuator(s) or Smart Actuator(s) Smart Actuators support dynamic fade times and can only be used with the Lighting Controller. | - |
| Overrun time motion | The 'Motion' input (Mo) will remain ON for this duration after detecting the last movement. The longer this overrun period, the more energy efficient the device can operate. If a motion sensor is used as part of a burglar alarm, then the overrun period will automatically be set to 3 seconds when the alarm is armed and the device reports to the Miniserver. | - |
| Sensitivity | Effects the sensitivity and thus also the distance over which motion can be sensed | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is protected from water.

---

## Documents

[**Datasheet LED Ceiling Light RGBW Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDCeilingLightAir_100286,100287,100500.pdf)

[**Datasheet LED Ceiling Light RGBW Air US**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDCeilingLightAirUS_100635,100636.pdf)

---