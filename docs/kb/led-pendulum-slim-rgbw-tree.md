# LED Pendulum Slim RGBW Tree

Source: https://www.loxone.com/enen/kb/led-pendulum-slim-rgbw-tree/

---

The LED Pendulum Slim RGBW Tree is a tubular pendant fitting with warm white and coloured light, for supply with 24VDC. The connection to the Miniserver is made via Loxone Tree technology.

[**Datasheet LED Pendulum Slim RGBW Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDPendulumSlimTree_100308,100309,100492.pdf)

[**Datasheet LED Pendulum Slim RGBW Tree US**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDPendulumSlimRGBWTreeUS_100369.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Tree Communication and Voltage Drop](#TreeVdrop)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Switch off the supply voltage before installation.

If necessary, shorten the connection cable to the desired length.

Mount the base plate to the ceiling. The canopy is later screwed onto the base plate.

Connect the power supply (orange/white wire) and Tree communication wires (green/white wire).

Then screw the lamp onto the base plate. Make sure that no wires are jammed.

---

## Commissioning

For lights, a commissioning mode is active when the devices are supplied with power and have not yet been paired.
In this mode, the device indicates whether it has been connected correctly and, if the connection is successful, provides lighting until it has been paired.

![Tree CommissioningMode SpotRGBW](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Tree_CommissioningMode_SpotRGBW.png)

This function is supported since Loxone Config 14.0.3.28, depending on the device firmware.

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Tree Communication and Voltage Drop

If the current flow on the GND line results in a voltage drop too high, this potential difference also affects the tree communication.
To solve this problem, split consumers of higher power over longer distances to several supply lines, or use a supply line with a higher cross-section, or a power supply close to the consumers.
For existing installations, doubling the cross-section of the GND line is often sufficient to eliminate the potential difference.

---

## Actuators

| Summary | Description | Value Range |
| --- | --- | --- |
| Smart actuator RGBW | Smart actuator | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status LED Pendulum Slim RGBW Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - |
| Actuator Type | Use device with Standard Actuator(s) or Smart Actuator(s) Smart Actuators support dynamic fade times and can only be used with the Lighting Controller. | - |
| Lighting Group | Assigned Lighting Group. To create a new group simply start typing a name. | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is protected from water.

---

## Documents

[**Datasheet LED Pendulum Slim RGBW Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDPendulumSlimTree_100308,100309,100492.pdf)

[**Datasheet LED Pendulum Slim RGBW Tree US**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDPendulumSlimRGBWTreeUS_100369.pdf)

---