# IR Extension

Source: https://www.loxone.com/enen/kb/ir-extension/

---

The **IR Extension** is designed to control up to 8 IR modules and is used to control IR capable devices like TVs, AV receivers, projectors, air conditioners, etc.

The Loxone IR Extension and the IR Modules are no longer available. Use the IR Control Air instead.

## Table of Contents
- [Commissioning](#Commissioning)
- [Adding an IR Module](#desc1)
- [Inputs, Outputs, Properties](#Diagnostic)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The IR Extension is installed on a DIN rail in a suitable enclosure.

![100022 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100022 install.png)

![100023 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100023 install.png)

Connect the power supply, Link communication to the Miniserver and the data lines connecting the IR Modules.

The IR modules are connected via RJ45 plugs and daisy-chained as shown in the diagram.
A CAT5/6/7 cable is used for wiring, the max. cable length is 500m.
The last IR module must be terminated by setting the jumper to pin 2/3.

2 IR transmitters can be connected to each IR module (3.5 mm connector). If only one transmitter is connected, it must be plugged into the left socket.
The IR receiver is integrated in the IR module.

The Extension starts after switching on the power supply, and the status LED will flash orange after a short time when the connection to the Miniserver is established.

**[Then follow the pairing procedure on the Link Interface.](https://www.loxone.com/help/link-interface#LinkPair)**

---

## Adding an IR Module

To search for IR Modules, first click on the IR Extension in Loxone Config, and then activate **IR Device Search**.

The search results at the bottom of the Loxone Config, will list all connected IR Modules.

Now select a Module, enter a name and then click on **Create device.**

**To apply the changes, save the program in the Miniserver.**

Now the added Modules are ready for use and available in Loxone Config.

Additional steps to create IR sensors (receiver) and actuators (transmitter), as well as for creating and learning remote controls is described at [**IR Control Air**](https://www.loxone.com/enen/kb/ir-control-air/).

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status IR Extension | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair an Extension with unknown serial number. This can only be used if there is only one Extension of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Extension. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Extension into the program. | - |
| Maximum cable length (m) | Maximum cable length in metres to the last IR-module. | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted on a DIN rail in an electrical distribution enclosure to ensure protection against contact, water and dust.

---

## Documents

---