# Monoflop

Source: https://www.loxone.com/enen/kb/monoflop/

---

Provides an output pulse of duration (D) for any length of input pulse

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tr | Trigger | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| O | Output | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| D | Duration output pulse | Input parameter - duration of output pulse | s | 0...∞ | 2 |

---

## Timing Diagram

![Monoflop timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Monoflop-timediag.png)