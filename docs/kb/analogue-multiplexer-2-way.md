# Analogue Multiplexer (2-Way)

Source: https://www.loxone.com/enen/kb/analogue-multiplexer-2-way/

---

This block can select between 2 analog values via a parameter input (Sel).

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| V1 | Value 1 | ∞ |
| V2 | Value 2 | ∞ |
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
| Sel | Select value | 0: (V) = (V1) 1: (V) = (V2) | 0/1 | 0 |

---