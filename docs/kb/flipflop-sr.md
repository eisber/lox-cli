# SR Flipflop

Source: https://www.loxone.com/enen/kb/flipflop-sr/

---

Flipflop with toggle input. Set is dominant.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| S | Set | A pulse switches Output (O) on, dominating input | 0/1 |
| Tg | Toggle | A pulse toggles Output (O) | 0/1 |
| R | Reset | A pulse switches Output (O) off | 0/1 |

---

## Outputs

| Abbreviation | Description | Value Range |
| --- | --- | --- |
| O | Output | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |

---

## Timing Diagram

![SRFlipFlop timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/SRFlipFlop-timediag.png)