# Pulse by

Source: https://www.loxone.com/enen/kb/pulse-by/

---

The block analyzes the text at the T input using user-defined search criteria.

Up to four character strings can be defined as search patterns. Wildcards can be used as placeholders.

If the text contains one of the search patterns, output (P) provides a pulse with the set length.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

> **ℹ️ Note:** Text

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| P | Pulse when search pattern is found. | 0/1 |

---

## Parameters

| Abbreviation | Summary | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Pd | Pulse Duration | s | 0...∞ | 1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Search Pattern 1 | If the search pattern is included in the text at input T, a pulse is generated at output P. The two wildcards '*' and '?' can be used ('*' = sequence of any characters of any length, '?' = any character). Example: '???nder*' applies to 'grinder', but also to 'thunderstorm' | - |
| Search Pattern 2 | If the search pattern is included in the text at input T, a pulse is generated at output P. The two wildcards '*' and '?' can be used ('*' = sequence of any characters of any length, '?' = any character). Example: '???nder*' applies to 'grinder', but also to 'thunderstorm' | - |
| Search Pattern 3 | If the search pattern is included in the text at input T, a pulse is generated at output P. The two wildcards '*' and '?' can be used ('*' = sequence of any characters of any length, '?' = any character). Example: '???nder*' applies to 'grinder', but also to 'thunderstorm' | - |
| Search Pattern 4 | If the search pattern is included in the text at input T, a pulse is generated at output P. The two wildcards '*' and '?' can be used ('*' = sequence of any characters of any length, '?' = any character). Example: '???nder*' applies to 'grinder', but also to 'thunderstorm' | - |

---