# Belimo Tree

Source: https://www.loxone.com/enen/kb/belimo-tree/

---

Belimo is one of the leading manufacturers of drives in the area of climate control.
The Loxone Belimo Tree is an interface to integrate up to 16 Belimo MP-Bus devices. With the native MP-Bus integration, Belimo products become a native member of Loxone Config and the Miniserver.

[**Datasheet Belimo Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_BelimoTree_100519.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Commissioning Belimo devices](#commbelimo)
- [Limitation of inputs/outputs](#limitIOs)
- [Data Reading](#dataread)
- [Inputs, Outputs, Properties](#Diagnostic)
- [Documents](#Documents)

---

## Mounting

Install the device in a suitable installation box.

![belimotree connection diagram](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/belimotree_connection_diagram.png)

Connect the power supply (orange/white terminal) and Tree communication wires (green/white terminal).

Connect the orange MP-Bus lead to the MP-Bus nodes. The GND of the MP-Bus nodes must be connected to the GND of the Belimo Tree.

While 24VAC power supply is possible for many MP-Bus devices, the Belimo Tree can only be supplied with 24VDC.

Shortly after power-up, the status LED will blink orange if the wiring is correct (connection to Tree Extension and Miniserver is established).

---

## Commissioning

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Commissioning Belimo devices

There are 3 ways to insert Belimo devices:

![belimo add](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/belimo_add.png)

![belimo addmanual](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/belimo_addmanual.png)

Belimo Datapool devices are added in the following steps:
1. Download suitable template for device from [Loxone Library](https://library.loxone.com/)
2. [Import template](https://www.loxone.com/help/templates)
3. Insert template into programming:

![belimo addtemplate](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/belimo_addtemplate.png)

Then configure / address the inserted Belimo device:

![belimo configtemplate](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/belimo_configtemplate.png)

If Belimo devices are addressed via the Belimo App, their address and serial number is entered in the properties:

![belimo appadressing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/belimo_appadressing.png)

For more information, please visit the [Belimo Documentation](https://www.belimo.com/de/en_GB/products/systems/product-documentation/bus-system-integration.html).

---

## Limitation of inputs/outputs

The number of usable inputs/outputs (IOs) is limited to 128 IOs in total per Belimo Tree/Air and to 20 IOs per Belimo device. Only IOs that are actually used on a programming page are relevant for this.

If these numbers are exceeded, an error message is displayed when inserting further IOs.

To solve this, IOs that are not absolutely required can be removed from the programming. Alternatively, use an additional Belimo Tree/Air.

---

## Data Reading

Only the values that are actually sent to the Miniserver are displayed in the Belimo Monitor.

This means that only values that have changed are shown. The data is not displayed in the monitor with every polling cycle.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Belimo Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - | - |
| MP-Bus Supply | Specifies if MP-Bus supply is active | - | - | - |
| Device Status Update Cycle | Belimo device specific status update cycle time in seconds (10 - 600) | s | 10...600 | 60 |

---

## Documents

[**Datasheet Belimo Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_BelimoTree_100519.pdf)

[Inputs, outputs, status values and error messages](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/IOs_StatVal_Error.pdf)

[Belimo Technical Documentation](https://www.belimo.com/mam/europe/technical-documentation/project_planning_notes/belimo_notes-for-project-planning_introduction-MP-bus-technology_en-gb.pdf)

[Belimo Cable Length Calculator](https://www.belimo.com/mam/europe/technical-documentation/application_programs_and_plug-ins/MP-Bus/MP-Bus_Application_MP-Bus-cable-length-calculator.xlsm)

---