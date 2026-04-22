# MinMax

Source: https://www.loxone.com/enen/kb/minmax/

---

The MinMax block determines the lowest and highest value at its analogue inputs, and outputs these two values at the outputs.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Application Examples](#baseconf)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| Min | Current minimum value | ∞ |
| Max | Current maximum value | ∞ |

---

## Parameters

| Abbreviation | Summary | Value Range | Default Value |
| --- | --- | --- | --- |
| V1-4 | Value 1-4 | ∞ | 0 |

---

## Application Examples

The following is an example of how to use the block:

![minmaxbase](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/minmaxbase.png)

The function block is provided with two values, the result is available at the outputs.

The inputs (V1) and (V2) are always analyzed, inputs (V3) and (V4) only if they are used.
The next example illustrates this:

![minmaxbase2](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/minmaxbase2.png)

(V1) is included in the analysis, the value 0 is used. (V4) is not included in the analysis, because nothing is connected.

By **cascading** the function block more than four inputs can be processed:

![minmaxcascade](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/minmaxcascade.png)

Since Loxone Config 14.5, the block also processes several values per input:

![MIM MinMax](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/MIM_MinMax.png)