# Push Switch

Source: https://www.loxone.com/enen/kb/push-switch/

---

Converts a switch to a push button and generates a pulse on each edge (switch operation).

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tr | Trigger | Pulse at input switches output (O) on for the duration set in parameter (Don). | 0/1 |
| Off | Off | Switches output (O) off. Permanent 1 = Locks function block. | 0/1 |
| On | On | Switches output (O) on. | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Tr), (Off), (On) when On. (e.g Child lock, cleaning) | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O | Output | Pulse on each edge at input (Tr) for the duration set in parameter (Don). | 0/1 |
| Off | Falling edge input (Tr) | Pulse on falling edge at input (Tr) for parameter (Don). | 0/1 |
| On | Rising edge input (Tr) | Pulse on rising edge at input (Tr) for parameter (Don). | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Don | On-duration of outputs (O), (Off), (On) | s | 0...∞ | 0,02 |

---

## Timing Diagram

![Switch2Button Timechart](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Switch2Button_Timechart.jpg)