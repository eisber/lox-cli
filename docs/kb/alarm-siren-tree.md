# Alarm Siren Tree

Source: https://www.loxone.com/enen/kb/alarm-siren-tree/

---

The Alarm Siren Tree is an external warning device with strobe light, for example as part of the burglar alarm. When triggered, the device emits an audible and visual alarm. The integrated tamper switch provides additional protection.

[**Datasheet Alarm Siren Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AlarmSirenTree_100313.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Select an installation location where the device is protected from extreme weather conditions and damage. Make sure that the cable outlet is covered by the unit after installation.

Connect the power supply (orange/white) and Tree communication wires (green/white) using the supplied 3M jelly crimp connectors. The wires including insulation are inserted into the connector, then the connector is crimped with pliers.

To complete the installation, place the top cover on the base unit and secure it to the bottom with the two screws.

---

## Commissioning

Shortly after power-up, the status LED will blink orange if the wiring is correct and a connection to the Miniserver (Tree Extension is online) has been established.

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Sensors

| Summary | Description | Value Range |
| --- | --- | --- |
| Tamper switch | Input is active in normal state. The input is deactivated when sabotage is detected by the contact on the device. | 0/1 |

---

## Actuators

| Summary | Description | Value Range |
| --- | --- | --- |
| Alarm light | Output to activate the alarm light | 0/1 |
| Alarm sound | Output to activate the alarm sound | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Alarm Siren Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | You will be notified via System Status or Could Mailer if the device is no longer available or offline. As this device and it's functionality are critical to safety, it is not possible to disable this setting for this device. | - | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - | - |
| Maximum audible alarm duration [s] | Maximum audible alarm duration (0 - 1800 seconds, 0 = no limit) | s | 0...1800 | 120 |

---

## Safety Instructions

When connecting to an external power supply, the installation must be carried out by a qualified technician in accordance with all applicable regulations.

---

## Documents

[**Datasheet Alarm Siren Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AlarmSirenTree_100313.pdf)

---