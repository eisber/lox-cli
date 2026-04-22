# Presence Sensor Air

Source: https://www.loxone.com/enen/kb/presence-sensor-air/

---

The Loxone Presence Sensor Air is used to detect presence, motion and brightness in a room. The device can either be powered by batteries or external power supply.

In addition to the passive infrared sensor, it also features an acoustic presence detection, which is only available with external power supply.

> **ℹ️ Note:** The Presence Sensor can also detect some pets. Although the sensitivity of the sensor can be reduced, there is no dedicated technology that excludes the detection of pets.

[**Datasheet Presence Sensor Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_PresenceSensorAir_100420.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Set volume thresholds](#desc1)
- [Usage and Functionality](#desc2)
- [Battery Replacement](#battery_change)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

![100190 100420 labeled back view](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100190 100420 labeled back view.png)

> **ℹ️ Note:** For motion to be detected accurately, correct placement of the Presence Sensor on the ceiling is essential. The following drawings can assist with this:

> **ℹ️ Note:** Sensitivity does not create a strict cutoff for the detection range; instead, it adjusts internal parameters that influence the range. To create blind spots or define specific detection zones, please use the stickers provided in the package.

![motiondetector range1](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/motiondetector_range1.jpg)

![motiondetector range2](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/motiondetector_range2.jpg)

Install the mounting ring at the desired location. Insert the batteries or connect the 24VDC power supply. To complete the installation, attach the Presence Sensor to the mounting ring.

---

## Commissioning

In delivery state, pairing mode will be active after inserting the battery or after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply. The pairing button is located on the back of the device.

The available functions differ depending on whether the device is battery operated or connected to an external power supply. This is determined during pairing. The device must therefore be paired in the same supply method in which it is to be operated later.
If the supply method is changed later, the device must be deleted from the configuration and paired again.

---

## Set volume thresholds

From the Presence Sensor's properties you can open a diagram for setting the volume thresholds (Only available if the sensor is connected with 24V). Here you set the ambient volume thresholds for presence detection and volume alarm:

![PresenceSensor SetVolumeThresholds](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PresenceSensor_SetVolumeThresholds.png)

A graph is drawn using the current ambient volume of the selected Presence Sensor. Use the graph to adjust the volume thresholds for presence and sound level alarm to the specific location. When doing so, also take into account intermittent sounds that could trigger presence or an alarm even when no one is present. For example, dishwashers (keeping presence active) or high levels of noise from outside such as construction sites, heavy traffic or the like (volume alarm).

---

## Usage and Functionality

The presence input of the sensor can also be used for blocks with motion sensor (Mo) inputs (e.g. Lighting Controller). However, the overrun time is then no longer specified in the blocks but via the Presence Sensor. The overrun time (Moet) of the blocks is set to 0.1s when using the presence input. As a result, lighting, for example, remains active for as long as the sensor indicates presence.

In a quiet setting, it may happen that no presence is detected although a person is in fact present. For example, if you do not move for a long time when reading and no sounds occur. As a result, the lighting is switched off. Within 10 seconds of the end of presence, it is possible to reactivate the presence input using sound alone. You could shout "Hey", for example, and switch the lighting back on. It is not necessary to move during this time.

---

## Battery Replacement

To replace the batteries, remove the unit from its base by briefly turning it counterclockwise. Remove the batteries from the back of the unit and insert new AA batteries.

![100190 100420 battery](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100190 100420 battery.png)

Shortly after that, the device flashes green 3 times once the connection to the Miniserver has been successfully established.

> **ℹ️ Note:** Devices with serial number 504F94FFFE-B/C..... do not flash their LED after removing and reinserting the battery. For such devices, after removing the battery, either press a button or wait for a minute to display the status after reinserting the battery.

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Volume Minimum | Provides the minimum volume periodically based on the value set in "Volume Transmission Cycle". Not available when battery operated. | - | 20...2000 |
| Volume Maximum | Provides the maximum volume periodically based on the value set in "Volume Transmission Cycle". Not available when battery operated. | - | 20...2000 |
| Brightness | Measured value of the current brightness | Lx | 0...83000 |
| Motion | Input is active when motion is detected | - | 0/1 |
| Presence | Input is active as long as presence is detected. Motion activates the input, volume and motion keep it active. Not available when battery operated. | - | 0/1 |
| Sound Level Alarm | Input provides a pulse when the volume exceeds the threshold for the sound level alarm. 1 pulse per 10s. Not available when battery operated. | - | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Presence Sensor Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - | - | - |
| Overrun Time Presence | Overrun time for presence and motion detector input | s | 2...60000 | - |
| Brightness transmission cycle | Brightness transmission cycle (0=OFF). | min | 0...120 | - |
| Sensitivity | Effects the sensitivity and thus also the distance over which motion can be sensed | - | - | - |

---

## Safety Instructions

When connecting to an external power supply, the installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is protected from water.

---

## Documents

[**Datasheet Presence Sensor Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_PresenceSensorAir_100420.pdf)

---