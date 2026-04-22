# Multi Extension Air

Source: https://www.loxone.com/enen/kb/multi-extension-air/

---

The Loxone Multi Extension Air features 8 dry contact latching relay outputs, 12 digital inputs, a 1-wire interface and 4 PWM outputs to control RGBW LEDs.

The connection to the Miniserver is via the Loxone Air wireless interface.

[**Datasheet Multi Extension Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MultiExtensionAir_100116.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [RGBW Outputs](#RGBW)
- [Note](#notes)
- [1-wire Interface](#1winterface)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

The Multi Extension Air is installed on a DIN rail in a suitable enclosure.

![100116 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100116 install.png)

> **ℹ️ Note:** For thermal reasons the total load rating must not exceed 48A (IEC) or 45A (UL).

> **ℹ️ Note:** The Multi Extension Air uses latching relays. Relays of this type can remain in the ON position even when the device is disconnected from power.

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.

---

## RGBW Outputs

The RGBW outputs of the Multi Extension Air support the PWM dimming of low voltage LED lamps in RGBW, Tunable White or single channels.
Alternatively, the outputs can be used for a PWM control signal with adjustable frequency.

The lamps must be supplied externally with the required voltage. The + terminal does not output any voltage, but optionally serves as a terminal point for the positive pole:

![multi rgbw](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/multi_rgbw.png)

### Tunable White

Tunable White (TW) refers to lighting with white light that is adjustable in color temperature.
For this purpose, light sources such as LED strips are available that combine warm white light and cool white light.
The color temperature can be adjusted from warm white to cool white by mixing the two colors.

For this purpose, Loxone Devices with RGBW Dimming outputs support the actuator type Smart Tunable White:

![stw actorselect](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/stw_actorselect.png)

Two outputs of the dimmer for cool white (CW) and warm white (WW) are combined per actuator.
Smart Tunable White actuators are supported by the Lighting Control block.

Two Smart TW actuators or one Smart TW actuator and two individual channels can be configured.
The assignment of the connections is as follows:

![multi smart tw](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/multi_smart_tw.png)

In the settings of the Smart TW actuator, the color temperature of the light source is set for warm white and cool white:

![stw colorselect](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/stw_colorselect.png)

This information can be found in the technical data of the light source.

---

## Note

The digital inputs can also be used as frequency meters. This allows for pulse counters such as electricity or gas meters to be connected. Since the counted pulses are only transmitted once per minute, the Multi Extension is not suitable for the connection of pulse output wind sensors.

---

## 1-wire Interface

1-Wire sensors can be connected to the 1-Wire interface of the device.
More about this in the **[Documentation of the 1-Wire Extension](https://www.loxone.com/help/1-wire#1wconnect)**, the properties of the interface are identical.

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
| Input 9 | Digital | 0/1 |
| Input 10 | Digital | 0/1 |
| Input 11 | Digital | 0/1 |
| Input 12 | Digital | 0/1 |

---

## Actuators

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Smart actuator RGBW | Smart actuator | - | ∞ |
| Relay 1 | Digital | 0/1 |
| Relay 2 | Digital | 0/1 |
| Relay 3 | Digital | 0/1 |
| Relay 4 | Digital | 0/1 |
| Relay 5 | Digital | 0/1 |
| Relay 6 | Digital | 0/1 |
| Relay 7 | Digital | 0/1 |
| Relay 8 | Digital | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Multi Extension Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| System temperature | Provides the internal device temperature. This is often the temperature of the CPU or another location in the device. | ° | ∞ |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Actuator Type | Use device with Standard Actuator(s) or Smart Actuator(s) Smart Actuators support dynamic fade times and can only be used with the Lighting Controller. | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

The installation requires a suitable enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Multi Extension Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_MultiExtensionAir_100116.pdf)

[Datasheet Relay](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_Relay_RT9S0015WG.pdf)

[Thermal Shutdown Temperatures](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Thermal_Shutdown_Temperatures.pdf)

---