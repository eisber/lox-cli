# RS232 Extension

Source: https://www.loxone.com/enen/kb/rs232-extension/

---

The **RS232 Extension** is designed to integrate devices with RS232 interface.

From the [Loxone Library](https://library.loxone.com/) suitable [templates](https://www.loxone.com/help/templates) for the integration of devices can be imported.

For additional information on how to integrate devices without template, please visit:
[Communication with RS232/485](https://www.loxone.com/enen/kb/communication-with-rs232485/)

The Extension does not support software flow control.

[**Datasheet RS232 Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RS232Extension_100013.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Diagnostic)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The RS232 Extension is installed on a DIN rail in a suitable enclosure.

![100013 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100013 install.png)

Connect the power supply and Link communication to the Miniserver.

Connect a device with RS232 interface as shown in the example above.

The Extension starts after switching on the power supply, and the status LED will flash orange after a short time when the connection to the Miniserver is established.

**[Then follow the pairing procedure on the Link Interface.](https://www.loxone.com/help/link-interface#LinkPair)**

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status RS232 Extension | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair an Extension with unknown serial number. This can only be used if there is only one Extension of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Extension. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Extension into the program. | - | - | - |
| Serial Protocol | Selection of the protocol used to decode the data and if available, automatic polling for the device. | - | - | - |
| Polling cycle | Polling cycle for the selected protocol 0 means that no automatic polling will take place. If no protocol is selected this value is ignored. | s | 0...3600 | -1 |
| Protocol Data | Protocol data depends on the selected protocol. For Kostal a comma separated list of the addresses. If no protocol is selected this value is ignored. | - | - | - |
| Baud Rate | Baud rate in bits per second for the serial connection | Bit/s | 0...2147483647 | 9600 |
| Number of data bits | Number of data bits of the serial connection. The extension supports only 8 data bits. | - | 8...8 | 8 |
| Stop Bits | Number of stop bits used (1-2) for the serial communication | - | 1...2 | 1 |
| Parity | Parity for serial connection | - | - | - |
| Pause | Sets the pause between packets during transmission. | s | 0...1 | 0.01 |
| End character | Identifier for the end of a received serial data package. When received the data is passed on to the Miniserver for processing. Enter the identifier using hex. (e.g. 0x0A) | - | - | - |
| Checksum | Checksum or frame structure for this connection | - | - | - |
| Positive acknowledge | Reply for a positive acknowledgement of received block e.g.: 0x06 | - | - | - |
| Negative acknowledge | Reply for a negative acknowledgement of received block e.g.: 0x15 | - | - | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted on a DIN rail in an electrical distribution enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet RS232 Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RS232Extension_100013.pdf)

---