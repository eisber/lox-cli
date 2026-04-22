# AO Extension

Source: https://www.loxone.com/enen/kb/ao-extension/

---

The **AO Extension** features 4 analogue 0-10V outputs.

[**Datasheet AO Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AOExtension_100382.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The AO Extension is installed on a DIN rail in a suitable enclosure.

![100382 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100382 install.png)

Connect the power supply and Link communication to the Miniserver.

The analogue outputs are used to connect devices that are controlled with a 0-10V signal, e.g. 0-10V actuators.

If the devices are connected to different power supplies, it is essential that the GNDs are interconnected.

The Extension starts after switching on the power supply, and the status LED will flash orange after a short time when the connection to the Miniserver is established.

**[Then follow the pairing procedure on the Link Interface.](https://www.loxone.com/help/link-interface#LinkPair)**

---

## Actuators

| Summary | Value Range |
| --- | --- |
| Voltage 1 | ∞ |
| Voltage 2 | ∞ |
| Voltage 3 | ∞ |
| Voltage 4 | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status AO Extension | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair an Extension with unknown serial number. This can only be used if there is only one Extension of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Extension. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Extension into the program. | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted on a DIN rail in an electrical distribution enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet AO Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AOExtension_100382.pdf)

---