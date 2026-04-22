# LED Surface Mount Spot WW Tree

Source: https://www.loxone.com/enen/kb/led-surface-mount-spot-ww-tree/

---

The Loxone LED Ceiling Spot WW Tree provides warm white light. It is controlled and dimmed directly via the Tree interface.

[**Datasheet LED Ceiling Spot WW Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDCeilingSpotWWTree_100238.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Switch off the supply voltage before installation.

Mount the base plate to the ceiling. The spot will be held to the base plate by 2 magnets.

Connect the power supply and Tree communication wires.

![treespot cabling](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/treespot cabling.png)

Then place the lamp on the base plate. Make sure that the magnets are positioned correctly and that no wires are jammed.

The additional safety rope is a security feature and must not be removed!

---

## Commissioning

For lights, a commissioning mode is active when the devices are supplied with power and have not yet been paired.
In this mode, the device indicates whether it has been connected correctly and, if the connection is successful, provides lighting until it has been paired.

![Tree CommissioningMode SpotWW](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Tree_CommissioningMode_SpotWW.png)

This function is supported since Loxone Config 14.0.3.28, depending on the device firmware.

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Actuators

| Summary | Description | Value Range |
| --- | --- | --- |
| Smart Actuator WW | Smart actuator | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status LED Ceiling Spot WW Tree Gen. 1 | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

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

[**Datasheet LED Ceiling Spot WW Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDCeilingSpotWWTree_100238.pdf)

---