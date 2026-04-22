# Combined Window Contact

Source: https://www.loxone.com/enen/kb/combined-window-contact/

---

This function block combines the signal of multiple window contacts used to determine the difference between open and tilted into one combined signal.
According to where on the window a window sensor in installed (it's position), the inputs then determine the right analog value to be passed on to the Door and Window Monitor block.
The block is designed so that only one sensor can be connected to each of the inputs (Open) & (Tilt)!

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Open | 0 = closed, 1 = open | 0/1 |
| Tilt | 0 = closed, 1 = tilt | 0/1 |
| Secured | 0 = not secured, 1 = secured | 0/1 |

---

## Outputs

| Abbreviation | Description | Value Range |
| --- | --- | --- |
| S | 1 = closed, 2 = tilted, 3 = open, 4 = closed and not locked, 5 = closed and locked, 0 = one or more sensors offline | ∞ |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Sensor position | Indicates the position of the sensor on the frame for both the open and tilt contacts to produce the combined signal. | - |

---