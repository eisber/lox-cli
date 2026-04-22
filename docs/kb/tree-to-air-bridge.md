# Tree to Air Bridge

Source: https://www.loxone.com/enen/kb/tree-to-air-bridge/

---

The Tree to Air Bridge is used to integrate various devices that are equipped with the Loxone Air wireless technology. It is connected via the Tree interface and features a compact design with integrated antenna. 128 Air devices can be paired to the Tree to Air Bridge.

It can be used, for example, in areas where the wireless range of an Air Base Extension is no longer sufficient, but where connection to a Tree Branch is possible.

[**Datasheet Tree to Air Bridge**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TreetoAirBridge_100451.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Pairing Air Devices](#desc1)
- [Tree To Air Bridge Limitation](#desc2)
- [Inputs, Outputs, Properties](#Diagnostic)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The Tree to Air Bridge is installed in a suitable enclosure.

![100451 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100451 install.png)

> **ℹ️ Note:** The Miniserver (no matter which generation or model is used) and Air signals can negatively influence each other when in close proximity. Therefore, a distance of 2 division / breaker units should be maintained between a Miniserver and an Air Base.

Connect the power supply and Tree communication wires.

> **ℹ️ Note:** For best signal performance, make sure that the device is not in close range of metal objects.

The Tree to Air Bridge starts after switching on the power supply, and the status LED will flash orange after a short time when the connection to the Miniserver is established.

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Pairing Air Devices

Up to 128 Air devices can be paired with a Tree to Air Bridge.

Like the Air Base Extension, the Tree to Air Bridge does not work as a repeater. If Air devices are to communicate with the system via the Tree to Air Bridge, they must be paired with it.

**[Instructions for pairing Air devices](https://www.loxone.com/help/air-interface)**.

---

## Tree To Air Bridge Limitation

A Tree to Air Bridge can support up to **128 Air devices**, whereas a Tree branch can handle a maximum of **50 Tree devices**. Theoretically, this allows for up to **6,400 Air devices** to be connected to a single Tree branch.

However, as the number of connected devices increases, so does data traffic. This can lead to occasional delays due to the **limited data transmission capacity (maximum 20 packets per second)**.

**Key Considerations:**

A **Tree branch** supports up to **50 Tree devices.**

A **Tree to Air Bridge** supports up to **128 Air devices.**

Higher device counts increase data traffic, potentially causing **latency**, especially in high-traffic scenarios

For **optimal performance**, we recommend a **balanced distribution** of devices.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Air Base Extension | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Channel Free Air Base Extension | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - |
| Region (channel) | The selected region affects the operation frequency Please be aware of local government regulations! | - |
| Update Air Devices Individually | Only one Air Device is updated at a time. This reduces the utilisation of the radio channel. | - |
| Automatic Air device pairing until… | Until this date, Air devices that were added manually with their serial number will be paired automatically as soon as they are found. | - |
| Encryption Key | Key for the 128bit AES encryption of Air communication. Hexadecimal, 32 byte length. | - |
| Recreate Encryption Key | Create a new encryption key. Already learned devices need to be relearned! | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted in an electrical enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Tree to Air Bridge**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TreetoAirBridge_100451.pdf)

---