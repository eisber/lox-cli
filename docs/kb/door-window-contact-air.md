# Door and Window Contact Air

Source: https://www.loxone.com/enen/kb/door-window-contact-air/

---

The Loxone Door & Window Contact Air is a wireless reed-contact sensor that uses a magnet to reliably monitor whether doors or windows are open or closed.

**[Datasheet Door & Window Contact Air White / ](https://pim.loxone.com/datasheet/100666-door-window-contact-air-white)****[Anthracite](https://pim.loxone.com/datasheet/100667-door-window-contact-air-anthracite)**

[**Datasheet Door & Window Contact Air Gen.1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_DoorWindowContactAir_100210.pdf)

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

![DoorWindowContactAir conn](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/DoorWindowContactAir_conn.png)

---

## Assembly

Attach the included adhesive strip to the back of the device and mount it to a door or window.

Mount the magnet parallel to the Door & Window Contact Air. For reliable detection, the maximum distance must not exceed 5 mm. Any other alignment will reduce the detection range.

![DoorWindowContactAir mount](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/DoorWindowContactAir_mount.png)

> **ℹ️ Note:** When mounted to metallic surfaces, the range may be significantly reduced.

---

## Battery Replacement

Carefully remove the cover by hand and take out the two AA batteries.

After inserting the new batteries, the device will automatically restart, and the status LED will blink green three times.

![DoorWindowContactAir battery](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/DoorWindowContactAir_battery.png)

Gen.1: Use the white button to eject the battery. Insert a new CR2032 lithium battery and ensure it is completely pushed in for proper button operation.

> **ℹ️ Note:** Devices with serial number 504F94FFFE-B/C..... do not flash their LED after removing and reinserting the battery. For such devices, after removing the battery, either press a button or wait for a minute to display the status after reinserting the battery.

---

## Sensors

| Summary | Description | Value Range |
| --- | --- | --- |
| Input 1 | To use the pairing button as Input | 0/1 |
| Contact | Input is active when magnet is detected (e.g. door closed) | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Door and Window Contact Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
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

**[Datasheet Door & Window Contact Air White / ](https://pim.loxone.com/datasheet/100666-door-window-contact-air-white)****[Anthracite](https://pim.loxone.com/datasheet/100667-door-window-contact-air-anthracite)**

[**Datasheet Door & Window Contact Air Gen.1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_DoorWindowContactAir_100210.pdf)

---