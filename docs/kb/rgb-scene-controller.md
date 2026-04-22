# RGB Lighting Controller

Source: https://www.loxone.com/enen/kb/rgb-scene-controller/

---

RGB lighting controller with one- or two-button control. The scenes can be cycled through with the "+" input.
At the outputs AQr, AQg, and AQb, the colors are then split into RGB, or output together at the output AQa.
By double-clicking on the block, the light scenes can be defined and edited.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| AI | Analogue input RGB %-value red + %-value green * 1000 + %-value blue * 1000000 | ∞ |
| + | Trigger next scene | Next RGB lighting scene | 0/1 |
| - | Trigger previous scene | Previous RGB lighting scene | 0/1 |
| AIs | Scene | Input for selecting RGB lighting scene(0-x) | ∞ |
| Dis | Disable | Disables all inputs (child lock) | 0/1 |
| R | Reset | Resets the lighting scene The name of the connected sensor is used in the user interface. | 0/1 |
| O | On | All ON Sets all outputs to maximum (white) | 0/1 |

---

## Outputs

| Abbreviation | Description | Value Range |
| --- | --- | --- |
| AQr | Analogue output for the red LED | ∞ |
| AQg | Analogue output for the green LED | ∞ |
| AQb | Analogue output for the blue LED | ∞ |
| AQs | Analogue Output - Value indicating currently active scene | ∞ |
| AQa | Analog output RGB %-value red + %-value green * 1000 + %-value blue * 1000000 | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Lighting scenes | Lighting scene management | - |

---