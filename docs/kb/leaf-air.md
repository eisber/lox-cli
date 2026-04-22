# Leaf Air

Source: https://www.loxone.com/enen/kb/leaf-air/

---

The Leaf Air is a decentralized ventilation unit with heat recovery that is controlled via Loxone Air technology. Several units are combined, providing room ventilation. More information about the operating method and a configuration tool for the recommended number of units can be found on the website of our cooperation partner: [Leaf Ventilation](https://www.leaf-ventilation.de/produkte/dezentrales-lueftungsgeraet/).

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Maintenance, filter replacement](#Wartung, Filterwechsel)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

[Install the device according to the manufacturer's installation and operating instructions.](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Leaf1AirTree_Assembly_DE_EN.pdf)

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for two minutes, then pairing mode is activated for 30 minutes.

---

## Maintenance, filter replacement

The fan filter must be replaced or cleaned at regular intervals based on the maintenance manual. The filter change interval can be defined in the Leaf 1 properties. This setting is 3 months (2200h) by default. The actual run-time of the fan is considered. You will be notified of an upcoming maintenance in the user interface.

---

## Actuators

| Summary | Unit |
| --- | --- |
| API Connector | Text |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Leaf Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Filterwarnung | Digital | 0/1 |
| Fan error | Digital | 0/1 |
| Aperture Error | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - | - | - |
| Filter Change Interval | After this time (in hours) the filter warning becomes active. | h | ∞ | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Please note the safety instructions in the following installation and operating manual.

---

## Documents

[**Installation and operating instructions with technical data**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Leaf1AirTree_Assembly_DE_EN.pdf)

[Maintenance manual](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Leaf1AirTree_Maintenance_DE_EN.pdf)

---