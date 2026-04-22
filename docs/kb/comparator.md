# Comparator

Source: https://www.loxone.com/enen/kb/comparator/

---

On the comparator the output (De) is activated as soon as the difference of the values of the inputs (V1) and (V2) exceed the value of parameter (Von) and deactivated as soon as this difference falls below the value of parameter (Voff).

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| V1 | Value 1 | ∞ |
| V2 | Value 2 | ∞ |

---

## Outputs

| Abbreviation | Description | Value Range |
| --- | --- | --- |
| De | 1 when difference exceeded | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Von | On-value | Input parameter - On-value of the comparator | ∞ | 5 |
| Voff | Off-value | Input parameter - Off-value of the comparator | ∞ | 1 |

---

## Timing Diagram

![Comparator timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Comparator-timediag.png)