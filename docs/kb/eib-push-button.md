# EIB push-button

Source: https://www.loxone.com/enen/kb/eib-push-button/

---

On/Off push-button with trigger, central On/Off function

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [EIB/KNX interface](#EIB/KNX)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tg | Toggle | Toggles between on and off. | 0/1 |
| Off | Off | Switches output (O) off. Permanent 1 = Locks function block. The name of the connected sensor is used in the user interface. | 0/1 |
| On | On | Switches output (O) on. | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Tg), (Off), (On) when On. (e.g Child lock, cleaning) | 0/1 |
| S | State | This input can forward the status of a EIB-actuator to the output without triggering an action on the output. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O | Output | Connect this output to an EIB-switching actuator (group address switching) | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |

---

## EIB/KNX interface

The Loxone Miniserver Gen. 1 has a full-fledged EIB interface for a wide range of applications.

EIB sensors: Use of EIB sensors instead of classic buttons in a Loxone installation

User interface: Supplementing existing EIB installations for user interface

Logic: Supplementing existing EIB installations for logic