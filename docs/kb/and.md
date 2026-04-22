# AND

Source: https://www.loxone.com/enen/kb/and/

---

This function block creates a logic AND operation. The output will be active when all connected inputs are active.
Multiple signals connected to the same input will also be AND-linked.

![And table](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/And_table.png)

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Timing Diagram](#timediag)
- [Functionality](#function)

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

Logic AND operation based on truth table.

![And timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/And-timediag.png)

---

## Functionality

Both examples will result in the same output.

![And function](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/And_function.jpg)

Even with a negation, the upper circuit works the same as the lower one (Attention: the negation has to be done a the right AND function block)

![And negation](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/And_negation.jpg)