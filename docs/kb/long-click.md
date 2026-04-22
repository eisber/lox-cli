# Long Click

Source: https://www.loxone.com/enen/kb/long-click/

---

Multiple function push-button with up to four functions, pulse after click

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Tr | Trigger | 0/1 |
| R | Reset | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| O1 | Pulse when (Tr) ≤ (TI) | 0/1 |
| O2 | Pulse when (TI) ≤ (Tr) ≤ 2x(TI) | 0/1 |
| O3 | Pulse when 2x(TI) ≤ (Tr) ≤ 3x(TI) | 0/1 |
| O4 | Pulse when 3x(TI) ≤ (Tr) | 0/1 |
| V | Value (V1-4), depending on time of input (Tr) | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| TI | Interval | s | 0...∞ | 0,35 |
| D | Duration output pulse | s | 0...∞ | 0,1 |
| V1 | Value 1 | - | ∞ | 1 |
| V2 | Value 2 | - | ∞ | 2 |
| V3 | Value 3 | - | ∞ | 3 |
| V4 | Value 4 | - | ∞ | 4 |

---

## Timing Diagram

![LongClick timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/LongClick-timediag.png)