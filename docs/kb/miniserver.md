# Miniserver

Source: https://www.loxone.com/enen/kb/miniserver/

---

The **Loxone Miniserver** serves as central control unit for all kinds of automation tasks.

The replaceable microSD card contains the operating system as well as the user programming and settings. The LAN interface is used for programming and the integrated web server allows the control of the system via a web interface or the Loxone App (up to 63 simultaneous connections).

It features 8 potential free relays, 8 digital inputs and 4 analogue inputs (0-10V). Up to 30 Extensions can be added to the Miniserver via the Link interface to add additional functions such as inputs, outputs or interfaces.

A Loxone Tree interface is also integrated to connect sensors and actuators throughout the building.

[**Datasheet Miniserver**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_Miniserver_100335.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Connecting Extensions](#LinkConnect)
- [Connecting Tree Devices](#TreeConnect)
- [Connecting Digital Inputs](#DIinput)
- [Connecting Analog Inputs](#connect-ai)
- [Loxone Health Check](#HealthCheck)
- [Device Status](#DevStat)
- [SD Card](#sdcard)
- [LED Status](#led_state)
- [Additional information](#desc)
- [Replace Miniserver](#msreplace)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The Miniserver is installed on a DIN rail in a suitable enclosure.

> **ℹ️ Note:** Installation Environment Requirement The Miniserver configuration may contain sensitive data, such as phone numbers and email addresses. To protect this information, the Miniserver must be installed in a secure location with restricted access, ensuring that only authorized personnel can physically reach the device. Suitable installation locations include locked electrical enclosures or secured technical rooms.

![100335 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100335 install.png)

> **ℹ️ Note:** The Miniserver and Air signals can negatively influence each other when in close proximity. Therefore, a distance of 2 division / breaker units should be maintained between a Miniserver and an Air Base.

Connect the power supply, as well as inputs/outputs and interfaces, if applicable.

Via the LAN port the Miniserver is connected to the local network or a WiFi router.

[Information on connecting additional devices.](https://www.loxone.com/enen/kb-cat/cabling/)

The Miniserver starts after the power supply is turned on, and will be operational within a few seconds.
Once the boot up process is complete, the left status LED will blink green once every second.
The right LED indicates whether any [System Status](https://www.loxone.com/enen/kb/systemstatus/) messages are available.

At first startup with factory settings, the Miniserver is assigned an IP address by the router via DHCP.

If there is no DHCP server in your network, or if the Miniserver is connected directly to a PC, link-local addressing via Zeroconf is supported.
If both Miniserver and computer are set to DHCP, they will use a 169.254.x.x link-local address.
Alternatively, you can [manually assign a static IP address](https://www.loxone.com/enen/kb/miniserver-setup/#channetwdeta) to the Miniserver and computer to enable a direct connection.

You can then search for the Miniserver in Loxone Config and connect. The factory settings for user and password are: admin/admin

![V15 Miniserver Search](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/V15 Miniserver Search.png)

![10.5 MS start search classic](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/10.5 MS start search classic.png)

**Then follow the instructions for the [Initial Setup](https://www.loxone.com/enen/kb/miniserver-setup/) to create your new project with the Miniserver.**

---

## Connecting Extensions

Up to 30 Extensions can be connected to a Miniserver's Link interface according to the following diagram:

![link wiring example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/link wiring example.png)

Connect the Extensions to a Miniserver by daisy-chaining the Link interfaces. If any of the Extensions are connected to a separate power supply, then all power supply's GND (negative) have to be interconnected. This connection is crucial for reliable data transmission.

The blue/white wires are used to wire the Link throughout a building. This allows a maximum length of 500m/1640ft.

The Link interface is terminated at the last Extension using the 120 Ohm resistor that is included with the Miniserver.

After switching on the power supply, the Extensions first flash red, and after successful connection to the Miniserver they flash orange.

You can now proceed with pairing the Extensions.

---

## Connecting Tree Devices

Up to 50 Tree Devices can be connected to the Tree interface of a Miniserver, or each of the two Branches of a Tree Extension. There must be no connection between different Tree Branches.
The following diagram shows the wiring of several Tree devices connected to two different Branches of a Tree Extension.

![tree wiring example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tree wiring example.png)

The following wiring topologies are possible, at a maximum length of 500m/1640ft:

![tree wiring topologies](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tree wiring topologies.png)

The Loxone Tree Cable is used for wiring, it contains the Tree communication lines as well as the lines for the 24VDC power supply in the corresponding colors. For devices with higher power requirements such as Tree lighting products, the Tree Cable also contains a pair of wires with 1.5mm² cross-section (AWG16) for power supply.

We recommend running one Tree Cable from the panel per room and then branching off to the individual Tree devices in that room. This corresponds to a Tree topology.

The following image illustrates favorable and less favorable wiring structures within a building, based on the permitted topologies:

![Tree Wiring Topologies Extended](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Tree Wiring Topologies Extended.png)

Within the enclosure, panel wire with twisted pairs is used for wiring.

Devices that communicate with each other have to share a common GND. This is necessary if more than one power supply is used. (e.g. in a sub-panel, or if separate power supplies are used for lighting). For this all power supply's GND (negative) have to be interconnected. This connection is crucial for reliable data transmission.

The cable length of max. 500m per tree branch refers purely to the data lines, not to the 24V power supply. For consumers with higher power such as tree lamps or tree dimmers, the achievable cable length is often significantly shorter due to the resulting voltage drop.

If the current flow on the GND line results in a voltage drop too high, this potential difference also affects the tree communication.
To solve this problem, split consumers of higher power over longer distances to several supply lines, or use a supply line with a higher cross-section, or a power supply close to the consumers.
For existing installations, doubling the cross-section of the GND line is often sufficient to eliminate the potential difference.

You can now proceed with pairing the Tree devices.

---

## Connecting Digital Inputs

![DI Example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/DI_Example.png)

---

## Connecting Analog Inputs

**0-10V transmitter with 2 outputs, common power supply:**

![conex AnalogInUni 2x0 10 1xpsu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/conex_AnalogInUni_2x0-10_1xpsu.png)

**0-10V transmitter with 2 outputs, separate power supply:**

![conex AnalogInUni 2x0 10 2xpsu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/conex_AnalogInUni_2x0-10_2xpsu.png)

---

## Loxone Health Check

The diagnostics of the Miniserver and the Loxone interfaces can be started via the Loxone Health Check:

![HealthCheck MS](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/HealthCheck_MS.png)

---

## Device Status

The Device Status serves as a central overview of the status of all devices in the programming, This enables a fast, but also detailed error diagnosis.

The Device Status can be opened via the menu bar:

![devstat open](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/devstat_open.png)

If a device is offline, currently being updated or has not yet been paired, this is highlighted in color in the status column:

![devstat overview](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/devstat_overview.png)

### Diagnostic Options

With a right click on the desired device, individual information can be retrieved and actions triggered. These available options are device-dependent.

![devstat options](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/devstat_options.png)

1. Text file with details of the selected device
2. Detailed troubleshooting details are summarized in a text file (**TechReport**), battery powered air devices must be [awake](https://www.loxone.com/enen/kb/air-base-extension/#AirWake/) for this purpose
3. Reboot device*
4. Get central log file **def.log**, where important events in the system are logged*
5. Show structure file LoxApp3.json, file to enable communication between user interface and Miniserver*
6. Send [device command](https://www.loxone.com/help/device-command/)
7. Send device to [sleep](https://www.loxone.com/enen/kb/air-base-extension/#AirWake/)
8. Keep device [awake](https://www.loxone.com/enen/kb/air-base-extension/#AirWake/)

* only available for Miniserver

**Additional information**
[Update and Diagnostics for Tree Devices](https://www.loxone.com/enen/kb/tree-extension/#TreeDiag/)
[Update and Diagnostics for Air Devices](https://www.loxone.com/enen/kb/air-base-extension/#AirDiag/)

---

## SD Card

The Micro SD card, located at the top edge of the Loxone Miniserver, contains the operating system and settings.

The SD card can be removed as follows:
Gently press the visible edge of the SD card inward with your fingernail.
This will release and partially eject the card so it can be removed.

If the SD card fails to latch when inserting it, it needs to be fully removed to reactivate the latching mechanism.

---

## LED Status

| Left LED | Right LED | Description |
| --- | --- | --- |
|  |  | Everything OK, device is online. |
|  |  | One or more System Status messages are active. |
|  |  | Device was selected in Loxone Config and is identifying. |
|  |  | Update is in progress. |

**Boot Phase:**

| Left LED | Right LED | Description |
| --- | --- | --- |
|  |  | Miniserver is booting. |
|  |  | Miniserver is loading the bootloader image from the SD card. |
|  |  | Miniserver has successfully loaded the image and will unpack it as the next step. |
|  |  | Miniserver has successfully unpacked the image. |
|  |  | Operating system is started. |
|  |  | Miniserver is loading the program file. |
|  |  | SD card cannot be read. Check SD card. |
|  |  | No compatible operating system on the SD card. |

> **ℹ️ Note:** Network connection

---

## Additional information

[Restore factory defaults and format SD card](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/)

[Creating a backup](https://www.loxone.com/enen/kb/backup-sd-card/)

[Update the Miniserver](https://www.loxone.com/enen/kb/installing-updates/)

[How to set up remote access](https://www.loxone.com/enen/kb/remote-connect/)

[Connecting several Miniservers via Tree Intercommunication](https://www.loxone.com/enen/kb/tree-intercommunication/)

[Connecting multiple Miniservers via Gateway-Client function](https://www.loxone.com/enen/kb/miniserver-clientgateway-concentrator/)

---

## Replace Miniserver

If a Miniserver needs to be replaced by another one, a wizard is available in Loxone Config to guide you through the necessary steps.
Start the wizard and follow the instructions:

![msreplace start](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/msreplace_start.png)

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
| Voltage 1 | - | ∞ |
| Voltage 2 | - | ∞ |
| Voltage 3 | - | ∞ |
| Voltage 4 | - | ∞ |

---

## Actuators

| Summary | Unit | Value Range |
| --- | --- | --- |
| Actuator (Relay) 1 | Digital | 0/1 |
| Actuator (Relay) 2 | Digital | 0/1 |
| Actuator (Relay) 3 | Digital | 0/1 |
| Actuator (Relay) 4 | Digital | 0/1 |
| Actuator (Relay) 5 | Digital | 0/1 |
| Actuator (Relay) 6 | Digital | 0/1 |
| Actuator (Relay) 7 | Digital | 0/1 |
| Actuator (Relay) 8 | Digital | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Computing power throttling | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Serial Number | Specifies the serial number of the device. | - |
| Local Address | Enter the local address of the Miniserver, this is the IP address or hostname that can be used to reach it on the local network. Hostnames are not supported in Gateway-Client projects. | - |
| External Address | Enter the address at which the Miniserver is accessible via the internet (hostname or IP address). If you are using Loxone Cloud DNS, then please enter connect.loxonecloud.com or connect.loxonecloud.com: if not using default port 80. | - |
| External Port HTTP | The external port that you specified in the port forwarding settings of your router for the HTTP port of the Miniserver | - |
| External Port HTTPS | The external port that you specified in the port forwarding settings of your router for the HTTPS port of the Miniserver | - |
| Miniserver Configuration | Edit Miniserver settings. A connection to the Miniserver is required. | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted on a DIN rail in an electrical distribution enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Miniserver**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_Miniserver_100335.pdf)

[Loxone Ports & Domains](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Loxone-Required-Ports-Domains.pdf)

[Thermal Shutdown Temperatures](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Thermal_Shutdown_Temperatures.pdf)

---