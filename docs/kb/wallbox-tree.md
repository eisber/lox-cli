# Wallbox 11kW 16A Tree

Source: https://www.loxone.com/enen/kb/wallbox-tree/

---

The Loxone Wallbox is a charging station which enables dynamic charging from 1.38kW to 11kW for electric vehicles.

With the robust Tree interface, the device seamlessly integrates into the Loxone system, offering complete flexibility for energy management.
To measure electrical energy, a Loxone Energy Meter Tree can be easily mounted on the integrated DIN rail and connected to the Tree Interface. Additionally, a Modbus meter can be installed and connected to the [Modbus interface.](https://www.loxone.com/enen/kb/modbus-extension/#Commissioning)
Optionally, products like the NFC Code Touch, Touch Pure Flex,... can be mounted directly on the device to set your charging mode or execute any other control.

[**Datasheet Wallbox 11kW 16A Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_Wallbox11kW16ATree_100526.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Programming](#baseconf)
- [LED Status](#led_state)
- [Charging Cable](#chargingcable)
- [Limiting charging power](#powerlimits)
- [Testing the Wallbox](#WboxTestDevice)
- [Modbus Data Reading](#wboxdataread)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Attach the Wallbox vertically by screwing it onto a solid, even and closed surface.
Select screws and fastening material according to the surface.
Make sure that the mounting frame rests flush on the surface without any gaps.

![wb mount base 16A Tree](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wb_mount_base_16A-Tree.png)

![wb mainsconnect 16A tree](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wb_mainsconnect_16A_tree.png)

![wb connect meter 16A Tree](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wb_connect_meter_16A-Tree.png)

![wb mount touch tree](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wb_mount_touch_tree.png)

![wb mount frontcovers](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wb_mount_frontcovers.png)

After mounting, the supply voltage (mains voltage and 24V) is switched on, the Wallbox flashes orange after a short time if the connection to the Miniserver is successful and is ready for pairing.

**Note:** Do not install the Wallbox using magnets or place it near magnetic sources (closer than 0.5 meters). Magnetic fields may interfere with the internal residual current device, potentially affecting the charging functionality.

---

## Commissioning

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Programming

The Loxone Wallbox is programmed and set up using the [Wallbox function block](https://www.loxone.com/help/wallbox-block) in Loxone Config:

![wb dragdrop block](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wb_dragdrop_block.gif)

---

## LED Status

> **ℹ️ Note:** Device has just been paired or has restarted and is now online.

---

## Charging Cable

If it is necessary to replace the charging cable, this must be installed and tested by a specialist in accordance with national regulations.
For this purpose, a charging cable with the same properties must be used; this may also be shorter or longer (max. 7.5 meters).
Extending the charging cable in any other way is not permitted.

---

## Limiting charging power

The actual charging power depends on the **vehicle**, **supply voltage**, and **battery level**, and may be above or significantly below the target charging power (**Tp**).

### Ensuring Consistent 3-Phase Charging

Some vehicles may encounter difficulties when it comes to 1-phase charging or switching between 1-phase and 3-phase charging. To address this, the default minimum charging power of the [Wallbox function block](https://www.loxone.com/help/wallbox-block) is set at **4.16 kW**. This ensures that the charging process exclusively involves **3-phase charging**.

![WB3PH16APowerDia](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/WB3PH16APowerDia.png)

### Vehicle Limitations on Phase Switching

Some vehicles only support a **limited number of switches** between 1-phase and 3-phase charging during a charging session.

Therefore, with dynamic charging currents (e.g., during surplus charging), certain vehicles may **terminate the charging process** after multiple phase changes. This situation can occur on cloudy days, for example, when solar production fluctuates rapidly.

In this case, the Wallbox remains in the "**Connected**" state and waits for the vehicle to resume charging.

---

## Testing the Wallbox

Note on testing the Wallbox with EVSE Testers:
Due to the testing of the internal residual-current sensor, a waiting period of at least 10 seconds is required when switching from Position B (vehicle connected) to Position C (vehicle ready to charge). Also, charging power for the charging process must be authorised to the wallbox. Otherwise, an unknown charging error may occur.

---

## Modbus Data Reading

Only the values that are actually sent to the Miniserver are displayed in the Modbus Monitor.

This means that only values that have changed are shown. The data is not displayed in the monitor with every polling cycle.

---

## Actuators

| Summary | Description |
| --- | --- |
| API Connector | Intelligent API based connector. API Commands |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Charging error | Reports a charging error. | - | 0/1 |
| Fault current | Reports a fault current. | - | 0/1 |
| Online Status Wallbox Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| System temperature | Provides the internal device temperature. This is often the temperature of the CPU or another location in the device. | ° | ∞ |
| Error Code | - | ∞ |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

Only mount on a solid, even and closed surface.

The selection of conductor cross-sections and associated overcurrent protection devices is dictated by national and international standards and installation guidelines. This requires choosing a conductor cross-section suitable for the loads rated current, as well as considering the insulation material of the cable, method of installation, and ambient temperature.

---

## Documents

[**Datasheet Wallbox 11kW 16A Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_Wallbox11kW16ATree_100526.pdf)

[Thermal Shutdown Temperatures](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Thermal_Shutdown_Temperatures.pdf)

---