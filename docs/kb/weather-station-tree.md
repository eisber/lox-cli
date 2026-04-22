# Weather Station Tree

Source: https://www.loxone.com/enen/kb/weather-station-tree/

---

The Weather Station Tree is used to capture data relevant for building automation such as wind speed, brightness, rain and temperature. By combining it with the [Loxone Weather Service](https://www.loxone.com/enen/kb/weather-service/), additional weather data and forecasts are integrated.

The rain sensor heater is permanently active and prevents rain detection due to condensation. It is also used to dry the sensor after it has rained.

[**Datasheet Weather Station Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WeatherStationTree_100246.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Storm warning threshold](#warning_velocity)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Mount the weather station in a location where wind, rain and brightness can be detected without interference.

![100246 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100246 install.jpg)

The device is mounted as shown either to a post with the enclosed accessories, or directly to a wall. Ensure exact horizontal alignment.

Air turbulence caused by buildings or objects should be taken into consideration, as this can falsify the measurement depending on the wind direction. A good location is about 1,5 m above the roof. For flat roofs, positioning in the middle of the roof is recommended.

Measuring the air temperature and brightness at the same location is not always suitable due to the direct sunlight and the subsequent internal heating of the weather station. The installation location should therefore be chosen according to the application.

The Weather Station is not suitable for installation near the coast, as the salty air can lead to corrosion of the rain sensor

Make sure the Weather Station cover is properly seated. Otherwise, the rain sensor functionality might be affected.

![100246 cabling](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100246 cabling.png)

Connect the power supply (orange/white terminal) and Tree communication wires (green/white terminals).

---

## Commissioning

Shortly after power-up, the status LED will blink orange if the wiring is correct and a connection to the Miniserver (Tree Extension is online) has been established.

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Storm warning threshold

The warning speed is set by the "Warning speed" value in the device's properties

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Brightness | Provides the measured value of the current brightness | lx | 0...188000 |
| Wind speed | Provides the maximum wind speed measured during the last five seconds and only if the value has changed by at least 1 km/h. | km/h | 9...145 |
| Temperature | Provides the measured value of the integrated temperature sensor | ° | ∞ |
| Rain | Input is active when rain is detected | - | 0/1 |
| Wind warning | Input is active when the set wind speed is exceeded | - | 0/1 |
| Sunshine | Input is active when sunshine is detected. Sunshine is determined by the measured brightness, the sun's elevation, and the sunshine threshold setting. | - | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Weather Station Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - |
| Storm Warning Threshold [km/h] | Windspeed above which the storm warning will be triggered | 0...150 | 35 |
| Sunshine Threshold | When the brightness exceeds the threshold calculated based on the sun angle, the sunshine input becomes active. | - | - |
| Transmission cycle | Specifies how often analog values from sensors are transmitted [s] 0 = Values are updated at least every 5 min, or if temperature deviates 0.2 °C, wind speed deviates 1 kph or brightness deviates by '3% if measured value is = 10 lx and = 100 lx' to the last transmitted value | 0...3600 | 300 |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is level.

---

## Documents

[**Datasheet Weather Station Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WeatherStationTree_100246.pdf)

---