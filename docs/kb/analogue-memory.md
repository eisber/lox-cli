# Analogue Memory

Source: https://www.loxone.com/enen/kb/analogue-memory/

---

The analog value on the input is placed at the output upon trigger and will be held there.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| V | Value | ∞ |
| Set | Set value to output (V) | On rising edge. | 0/1 |
| Off | Off | Pulse: Output is reset / switched off. | 0/1 |

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

---

## Timing Diagram

![AMemory Table](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AMemory_Table.jpg)