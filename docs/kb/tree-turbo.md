# Tree Turbo

Source: https://www.loxone.com/enen/kb/tree-turbo/

---

Tree Turbo is a high-performance interface that enables the connection of audio devices such as the [Stereo Extension](https://www.loxone.com/help/stereo-extension), [Install Speaker 7 Master](https://www.loxone.com/help/IS7M), [Install Speaker 10 Master](https://www.loxone.com/help/IS10M), [Install Sub 10 Master](https://www.loxone.com/help/Sub10M), and [Satellite Speaker IP64 Master](https://www.loxone.com/help/Sat64M) to the [Audioserver](https://www.loxone.com/help/audioserver) or the [Miniserver Compact](https://www.loxone.com/help/miniserver-compact).
Each Tree Turbo interface supports up to 10 devices with a maximum cable length of 150 m /492 ft.

## Table of Contents
- [Wiring & Topology](#TreeTurboConnect)
- [Pairing Tree Turbo Devices](#TreeTurboPair)
- [Tree Turbo Speed Requirements for Audio Devices](#TreeTurboSpeed)

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