# M-Bus Extension

Source: https://www.loxone.com/enen/kb/m-bus-extension/

---

The M-Bus Extension serves as an M-Bus master, allowing for the integration of up to 30 M-Bus slaves. This enables the collection of data from a wide range of meters for various utilities such as electricity, gas, water, heat, and more.

> **ℹ️ Note:** Several electric utility company meters work as M-Bus master, like the M-Bus Extension. Meters of this type are not compatible, because only one master is allowed at the M-Bus. Also the M-Bus Extension does not support encryption.

[**Datasheet M-Bus Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MBusExtension_100569.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Pairing M-Bus Devices](#pairmbus)
- [Sensors and Usage](#sensors)
- [Addressing and conflicts](#addrconflict)
- [Change Baud Rate](#baudrate)
- [Inputs, Outputs, Properties](#Diagnostic)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

The M-Bus Extension is installed on a DIN rail in a suitable enclosure.
Connect the M-Bus Extension to the 24V supply and Loxone Link, as well as the devices to the M-Bus.

![mbus connect](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/mbus_connect.png)

In general, it is recommended to keep the M-Bus as short as possible, with a maximum length of 100 meters, and use a cable with a twisted pair of 2x0.8mm / 0.5mm² / AWG 20. This has a positive impact on achievable baud rates and polling cycles.
Shielding is not necessary in common installations and can even have negative effects, when using long cable lengths.

If multiple M-Bus devices share the same secondary address and baud rate, this address conflict will have to be [resolved](#addrconflict).

After the power supply is turned on, the Extension is ready to pair.

---

## Commissioning

The Extension starts after switching on the power supply, and the status LED will flash orange after a short time when the connection to the Miniserver is established.

**[Then follow the pairing procedure on the Link Interface.](https://www.loxone.com/help/link-interface#LinkPair)**

---

## Pairing M-Bus Devices

To search for M-Bus devices, first click on the M-Bus Extension in Loxone Config, and then activate **M-Bus search**:

![mbus search](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/mbus_search.png)

It will then search for devices with different baud rates and display the discovered devices. By clicking the right arrow, selected devices are added to the programming. Finally, save the program to the Miniserver.

---

## Sensors and Usage

After inserting the M-Bus devices, they will appear in the peripheral tree of Loxone Config, and various sensors will have been transferred from the M-Bus devices during the setup.

The available physical quantities, along with their units, vary depending on the device and manufacturer, and are defined in the M-Bus standard through [VIF and VIFE codes](https://m-bus.com/documentation-wired/08-appendix).

It is advantageous to only drag the sensors onto the program page that are absolutely necessary. This can reduce the bus load on some M-Bus devices and allow shorter polling cycles. However, this has no effect on many M-Bus devices, which send all available sensor data at once.

The fewer M-Bus devices (sensors) are polled, and the higher the baud rate, the shorter the polling cycles can be.

A typical readout of a meter at 2400 baud takes about 1 second. During this time, no other meter can be read on the M-Bus. If the baud rate is increased to 9600, this time can be reduced to around 0.5 seconds. This means that more devices can be read in less time. Conversely, reading a meter at a baud rate of 300 baud can take up to 5 seconds, which occupies the M-Bus for this time.

---

## Addressing and conflicts

### Addressing

Only M-Bus devices that support both primary and secondary addressing can be used. This is standard for almost all M-Bus devices.

M-Bus devices usually come from the factory with a preset primary and different secondary address. Together with the large address space, an address conflict is therefore rather the exception.

Even an operation of devices with the same address is possible, if they work with different baud rates.

### Resolve address conflicts

If devices with the same baud rate and the same secondary address are present on the M-Bus, this leads to address conflicts already during the device search. Such devices can then not be added.

To solve such a conflict, first identify the devices with the same address and baud rate. The identification numbers (the first 4 bytes of the 8 byte long secondary address) are decisive.This is often printed on the M-Bus device.
Disconnect all but one of the devices with the same address from the bus, then search and add only this one.
Then [assign a different baud rate](#baudrate) to the device. Repeat this step by step for each device.

If there are more devices with the same address than different baud rates available, another M-Bus Extension can be used.

---

## Change Baud Rate

Even if M-Bus devices work with different baud rates, they can be operated together.
If necessary, the baud rate of M-Bus devices can be changed in Loxone Config:

![mbus baudrate](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/mbus_baudrate.png)

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status M-Bus Extension | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

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

[**Datasheet M-Bus Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MBusExtension_100569.pdf)

---