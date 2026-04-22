# Miniserver Go Gen. 1

Source: https://www.loxone.com/enen/kb/miniserver-go-gen-1/

---

The **Loxone Miniserver Go** serves as central control unit for all kinds of automation tasks.

The replaceable microSD card contains the operating system as well as the user programming and settings. The LAN interface is used for programming and the integrated web server allows the control of the system via a web interface or the Loxone App (up to 31 simultaneous connections).

An Air Base Extension for connecting devices via Loxone Air wireless technology is already integrated.

Up to 30 Extensions can be added to the Miniserver Go via the Link interface to add additional functions such as inputs, outputs or interfaces.

[**Datasheet Miniserver Go Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MiniserverGo_100139.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Connecting Extensions](#Anschließen von Extensions)
- [Pairing Extensions](#LinkPair)
- [Pairing Air Devices](#AirPair)
- [Loxone Health Check](#HealthCheck)
- [Device Status](#DevStat)
- [Device Status](#DevStat)
- [SD Card](#sdcard)
- [LED Status](#led_state)
- [Additional information](#desc)
- [Replace Miniserver](#msreplace)
- [Inputs, Outputs, Properties](#Property)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

Install the unit in a place where it is protected from water, dirt and possible damage. The bracket on the back allows for screw mounting.

![100139 100336 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100139 100336 install.png)

> **ℹ️ Note:** The Miniserver Go Gen.1 and Air signals can negatively influence each other when in close proximity. Therefore, a distance of 2 division / breaker units should be maintained between a Miniserver Go Gen.1 and an Air Base.

Plug the included power supply into the Micro-USB connector.

Via the LAN port the Miniserver is connected to the local network or a WiFi router.

The Miniserver Go starts after the power supply is connected, and will be operational within a few seconds.
Once the boot up process is complete, the status LED will be permanently lit.
The LEDs on the RJ45 port are disabled on the Miniserver Go and are off during operation.

When starting for the first time with factory settings, the Miniserver is assigned an IP address by the router via DHCP. If there is no DHCP server in your network, or if the Miniserver is connected directly to a PC, its default IP address is 192.168.1.77. In this case, assign a matching IP address to the PC to make the connection possible.
Zeroconf link-local addressing is not supported by the Miniserver Go Gen. 1.

You can then search for the Miniserver in Loxone Config and connect. The factory settings for user and password are: admin/admin

![V15 Miniserver Search](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/V15 Miniserver Search.png)

![10.5 MS start search classic](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/10.5 MS start search classic.png)

**Then follow the instructions for the [Initial Setup](https://www.loxone.com/enen/kb/miniserver-setup/) to create your new project with the Miniserver.**

---

## Connecting Extensions

Up to 30 Extensions can be connected to the Miniserver Go according to the following diagram:

![100139 100336 connect extension](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100139 100336 connect extension.png)

Connect the Extensions to the Miniserver Go by daisy-chaining the Link interfaces. Since the Extensions are connected to a separate power supply, GND must also be connected. This connection is crucial for reliable data transmission.

One twisted pair of a CAT5/6/7 cable can be used to wire the Link throughout a building. We recommend using the blue/white pair.

The Link interface is terminated at the last Extension using the 120 Ohm resistor that is included with the Miniserver.

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

## Pairing Air Devices

**Pairing Mode**

All Air devices have to be paired in Loxone Config via the pairing mode. In delivery state, pairing mode will be active after the power supply has been established.

On most Air devices, the status LED will indicate this by blinking red/green/orange. You can find the exact method of indicating pairing mode in the documentation of the respective Air device.

On most battery powered Air devices the pairing mode is only active for 5 minutes to conserve energy. Briefly remove the battery and then reinsert it to reactivate pairing mode, if necessary.

If an Air device blinks only orange, then it was previously paired, but can no longer establish a connection. In this case, you have to [activate the pairing mode manually](https://www.loxone.com/help/air-interface#AirPairMode). On most devices, this is done by pressing the pairing button or power cycling the device.

**Searching and pairing**

To search for Air devices, first click on an Air interface in Loxone Config, and then activate **Air Device Search**

The window that opens will list all Air devices that are in pairing mode. This can take a few minutes:

![10.5 air search](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/10.5 air search.png)

If you select a device here, it will **identify** itself in different ways. Devices with a status LED will flash it, lighting products pulse a white light, other devices such as the Loxone Touch emit an audible click. This allows you to assign and name the devices correctly.

Select the desired device, assign a name, room and installation location and add it to the programming using the **Pair Device** or **+** button.

The right window lists all the devices that are currently part of the program. You can display them by clicking the button **Show my Air devices**. You can also replace an existing device with a new device of the same type that was found in the search. This is useful when a device needs to be replaced or devices are added to a pre-configured program. Select the device to be added and the device to be replaced. By clicking on the button with the arrow pointing right, the old device is replaced with the new one in the program.

**To apply the changes, save the program in the Miniserver.**

Now the added devices are ready for use and the functions are available in the Periphery Tree in Loxone Config.

The Loxone App, under Settings, also supports searching for and pairing Air devices.

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

The Micro SD card, located at the back of the Loxone Miniserver Go, contains the operating system and settings.

The SD card can be removed as follows:
Gently press the visible edge of the SD card inward with your fingernail or a small flat screwdriver.
This will release and partially eject the card so it can be removed.

If the SD card fails to latch when inserting it, it needs to be fully removed to reactivate the latching mechanism.

---

## LED Status

| Top LED | Back LED | Description |
| --- | --- | --- |
|  |  | Everything OK, device is online. |
|  |  | One or more System Status messages are active. |
|  |  | Device was selected in Loxone Config and is identifying. |
|  |  | Update is in progress. |

**Boot Phase:**

| Top LED | Back LED | Description |
| --- | --- | --- |
|  |  | Miniserver is booting. |
|  |  | Miniserver is loading the bootloader image from the SD card. |
|  |  | Miniserver has successfully loaded the image and will unpack it as the next step. |
|  |  | Miniserver has successfully unpacked the image. |
|  |  | SD card cannot be read. Check SD card. |

The LEDs on the RJ45 port are disabled and are off during operation.

---

## Additional information

[Restore factory defaults and format SD card](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/)

[Creating a backup](https://www.loxone.com/enen/kb/backup-sd-card/)

[Update the Miniserver](https://www.loxone.com/enen/kb/installing-updates/)

[How to set up remote access](https://www.loxone.com/enen/kb/remote-access/)

---

## Replace Miniserver

If a Miniserver needs to be replaced by another one, a wizard is available in Loxone Config to guide you through the necessary steps.
Start the wizard and follow the instructions:

![msreplace start](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/msreplace_start.png)

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

Connecting additional Extensions to the Link interface must be carried out by a qualified electrician in accordance with the applicable regulations.

---

## Documents

[**Datasheet Miniserver Go Gen. 1**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MiniserverGo_100139.pdf)

[Loxone Ports & Domains](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Loxone-Required-Ports-Domains.pdf)

---