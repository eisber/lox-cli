# Motion Sensor Tree

Source: https://www.loxone.com/enen/kb/motion-sensor-tree/

---

The Loxone Motion Detector Tree is used to detect motion and brightness in a room.

Note: The Motion Sensor Tree is no longer available and has been replaced by the [Presence Sensor Tree](https://www.loxone.com/help/presence-sensor-tree).

> **ℹ️ Note:** The Motion Sensor can also detect some pets. Although the sensitivity of the sensor can be reduced, there is no dedicated technology that excludes the detection of pets.

[**Datasheet Motion Sensor Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MotionSensorTree_100223.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

![100223 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100223 install.png)

> **ℹ️ Note:** For motion to be detected accurately, correct placement of the Motion Sensor on the ceiling is essential. The following drawings can assist with this:

![motiondetector range1](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/motiondetector_range1.jpg)

![motiondetector range2](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/motiondetector_range2.jpg)

Connect the power supply (orange/white terminal) and Tree communication wires (green/white terminals). To complete the installation, attach the Presence Sensor to the mounting ring. Shortly after power-up, the status LED will blink orange if the wiring is correct (connection to Tree Extension and Miniserver is established).

---

## Commissioning

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Motion | Input is active when motion is detected | - | 0/1 |
| Brightness | Provides the measured value of the current brightness | Lx | 0...188 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Motion Sensor Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - | - |
| Sensitivity | Effects the sensitivity and thus also the distance over which motion can be sensed | - | - | - |
| Overrun time motion | The 'Motion (Mo)' input remains active following the last detected motion for the set time. A higher value means less packets have to be sent via Tree and Link. If the motion detector is used for the alarm system, the time is automatically set to 3 seconds when the alarm system is armed. | s | 3...900 | - |
| Transmission cycle | Specifies how often analog values from sensors are transmitted [s] 0 = Values are updated at least every 60 min, or if brightness deviates 5% to the previous transmitted value | - | 0...3600 | 900 |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is protected from water.

---

## Documents

[**Datasheet Motion Sensor Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MotionSensorTree_100223.pdf)

---