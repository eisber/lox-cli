# Alarm Chain

Source: https://www.loxone.com/enen/kb/alarm-chain/

---

Creates a 10-level Alarm Sequence with text outputs. After the set reaction time, the next step becomes active. If the alarm continues to be active after the last stage, the chain starts over again.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Use with Loxone Caller Service](#caller)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| A | Alarm | ON: alarm start, OFF: alarm end | 0/1 |
| Au | Urgent alarm | ON: alarm start at all outputs (A1-10), OFF: alarm ends at all outputs (A1-10). If (A) and (Au) are (ON), (Au) is dominating. | 0/1 |
| AEs | Alarm emergency service | ON: emergancy alarm start, OFF: emergency alarm ends if (A) and (Au) are OFF. | 0/1 |
| T1-3 | Alarm texts 1-3 | Text provided on this input can be used in the alarm messages of output (A1-10). | - |
| Ca | Confirm alarm | Pulse: Resets all alarm outputs. On = Alarm outputs are locked. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| A1-10 | Alarm chain stage 1-10 | Text output for alarm chain level 1-10. Is skipped if not connected. | - |
| AEs | Alarm text emergency services | ON when the alarm chain have reached their maximum repetition (MaxR) or when input (AEs) is activated. | - |
| As | Current alarm stage | Number of the current level of the alarm chain (A1-10) -1 = All alarm levels active | -1...10 |
| Ton | Time of last alarm start | - |
| Toff | Time of last alarm stop | - |
| Cc | Cause of confirmation | - |
| Tc | Time of confirmation | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rt | Reaction time | Time before next level is activated. If input (Au) is enabled then this parameter refers to the duration before outputs are activated again. 0 = All alarm outputs are activated at the same time and only once. | s | 0...∞ | 60 |
| MaxR | Maximum repetitions | The maximum number of times the alarm chain is repeated. 0 = Unlimited repetitions | - | 0...∞ | 4 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Alarm text emergency services | Is provided at output (AEs) when active. If this text is empty, the default alarm text is used. The following values can be used: : Name of the block  - : Text at inputs (T1-3) : Time of the alarm when (A) or (Au) is activated : Customer name from the project settings : Customer address from project settings | - |
| Alarm text | Is provided at outputs (A1-10) when active. The following values can be used: : Name of the block  - : Text at inputs (T1-3)  - : Value without decimal places at inputs (T1-3)  - : Value with 2 decimal places at inputs (T1-3) : Time of the alarm when (A) or (Au) is activated : Customer name from the project settings : Customer address from project settings : Time in seconds before activating output (AEs) | - |

---

## Use with Loxone Caller Service

The outputs (A1-10) can be used with the Loxone Caller Service. However, please be aware of the limitations of the number and frequency of calls that can be made. Further information about this can be found [here](https://www.loxone.com/help/caller).