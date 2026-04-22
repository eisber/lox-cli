# Pulse Width Modulation

Source: https://www.loxone.com/enen/kb/pulse-width-modulation/

---

Pulse width modulator to convert analog values into digital signals.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| V | Value | 0...10 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| PWM | PWM output | 0/1 |

---

## Parameters

| Abbreviation | Summary | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| P | Period | s | 0...∞ | 1 |

---

## Timing Diagram

![PWM timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PWM-timediag.png)