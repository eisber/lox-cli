# BACnet

Source: https://www.loxone.com/enen/kb/bacnet/

---

BACnet, which stands for Building Automation and Control Networks, is a standard communication protocol used for building automation and control systems for applications such as heating, ventilating, and air-conditioning control (HVAC), lighting control, access control, and fire detection systems and their associated equipment.
It allows different devices and systems to communicate and exchange information seamlessly.

The Loxone Miniserver implements BACnet/IP with the following BACnet device profiles:
B-SA (BACnet Smart Actuator)
B-GW (BACnet Gateway)

You can find detailed information about the supported functions in the [Protocol Implementation Conformance Statement (PICS)](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/06/BACnet-PICS-Loxone-Miniserver.pdf).

The Loxone Miniserver is [BACnet certified](https://www.bacnetinternational.net/btl/index.php?m=368), ensuring its compatibility and compliance with BACnet standards.

![BACnetBTL logo](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/BACnetBTL-logo.png)

> **ℹ️ Note:** BACnet requires the current Miniserver, the Miniserver Gen. 1 is not supported!

**Explanation of BACnet Server and Client in the context of the Loxone Miniserver**
- A BACnet server is a device that offers data, information, or services.
- A BACnet client is a device (or software) that requests or reads information from the server.

The Loxone Miniserver acts as a BACnet server.

This means:
- It makes its data (like temperatures, switch states, energy values, etc.) available on the BACnet network.
- It cannot request or read data from other BACnet devices.

To enable communication between the Loxone system and another BACnet device, such as those from Siemens or Schneider (which are also servers), a BACnet client is required to act as an intermediary.

The BACnet client establishes connections with both servers and is responsible for:
- Read values from one server (for example, Siemens).
- Write values to another server (for example, Loxone).
- Synchronize data between them.

This client is usually a software application running on a PC or a dedicated gateway device.

To enable BACnet, click on 'Network Periphery' in the 'Periphery' tab, then 'Add Network Device' and select 'BACnet'. Or you can quickly add it by clicking F5 and searching for 'BACnet'.

## Table of Contents
- [Properties](#Property)

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Port | TCP port on which this BACnet server can be accessed | 1...65535 | 47808 |
| Password | Password used for the BACnet reinitialize device command. If there is no password the command will not be allowed. | - | - |
| Instance number | BACnet instance number, which the Miniserver uses in the BACnet network. | ∞ | 0 |

---