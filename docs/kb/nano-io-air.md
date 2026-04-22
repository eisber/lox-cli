# Nano IO Air

Source: https://www.loxone.com/enen/kb/nano-io-air/

---

The Loxone Nano IO Air has 2 relay outputs and 6 digital inputs (24VDC), for installation in an electrical back box. The Touch for Nano keypad module is available as an option.

[**Datasheet Nano IO Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_NanoIOAir_100153.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Alternative operation with 24V](#supply)
- [Wiring the digital inputs](#digital_inputs)
- [Touch for Nano](#touch)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Install the device in a suitable installation box. Connect the device according to the following wiring diagram:

![100153 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100153 install.png)

In this operating mode power is supplied via mains voltage (110-230V AC) and the outputs of the Nano IO then also supply mains voltage. In this case, they are not suitable for DC or low voltages. Mains voltage loads can be connected directly.

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.

---

## Alternative operation with 24V

Install the device in a suitable installation box. Connect the device according to the following wiring diagram:

![100153 install24V](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100153 install24V.png)

In this operating mode the Nano IO is supplied by an external 24VDC power supply via the 24V in and GND terminals. In this case, the Nano IO must not be connected to mains voltage.

The two outputs L' and L" can be compared to volt-free relay contacts in this operating mode. For example, 12V AC can be switched via the relays to control an electric door strike. L is the common contact of the two relays and L' and L" are the switched contacts. N must not be connected in this operating mode.

---

## Wiring the digital inputs

6 digital inputs are available. The 24V of the Nano IO are switched for example via push buttons and connected to the digital inputs:

![100153 install inputs](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100153 install inputs.png)

The 24V output of the Nano IO must not be loaded with more than 1W by the sensors. For higher loads, a separate power supply must be used for the sensors. The ground potentials (GND) of the Nano IO and the power supply must be connected in such cases.

---

## Touch for Nano

The optional plug-on modules Touch for Nano and Touch Pure for Nano feature five touch points to control the most important functions of a room. When a button is touched, an audible click confirms the action. Another compatible module is the NFC Code Touch for Nano, a keypad for entering codes and reading NFC tags, used for access control.

The plug-on module mounting frame is snapped onto the Nano IO and then screwed onto the installation box. Finally, the Touch for Nano is plugged on along with the frame.

![100154 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100154 install.png)

When using the Touch for Nano or NFC Code Touch for Nano plug-on module, it must be activated in the properties of the Nano IO Air in Loxone Config. The inputs will then be available in the peripheral tree.

> **ℹ️ Note:** The Touch for Nano's large centre touch zone is ideal for controlling the lighting, while the corner zones are suitable for controlling music and shading. It is based on the Loxone switch standard. The buttons can also be freely used for other applications. To use the individual buttons as inputs, activate the checkboxes in the Properties window. The audible confirmation can also be disabled here.

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
| T5 | - | ∞ |

---

## Actuators

| Summary | Unit | Value Range |
| --- | --- | --- |
| Relay 1 | Digital | 0/1 |
| Relay 2 | Digital | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Nano IO Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| System temperature | Provides the internal device temperature. This is often the temperature of the CPU or another location in the device. | ° | ∞ |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Disable Repeater functionality | Disable repeater functionality of this Air device. Loxone Air is based on mesh technology. Any air device connected to the power supply can repeat packets from other Air devices, thus extending the range and stability of the overall system. In large systems with a large number of air devices in a confined space, the communication between the air devices can lead to a very high radio channel utilization. A reliable accessibility of the air devices can not be guaranteed. Disabling repeater functionality on individual Air devices can help. Do not disable this function recklessly as this may affect the range and stability of the system. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Application | Specifies the intended use of the actuators. Universal: Outputs can be used freely Shading: Auto-Configuration will create a Automatic Shading function block | - |
| Expansion module | Expansion module for Nano IO Air | - |
| Button Behaviour | Specifies the behavior when a button is pressed. Pulse: Sends a pulse on rising edge OnOff: Sends ON on rising edge and OFF on falling edge, used for long click Automatic: Sends a pulse on rising edge for buttons 1 & 4 (shading) and 3 (lighting). Sends ON on rising edge and OFF on falling edge for buttons 2 & 5 (music) to enable volume up/down via long press | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

The installation requires a suitable enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Nano IO Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_NanoIOAir_100153.pdf)

[Datasheet Relay](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_Relay_HF32FA-G.pdf)

[Datasheet Touch for Nano](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchforNano_100154.pdf)

**[Datasheet Touch Pure for Nano White / ](https://pim.loxone.com/datasheet/100801-touch-pure-nano-white)****[Anthracite / ](https://pim.loxone.com/datasheet/100802-touch-pure-nano-anthracite)****[Gold](https://pim.loxone.com/datasheet/100803-touch-pure-nano-gold)**

[Datasheet Touch Pure Classic for Nano](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchPureNano_100661,100586.pdf)

[Datasheet NFC Code Touch for Nano](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_NFCCodeTouchforNano_100301.pdf)

[Thermal Shutdown Temperatures](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Thermal_Shutdown_Temperatures.pdf)

---