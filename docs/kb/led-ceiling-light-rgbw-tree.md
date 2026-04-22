# LED Ceiling Light RGBW Tree

Source: https://www.loxone.com/enen/kb/led-ceiling-light-rgbw-tree/

---

The Loxone LED Ceiling Light RGBW Tree is a surface mounted 24VDC fixture with warm white and coloured light. It features an integrated motion and brightness sensor. It is connected to the Miniserver via the Loxone Tree technology.

[**Datasheet LED Ceiling Light RGBW Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDCeilingLightTree_100288,100289,100495.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Tree Communication and Voltage Drop](#TreeVdrop)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Switch off the supply voltage before installation.

Mount the base plate to the ceiling. The lamp will be held to the base plate by 3 magnets.

Connect the power supply and Tree communication wires.

![100288 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100288 install.png)

Then place the lamp on the base plate. Make sure that the magnets are positioned correctly and that no wires are jammed.

The additional safety rope is a security feature and must not be removed!

Shortly after power-up, the status LED will blink orange if the wiring is correct (connection to Tree Extension and Miniserver is established).

---

## Commissioning

For lights, a commissioning mode is active when the devices are supplied with power and have not yet been paired.
In this mode, the device indicates whether it has been connected correctly and, if the connection is successful, provides lighting until it has been paired.

![Tree CommissioningMode CeilingLightRGBW](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Tree_CommissioningMode_CeilingLightRGBW.png)

This function is supported since Loxone Config 14.0.3.28, depending on the device firmware.

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Tree Communication and Voltage Drop

If the current flow on the GND line results in a voltage drop too high, this potential difference also affects the tree communication.
To solve this problem, split consumers of higher power over longer distances to several supply lines, or use a supply line with a higher cross-section, or a power supply close to the consumers.
For existing installations, doubling the cross-section of the GND line is often sufficient to eliminate the potential difference.

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Motion | Input is active when motion is detected | - | 0/1 |
| Brightness | Provides the measured value of the current brightness. Value is not updated when the light is on | Lx | ∞ |

---

## Actuators

| Summary | Value Range |
| --- | --- |
| Smart Actuator | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status LED Ceiling Light RGBW Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - | - |
| Actuator Type | Use device with Standard Actuator(s) or Smart Actuator(s) Smart Actuators support dynamic fade times and can only be used with the Lighting Controller. | - | - | - |
| Lighting Group | Assigned Lighting Group. To create a new group simply start typing a name. | - | - | - |
| Sensitivity | Effects the sensitivity and thus also the distance over which motion can be sensed | - | - | - |
| Overrun time motion | The 'Motion (Mo)' input remains active following the last detected motion for the set time. A higher value means less packets have to be sent via Tree and Link. If the motion detector is used for the alarm system, the time is automatically set to 3 seconds when the alarm system is armed. | s | 3...900 | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is protected from water.

---

## Documents

[**Datasheet LED Ceiling Light RGBW Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_LEDCeilingLightTree_100288,100289,100495.pdf)

---