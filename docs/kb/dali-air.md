# DALI Air

Source: https://www.loxone.com/enen/kb/dali-air/

---

DALI Air is a compact Interface to wirelessly integrate up to 10 DALI devices, either in broadcast mode or by assigning dedicated addresses. It is DALI-2 certified and supports also DALI Sensors.

[**Datasheet DALI Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_DALIAir_100570.pdf)

Using the [DALI Alliance database](https://www.dali-alliance.org/products), you can check which functions specific DALI devices support and what certifications they have.

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Pairing DALI Devices](#DALIpair)
- [DALI Broadcast](#DALIbroadcast)
- [Address conflict](#daliaddressconflict)
- [Create DALI groups](#daligroups)
- [Diagnostics for DALI devices](#dalidiag)
- [Supported DALI device types](#dalitypes)
- [Supported DALI-2 Instance Types](#dali2types)
- [Inputs, Outputs, Properties](#Diagnostic)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Install the device in a suitable installation box.

![100570 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100570-install.png)

Connect the AC mains supply (brown/blue wire), as well as the DALI data lines (grey screw terminal).

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for two minutes, then pairing mode is activated for 30 minutes.

---

## Pairing DALI Devices

To search for DALI devices, first click on the DALI Air in Loxone Config, and then activate **DALI search**:

![DALIsearch](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/DALIsearch.png)

It will then search for devices and display the discovered devices. By clicking the right arrow, selected devices are added to the programming.

After that, the added devices are available in the periphery tree, and can be used on the programming page:

![DALIAir DeviceActor](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/DALIAir-DeviceActor.png)

---

## DALI Broadcast

Using a DALI broadcast, you can effortlessly control all compatible DALI bus devices at once, without the need for individual pairing or programming. To enable this feature, simply activate the Broadcast Application in the DALI settings.
When using DALI broadcast, DALI sensors cannot be added.

![DALIAir BroadcastActor](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/DALIAir-BroadcastActor.png)

---

## Address conflict

If an **address conflict** is detected, e.g. if two or more devices have the same address, the function "Resolve Address Conflict" can be used:

![dali AddressConflict](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/dali_AddressConflict.png)

The affected devices are assigned a new address. If devices were already paired, they can be added again and the old devices replaced. To do this, select the newly addressed device and the device to be replaced and click on the button with the arrow pointing right.

Alternatively, you can also start a search with "New Installation", where all device addresses are deleted and reassigned.

---

## Create DALI groups

If several DALI lights are to be controlled simultaneously, a DALI group is recommended.
This offers the advantage that the programming is simpler and there is no time delay when dimming the lights. Also, central functions can be handled easier.

When pairing DALI devices, existing group assignments are imported and created in Loxone Config.

To create a new group, the DALI devices must already be paired.

The devices can be selected in the DALI group's properties, thus assigning them to that group.

The group actuator can then be used in programming.

---

## Diagnostics for DALI devices

With the function "Activate all" all connected DALI lights are set to full brightness, so the wiring can be checked.

![dali ActivateAll](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/dali_ActivateAll.png)

The option **Display diagnostic inputs** can be used to display an error input for DALI devices, that is active in case of errors.

Possible errors are: no bulb or defective bulb in the DALI device.

The DALI Monitor provides an analysis of the communication between the DALI Extension and devices. First connect to the Miniserver, click on the DALI Extension, and then on **DALI Monitor**.

The DALI Monitor displays the name of the device, the transmitted values and the DALI commands.

**Note:**
The DALI Monitor for DALI Air is deactivated by default due to the high volume of communication on the Air network when the DALI Monitor is enabled. Each DALI command is transmitted over Air, creating a significant load.

To **activate** the DALI Monitor for the DALI Air, go to Device Status -> Right Click on the DALI Air device -> Send Device Command -> **MonitorOn**

To **deactivate** the DALI Monitor for the DALI Air, go to Device Status -> Right Click on the DALI Air device -> Send Device Command -> **MonitorOff**

These are defined in IEC 62386-102, the following is an excerpt:

| Command | Description |
| --- | --- |
| DIRECT_ARC_POWER | Dimming value is sent (0-255) |
| CMD_QUERY_X | A data set is queried from the DALI dimmer |
| SEARCH_ADDR_H SEARCH_ADDR_M SEARCH_ADDR_L | This sets the current search address |
| COMPARE | Prompts the dimmer to compare its own random number with the current search address |
| WITHDRAW | Found dimmer is removed from the search |
| PROG_SHORT_ADR | The DALI short address is stored in the dimmer |
| VERIFY_SHORT_ADR | The DALI short address of the dimmer is verified |
| TERMINATE | Search is ended |

---

## Supported DALI device types

> **ℹ️ Note:** Fluorescent lamps

---

## Supported DALI-2 Instance Types

> **ℹ️ Note:** Push Buttons

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status DALI Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Disable Repeater functionality | Disable repeater functionality of this Air device. Loxone Air is based on mesh technology. Any air device connected to the power supply can repeat packets from other Air devices, thus extending the range and stability of the overall system. In large systems with a large number of air devices in a confined space, the communication between the air devices can lead to a very high radio channel utilization. A reliable accessibility of the air devices can not be guaranteed. Disabling repeater functionality on individual Air devices can help. Do not disable this function recklessly as this may affect the range and stability of the system. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Application | Specifies the intended use of the DALI devices. Broadcast: all DALI devices can be controlled simultaneously Individual devices: DALI devices can be controlled individually Adding the devices manually or via the DALI device seach is only available when the application Individual devices is activated | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted in an electrical enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet DALI Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_DALIAir_100570.pdf)

---