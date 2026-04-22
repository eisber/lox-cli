# Window Handle Air

Source: https://www.loxone.com/enen/kb/window-handle-air/

---

The Loxone Window Handle Air detects whether a window is open, closed or tilted based on the three positions of the handle. In addition, an internal sensor detects vibrations and can thus warn of a forced entry.

[**Datasheet Window Handle Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WindowHandleAir_100177.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Note](#notes)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Remove the plastic cover from the window handle by pulling it towards the handle. Adjust the length of the square pin to the window. Once the correct length has been determined, secure the square pin with the set screw located at the bottom of the handle. Attach the Window Handle Air to the window frame using the supplied screws. Then insert the batteries.

![100177 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100177 install.png)

> **ℹ️ Note:** The detection of the handle position is based on gravity. Therefore, the device can only be used on conventional windows that are aligned vertically/straight when closed. The detection of the handle position does not work on slanted windows, e.g. on roof windows.

> **ℹ️ Note:** On windows for handles with 12mm cams, a cam slip-on ring from 10mm to 12mm must be used so that the housing of the window handle is positioned correctly and the function is ensured.

---

## Commissioning

In delivery state, pairing mode will be active after inserting the battery. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds immediately after inserting the battery.

---

## Note

The handle itself is not removable. The mounting screws are 43mm apart, according to DIN 18267. The window handle Air complies with V ENV 1627 to V ENV 1630, so it can be used throughout the EU.

> **ℹ️ Note:** Please note that correct detection of the handle's position is only possible when the handle is mounted and the "Closed position" setting is correct.

---

## Sensors

| Summary | Description | Value Range |
| --- | --- | --- |
| Position | Analogue input to display the position. 1=closed 2=tilted 3=open 0=device offline | ∞ |
| Closed | Input is active when handle is in closed position | 0/1 |
| Tilted | Input is active when handle is in tilted position | 0/1 |
| Alarm | Input is active when vibration is detected (only when in Closed position). | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Window Handle Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | You will be notified via System Status or Could Mailer if the device is no longer available or offline. As this device and it's functionality are critical to safety, it is not possible to disable this setting for this device. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Closed position | Position of handle when window is closed. | - |
| Tilt Before Open | Default tilt mode - Tilt after open. If checked, tilt mode of window handle is set to tilt before open configuration. | - |

---

## Safety Instructions

Ensure that the device is protected from water.

---

## Documents

[**Datasheet Window Handle Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WindowHandleAir_100177.pdf)

---