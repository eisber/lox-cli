# Edge Detection

Source: https://www.loxone.com/enen/kb/edge-detection/

---

This function block provides a pulse when the value of a digital signal changes. It can be distinguished between rising and falling edge.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Timing Diagram](#timediag)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| I | Input | Input at which edges are detected | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| P | Pulse at every edge | Outputs a pulse at every edge. | 0/1 |
| On | Pulse at rising edge | Outputs a pulse at a rising edge. | 0/1 |
| Off | Pulse at falling edge | Outputs a pulse at a falling edge. | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Pd | Pulse duration | Pulse duration at the outputs when an edge was detected. | s | 0...∞ | 1 |

---

## Timing Diagram

![EdgeDetection timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/EdgeDetection-timediag.png)