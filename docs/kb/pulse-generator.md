# Pulse Generator

Source: https://www.loxone.com/enen/kb/pulse-generator/

---

Creates pulses at the output with adjustable On/Off (Mark/Space) durations

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |
| Inv | Invert | Invert input of the pulse generator, inverts output (P) | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| P | Pulse | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| Don | Duration On | s | 0.1...∞ | 1 |
| Doff | Duration Off | s | 0.1...∞ | 1 |

---

## Timing Diagram

![PulseGenerator timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PulseGenerator-timediag.png)