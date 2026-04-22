# LED Spot RGBW Tree Gen. 1

Source: https://www.loxone.com/enen/kb/led-spot-rgbw-tree-gen-1/

---

The LED Spot RGBW Tree is a recessed LED spot with warm white and coloured light for supply with 24VDC. The connection to the Miniserver is via Loxone Tree technology.

[**Datasheet LED Spot RGBW Tree Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDSpotRGBWTreeGen1_100269.pdf)

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

Connect the power supply (orange/white wire) and Tree communication wires (green/white wire).

Flip the spring clips back and snap the spotlight into the mounting hole. Make sure that no wires are jammed.

![treespot cabling](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/treespot cabling.png)

---

## Commissioning

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
| Online Status LED Spot RGBW Tree Gen. 1 | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

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

[**Datasheet LED Spot RGBW Tree Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDSpotRGBWTreeGen1_100269.pdf)

---