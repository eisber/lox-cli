# Room Comfort Sensor Air

Source: https://www.loxone.com/enen/kb/room-comfort-sensor-air/

---

The Room Comfort Sensor Air is used to measure temperature and humidity in rooms or similar applications. The openings in the housing ensure rapid response to changes in temperature or humidity.

[**Datasheet Room Comfort Sensor Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RoomComfortSensorAir_100264.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

The recommended mounting height for the indoor climate sensor is between 1.2 m (4 ft) - 1.5 m (5 ft) when mounted on the wall. The positioning of the sensor should be adapted to the application. (e.g. according to what area of a space is generally occupied)

Install the mounting frame at the desired location and connect the power supply (orange/white terminal). When using batteries, remove the orange/white terminal and insert the supplied AAA batteries.

The sensor is then snapped into the mounting frame. For free-standing use, install the supplied back cover instead of the mounting frame.

![100264 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100264 install.png)

The available functions differ depending on whether the device is battery operated or connected to an external power supply. This is determined during pairing. The device must therefore be paired in the same supply method in which it is to be operated later.
If the supply method is changed later, the device must be deleted from the configuration and paired again.

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Temperature | Provides the measured value of the current temperature | ° | -40...125 |
| Air humidity | Provides the measured value of the current air humidity | % | 0...100 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Room Comfort Sensor Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - | - | - |
| Transmission cycle | Request or transmit cycle for temperature and humidity (1-120 min, 0 = Off) | min | 0...120 | - |

---

## Safety Instructions

When connecting to an external power supply, the installation must be carried out by a qualified technician in accordance with all applicable regulations.

The installation requires a suitable enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Room Comfort Sensor Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RoomComfortSensorAir_100264.pdf)

---