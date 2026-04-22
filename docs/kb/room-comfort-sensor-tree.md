# Room Comfort Sensor Tree

Source: https://www.loxone.com/enen/kb/room-comfort-sensor-tree/

---

The Room Comfort Sensor Tree is used to measure temperature, humidity and CO2 in rooms or similar applications. The openings in the housing ensure rapid response to changes in measured values.

[**Datasheet Room Comfort Sensor Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RoomComfortSensorTree_100276.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Calibrate CO2 sensor](#calco2)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

The recommended mounting height for the indoor climate sensor is between 1.2 m (4 ft) - 1.5 m (5 ft) when mounted on the wall. The positioning of the sensor should be adapted to the application. (e.g. according to what area of a space is generally occupied)

Install the mounting frame at the desired location.

Connect the power supply (orange/white terminal) and Tree communication wires (green/white terminals). Shortly after power-up, the status LED will blink orange if the wiring is correct (connection to Tree Extension and Miniserver is established).

Mount the device by snapping it onto the mounting frame.

![100276 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100276 install.png)

---

## Commissioning

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Calibrate CO2 sensor

Manual calibration of the CO2 sensor is not necessary, but can be performed for special applications.
To do this, first ensure that the CO2 value at the measuring point is as low and constant as possible.
Now measure the CO2 value at the measuring point with a suitable measuring device.
Immediately afterwards, assign the measured value to the sensor via a [device command](https://www.loxone.com/help/device-command/) or [webservice](https://www.loxone.com/enen/kb/web-services) command, in the following example 450ppm:

miniserver/dev/sys/wsdevice/devicename-or-serialnr/CalCO2/450

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Temperature | Provides the air temperature. | ° | -40...125 |
| Humidity | Provides the air humidity. | % | 0...100 |
| CO2 | Provides the CO2 value. | ppm | 400...10000 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Room Comfort Sensor Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - |
| Transmission cycle | Specifies how often analog values from sensors are transmitted (only 2 s steps are possible) [s] 0 = Values are updated at least every 60 min, or if temperature deviates by 0.2 °C, humidity deviates by 0.2% or CO2 deviates by 20 ppm to the last transmitted value | 0...3600 | 900 |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

The installation requires a suitable enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Room Comfort Sensor Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RoomComfortSensorTree_100276.pdf)

---