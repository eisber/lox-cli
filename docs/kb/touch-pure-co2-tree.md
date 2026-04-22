# Touch Pure CO2 Tree

Source: https://www.loxone.com/enen/kb/touch-pure-co2-tree/

---

The Loxone Touch Pure CO2 Tree features five touch points on a glass surface to control the most important functions of a room. When a button is touched, an audible click confirms the action.

The large center touch zone is ideal for controlling the lighting, while the corner zones are suitable for controlling music and shading. It is based on the [Loxone switch standard](https://www.loxone.com/enen/smart-home/switch-standard/). The buttons can also be freely used for other applications.

It also features a controllable orientation light and the integrated sensor measures temperature, relative humidity and CO2.

Please note that there is a certain delay when measuring humidity due to the housing. The Room Comfort Sensor is better suited for a fast detection of changes in humidity.

**[Datasheet Touch Pure CO2 Tree White / ](https://pim.loxone.com/datasheet/100795-touch-pure-co2-tree-white)****[Anthracite / ](https://pim.loxone.com/datasheet/100796-touch-pure-co2-tree-anthracite)****[Gold](https://pim.loxone.com/datasheet/100797-touch-pure-co2-tree-gold)**

[**Datasheet Touch Pure Classic CO2 Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureCO2Tree_100517,100518.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Calibrate CO2 sensor](#calco2)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Install the mounting frame at the desired location.

Connect the power supply (orange/white terminal) and Tree communication wires (green/white terminals). Shortly after power-up, the status LED will blink orange if the wiring is correct (connection to Tree Extension and Miniserver is established).

Mount the device by snapping it onto the mounting frame.

![100396 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100396 install.png)

---

## Commissioning

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Calibrate CO2 sensor

The CO2 sensor automatically calibrates itself weekly if it is in continuous operation for this period.

Manual calibration of the CO2 sensor is not necessary, but can be performed for special applications.
To do this, first ensure that the CO2 value at the measuring point is as low and constant as possible.
Now measure the CO2 value at the measuring point with a suitable measuring device.
Immediately afterwards, assign the measured value to the sensor via a [device command](https://www.loxone.com/help/device-command/) or [webservice](https://www.loxone.com/enen/kb/web-services) command, in the following example 450ppm:

miniserver/dev/sys/wsdevice/devicename-or-serialnr/ForceRecalibration/450

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| T5 | Combined input for the 5 touch points according to the Loxone Switch Standard. | - | ∞ |
| Temperature | Provides the air temperature. | ° | -40...125 |
| Humidity | Provides the air humidity. | % | 0...100 |
| CO2 | Provides the CO2 value. | ppm | 0...40000 |

---

## Actuators

| Summary | Description | Value Range |
| --- | --- | --- |
| Orientation light | Digital output to control the orientation light. | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Touch Pure CO2 Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - |
| Show Button 1 | Show individual button | - | - |
| Show Button 2 | Show individual button | - | - |
| Show Button 3 | Show individual button | - | - |
| Show Button 4 | Show individual button | - | - |
| Show Button 5 | Show individual button | - | - |
| Audible acknowledgement | Audible acknowledgement on button press | - | - |
| Transmission cycle | Specifies how often analog values from sensors are transmitted [s] 0 = Values are updated at least every 60 min, or if temperature deviates by 0.2 °C, humidity deviates by 2% or CO2 deviates by 50 ppm to the last transmitted value | 0...3600 | 0 |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is protected from water.

---

## Documents

**[Datasheet Touch Pure CO2 Tree White / ](https://pim.loxone.com/datasheet/100795-touch-pure-co2-tree-white)****[Anthracite / ](https://pim.loxone.com/datasheet/100796-touch-pure-co2-tree-anthracite)****[Gold](https://pim.loxone.com/datasheet/100797-touch-pure-co2-tree-gold)**

[**Datasheet Touch Pure Classic CO2 Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureCO2Tree_100517,100518.pdf)

---