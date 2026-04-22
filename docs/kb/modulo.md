# Modulo

Source: https://www.loxone.com/enen/kb/modulo/

---

Divides analogue values as integers or decimals and outputs the remainder of the division.

## Table of Contents
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Calculation of outputs](#calcoutputs)

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Int | Integer | Result of the integer calculation | ∞ |
| Dec | Decimals | Result of the decimal calculation | ∞ |

---

## Parameters

| Abbreviation | Summary | Value Range | Default Value |
| --- | --- | --- | --- |
| V1-2 | Value 1-2 | ∞ | 0 |

---

## Calculation of outputs

(Int) outputs the result of the integer calculation and (Dec) of the decimal calculation in the corresponding data type.

![modulo calc](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/modulo_calc.png)

(Int): 13 mod 5 = 3
(Dec): 13.3 mod 5.2 = 2.9