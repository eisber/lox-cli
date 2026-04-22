# Shading Actuator Air

Source: https://www.loxone.com/enen/kb/shading-actuator-air/

---

The Shading Actuator Air is a wireless device for mains voltage controlled via Loxone Air technology. The two relay outputs can be used to connect either shading devices or other loads. The device is equipped with Hirschmann STAK3/STAS3 connectors and can be plugged into the supply line of the shading device.

[**Datasheet Shading Actuator Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_ShadingActuatorAir_100290.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Programming](#Programmierung)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Install the device at the desired location and connect the plugs.

![100290 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100290 install.png)

Wiring Example Shading:

![ShadingActuatorAir Shading](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ShadingActuatorAir_Shading.png)

Wiring Example Lighting:

![ShadingActuatorAir Lighting](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ShadingActuatorAir_Lighting.png)

> **ℹ️ Note:** Install the device and cables in a way that they will not be damaged by the shading device.

> **ℹ️ Note:** In order to prevent water ingress, careful assembly is necessary. Make sure that the gaskets on both connectors are properly positioned, the safety brackets are locked, and the cable glands are tight. If possible, position the device so that the Loxone logo faces up and the device is protected from heavy rain, submersion and standing water.

Finally, switch on the power supply.

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by a short audible click of the relays or a brief up/down movement of the shading device.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for two minutes, then pairing mode is activated for 30 minutes.

---

## Programming

In the properties under Application, you can select whether to use the Shading Actuator Air for Shading or as a Universal actuator.

When used as a Shading Actuator, drag and drop the device onto the programming page to create a [Automatic Shading](https://www.loxone.com/help/auto-shading) function block. In the properties of the function block, the Shading Actuator is already assigned.

In order to learn the travel times, the shading device must fully travel three times in opposite directions. (fully open - fully closed - fully open). The travel must not be interrupted. The times are measured and automatically transferred to the Automatic Shading block.

When used as a universal actuator, two freely programmable digital outputs and one digital input for current flow are available. The threshold value for the current flow cannot be changed.

---

## Actuators

| Summary | Unit |
| --- | --- |
| API Connector | Text |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Shading Actuator Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
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
| Automatically set drive times | Automatically learn travel durations for complete travels (Application: Shading). For learning, perform three uninterrupted complete travels to an end position. Open-close-open or close-open-close. This is not possible for motors with less than 90W power. In this case, please deactivate the option and set the travel durations in the Shading Block via the parameters (Opd) and (Cld). | - |
| Inverted Direction | Invert direction. Required when Up/Down connection is made the wrong way round. | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

---

## Documents

[**Datasheet Shading Actuator Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_ShadingActuatorAir_100290.pdf)

[Thermal Shutdown Temperatures](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Thermal_Shutdown_Temperatures.pdf)

---