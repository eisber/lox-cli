# Ramp Controller

Source: https://www.loxone.com/enen/kb/ramp-controller/

---

Ramp control using 2 levels

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |
| S | Step selection - 0 = (L1), 1 = (L2) | 0/1 |
| St | Stop | When 1 (V) = (Sv) | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| V | Value | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |
| Sv | Start value | When 1 (V) = (Sv) | ∞ | 5 |
| Sts | Step size | (V) is changed by this value every 100ms until the target value is reached. | 0...∞ | 1 |
| L1 | Step 1 | ∞ | 7 |
| L2 | Step 2 | ∞ | 3 |

---