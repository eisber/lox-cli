# Multi-Click

Source: https://www.loxone.com/enen/kb/multiclick/

---

Multiple function push-button with up to four functions, pulse after click
The multi-click program block determines whether a button has been pressed one, two, three, or four times.
So you can, for example, create a function where a triple-click performs a different function to a double-click.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tr | Trigger | Sends a pulse to the output (1C-4C) depending on the number of clicks. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| 1C | Pulse on single-click | Pulse on single-click for the duration set in parameter (On). | 0/1 |
| 2C | Pulse on double-click | Pulse on double-click for the duration set in parameter (On). | 0/1 |
| 3C | Pulse on triple-click | Pulse on triple-click for the duration set in parameter (On). | 0/1 |
| 4C | Pulse on quad-click | Pulse on quad-click for the duration set in parameter (On). | 0/1 |
| V | Value triggered output | Sets the value (V1c-V4c) of the triggered output. | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | - | 0/1 | 0 |
| Tmc | Time multi-click | Maximum time between two pulses to count as multi-click. | s | 0...∞ | 0,35 |
| On | On-duration of output (1C-4C) | s | 0...∞ | 0,1 |
| V1c | Value for single-click | - | ∞ | 1 |
| V2c | Value for double-click | - | ∞ | 2 |
| V3c | Value for triple-click | - | ∞ | 3 |
| V4c | Value for quad-click | - | ∞ | 4 |

---