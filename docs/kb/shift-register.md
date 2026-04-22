# Shift Register

Source: https://www.loxone.com/enen/kb/shift-register/

---

Delay of digital pulses

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Basic Programming](#TimeTable)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tr | Trigger | With each pulse the value of (D) is stored and the register contents are shifted. | 0/1 |
| D | Data | Data input of shift register | 0/1 |
| Dir | Shift direction | Shift direction of the shift register | 0/1 |

---

## Outputs

| Abbreviation | Description | Value Range |
| --- | --- | --- |
| O | Output | 0/1 |

---

## Parameters

| Abbreviation | Summary | Value Range | Default Value |
| --- | --- | --- | --- |
| Rb | Register bit | ∞ | 8 |

---

## Basic Programming

For each pulse at (Tr), the value of the data input (D) is stored in accordance with the shift direction (Dir) and the register contents are shifted in the corresponding direction beforehand. Parameter (Rb) is used to select the register bit whose value is passed on to the output (O).

![Shift TimeTable](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Shift_TimeTable.jpg)