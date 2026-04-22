# KNX Extension

Source: https://www.loxone.com/enen/kb/knx-extension/

---

The **KNX Extension** is designed to integrate devices with KNX interface.

In Loxone Config KNX sensors and actuators can be paired and configured, and then used in programming.

> **ℹ️ Note:** To address and configure the KNX devices the ETS software by the KNX Association and an external KNX gateway are required.

[**Datasheet KNX Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_KNXExtension_100322.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [KNX integration](#desc1)
- [Inputs, Outputs, Properties](#Diagnostic)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The KNX Extension is installed on a DIN rail in a suitable enclosure.

![100322 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100322 install.png)

Connect the power supply, Link communication to the Miniserver and the KNX data lines.

A separate KNX power supply is required to power the KNX bus.

The Extension starts after switching on the power supply, and the status LED will flash orange after a short time when the connection to the Miniserver is established.

**[Then follow the pairing procedure on the Link Interface.](https://www.loxone.com/help/link-interface#LinkPair)**

---

## KNX integration

To integrate KNX devices, the corresponding sensors and actuators must be created or paired in Loxone Config:

![10.5 KNX periphery](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/10.5 KNX periphery.png)

For additional information please visit the documentation on our website:

[**EIB/KNX Introduction**](https://www.loxone.com/enen/kb/eibknx-introduction/)

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status KNX Extension | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| No KNX Connection | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| KNX Sending not possible | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair an Extension with unknown serial number. This can only be used if there is only one Extension of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Extension. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Extension into the program. | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Own EIB Address | The EIB line's own physical address (x.x.x). E.g.: 1.1.250 | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted on a DIN rail in an electrical distribution enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet KNX Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_KNXExtension_100322.pdf)

---