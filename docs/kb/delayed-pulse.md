# Delayed Pulse

Source: https://www.loxone.com/enen/kb/delayed-pulse/

---

Provides a delayed pulse at the output (delay adjustable)

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| P | Pulse | 0/1 |
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
| Dd | Duration of delay | s | 0...∞ | 5 |
| Dp | Duration output pulse | s | 0...∞ | 0,5 |

---

## Timing Diagram

![DelayedPulse timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/DelayedPulse-timediag.png)