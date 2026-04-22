# Up Counter

Source: https://www.loxone.com/enen/kb/counter/

---

Simple counter with limit
**Tip:** If (M) = 1 the counter will only start over if a reset (Off) is triggered or (M) is set to 0.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| C | Count | Pulse increases (V) by 1. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Lr | 1 when (V) = (L) | 0/1 |
| V | Counter value | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |
| L | Limit | ∞ | 1000 |
| M | 0 = counter loops automatically, 1 = counter stops at limit | 0/1 | 0 |

---