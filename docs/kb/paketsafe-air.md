# Paketsafe Air

Source: https://www.loxone.com/enen/kb/paketsafe-air/

---

The Paketsafe allows the safe deposit of packages delivered in your absence. Via the connection to the Loxone system you will be notified about newly delivered packages.

The Paketsafe Air is no longer available.

[**Datasheet Paketsafe**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_PaketsafePlus_100444.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Programming](#Programmierung)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

We recommend installing the Paketsafe at a location that is easily visible and accessible for the carrier. Please also consider the space required by the filled bag. Please follow the installation instructions, which you will find at the bottom of this page.

![100444 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100444 install.png)

Finally, connect the power supply (orange/white terminal) or insert the two AA batteries.

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.

The available functions differ depending on whether the device is battery operated or connected to an external power supply. This is determined during pairing. The device must therefore be paired in the same supply method in which it is to be operated later.
If the supply method is changed later, the device must be deleted from the configuration and paired again.

---

## Programming

For programming the device, the Packetsafe function block is used. Drag the device from the peripheral tree to the programming page to create the block.

---

## Actuators

| Summary | Unit |
| --- | --- |
| API Connector | Text |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Paketsafe Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Show inputs | Display inputs of the Paketsafe to be able to use them in configuration independent of the function block. | - |

---

## Safety Instructions

When connecting to an external power supply, the installation must be carried out by a qualified technician in accordance with all applicable regulations.

---

## Documents

[**Mounting Instructions Paketsafe**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Instructions_PaketsafePlus_100444.pdf)

[**Datasheet Paketsafe**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_PaketsafePlus_100444.pdf)

---