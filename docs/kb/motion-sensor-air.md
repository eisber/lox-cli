# Motion Sensor Air

Source: https://www.loxone.com/enen/kb/motion-sensor-air/

---

The Loxone Motion Detector Air is used to detect motion and brightness in a room. It can be operated either by battery or external power supply.

Note: The Motion Sensor Air is no longer available and has been replaced by the [Presence Sensor Air](https://www.loxone.com/help/presence-sensor-air).

> **ℹ️ Note:** The Motion Sensor can also detect some pets. Although the sensitivity of the sensor can be reduced, there is no dedicated technology that excludes the detection of pets.

[**Datasheet Motion Sensor Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MotionSensorAir_100190.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Battery Replacement](#battery_change)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

![100190 100420 labeled back view](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100190 100420 labeled back view.png)

> **ℹ️ Note:** For motion to be detected accurately, correct placement of the Motion Sensor on the ceiling is essential. The following drawings can assist with this:

![motiondetector range1](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/motiondetector_range1.jpg)

![motiondetector range2](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/motiondetector_range2.jpg)

Install the mounting ring at the desired location. Insert the batteries or connect the 24VDC power supply. To complete the installation, attach the Motion Sensor to the mounting ring.

---

## Commissioning

In delivery state, pairing mode will be active after inserting the battery or after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply. The pairing button is located on the back of the device.

The available functions differ depending on whether the device is battery operated or connected to an external power supply. This is determined during pairing. The device must therefore be paired in the same supply method in which it is to be operated later.
If the supply method is changed later, the device must be deleted from the configuration and paired again.

---

## Battery Replacement

To replace the batteries, remove the unit from its base by briefly turning it anticlockwise. Remove the batteries from the back of the unit and insert new AA batteries. The device will restart and the status LED will flash green 3 times.

![100190 100420 battery](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100190 100420 battery.png)

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Motion | Input is active when motion is detected | - | 0/1 |
| Brightness | Provides the measured value of the current brightness | Lx | 0...188 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Motion Sensor Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - | - | - |
| Brightness transmission cycle | Brightness transmission cycle (0=OFF). | min | 0...120 | - |
| Overrun time motion | The 'Motion' input (Mo) will remain ON for this duration after detecting the last movement. The longer this overrun period, the more energy efficient the device can operate. If a motion sensor is used as part of a burglar alarm, then the overrun period will automatically be set to 3 seconds when the alarm is armed and the device reports to the Miniserver. | - | - | - |
| Sensitivity | Effects the sensitivity and thus also the distance over which motion can be sensed | - | - | - |

---

## Safety Instructions

When connecting to an external power supply, the installation must be carried out by a qualified technician in accordance with all applicable regulations.

Ensure that the device is protected from water.

---

## Documents

[**Datasheet Motion Sensor Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MotionSensorAir_100190.pdf)

---