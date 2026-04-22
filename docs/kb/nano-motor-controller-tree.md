# Nano Motor Controller Tree

Source: https://www.loxone.com/enen/kb/nano-motor-controller-tree/

---

The Nano Motor Controller Tree is a compact module with DC outputs, controlled via the Loxone Tree interface.

Its configurable outputs allow the control of DC motors or LEDs in the following [operating modes](#modes):

**Motor bidirectional:** One output for DC motors with control of direction and speed.

**Motor unidirectional:** Two outputs for DC motors and speed control, no direction change possible.

**Dimmer:** One output for dimming low voltage LED lights.

![100473 picture](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100473 picture.png)

An internal H-bridge is used to switch the outputs and reverse their polarity. Motor speed and dimming are controlled through pulse-width modulation.

[**Datasheet Nano Motor Controller Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_NMCTree_100473.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Operating modes](#modes)
- [Notes on operating motors](#motors)
- [Set current threshold values](#nmcdiag)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Install the device in a suitable installation box.

![100473 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100473 install.png)

Connect the power supply (orange/white screw terminal) and Tree communication wires (green/white terminals).

The supply voltage will depend on the load, but must be in the range of 9...26VDC.

The selected [operating mode](#modes) specifies how the outputs are connected.

Shortly after power-up, the status LED will blink orange if the wiring is correct (connection to Tree Extension and Miniserver is established).

---

## Commissioning

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Operating modes

The Nano Motor Controller supports three operating modes that can be set in Loxone Config. They differ in function and how the outputs are connected:

### Motor Bidirectional

This operating mode provides one output for DC motors with control of direction and speed.
The rotation direction can be changed by the Nano Motor Controller by reversing the polarity of the outputs:

![NMCmodeBidir](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/NMCmodeBidir.png)

This is useful for applications where a change of rotation direction is required, e.g. motors for shades, curtains, or motorized windows.

### Motor Unidirectional

This operating mode provides two output for DC motors with speed control, the outputs are independently controllable.
The rotation direction is determined by the connection:

![NMCmodeUnidir](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/NMCmodeUnidir.png)

This is useful for applications where two motors are to be controlled separately.

### Dimmer

This operating mode provides one output for dimming low voltage LED lights:

![NMCmodeDimmer](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/NMCmodeDimmer.png)

This is useful for lights like LED strips or spots that operate at a constant voltage, e.g. 12V or 24V and can be dimmed by pulse width modulation (PWM).

---

## Notes on operating motors

The speed and direction of motors are controlled directly via their supply voltage.
The voltage at the outputs of the Nano Motor Controller is pulse-width modulated to control speed, and in bidirectional mode it reverses polarity to change the direction.

True DC motors, that have a commutator and brushes, are especially well suited.

Brushless DC motors are often equipped with an internal controller which makes control of the motor speed via PWM control of the supply voltage impossible.
Such motors can only be switched on or off. The speed must be set to 100%, the acceleration value can be set to Jump.
Reversing the rotation direction is often not possible, and there is a risk of damaging the motor electronics if it is not designed for changing the direction and does not have reverse polarity protection.

Connecting motors in parallel to one output is only possible under certain conditions.
It must be ensured that only motors of the same type are used, and that they are equally loaded.

The overcurrent shutdown of the Nano Motor Controller may trigger despite a high overcurrent limit value when motors with a high moment of inertia start or decelerate (brake).
This can be avoided by decreasing the value for acceleration and braking to e.g. 20%/s in order to reduce the current drawn by the motor during start-up and deceleration.
During the first second of motor start-up, the set overcurrent threshold is ignored and is fixed at 5A. If this is too short, the overcurrent limit can be increased up to 5A to allow operation of motors with longer start-up times. The operation of motors in the overcurrent range of 2.2 - 5A is allowed for max. 30 seconds.

When stopping the motor it does not coast to a stop, but due to technical reasons the Nano Motor Controller applies a braking force.
If coasting to a stop without a braking force is desired, the value for deceleration must be set to a lower value, resulting in a more natural coasting.

---

## Set current threshold values

From the Nano Motor Controller's properties you can open a diagram for setting the current thresholds:

![nmcdiag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/nmcdiag.png)

A graph is drawn using the current drawn by the motor. The threshold values for current flow and overcurrent can be calibrated by performing a test run of the motor or drive used.

It is recommended to not set the threshold values too close, in case the motor load and current draw changes somewhat due to wear or temperature influence.

The values are then applied by saving the program to the Miniserver.

---

## Sensors

| Summary | Description | Value Range |
| --- | --- | --- |
| Current Flow | Input activates when the current drawn by the motor is above the threshold value for current flow (bidirectional operating mode) | 0/1 |
| Overcurrent | Input activates when the current drawn by the motor is above the threshold value for overcurrent (bidirectional operating mode) | 0/1 |
| Current flow A | Input activates when the current drawn by motor A is above the threshold value for current flow A (unidirectional operating mode) | 0/1 |
| Current flow B | Input activates when the current drawn by motor B is above the threshold value for current flow B (unidirectional operating mode) | 0/1 |
| Overcurrent A | Input activates when the current drawn by motor A is above the threshold value for overcurrent A (unidirectional operating mode) | 0/1 |
| Overcurrent B | Input activates when the current drawn by motor B is above the threshold value for overcurrent B (unidirectional operating mode) | 0/1 |

---

## Actuators

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Clockwise Rotation | Output activates the motor clockwise rotation A+/B- (bidirectional operating mode) | - | 0/1 |
| Anticlockwise Rotation | Output activates the motor anticlockwise rotation A-/B+ (bidirectional operating mode) | - | 0/1 |
| Start/Stop A | Output activates motor A (unidirectional operating mode) | - | 0/1 |
| Start/Stop B | Output activates motor B (unidirectional operating mode) | - | 0/1 |
| Speed | Output specifies the speed for the motor (bidirectional operating mode) If the output is not used, the default speed is used | % | 0...100 |
| Speed A | Output specifies the speed for motor A (unidirectional operating mode) If the output is not used, the default speed is used | % | 0...100 |
| Speed B | Output specifies the speed for motor B (unidirectional operating mode) If the output is not used, the default speed is used | % | 0...100 |
| Dimmer WW | Standard actuator with one channel to control lighting (dimmer operating mode) | % | 0...100 |
| Smart Actuator WW | Smart Actuator WW to control lighting, use on compatible lighting blocks (dimmer operating mode) | - | - |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Nano Motor Controller Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| System temperature | Provides the internal device temperature. This is often the temperature of the CPU or another location in the device. | ° | ∞ |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - | - |
| Operating mode | Specifies the operating mode of the Nano Motor Controller. Motor bidirectional: One output for DC motors with control of direction and speed. Motor unidirectional: Two outputs for DC motors and speed control, no direction change possible. Dimmer: One output for dimming extra low voltage LED lights. | - | - | - |
| Current threshold values | Configure the current threshold values using a graph of the measured current. | - | - | - |
| PWM frequency | Frequency of the pulse width modulation. Used to adapt to the motor. This can eliminate whistling noises, for example. | Hz | 1000...10000 | 5000 |
| Current flow threshold | Current flow is detected above this value. | mA | 100...5000 | 100 |
| Overcurrent threshold | When this value is exceeded, overcurrent is detected, the output switches off and the overcurrent input is activated. During the first second of motor start-up, the set overcurrent threshold is ignored and is fixed at 5A. The operation of motors in the overcurrent range of 2.2 - 5A is allowed for max. 30 seconds. (Motor start-up) The total current is relevant. | mA | 100...5000 | 3500 |
| Speed up | Rate of change when powering on. Jump means that the target value is set immediately. | - | - | - |
| Slow down | Rate of change when slowing down. Jump means that the target value is set immediately. | - | - | - |
| Default speed | Default speed when the speed output is not used. | % | 0...100 | 100 |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

The installation requires a suitable enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Nano Motor Controller Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_NMCTree_100473.pdf)

[Thermal Shutdown Temperatures](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Thermal_Shutdown_Temperatures.pdf)

---