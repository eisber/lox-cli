# Exclusive OR

Source: https://www.loxone.com/enen/kb/exclusive-or/

---

This function block creates a logic XOR operation. The output will be active when the connected inputs have unequal values.
Note that multiple signal connected to same input will be OR-linked.

![Xor table](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Xor_table.png)

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| I1 | Input 1 | 0/1 |
| I2 | Input 2 | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| O | Output | 0/1 |

---

## Timing Diagram

Logical XOR operation based on a truth table.

![XOR timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/XOR-timediag.png)