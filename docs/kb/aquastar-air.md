# Aquastar Air

Source: https://www.loxone.com/enen/kb/aquastar-air/

---

The Aquastar Air allows automatic backwashing in swimming pools. Additionally, connections for valves, pressure sensors and temperature sensors are integrated.

[**Installation and operating instructions with technical data**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/InstructionManual_AquastarAirDE_EN.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Water Level](#level)
- [1-wire Interface](#1winterface)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Follow the installation and operating instructions to install the device. These contain important information and notes on the valve and Valve Actuator.

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.
The pairing button is located on the circuit board, see [Installation manual](#Documents) page 10.

---

## Water Level

The water level in the pool can be determined with the help of a pressure sensor.
For this purpose, a 0-0.3 bar pressure sensor is installed in a location where it is exposed to the pressure of the water inside the pool. For example, the main drain pipe can be used for this.
Not suitable are pipes in the filter circuit, or other pipes with changing pressure conditions.

This pressure sensor is connected to the 0-10V input (Ps1) of the Aquastar Air.

---

## 1-wire Interface

1-Wire sensors can be connected to the 1-Wire interface of the device.
More about this in the **[Documentation of the 1-Wire Extension](https://www.loxone.com/help/1-wire#1wconnect)**, the properties of the interface are identical.

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Pressure sensor 1 | For connecting a 0-10V pressure sensor. 0-6 bar: System pressure 0-0,3 bar: Water level Transmission cycle: 15 minutes | V | 0...10 |
| Pressure sensor 2 | For connecting a 0-5V (0,5-4,5V) pressure sensor. 0-6 bar: System pressure Transmission cycle: 15 minutes | V | 0...5 |

---

## Actuators

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Relay 1 | Relay | p | 0/1 |
| Relay 2 | Relay | p | 0/1 |
| Relay 3 | Relay | p | 0/1 |
| API Connector | Text | - |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Aquastar Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| System temperature | Provides the internal device temperature. This is often the temperature of the CPU or another location in the device. | ° | ∞ |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Please note the safety instructions in the following installation and operating manual.

---

## Documents

[**Installation and operating instructions with technical data**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/InstructionManual_AquastarAirDE_EN.pdf)

---