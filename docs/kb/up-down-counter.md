# Up-Down Counter

Source: https://www.loxone.com/enen/kb/up-down-counter/

---

Counting Up/Down with trigger and direction input

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| C | Count | With each pulse, the counter is increased/decreased by 1 | 0/1 |
| Dir | Direction | 0 = up, 1 = down | 0/1 |
| R | Reset | When 1 (O) = 0 and (V) = (Sv) | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O | Output | Switches dependent on the parameter (Von) & (Voff) | 0/1 |
| V | Counter value | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |
| Sv | Start value | Input parameter - start value of the ramp control | ∞ | 0 |
| Von | On-value | ∞ | 10 |
| Voff | Off-value | ∞ | 5 |

---

## Timing Diagram

![UpDownCounter timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/UpDownCounter-timediag.png)