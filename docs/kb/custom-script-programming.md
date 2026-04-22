# Custom Script Programming

Source: https://www.loxone.com/enen/kb/custom-script-programming/

---

Allows the realization of complex functions and processes in script language Pico C.
Note that if an error is detected in the program, the Miniserver may reboot to ensure data consistency. Therefore, a high level of programming skill in C is a must.
This control is for developers only, therefore no support is provided.

A maximum of 8 program blocks are supported.

[**Custom Script Programming**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/CustomScriptProgramming.pdf)

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| T1-3 | Text input 1-3 | - |
| I1-13 | Input 1-13 | ∞ |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Txt1-3 | Text output 1-3 | Maximum length for text output: 4096 bytes | - |
| O1-13 | Output 1-13 | ∞ |
| Etxt | Error text | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Program code | Program code editor | - |

---