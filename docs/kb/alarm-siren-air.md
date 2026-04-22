# Alarm Siren Air

Source: https://www.loxone.com/enen/kb/alarm-siren-air/

---

The Alarm Siren Air is an external warning device with strobe light, for example as part of the burglar alarm. When triggered, the device emits an audible and visual alarm. The integrated tamper switch provides additional protection.

[**Datasheet Alarm Siren Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AlarmSirenAir_100312.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Select an installation location where the device is protected from extreme weather conditions and damage. Make sure that the cable outlet is covered by the unit after installation.

Connect the power supply (orange/white) using the supplied 3M jelly crimp connectors. The wires including insulation are inserted into the connector, then the connector is crimped with pliers.

To complete the installation, place the top cover on the base unit and secure it to the bottom with the two screws.

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for two minutes, then pairing mode is activated. The device can then be paired again.

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
| Online Status Alarm Siren Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | You will be notified via System Status or Could Mailer if the device is no longer available or offline. As this device and it's functionality are critical to safety, it is not possible to disable this setting for this device. | - | - | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - | - | - |
| Maximum audible alarm duration [s] | Maximum audible alarm duration (0 - 1800 seconds, 0 = no limit) | s | 0...1800 | 120 |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

---

## Documents

[**Datasheet Alarm Siren Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AlarmSirenAir_100312.pdf)

---