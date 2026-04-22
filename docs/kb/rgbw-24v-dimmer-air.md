# RGBW 24V Dimmer Air

Source: https://www.loxone.com/enen/kb/rgbw-24v-dimmer-air/

---

With the RGBW Dimmer Air, extra low-voltage LED fittings and LED strips can be dimmed using PWM and any useable colours can be mixed. The 4 outputs can be configured as one RGBW channel, or can be used as 4 separate channels.

The device is also available in a compact housing (RGBW Compact Dimmer Air). The functions are identical.

[**Datasheet RGBW 24V Dimmer Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RGBW24VDimmerAir_100125.pdf)

[**Datasheet RGBW 24V Compact Dimmer Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RGBW24VCompactDimmerAir_100324.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Individual channels](#channels)
- [Smart Tunable White](#SmartTW)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

![100125 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100125 install.png)

The RGBW Dimmers is installed on a DIN rail in a suitable enclosure. The Compact Dimmer can be used without a separate enclosure. The cable should be kept as short as possible to reduce voltage drop.

The wire size should be selected so that the voltage drop is not more than 1 V. This can be determined using the following formula:

ΔU = I · R = I · ((2 · L · ρ) / A)

I … current [A], L … cable length [m], A … wire cross-section [mm²], ΔU … voltage drop [V], ρ … resistance [( Ω*mm²)/m], ρ= constant (0,0172 for copper)

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for two minutes, then pairing mode is activated for 30 minutes.

---

## Individual channels

If "Individual channels " is selected as actuator type in the properties of the device, the channel assignment is as follows:

Output 1 - Channel Red, output 2 - Channel Green, output 3 - Channel Blue, output 4 - Channel White

![RGBW 24V Dimmer Air WW](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RGBW_24V_Dimmer_Air_WW.png)

---

## Smart Tunable White

Tunable White (TW) refers to lighting with white light that is adjustable in color temperature.
For this purpose, light sources such as LED strips are available that combine warm white light and cool white light.
The color temperature can be adjusted from warm white to cool white by mixing the two colors.

For this purpose, Loxone Devices with RGBW Dimming outputs support the actuator type Smart Tunable White:

![stw actorselect](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/stw_actorselect.png)

Two outputs of the dimmer for cool white (CW) and warm white (WW) are combined per actuator.
Smart Tunable White actuators are supported by the Lighting Control block.

Two Smart TW actuators or one Smart TW actuator and two individual channels can be configured.
The assignment of the connections is as follows:

![stw channelconnect](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/stw_channelconnect.png)

In the settings of the Smart TW actuator, the color temperature of the light source is set for warm white and cool white:

![stw colorselect](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/stw_colorselect.png)

This information can be found in the technical data of the light source.

---

## Actuators

| Summary | Description | Value Range |
| --- | --- | --- |
| Smart actuator RGBW | Smart actuator | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status RGBW 24V Dimmer Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Actuator Type | Use device with Standard Actuator(s) or Smart Actuator(s) Smart Actuators support dynamic fade times and can only be used with the Lighting Controller. | - |
| Switch off status LEDs | If checked, the status LEDs on the device are switched off in normal operation. In the event of a fault, the device will continue to alert you to its status LEDs. | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

The installation requires a suitable enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet RGBW 24V Dimmer Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RGBW24VDimmerAir_100125.pdf)

[**Datasheet RGBW 24V Compact Dimmer Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RGBW24VCompactDimmerAir_100324.pdf)

---