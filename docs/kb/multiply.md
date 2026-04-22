# Multiply

Source: https://www.loxone.com/enen/kb/multiply/

---

Multiplies two or more analog values and makes the result available at output (O).

Since Loxone Config 14.5, the block also processes several values per input:

![MIM Multiply](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/MIM_Multiply.png)

If one of the two inputs is disconnected or reads 0, the 0 value will be ignored. In this case, only the remaining connected input will be multiplied.

## Table of Contents
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Outputs

| Abbreviation | Description | Value Range |
| --- | --- | --- |
| O | (O) = (V1) x (V2) | ∞ |

---

## Parameters

| Abbreviation | Summary | Value Range | Default Value |
| --- | --- | --- | --- |
| V1-2 | Value 1-2 | ∞ | 0 |

---