# Analogue Value Monitor

Source: https://www.loxone.com/enen/kb/analogue-watchdog/

---

The function block checks an analog input (V) for exceeding or falling below thresholds.
The output (Te) is activated as soon as the value of input (V) is outside of the tolerance range.
The tolerance range is defined by the parameter inputs upper threshold (TU) and lower threshold (TL).

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
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Te | 1 when threshold exceeded | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |
| TU | Upper threshold | ∞ | 7 |
| TL | Lower threshold | ∞ | 3 |

---

## Timing Diagram

![AnalogValueMonitor timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AnalogValueMonitor-timediag.png)