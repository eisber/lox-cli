# Smoke Detector Air

Source: https://www.loxone.com/enen/kb/smoke-detector-air/

---

The Loxone Smoke Detector Air is a photo-electric smoke detector. When smoke is detected, the device emits a loud alarm signal. The alarm is also transmitted to the Miniserver. This allows for further steps to be initiated via programming.

[**Datasheet and User Manual Smoke Detector Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_SmokeDetectorAir_100142.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Operating and alarm signals](#signals)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

The correct amount and position of smoke detectors is essential to achieve effective protection. Local code and regulations must be observed.

Start by mounting the base plate. Then insert the 9V battery and plug it in. Place the detector on the base plate and lock the device with a short clockwise turn. (This is only possible when the battery is inserted)

![100142 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100142 install.png)

---

## Commissioning

In delivery state, pairing mode will be active after inserting the battery. This is indicated by the status LED (light guiding key) flashing red every second

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply. The pairing button is located on the circuit board on the back of the device.

![100142 button](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100142 button.jpg)

---

## Operating and alarm signals

| Description | Signal tone | Red LED |
| --- | --- | --- |
| Normal operation | No sound | flashes every 40 seconds |
| Alarm state | Loud intermittent sound in 0.5 sec. intervals | 2 flashes per second |
| Malfunction / Contamination | 3 short beeps every 40 seconds | LED off |
| Low battery indicator | Short beep every 40 seconds | flashes every 40 seconds simultaneously with the signal tone |
| Alarm muting | No sound | flashes every 10 seconds |
| Alarm Memory active Alarm was active within the last 24h | No sound | flashes 3 times every 43 seconds |
| Functional test | Loud intermittent sound | flashes twice every second as long as the light guide button is pressed |
| Pairing mode | No sound | flashes every second |

---

## Sensors

| Summary | Description | Value Range |
| --- | --- | --- |
| Alarm | Input is active when smoke is detected | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Smoke Detector Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
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

> **ℹ️ Note:** Smoke detectors should be checked once a year.

Remove dust and insects from the smoke detector grille if necessary. Any smoke that may occur must be able to enter the smoke detector unhindered.

The smoke detector has a button for test purposes. Press the light guide button on the top until the smoke detector sounds the audible alarm. Please do not forget to activate the alarm suppression before doing so.

---

## Documents

[**Datasheet and User Manual Smoke Detector Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_SmokeDetectorAir_100142.pdf)

---