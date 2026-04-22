# Moving Average

Source: https://www.loxone.com/enen/kb/moving-average/

---

The function block takes the value from the input every 'C' seconds and places it in a buffer that has a maximum of 'N' entries. The oldest entry from this buffer is always removed. The average value is calculated from these entries.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| V | Value | Analogue input from which the moving average is calculated. | ∞ |
| R | Reset | Deactivates the averaging function. Output (Avg) is equal to the value at input (V). | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Avg | Average | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| C | Polling cycle | Used to set the interval at which the value on the input read to then be averaged. | s | 0...∞ | 60 |
| N | Number of readings | Number of values used for calculating average. | - | 0...1000 | 60 |

---