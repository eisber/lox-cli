# Toilet Ventilation Controller

Source: https://www.loxone.com/enen/kb/toilet-ventilation-controller/

---

The Toilet Ventilation Controller works in "Sessions". A pulse at the input (Tg) will start or end the session.
Output (S) is switched on directly after being triggered. Output (Fan) switches on after the delay set in Parameter (Fsd).
If input (P) is used, the session starts with a rising edge at input (P) and ends the session at a falling edge, delayed by the duration set in parameter (FPet).

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tg | Toggle | Start / end session. | 0/1 |
| P | Presence | Start session when 1. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |
| DisPc | Disable periphery control | Disables all inputs when On. (e.g Child lock, cleaning) Control via user interface is still possible. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| S | Session status | On as long as the session is active. | 0/1 |
| Fan | Fan | Output for fan control. | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| Fsd | Fan start delay | Delay before the fan is switched on after the session started. | s | 0...∞ | 30 |
| FPet | Fan / Movement extend time | 1. Time starts with the falling edge of output (S) and extends output (Fan) by set time. 2. When using input (P), time starts with the falling edge of input (P) and extends outputs (S) and (Fan) by set time. | s | 0...∞ | 180 |

---