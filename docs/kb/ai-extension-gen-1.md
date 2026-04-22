# AI Extension Gen. 1

Source: https://www.loxone.com/enen/kb/ai-extension-gen-1/

---

The **AI Extension Gen. 1** features 4 analog 0-10V inputs. These can also be used as digital inputs.

[**Datasheet AI Extension Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AIExtension_100381.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Connecting Analog Inputs](#connect-ai)
- [Additional information](#desc1)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The AI Extension is installed on a DIN rail in a suitable enclosure.

![100381 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100381 install.png)

Connect the power supply and Link communication to the Miniserver.

Connect devices that provide a 0-10V signal to the analogue inputs, e.g. 0-10V temperature sensors. When using the inputs as digital inputs, 24VDC can be connected to the inputs via retractive switches for example.

If the devices are connected to different power supplies, it is essential that the GNDs are interconnected.

The Extension starts after switching on the power supply, and the status LED will flash orange after a short time when the connection to the Miniserver is established.

**[Then follow the pairing procedure on the Link Interface.](https://www.loxone.com/help/link-interface#LinkPair)**

---

## Connecting Analog Inputs

**0-10V transmitter with 2 outputs, common power supply:**

![conex AnalogInUni 2x0 10 1xpsu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/conex_AnalogInUni_2x0-10_1xpsu.png)

**0-10V transmitter with 2 outputs, separate power supply:**

![conex AnalogInUni 2x0 10 2xpsu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/conex_AnalogInUni_2x0-10_2xpsu.png)

---

## Additional information

[Connect sensors with open collector outputs](https://www.loxone.com/enen/kb/wiring-accessories/#CONNECTING%20OPEN%20COLLECTOR%20OUTPUTS)

---

## Sensors

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
| Online Status AI Extension Gen. 1 | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

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

[**Datasheet AI Extension Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AIExtension_100381.pdf)

---