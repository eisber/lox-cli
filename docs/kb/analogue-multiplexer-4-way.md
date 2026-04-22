# Analogue Multiplexer (4-Way)

Source: https://www.loxone.com/enen/kb/analogue-multiplexer-4-way/

---

This block can select between 4 analog values via a parameter input (Sel).

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| V1-4 | Value 1-4 | ∞ |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| V | Value | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Sel | Select value | 0: (V) = 0 1: (V) = (V1) ... 4: (V) = (V4) | 0...4 | 1 |

---