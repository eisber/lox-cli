# LED Spot RGBW Tree

Source: https://www.loxone.com/enen/kb/led-spot-rgbw-tree/

---

The LED Spot RGBW Tree is a LED spot with warm white and colored light for 24VDC supply. The connection to the Miniserver is via Loxone Tree technology.

The spot is available in different variants for ceiling installation or surface mounting.

[**Datasheet LED Spot RGBW Tree **](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDSpotRGBWTreeEU_100330.pdf)

[**Datasheet LED Ceiling Spot RGBW Tree **](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LedCeilingSpotRGBWTree_100503,100504.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Tree Communication and Voltage Drop](#TreeVdrop)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

![LEDspotRGBW planning](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LEDspotRGBW planning.png)

Switch off the supply voltage before installation.

Connect the power supply (orange/white wire) and Tree communication wires (green/white wire).

Flip the spring clips back and snap the spotlight into the mounting hole. Make sure that no wires are jammed.

![treespot cabling](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/treespot cabling.png)

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
| Online Status LED Spot RGBW Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

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

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

Ensure that the device is protected from water.

The recessed spots are UL 1598 Type IC certified, making them suitable for installation on flammable surfaces.

---

## Documents

[**Datasheet LED Spot RGBW Tree **](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDSpotRGBWTreeEU_100330.pdf)

[**Datasheet LED Ceiling Spot RGBW Tree **](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LedCeilingSpotRGBWTree_100503,100504.pdf)

---