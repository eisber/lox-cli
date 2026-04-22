# Edge-Triggered Wiping Relay

Source: https://www.loxone.com/enen/kb/edge-triggered-wiping-relay/

---

Provides a fixed number of adjustable pulses at the output for every pulse on input

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
| P | Pulse | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| Don | Duration on | s | 0...∞ | 1 |
| Doff | Duration off | s | 0...∞ | 2 |
| C | Cycles | Number of cycles | - | 1...∞ | 10 |

---

## Timing Diagram

![EdgeTriggeredWipingRelay timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/EdgeTriggeredWipingRelay-timediag.png)