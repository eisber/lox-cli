# Audio Cabling

Source: https://www.loxone.com/enen/kb/audio-cabling/

---

[Note: When using more than one Audioserver, network failures may occur since firmware 2.6 due to reboots on Huawei routers.](https://www.loxone.com/dede/wp-content/uploads/sites/2/2021/04/Audioserver_KnownIssue.pdf)

The Loxone Audioserver is a highly versatile audio solution for any type of building.

In combination with the Miniserver, it is possible to play different music for each room and to select from a various audio sources. The doorbell, notifications and announcements as well as alarms are also enabled in this way.

The replaceable microSD card contains the operating system and settings. The LAN interface is used to connect to the Miniserver, and to stream radio stations or music services on the Internet, or media on the local network. The USB interface can be used to connect storage devices with music files.

The Audioserver provides two stereo speaker outputs, which can also be separated and used individually. An analogue audio input, an output and a digital (electrical) SPDIF output (all via 3.5mm jacks) are also integrated.

Using the Tree Turbo interface, every Audioserver can be expanded with [Tree Turbo Audio devices](https://www.loxone.com/help/treeturbo-interface) for additional zones or speaker outputs. For larger systems, multiple Audioservers can be used.

Using the [Audio Player function block](https://www.loxone.com/help/audioplayer) in Loxone Config, the system can be highly customized, the speaker outputs can be assigned to the individual rooms in mono or stereo mode. It is possible to create groups across rooms, allowing for a uniform music experience in open spaces.

For larger rooms, multiple speaker outputs are assigned to the same Audio Player function block. Each speaker output can be configured to output either the left or right channel signal, or a downmix of both channels.

The Audioserver is compatible with every type of the Loxone Miniserver.

[**Datasheet Audioserver**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_Audioserver_100428.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Pairing with the Miniserver](#desc0)
- [Factory Reset, Settings and Updates](#desc0.1)
- [Wiring & Topology](#TreeTurboConnect)
- [Pairing Tree Turbo Devices](#TreeTurboPair)
- [Tree Turbo Speed Requirements for Audio Devices](#TreeTurboSpeed)
- [Separate Stereo Outputs](#SeparateSpkrOutputs)
- [Line Out, SPDIF Out](#lineout)
- [Line In](#linein)
- [SD Card](#sdcard)
- [Wiring Example](#wiringexample)
- [LED Status](#led_state)
- [Selecting Power Supplies](#powersupply)
- [Network Share Security](#smbshare)
- [Speaker Installation Planning](#PlanSpkrs)
- [Inputs, Outputs, Properties](#Diagnostic)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The Audioserver is installed on a DIN rail in a suitable enclosure.

![100428 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100428 install.png)

Connect the power supply, the speakers and audio in/outputs as needed. The LAN port is used to connect the Audioserver to the local network or a WiFi router. Connect the Stereo Extensions to the Tree Turbo interface.
The [wiring examples](#wiringexample) provide a brief overview.

The Audioserver starts after switching on the power supply and is ready for operation after about 1.5 minutes. When starting for the first time, the file system on the SD card is unpacked, during this process the status LEDs remain off. Wait until the Audioserver has completed the startup process. After this initial set-up, the Audioserver is ready for pairing with the Miniserver. This is indicated by the status LED flashing red/green/orange.

**[Then follow the instructions for pairing with the Miniserver.](#desc0)**

---

## Pairing with the Miniserver

Once the Audioserver is operational and connected to the network, it can be paired with the Miniserver in Loxone Config. Pairing mode is indicated by the status LED flashing red/green/orange.

If this is not the case, reset the Audioserver to [factory settings](#desc0.1).

To search for Audioservers, first click on **Audio** in the Periphery tree of Loxone Config, and then on **Audioserver Search** in the menu bar

The window that opens will list all Audioservers that are ready for pairing. This can take a few minutes:

![11.1 AS pairing](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/11.1 AS pairing.png)

When highlighting one of the devices, it will identify by flashing its status LED or optionally also by an audible signal through the connected speakers. This allows you to easily identify and name the devices.

Select the desired Audioserver and click on **Configure Device** to assign a static IP address to the Audioserver. This can also be done via the Audioserver's [web interface](#desc0.1).

Then, select a name, room and installation location for the highlighted Audioserver and add it to the programming using the **Pair Device** or **+** button.

The right window lists all the devices that are currently part of the program. You can display them by clicking the button **Show my Devices**. You can also replace an existing device with a new device of the same type that was found in the search. This is useful when a device needs to be replaced or devices are added to a pre-configured program. Select the device to be added and the device to be replaced. By clicking on the button with the arrow pointing right, the old device is replaced with the new one in the program.

**To apply the changes, save the program in the Miniserver.**

Now the added devices are ready for use and available in the Periphery Tree in Loxone Config.

If multiple Audioservers are used, ensure they are connected within the same network segment.

---

## Factory Reset, Settings and Updates

If the Audioserver was previously paired with a Miniserver and is now to be used in another installation, the pairing must first be released. Connect to the Miniserver and delete the Audioserver from the programming. Then save the program into the Miniserver.

If this is not (or no longer) possible, reset the Audioserver to **factory settings** by [formatting an SD card for the Audioserver](https://www.loxone.com/enen/kb/restore-factory-defaults-format_sd-card/) in Loxone Config and then inserting it. Alternatively, you can reset the Audioserver to the factory settings via the web interface. Now the Audioserver is ready to be paired again.

> **ℹ️ Note:** Without these steps a previously paired Audioserver cannot be paired with another Miniserver!

The **web interface** of the Audioserver allows network configuration, status display and diagnostic options. You can access it via the IP address or the hostname of the Audioserver. When searching for Audioservers in Loxone Config, the search result includes the IP address or hostname of the Audioserver.

If the Audioserver is not yet paired with a Miniserver, the login credentials for the web interface are admin/admin.
Once the Audioserver is paired, the login credentials of a user of the user group Full Access (Administrators) of the paired Miniserver are required.

The Audioserver can perform **firmware updates** automatically, if any are available. This requires **Automatic Updates** to be enabled in the project properties. The Audioserver adopts this setting.

Alternatively, the Audioserver can be updated manually in Loxone Config. To do so, select the Audioserver in the periphery tree and click on **Update Audioserver Firmware** in the menu bar. The update can also be started from the Audioserver web interface.

Another option is to manually copy the update file (*.upd) for the Audioserver [to the SD card.](https://www.loxone.com/help/audioplayer#local) Shortly thereafter, the update will be applied from this file.

If there is no DHCP server in your network, or if the Audioserver is connected directly to a PC, link-local addressing via Zeroconf is supported. If both Audioserver and computer are set to DHCP, they will adopt a 169.254.x.x link-local address.
This way a connection to the Audioserver is possible even without a network. Although this is not suitable for normal operation, it allows you to restore the factory settings via the Audioserver's web interface, or to manually assign an IP address.

To check the network by a periodic ping, the Audioserver relies on the [ICMP](https://en.wikipedia.org/wiki/Internet_Control_Message_Protocol) service at the router/gateway. If this service does not respond, the Audioserver performs a security reboot about every 10min.

---

## Wiring & Topology

The following wiring topologies (Tree Turbo) are supported, with a maximum cable length of 150 m /492 ft:

![tree wiring topologies](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/tree wiring topologies.png)

We recommend using the Loxone Audio Cable for wiring. Use the green, green-white twisted pair for the Tree Turbo data line and the orange/white-orange pair with a cross-section of 1.5 mm² (AWG 16) for the 24 V DC power supply.

For longer cable runs or when connecting multiple Tree Turbo devices with high power consumption, additional power supplies can be installed near the devices, or multiple supply lines can be routed.

If separate power supplies are used, we recommend connecting the GNDs of all power supplies together.

![TreeTurbo cabeling](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/TreeTurbo_cabeling.png)

### Detailed wiring with the Audio Cable

**Master** Install Speaker (Install Speaker 7 Master, Install Speaker 10 Master, Install Sub 10 Master):

![TreeTurbo cabeling ISM](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/TreeTurbo_cabeling_ISM.png)

**Client** Install Speaker (Install Speaker 7 Client, Install Speaker 10 Client, Install Sub 10 Client):

![TreeTurbo cabeling ISC](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/TreeTurbo_cabeling_ISC.png)

**Satellite Speaker IP64 Master:**

![TreeTurbo cabeling Sat64M](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/TreeTurbo_cabeling_Sat64M.png)

**Satellite Speaker IP64 Client:**

![TreeTurbo cabeling Sat64C](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/TreeTurbo_cabeling_Sat64C.png)

**Stereo Extension:**

![TreeTurbo cabeling StereoExtension](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/TreeTurbo_cabeling_StereoExtension.png)

> **ℹ️ Note:** The Tree Turbo interface is based on a completely different technology than the well-known Tree interface. Therefore, the Tree and the Tree Turbo interface must not be connected! The Tree Turbo data lines should not be run together with other data or signal lines in the same cable.

The Tree Turbo communication is IP based, therefore IP addresses for all Tree Turbo devices will appear on the network.

---

## Pairing Tree Turbo Devices

To add Tree Turbo devices, first click on the desired Tree Turbo interface in Loxone Config, and then on **Tree Turbo Search**.

The window that opens will list all connected Tree Turbo devices that are not yet part of the program:

![treeturbo search](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/treeturbo_search.png)

When you highlight a device, it will identify itself by flashing its status LED or by emitting an audible signal through the speakers. This allows you to easily locate and name the device.

Select the desired device, assign a name, room and installation location and add it to the programming using the **Pair Device** or **+** button.

The right-hand section of the window lists all devices that are already part of the program. You can display them by clicking **Show my devices**.
You can also replace an existing device with a new one of the same type that was found during the search. This is useful when replacing a device or adding devices to a preconfigured program. Select both the device to be added and the one to be replaced, then click the right arrow button to replace the old device in the program with the new one.

**To apply the changes, save the program in the Miniserver.**

The added devices are now ready for use and available in the Periphery tree below the corresponding Tree Turbo interface.

---

## Tree Turbo Speed Requirements for Audio Devices

For reliable audio playback over Tree Turbo, it is important to verify data throughput using the Health Check diagnostic tool in Loxone Config.

**Recommended speed values:**
- **Above 180 Mbit/s** – Optimal performance
- **100–150 Mbit/s** – May cause audio dropouts, especially when using services such as Spotify Connect, Bluetooth, or AirPlay
- **Below 100 Mbit/s** – Can negatively affect all audio streams, depending on the number of clients and active streams

If the Tree Turbo speed is too low, please verify the following:
- We recommend using the Loxone Tree Cable or Loxone Audio Cable
- We recommend using the [Weidmüller](https://shop.loxone.com/enen/product/200469-feed-through-terminal-block-s4c-2-5-orange) terminals available in our webshop.
- Avoid parallel routing of Tree Turbo cables from different Audioservers or Miniserver Compacts. These cables must not be installed in close proximity to each other to prevent crosstalk.
- Observe the maximum cable length of 150 m /492 ft.
- The number of Tree Turbo devices is limited to 10 devices per Tree Turbo interface.

---

## Separate Stereo Outputs

The option to split one stereo output of the Audioserver or of a Stereo Extension into two separate channels allows you to cover two different rooms or areas with one speaker each. The two outputs can be used independently on separate Audio Player blocks.

To separate the channels, first click on the output of an Audioserver or a Stereo Extension in the Periphery tree, then click on the menu bar button **Separate Stereo Output**. Now two individual outputs are available in the Periphery tree. In order to merge the two outputs into a stereo output again, click on the button **Merge to Stereo Output**.

Note: If the outputs are separated, there can be slight crosstalk between the two channels.
This means that at a volume of 65% or higher, you may hear the audio signal on the adjacent channel, even if it is switched off.
For directly adjacent rooms, this effect is usually not noticeable, since at this volume the music from the adjacent room can also be heard through the walls.

The Line Out and SPDIF Out options are not available for separated outputs.

---

## Line Out, SPDIF Out

The **Line Out** (green jack socket) is an analog audio output. AV devices such as amplifiers, mixers or active speakers with analog inputs can be connected to this output. Use a 3.5mm to RCA cable.
The output volume is variable and corresponds to the volume that is currently set on the Audio Player.
Equalizer settings are also applied to the Line Out.

The **SPDIF Out** (black jack socket) is a digital electrical SPDIF output. AV devices like amplifiers or active speakers can be connected to this output. Use a 3.5mm to RCA cable, the electrical SPDIF signal is output on the left (white) RCA plug. Connect this plug to a digital coaxial audio input.
The output volume is variable and corresponds to the volume that is currently set on the Audio Player.
For a fixed output volume, select external volume mode in the settings of a Stereo Output set to SPDIF.
Equalizer settings are not applied to the SPDIF Out.

Use high-quality shielded cables for both outputs and run them separately from other cables.

The Line Out and the SPDIF Out can be activated in the properties of the respective output:

![AS speakertype](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AS_speakertype.png)

When Line Out or SPDIF Out is selected for an output, the speaker outputs are disabled and the signal is output at the Line Out or SPDIF Out instead.

The Line Out and SPDIF Out options are not available for separated outputs.

---

## Line In

The Line In (blue jack socket) is an analogue audio input. Devices with analogue audio or headphone outputs can be connected to this input.

Use a high-quality shielded 3.5mm to RCA cable, or a 3.5mm to 3.5mm cable and run it separate from other cables.

The Line In can be selected as a Source in the interface of the [Audio Player](https://www.loxone.com/help/audioplayer) function block.

### Latency and delay

When playing back audio signals from the Line In, there is a short delay.
This latency is caused by the recording, subsequent data transfer and synchronization and is at least 20ms.

If the same signal is played back on other Audioservers or Tree Turbo devices, a higher delay of more than 100 ms can occur due to the synchronization between the Audio Players. This is generally not noticeable when playing music.

Even when the sound is played back to a video, many people perceive it as synchronous with the image up to a delay of 50-100 ms, depending on the content.

### Using a microphone

The Line In is not suitable for the direct connection of a microphone.
In this case, additional equipment such as a microphone amplifier or mixer is required.

This enables applications such as announcements via a microphone, since latency is negligible here.
However, it should be ensured that the speaker does not hear himself over the loudspeakers if possible, as the delay is disturbing when speaking, or feedback can occur.

The system is not suitable for the use of a microphone in live performances, or other applications where real-time playback is required, due to latency.

### Adjusting Line-In Input Gain to Prevent Audio Distortion

Some external audio devices may deliver an analog output with a voltage level that is too high, leading to distortion in the incoming or sampled audio signal on the Audioserver. To solve this, you can manually adjust the input gain level of the line-in via an API command.

**Steps to Adjust Line-In Gain Level:**

**1. Access the Audioserver API:**
- Open your browser and navigate to: http://<ip-address-audioserver>/wsdev
- Log in using your Miniserver credentials.

**2. Modify the Line-In Gain Level:**
- In the wsdev command text field, use the following API command format: audio/cfg/props/LINEINVOLUME/<x>
- Replace <x> with your desired gain level. The gain level can be adjusted between 0 and 31 (factory default = 28).
- Example Command: To set the input gain to 25, enter: audio/cfg/props/LINEINVOLUME/25

**3. Send the Command:**
- Click the "Send" button.
- The system will display a confirmation of the updated gain value.

**4. Reboot the Audioserver:**
- To activate the new input volume settings, restart the Audioserver.

---

## SD Card

The Micro SD card, located at the top edge of the Loxone Audioserver, contains the operating system and settings.

The SD card can be removed as follows:
Gently press the visible edge of the SD card inward with your fingernail.
This will release and partially eject the card so it can be removed.

If the SD card fails to latch when inserting it, it needs to be fully removed to reactivate the latching mechanism.

---

## Wiring Example

The following image shows a simplified wiring diagram for an Audioserver and two Stereo Extensions:

![audioserver wiringexample](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/audioserver-wiringexample.png)

---

## LED Status

| Left LED | Right LED | Description |
| --- | --- | --- |
|  |  | Device is in pairing mode, ready for pairing. |
|  |  | Audioserver is booting, or no connection to the paired Miniserver is possible. |
|  |  | No network connection |
|  |  | Everything OK, device is online. |
|  |  | Device was selected in Loxone Config and is identifying. |
|  |  | Check power supply and SD card. At first start-up: File system on SD card is unpacking, wait for the process to complete. |
|  |  | No compatible operating system on the SD card. |

For the first few seconds after switching on the power supply, both status LEDs flash red and orange alternately.

> **ℹ️ Note:** Network connection, indicates data traffic.

---

## Selecting Power Supplies

When planning the power supply for audio installations, the following rule of thumb can be applied:
- **Normal sound levels** (low simultaneity factor, e.g., homes, offices, retail): **10W** per speaker.
- **Higher sound levels** (high simultaneity factor, e.g., bars, restaurants, event venues): **20W** per speaker.

---

## Network Share Security

You can define a Samba password to protect access to the shared network folder of the Audioserver or Miniserver Compact.

Three options are available: (1) disable network share, (2) enable network share without password, (3) enable network share with password.

If a password is configured, the username for authentication is 'audio'

---

## Speaker Installation Planning

More information about speaker planning can be found [here](https://www.loxone.com/enen/products/audio).

Loxone Install Speakers require installation in a closed enclosure or a cavity, such as in ceilings or walls, to fully develop their sound volume.

Suitable **[mounting enclosures for drywall or concrete installation](https://shop.loxone.com/enus/audio.html/?c=install-box)** are available in the Loxone Shop.

![PH Loxone Installation recommendations table](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PH-Loxone-Installation-recommendations-table.png)

While an enclosure is not strictly required for fully enclosed surfaces, speakers must be installed in a rear mounting enclosure when used in open surface structures, such as acoustic ceilings.

The required acoustic installation volume varies based on the speaker size and type:

| Speaker type | Minimal volume | Recommended volume |
| --- | --- | --- |
| Install Speaker 7 | 7.2l | 9l or more |
| Install Speaker 10 | 14.5l | 18l or more |
| Install Sub 10 | 18l | 30l |

Larger enclosures or cavities may also be used, provided they are closed.

### Speaker Quantity

In main living areas, at least two speakers should be used to achieve good sound quality. For small rooms or ancillary spaces, a single speaker is usually sufficient.
Depending on the room size, we recommend planning the following number of speakers per room:

![RoomsizeSpkrCountChart](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RoomsizeSpkrCountChart.png)

### Ceiling installation

Plan the speaker installation positions to be evenly distributed throughout the room. A minimum distance of 50cm/20″ from walls should be maintained to avoid sound reflections.

In ceiling installations, the stereo effect is barely noticeable and can often be neglected. Therefore, a full stereo signal is later assigned to each individual speaker via the connection to the Audio Player block (Downmix).

### Wall installation

At the most frequently used listening position, at least two speakers should be arranged to achieve a good stereo effect.
The left-right assignment of the speakers is made later through the connection to the Audio Player block.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Audioserver 1 | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Serial Number | Specifies the serial number of the device. | - | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - |
| Local Address | Address or hostname of the Audioserver or audio device e.g.: 192.168.1.7 or as1234. This setting configures the address the Miniserver will connect to but does not change the configuration of the audio device itself. | - | - |
| Notify of error as a voice message | If the selected source cannot be played or there is an error a voice message giving the reason for the error will be played in the corresponding zone. | - | - |
| Network Share | Configure the password protection for the network share | - | - |
| Line In ID | Use this ID on the input LineIn of an Audioplayer to select the Line In as source. | 1...100 | 1 |
| Device Configuration | Edit device settings. A connection to the Miniserver is required. | - | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted on a DIN rail in an electrical distribution enclosure to ensure protection against contact, water and dust.

Only mount the device on a horizontal DIN rail to ensure heat dissipation by convection.

---

## Documents

[**Datasheet Audioserver**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_Audioserver_100428.pdf)

[Loxone Ports & Domains](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Loxone-Required-Ports-Domains.pdf)

[Datasheet Install Speaker 7 Passive](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_InstallSpeaker7Passive_100497.pdf)

[Datasheet Install Speaker 10 Passive](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_InstallSpeaker10Passive_610148.pdf)

[Thermal Shutdown Temperatures](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Thermal_Shutdown_Temperatures.pdf)

---