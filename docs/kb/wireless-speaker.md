# Wireless Speaker

Source: https://www.loxone.com/enen/kb/wireless-speaker/

---

The Wireless Speaker is an audio solution designed for both standalone use and seamless integration into a Loxone. It's a portable speaker featuring a 2-way system with passive radiator. It is powered by 24V and receives its audio signal via Wi-Fi or Bluetooth.

**[Datasheet Wireless Speaker](https://pim.loxone.com/datasheet/610165-wireless-speaker)**

## Table of Contents
- [Commissioning](#Commissioning)
- [LED Status](#led_state)
- [Factory Reset](#reset)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

To set up the speaker, connect the power adapter and plug it into a wall outlet. When the LED turns solid orange, use your phone or tablet to join the "**wspXXXX**" Wi‑Fi network. If the setup page does not open automatically, open a web browser and go to 192.168.123.1 to add the speaker to your local network. Note that pairing mode is active for 30 minutes after the speaker is powered on, requiring a reboot to connect if this time has expired. After successful configuration, the speaker will restart and can then be used as a standalone player with **AirPlay or Spotify Connect**.

For integration into a Loxone system, ensure the speaker and Miniserver are connected to the same network. The speaker is ready for pairing when the Status LED cycles red, orange, and green. In Loxone Config, run an Audio Device Search, or use the Loxone App (Device Search) to run a Network Speaker Search. Follow the prompts to complete the room assignment and configuration. You can perform the entire setup directly through the Loxone App, with no need to access Loxone Config.

---

## LED Status

> **ℹ️ Note:** Booting

**Wi-Fi standalone mode**

| Status LED | Description | LED States |
| --- | --- | --- |
|  | Blue flashing for 10 seconds | Bluetooth connected |
|  | Green flashing for 10 seconds | AirPlay / Spotify Connect playback |

---

## Factory Reset

If the Wireless Speaker was previously paired with a Miniserver and is now to be used in another installation, the pairing must first be released. Connect to the Miniserver and delete the Wireless Speaker from the programming. Then save the program into the Miniserver.

If this is not (or no longer) possible, you can reset the Wireless Speaker to factory settings. After the speaker has fully booted, press the reset button for 1 second using a paper clip or similar object. Avoid applying excessive force while pressing the button. A confirmation sound will indicate that the reset was successful. Alternatively, you can restore factory settings via the web interface. After the reset is complete, the Wireless Speaker is ready to be paired again.

![wireless speaker resetbutton](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wireless_speaker_resetbutton.png)

> **ℹ️ Note:** Without these steps a previously paired Wireless Speaker cannot be paired with another Miniserver!

The **web interface** of the Wireless Speaker provides access to network settings, status information, and diagnostic tools. You can access the interface using either the speaker’s IP address or its hostname. When searching for audio devices in Loxone Config, the search results include the IP address or hostname of the Wireless Speaker.

If the Wireless Speaker is not yet paired with a Miniserver, the login credentials for the web interface are admin/admin.
Once the Wireless Speaker is paired, the login credentials of a user of the user group Full Access (Administrators) of the paired Miniserver are required.

The Wireless Speaker can perform **firmware updates** automatically when new versions become available. This requires **Automatic Updates** to be enabled in the project properties. The Wireless Speaker will automatically adopt this setting.

---

## Actuators

| Summary | Value Range |
| --- | --- |
| Wireless Speaker 1 | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Amplifier limit | If the temperature of the amplifier reaches a critical point, the volume of the zone is reduced. This may be due to overloading or excessively high ambient temperature. | - | 0/1 |
| Online Status Wireless Speaker 1 | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Serial Number | Specifies the serial number of the device. | - | - | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Maximum Volume | Determines the maximum (physical) output power of the amplifier in percent, thus limiting the maximum possible volume for this output. The volume values from 0-100% of the Audio Player or App are scaled accordingly. | % | 0...100 | 100 |
| Gain | Adjusts the volume of this output in decibels. This control helps balance the sound levels across different speakers or environments, ensuring consistent audio output. The volume is scaled and limited according to the specified maximum volume. | - | -6...6 | 0 |

---

## Safety Instructions

The device is maintenance-free and should only be cleaned with a dry cloth. It is designed for indoor use within a temperature range of -10°C to 50°C (14°F to 122°F).

---

## Documents

**[Datasheet Wireless Speaker](https://pim.loxone.com/datasheet/610165-wireless-speaker)**

---