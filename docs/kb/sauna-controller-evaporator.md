# Sauna Controller with Evaporator

Source: https://www.loxone.com/enen/kb/sauna-controller-evaporator/

---

Sauna controller with Evaporator
Using this Function Block, a Sauna with an Evaporator can be intelligently automated
All standard Sauna heaters can be controlled (switching of the heating elements, as well as analog control via 0-10 V signal).
An Evaporator can be switched On / Off, but also controlled via an analog 0-10 V signal.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Application](#usage)
- [Basic Programming](#basic)
- [No Remanence](#noRem)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Mode | Select sauna mode | 0 = Off / Manual 1 = Finnish manual 2 = Humidity manual 3 = Finnish sauna 4 = Herbal sauna 5 = Soft steam bath 6 = Hot-air bath | - | 0...6 |
| Fim | Finnish manual | Switches to this mode. (Temperature: manual, Humidity: Off) | - | 0/1 |
| Hum | Humidity manual | Switches to this mode. (Temperature: manual, Humidity: manual) | - | 0/1 |
| Fin | Finnish sauna | Switches to this mode. (Temperature: 80°C, Humidity: Off) | - | 0/1 |
| Her | Herbal sauna | Switches to this mode. (Temperature: 45°C, Humidity: 50%) | - | 0/1 |
| Sof | Soft Steam bath | Switches to this mode. (Temperature: 50°C, Humidity: 50%) | - | 0/1 |
| Hot | Hot-air bath | Switches to this mode. (Temperature: 45°C, Humidity: 45%) | - | 0/1 |
| Tg | Toggle | Toggles between Sauna On, Heating, Drying, Fan, Sauna Off. | - | 0/1 |
| ϑt | Target temperature | Min: 30°C Max: 110°C (Finnish manual) Max: 70°C (Humidity manual) | ° | ∞ |
| ϑc | Current temperature | ° | ∞ |
| Ht | Target humidity | % | 15...65 |
| Hc | Current humidity | % | 0...100 |
| Fan | Toggle fan | Switches fan on / off. The fan can only be switched on when the sauna is On. | - | 0/1 |
| St | Activate sand timer | Activates the sand timer for the duration set in parameter (Std). Each subsequent pulse on the input restarts the timer. | - | 0/1 |
| Dc | Door contact | The door state is only used for display in the user interface! 0 = opened, 1 = closed. | - | 0/1 |
| ϑb | Current temperature bench | If connected, the bench temperature is used as current temperature. | ° | ∞ |
| P | Presence | Used for safety shutdown. If no presence is detected, the sauna will shutdown automatically after the duration set in parameter (Ssdt). | - | 0/1 |
| Ws | Water shortage | If the water supply is low, the Evaporator is switched off. | - | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Tg), (Fan), (St) when On. (e.g Child lock, cleaning) Control via user interface is still possible. | - | 0/1 |
| On | On | Activate Sauna | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| So | Sauna output (0-10V) | Analog output 0-10V for sauna control. | - | ∞ |
| L1-3 | Sauna phase output (1-3) | Phase output (L1-3) for sauna control. | - | 0/1 |
| Ev | Evaporator output (0-10V) | Analog output 0-10V for Evaporator control. | - | ∞ |
| Evd | Evaporator digital output | Digital output for Evaporator control. | - | 0/1 |
| On | Sauna state | On as long as sauna and drying phase are active. | - | 0/1 |
| Fan | Fan | Output for fan control. | - | 0/1 |
| Stt | Sand timer remaining time | s | 0...∞ |
| Dry | Drying phase | - | 0/1 |
| Mode | Current sauna mode | 0 = Off / Manual 1 = Finnish manual 2 = Humidity manual 3 = Finnish sauna 4 = Herbal sauna 5 = Soft steam bath 6 = Hot-air bath | - | ∞ |
| Ssd | Safety shutdown | Pulse when the temperature exceeds the value set in parameter (Ssdϑ). | - | 0/1 |
| ϑt | Target temperature | Outputs the target temperature. | ° | ∞ |
| Ht | Target humidity | Outputs the target humidity. | % | 15...65 |
| Stoff | Sand timer end | Pulse when the sand timer ends. | - | 0/1 |
| St | Sand timer state | On when sand timer is active. | - | 0/1 |
| Ready | Sauna ready | Pulse when target temperature is reached. | - | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| ϑd | Temperature deviation | Deviation current temperature to seat temperature (if the input current bench temperature (ϑb) is not used). | ° | ∞ | 0 |
| Dryϑ | Drying phase temperature | The temperature required to start the fan when in drying phase. | ° | ∞ | 70 |
| Dryd | Drying phase duration | Fan duration after reaching the drying phase temperature (Dryϑ). | s | 0...∞ | 1800 |
| Std | Sand timer duration | s | 0...∞ | 600 |
| Ssdϑ | Safety shutdown temperature | If exceeded, all outputs are switched off, except for the output (Ssd). | ° | ∞ | 139 |
| Ssdt | Safety shutdown time | The sauna is automatically switched off at the set time. If input (P) is used, the time starts to run when presence is no longer detected. | s | 0...∞ | 7200 |
| PWMp | PWM period | Specifies the PWM period for the phase outputs (L1-3). | s | 0...∞ | 180 |
| G | Gain | Controller gain for the PWM modulated output. If the value is decreased, the temperature control responds slower, if it is increased, it responds faster. If necessary, change the value in small steps to adapt the control to the sauna. | - | 0...∞ | 1 |
| Pm | Phase mode | Number of phases used: 0 = 3 phases 1 = 1 phase 2 = 2 phases in Evaporator mode or 3 phases in mode without Evaporator. Parameter is only displayed if phase outputs (L2) and (L3) are used. | - | 0...2 | 2 |

---

## Application

Using this block, a Sauna can be Automated Optimally. All common sauna stoves can be controlled. (Switching of the Heating Elements, as well as Analogue control via 0-10V Signal). Also integrated in the block is a Sand Timer function, Airing, and Drying. There is also a Safety Shutdown, if you forget to switch off the heater. You can choose between different Operating modes such as: Hot Air Bath, Herbal Sauna, Finnish Sauna, etc… In Addition, the Sauna module can be operated via the Web Interface/App and, therefore, remotely. Please ensure you follow the relevant Safety and Legal Regulations for your respective Country.

---

## Basic Programming

The sauna temperature sensor is linked to input (ϑc) and the humidity sensor to (Hc).

The output (So) and the phase outputs (L1-3) are used to connect the respective outputs. If the sauna heater requires only one phase, use (L1). The fan is connected to output (Fan).

Phase outputs (L1-3) are pulse-width modulated by the controller to emulate an analog value. For example, a value of 20% means that the outputs are on for 20% of the period and off for 80%.

The steam generator is connected either to the output (Ev) (analog 0-10V) or to (Evd) (digital on/off) depending on the type of control.

---

## No Remanence

For security reasons, the block has no remanence.
Therefore, it is always switched off after a Miniserver restart.