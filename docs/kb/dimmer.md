# Dimmer

Source: https://www.loxone.com/enen/kb/dimmer/

---

Optimised dimmer with double click function for single button operation.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Presence Simulation](#PresenceSimulation)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tg | Toggle | Toggles output (D) between 0 and last value (or between 0 and 100 when parameter (Lv) is 1). Long-click for dimming up/down. | 0/1 |
| + | Dim+ | Toggles output (D) between 0 and last value (or between 0 and 100 when parameter (Lv) is 1). Long-click for dimming up. | 0/1 |
| - | Dim- | Toggles output (D) between 0 and last value (or between 0 and 100 when parameter (Lv) is 1). Long-click for dimming down. | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Tg), (+), (-) when On. (e.g Child lock, cleaning) Control via user interface is still possible. | 0/1 |
| Set | Sets output (D) to value of input (Set) | ∞ |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| D | Dimmer output | Analog output for dimmer control (e.g. 0-10V) | ∞ |
| S | Status | On when output (D) is greater than 0. | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Rem | Remanence input | Remanence input: If active, the function block retains its previous state after a Miniserver reboot. The state of the function block is saved: – When saving to the Miniserver – At a planned reboot – Before a backup – Once per hour The data is saved on the SD card. | 0/1 | 0 |
| Di | Delay input | Delays input for set time. If this parameter is set too low, the block may interpret the input as a held button press rather than a normal (short) click. Applies to inputs (Tg), (+), (-). | ∞ | 0,4 |
| Sts | Step size | ∞ | 0,5 |
| Str | Step rate | Dim value is in-/decreased by (Sts) every (Str) seconds on long-click. | ∞ | 0,2 |
| MinD | Minimum dim value | ∞ | 0 |
| MaxD | Maximum dim value | ∞ | 100 |
| Dm | Dim mode | 1 = Dim between parameters (MinD) and (MaxD) on long-click. 0 = Stop dimming when reaching parameter (MinD) or parameter (MaxD) on long-click. Only applies to input (Tg). | 0/1 | 0 |
| Lv | Last value | 1 = Sets output (D) to (MaxD) when switching on. 0 = Sets last value to ouput (D) when switching on. | 0/1 | 0 |

---

## Presence Simulation

This function block has a presence simulation.
Activate and define the presence simulation in the properties window:

![PresenceSimulation Dimmer](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PresenceSimulation_Dimmer.png)