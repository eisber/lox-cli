# Sauna Controller

Source: https://www.loxone.com/enen/kb/sauna-controller/

---

Sauna controller without Evaporator
With this Function Block a sauna can be optimally automated.
All standard Sauna heaters can be controlled (switching of the heating elements, as well as analog control via 0-10 V signal).
Also integrated in the module is an hourglass function, fan and post-drying, as well as a safety shutdown, if you forget to switch off the heater.

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
| Tg | Toggle | Toggles between Sauna On, Heating, Drying, Airing, Sauna Off. | - | 0/1 |
| ϑt | Target temperature | Min: 30°C Max: 120°C | ° | ∞ |
| ϑc | Current temperature | ° | ∞ |
| Fan | Toggle fan | Switches fan on / off. The fan can only be switched on when the sauna is On. | - | 0/1 |
| St | Activate sand timer | Activates the sand timer for the duration set in parameter (Std). Each subsequent pulse on the input restarts the timer. | - | 0/1 |
| Dc | Door contact | The door state is only used for display in the user interface! 0 = opened, 1 = closed. | - | 0/1 |
| ϑb | Current temperature bench | If connected, the bench temperature is used as current temperature. | ° | ∞ |
| P | Presence | Used for safety shutdown. If no presence is detected, the sauna will shutdown automatically after the duration set in parameter (Ssdt). | - | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Tg), (Fan), (St) when On. (e.g Child lock, cleaning) Control via user interface is still possible. | - | 0/1 |
| On | On | Activate Sauna | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| So | Sauna output (0-10V) | Analog output 0-10V for sauna control. | V | 0...10 |
| L1-3 | Sauna phase output (1-3) | Phase output (L1-3) for sauna control. | - | 0/1 |
| On | Sauna state | On as long as sauna and drying phase are active. | - | 0/1 |
| Fan | Fan | Output for fan control. | - | 0/1 |
| Stt | Sand timer remaining time | s | 0...∞ |
| Dry | Drying state | On as long as drying and airing are active. | - | 0/1 |
| Ssd | Safety shutdown | Pulse when the current temperature exceeds the value set in parameter (Ssdϑ). | - | 0/1 |
| ϑt | Target temperature | Outputs the target temperature. | ° | 30...120 |
| Stoff | Sand timer end | Pulse when sand timer ends. | - | 0/1 |
| St | Sand timer state | On when sand timer is active. | - | 0/1 |
| Ready | Sauna ready | Pulse when target temperature is reached. | - | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| ϑd | Temperature deviation | Deviation current temperature to seat temperature (if the input current bench temperature (ϑb) is not used). | ° | ∞ | 0 |
| Dryϑ | Drying phase target temperature | The temperature required to start the fan when in drying phase. | ° | ∞ | 70 |
| Dryd | Drying phase duration | Fan duration after reaching the drying phase temperature (Dryϑ). | s | 0...∞ | 1800 |
| Std | Sand timer duration | s | 0...∞ | 900 |
| Ssdϑ | Safety shutdown temperature | If exceeded, all outputs are switched off, except for the output (Ssd). | ° | ∞ | 139 |
| Ssdt | Safety shutdown time | The sauna is automatically switched off at the set time. If input (P) is used, the time starts to run when presence is no longer detected. | s | 0...∞ | 7200 |
| PWMp | PWM period | Specifies the PWM period for the phase outputs (L1-3). | s | 0...∞ | 180 |
| G | Gain | Controller gain for the PWM modulated output. If the value is decreased, the temperature control responds slower, if it is increased, it responds faster. If necessary, change the value in small steps to adapt the control to the sauna. | - | 0...∞ | 1 |
| Pm | Phase mode | Number of phases used: Off = 3 phases On = 1 phase Parameter is only displayed if phase outputs (L2) and (L3) are used. | - | 0/1 | 0 |

---

## Application

Using this block, a Sauna can be Automated Optimally. All common sauna stoves can be controlled. (Switching of the Heating Elements, as well as Analogue control via 0-10V Signal). Also integrated in the block is a Sand Timer function, Airing, and Drying. There is also a Safety Shutdown, if you forget to switch off the heater. In Addition, the Sauna module can be operated via the Web Interface/App and, therefore, remotely. Please ensure you follow the relevant Safety and Legal Regulations for your respective Country.

---

## Basic Programming

The sauna temperature sensor is linked to input (ϑc).

The output (So) and the phase outputs (L1-3) are used to connect the respective outputs. If the sauna heater requires only one phase, use (L1). The fan is connected to output (Fan).

Phase outputs (L1-3) are pulse-width modulated by the controller to emulate an analog value. For example, a value of 20% means that the outputs are on for 20% of the period and off for 80%.

---

## No Remanence

For security reasons, the block has no remanence.
Therefore, it is always switched off after a Miniserver restart.