# Weather Station Air

Source: https://www.loxone.com/enen/kb/weather-station-air/

---

The Weather Station Air is used to capture data relevant for building automation such as wind speed, brightness, rain and temperature. By combining it with the [Loxone Weather Service](https://www.loxone.com/enen/kb/weather-service/), additional weather data and forecasts are integrated. It can be powered either by AA batteries or external power supply.

The rain sensor heater is permanently active and prevents rain detection due to condensation. It is also used to dry the sensor after it has rained. When battery powered, however, this heater is not activated due to the power consumption. Thus, during battery operation, the rain sensor can already react to condensation.

[**Datasheet Weather Station Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WeatherStationAir_100245.pdf)

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

![100245 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100245 install.jpg)

The device is mounted as shown either to a post with the enclosed accessories, or directly to a wall. Ensure exact horizontal alignment.

Air turbulence caused by buildings or objects should be taken into consideration, as this can falsify the measurement depending on the wind direction. A good location is about 1,5 m above the roof. For flat roofs, positioning in the middle of the roof is recommended.

Measuring the air temperature and brightness at the same location is not always suitable due to the direct sunlight and the subsequent internal heating of the weather station. The installation location should therefore be chosen according to the application.

The Weather Station is not suitable for installation near the coast, as the salty air can lead to corrosion of the rain sensor

Make sure the Weather Station cover is properly seated. Otherwise, the rain sensor functionality might be affected.

![100245 cabling](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100245 cabling.png)

When not using batteries, connect the power supply (orange/white).

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply. The pairing button is located under the cover of the device, see image above.

The available functions differ depending on whether the device is battery operated or connected to an external power supply. This is determined during pairing. The device must therefore be paired in the same supply method in which it is to be operated later.
If the supply method is changed later, the device must be deleted from the configuration and paired again.

---

## Storm warning threshold

When powered by 24V, the warning speed is set by the "Warning speed" value in the device's properties. When battery operated, the setting is made via the potentiometer on the weather station according to the following table:

> **ℹ️ Note:** 25 / 15.5

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Brightness | Provides the measured value of the current brightness | lx | 0...188000 |
| Wind speed | 24V supply: Provides the maximum wind speed measured during the last five seconds and only if the value has changed by at least 1 km/h. Battery operation: Provides the maximum wind speed of the last 1.2 seconds in the set transmission cycle. For storm protection, the "Storm warning" input must be used when battery operated. | km/h | 9...145 |
| Temperature | Provides the measured value of the integrated temperature sensor | ° | -40...125 |
| Rain | Input is active when rain is detected | - | 0/1 |
| Wind warning | Input is active when the set wind speed is exceeded | - | 0/1 |
| Sunshine | Input is active when sunshine is detected. Sunshine is determined by the measured brightness, the sun's elevation, and the sunshine threshold setting. | - | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Weather Station Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - | - | - |
| Transmission cycle for battery operation | Energy Management for battery operation: The longer the transmission cycle for analogue values, the longer the battery lifetime. For devices powered by 24V the transmission is dynamically done based on the data. Digital inputs will always be updated instantly, even when a device is under battery operation. | min | 15...60 | - |
| Storm Warning Threshold [km/h] | Wind speed above which the storm warning is triggered. WARNING: This setting is only used if the device is powered with 24V. If the device is battery powered this parameter is set using a potentiometer on the weather station! | - | 0...150 | 35 |
| Sunshine Threshold | When the brightness exceeds the threshold calculated based on the sun angle, the sunshine input becomes active. | - | - | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is level.

---

## Documents

[**Datasheet Weather Station Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WeatherStationAir_100245.pdf)

---