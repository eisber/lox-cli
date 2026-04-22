# Water Sensor Air

Source: https://www.loxone.com/enen/kb/water-sensor-air/

---

The Loxone Water Sensor Air is a wireless sensor with two contacts on the bottom. The sensor activates when the contacts come into contact with a conductive liquid. This allows monitoring of indoor water intrusion or leakage.

**[Datasheet Water Sensor Air](https://pim.loxone.com/datasheet/100668-water-sensor-air)**

[**Datasheet Water Sensor Air Gen.1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WatersensorAir_100211.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Assembly](#assembly)
- [Battery Replacement](#battery_change)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

In delivery state, pairing mode will be active after removing the battery pull tab. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.

![WaterSensorAir conn](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/WaterSensorAir_conn.png)

---

## Assembly

Position the Water Sensor Air at the desired location.

> **ℹ️ Note:** If the sensor was submerged in water, it will no longer operate and must be replaced by a new sensor. When installing the sensor, make sure that it is not subjected to water in everyday use, for example when cleaning the floor.

> **ℹ️ Note:** When mounted to metallic surfaces, the range may be significantly reduced.

---

## Battery Replacement

Use a screwdriver to open the cover on the back and remove the two AA batteries.

After inserting the new batteries, the device will automatically restart, and the status LED will blink green three times.

![WaterSensorAir battery](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/WaterSensorAir_battery.png)

Gen.1: Use the white button to eject the battery. Insert a new CR2032 lithium battery and ensure it is completely pushed in for proper button operation.

> **ℹ️ Note:** Devices with serial number 504F94FFFE-B/C..... do not flash their LED after removing and reinserting the battery. For such devices, after removing the battery, either press a button or wait for a minute to display the status after reinserting the battery.

---

## Sensors

| Summary | Description | Value Range |
| --- | --- | --- |
| Input 1 | To use the pairing button as Input | 0/1 |
| Contact | Input is active when water is detected | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Water Sensor Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | You will be notified via System Status or Could Mailer if the device is no longer available or offline. As this device and it's functionality are critical to safety, it is not possible to disable this setting for this device. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Start Value ON | If checked, the digital input value will be ON when starting the program. | - |
| Display Error Output | If checked, error output will be displayed in 2nd row. | - |

---

## Safety Instructions

Ensure that the device is protected from water.

---

## Documents

**[Datasheet Water Sensor Air](https://pim.loxone.com/datasheet/100668-water-sensor-air)**

[**Datasheet Water Sensor Air Gen.1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WatersensorAir_100211.pdf)

---