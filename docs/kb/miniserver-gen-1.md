# Miniserver Gen. 1

Source: https://www.loxone.com/enen/kb/miniserver-gen-1/

---

The **Loxone Miniserver** serves as central control unit for all kinds of automation tasks.

The replaceable microSD card contains the operating system as well as the user programming and settings. The LAN interface is used for programming and the integrated web server allows the control of the system via a web interface or the Loxone App (up to 31 simultaneous connections).

It features 8 potential free relays, 8 digital inputs, 4 analogue inputs (0-10V) and 4 analogue outputs (0-10V). Up to 30 Extensions can be added to the Miniserver via the Link interface to add additional functions such as inputs, outputs or interfaces.

An interface for connecting KNX devices is also integrated.

[**Datasheet Miniserver Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MiniserverGen1_100001.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Connecting Extensions](#LinkConnect)
- [Pairing Extensions](#LinkPair)
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

![100001 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100001 install.png)

> **ℹ️ Note:** The Miniserver Gen.1 and Air signals can negatively influence each other when in close proximity. Therefore, a distance of 2 division / breaker units should be maintained between a Miniserver Gen.1 and an Air Base.

Connect the power supply, as well as inputs/outputs and interfaces, if applicable.

Via the LAN port the Miniserver is connected to the local network or a WiFi router.

[Information on connecting additional devices.](https://www.loxone.com/enen/kb-cat/cabling/)

The Miniserver starts after the power supply is turned on, and will be operational within a few seconds.
Once the boot up process is complete, the left status LED will blink green once every second.
The right LED indicates whether any [System Status](https://www.loxone.com/enen/kb/systemstatus/) messages are available.

When starting for the first time with factory settings, the Miniserver is assigned an IP address by the router via DHCP.
If there is no DHCP server in your network, or if the Miniserver is connected directly to a PC, its default IP address is 192.168.1.77. In this case, assign a matching IP address to the PC to make the connection possible.
Zeroconf link-local addressing is not supported by the Miniserver Gen. 1.

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

## Pairing Extensions

To search for Extensions, first click on a Link interface in Loxone Config, and then activate **Extension Search**.

The window that opens will list all connected Extensions that are not yet part of the program to the left:

![extension search](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/extension_search.png)

If you select an Extension here, it will **identify** itself by flashing its status LED. This allows you to assign and name the Extensions correctly.

Select the desired Extension, assign a name and installation location and add it to the programming using the **+** button.

The right window lists all the Extensions that are currently part of the program. You can display them by clicking the button **Show my Extensions**. You can also replace an existing Extension with a new one of the same type that was found in the search. This is useful when a device needs to be replaced or devices are added to a pre-configured program. Select the device to be added and the device to be replaced. By clicking on the button with the arrow pointing right, the old device is replaced with the new one in the program.

**To apply the changes, save the program in the Miniserver.**

Now the added Extensions are ready for use and available in the Periphery Tree in Loxone Config.

---

## Loxone Health Check

The diagnostics of the Loxone interfaces can be started via the Loxone Health Check:

![HealthCheck Gen1](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/HealthCheck_Gen1.png)

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

The Micro SD card, located at the left side of the Loxone Miniserver, contains the operating system and settings.

The SD card can be removed as follows:
Gently pull the visible edge of the SD card outward with your fingernail or a small flat screwdriver.

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
|  |  | Miniserver is loading the program file. |
|  |  | SD card cannot be read. Check SD card. |
|  |  | No compatible operating system on the SD card. |

> **ℹ️ Note:** Network connection

---

## Additional information

[Restore factory defaults and format SD card](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/)

[Creating a backup](https://www.loxone.com/enen/kb/backup-sd-card/)

[Update the Miniserver](https://www.loxone.com/enen/kb/installing-updates/)

[How to set up remote access](https://www.loxone.com/enen/kb/remote-access/)

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
| Voltage 1 | - | ∞ |
| Voltage 2 | - | ∞ |
| Voltage 3 | - | ∞ |
| Voltage 4 | - | ∞ |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Serial Number | Specifies the serial number of the device. | - |
| Local Address | Enter the local address of the Miniserver, this is the IP address or hostname that can be used to reach it on the local network. Hostnames are not supported in Gateway-Client projects. | - |
| External Address | Enter the address at which the Miniserver is accessible via the internet (hostname or IP address). If you are using Loxone Cloud DNS, then please enter connect.loxonecloud.com or connect.loxonecloud.com: if not using default port 80. | - |
| Miniserver Configuration | Edit Miniserver settings. A connection to the Miniserver is required. | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted on a DIN rail in an electrical distribution enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Miniserver Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MiniserverGen1_100001.pdf)

[**Datasheet Relay**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Relay_HF33F_en.pdf)

[Loxone Ports & Domains](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Loxone-Required-Ports-Domains.pdf)

---