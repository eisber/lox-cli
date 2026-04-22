# DI Extension

Source: https://www.loxone.com/enen/kb/di-extension/

---

The **DI Extension** features 20 digital inputs.

The inputs can also be used as frequency counters, to connect for example S0 meters or wind sensors that transmit pulses.

[**Datasheet DI Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_DIExtension_100283.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Wiring examples](#desc1)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The DI Extension is installed on a DIN rail in a suitable enclosure.

![100283 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100283 install.png)

Connect the power supply and Link communication to the Miniserver.

You can connect push buttons to the digital inputs, for example. When using the inputs as frequency counters, wind sensors or S0 meters can also be connected.

If the devices are connected to different power supplies, it is essential that the GNDs are interconnected.

The Extension starts after switching on the power supply, and the status LED will flash orange after a short time when the connection to the Miniserver is established.

**[Then follow the pairing procedure on the Link Interface.](https://www.loxone.com/help/link-interface#LinkPair)**

---

## Wiring examples

[Connect sensors with open collector outputs](https://www.loxone.com/enen/kb/wiring-accessories/#CONNECTING%20OPEN%20COLLECTOR%20OUTPUTS)

[Connecting a 230V motion sensor to a digital input](https://www.loxone.com/enen/kb/wiring-accessories/#CONNECTING%20A%20230V%20MOTION%20SENSOR)

---

## Sensors

| Summary | Unit | Value Range |
| --- | --- | --- |
| Input 1 | Digital | 0/1 |
| Input 2 | Digital | 0/1 |
| Input 3 | Digital | 0/1 |
| Input 4 | Digital | 0/1 |
| Input 5 | Digital | 0/1 |
| Input 6 | Digital | 0/1 |
| Input 7 | Digital | 0/1 |
| Input 8 | Digital | 0/1 |
| Input 9 | Digital | 0/1 |
| Input 10 | Digital | 0/1 |
| Input 11 | Digital | 0/1 |
| Input 12 | Digital | 0/1 |
| Input 13 | Digital | 0/1 |
| Input 14 | Digital | 0/1 |
| Input 15 | Digital | 0/1 |
| Input 16 | Digital | 0/1 |
| Input 17 | Digital | 0/1 |
| Input 18 | Digital | 0/1 |
| Input 19 | Digital | 0/1 |
| Input 20 | Digital | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status DI Extension | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

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

[**Datasheet DI Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_DIExtension_100283.pdf)

---