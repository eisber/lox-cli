# Nano Relay Air

Source: https://www.loxone.com/enen/kb/nano-relay-air/

---

The Loxone Nano Relay Air features a latching relay output, for installation in an electrical switch/outlet box. The Touch for Nano is an optional plug-on keypad.

It is controlled via the Loxone Air wireless technology.

[**Datasheet Nano Relay Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_NanoRelayAir_100426.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Touch for Nano](#touch)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Install the device in a suitable installation box. Connect the device according to the following wiring diagram:

![100426 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100426 install.png)

The power is supplied by mains voltage, the integrated relay can directly switch mains voltage loads.

The status LED, the pairing button and the connector for the optional Touch for Nano plug-on module are located on the front:

![100426 front](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100426 front.png)

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.

---

## Touch for Nano

The optional plug-on module Touch for Nano features five touch points to control the most important functions of a room. When a button is touched, an audible click confirms the action.

The plug-on module snaps onto the Nano Relay Air. The assembly is then mounted to the switch/outlet box with the supplied screws.

![TouchForNanoUS PlugOn](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/TouchForNanoUS-PlugOn.png)

When using the Touch for Nano plug-on module, it must be activated in the properties of the Nano Relay Air in Loxone Config. The combined button input T5 will then be available in the peripheral tree.

> **ℹ️ Note:** The Touch for Nano's large centre touch zone is ideal for controlling the lighting, while the corner zones are suitable for controlling music and shading. It is based on the Loxone switch standard. The buttons can also be freely used for other applications. To use the individual buttons as inputs, activate the checkboxes in the Properties window. The audible confirmation can also be disabled here.

---

## Sensors

> **ℹ️ Note:** ∞

---

## Actuators

| Summary | Unit | Value Range |
| --- | --- | --- |
| Nano Relay Air | Digital | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Nano Relay Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| System temperature | Provides the internal device temperature. This is often the temperature of the CPU or another location in the device. | ° | ∞ |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Disable Repeater functionality | Disable repeater functionality of this Air device. Loxone Air is based on mesh technology. Any air device connected to the power supply can repeat packets from other Air devices, thus extending the range and stability of the overall system. In large systems with a large number of air devices in a confined space, the communication between the air devices can lead to a very high radio channel utilization. A reliable accessibility of the air devices can not be guaranteed. Disabling repeater functionality on individual Air devices can help. Do not disable this function recklessly as this may affect the range and stability of the system. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Expansion module | Expansion module for Nano IO Air | - |
| Button Behaviour | Specifies the behavior when a button is pressed. Pulse: Sends a pulse on rising edge OnOff: Sends ON on rising edge and OFF on falling edge, used for long click Automatic: Sends a pulse on rising edge for buttons 1 & 4 (shading) and 3 (lighting). Sends ON on rising edge and OFF on falling edge for buttons 2 & 5 (music) to enable volume up/down via long press | - |
| Touch Rotation | Specifies the physical rotation of the Touch device (0°, 90°, 180°, or 270° clockwise). By default, this is set according to the country code. Adjust only if the device was installed in a different orientation. | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

The installation requires a suitable enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Nano Relay Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_NanoRelayAir_100426.pdf)

[**Datasheet Relay**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Relay_HF115F-L_1_pole_en.pdf)

[**Datasheet Touch for Nano**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchforNanoUS_100427.pdf)

---