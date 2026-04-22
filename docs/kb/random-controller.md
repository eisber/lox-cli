# Random Controller

Source: https://www.loxone.com/enen/kb/random-controller/

---

Creates random On and Off signals with varying intervals

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| En | Enable | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Ran | Random output | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| Son | Maximum duration switch-on delay | Input parameter - maximum duration switch-on delay | s | 0...∞ | 100 |
| Soff | Maximum duration switch-off delay | Input parameter - maximum duration switch-off delay | s | 0...∞ | 10 |

---

## Timing Diagram

![RandomController timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RandomController-timediag.png)