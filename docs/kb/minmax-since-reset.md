# MinMax since Reset

Source: https://www.loxone.com/enen/kb/minmax-since-reset/

---

The minimum and maximum values at input (V) are determined and output at the outputs (Min) and (Max).

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| V | Value | Analogue input, the values of which are used to determine the minimum and maximum. | ∞ |
| R | Reset | Resets (Min) and (Max) to the current value (V). | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Min | Minimum | ∞ |
| Max | Maximum | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |

---