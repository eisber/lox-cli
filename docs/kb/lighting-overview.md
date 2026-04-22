# Lighting Central

Source: https://www.loxone.com/enen/kb/lighting-overview/

---

With this function block, all Lighting Controller blocks can be controlled together. Double-click on the block to open the dialog for selecting the linked blocks.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)
- [Basic Programming](#basic)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Lc1-4 | Light circuit 1-4 | Activates output (Lc1-4). Long-click for dimming. | 0/1 |
| M+ | Next mood | Pulse: Selects next mood. Double-click: Turns off lights and sends pulse to output (2C). Triple-click: Turns off lights and sends pulse to outputs (3C) and (2C). | 0/1 |
| M- | Previous mood | Pulse: Selects previous mood. Double-click: Turns off lights and sends pulse to output (2C). Triple-click: Turns off lights and sends pulse to outputs (3C) and (2C). | 0/1 |
| Mood | Select mood by ID | 0...99 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. An alarm can still be triggered via the input (Alarm). The name of the connected sensor is used in the user interface. | 0/1 |
| T5/1-8 | T5 control | Pulse: First pulse selects the assigned mood, every subsequent pulse switches to the next mood. If there is no additional pulse for 30 seconds, a pulse selects the assigned mood again. Long-click: Either mixes in the mood (Mmd) or switches to the next mood. Double-click: Turns off lights and sends pulse to output (2C). Triple-click: Turns off lights and sends pulse to outputs (3C) and (2C). | ∞ |
| DisP | Disable presence / motion | Disables inputs (P) and (Mo) when 1. Lights activated by presence (P) are switched off immediately. If lights were activated by motion (Mo), (Moet) timer is started on rising edge at (DisP) and lights are switched off after expiration. If the extend time of (Met) is shorter than (Moet), (Met) is used instead. | 0/1 |
| On | All on | Activates mood with ID 99. If no mood with ID 99 is configured, all used outputs (Lc1-18) will be activated with brightness set in parameter (MaxAbr). | 0/1 |
| Alarm | Alarm | All used outputs (Lc1-18) start flashing when 1. Parameter (MaxAbr) defines the brightness, parameter (Afi) defines the flashing interval. ID 99 is displayed at the output (M). If the input (Off) is 1, an alarm can still be triggered. | 0/1 |
| Buzzer | Buzzer | Activates alarm clock mood (ID 98) when 1. Parameter (Fbu) defines the fading time. Fading is only supported by Smart actuators. If no alarm clock mood is configured, the mood with ID 99 is used instead. Lighting is switched off after the time set in parameter (Met) has expired. | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Lc1-4, M+, M-, Mood, Off, T5/1-8, On, Buzzer, MBr) when On. (e.g Child lock, cleaning) Control via user interface is still possible. | 0/1 |
| Rtd | Reset to default | Resets parameters and settings of the block to the default values as specified in the block preset. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| API | API Connector | Intelligent API based connector. API Commands | - |
| Na | Active Lights | Number of active lights | ∞ |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Selection | All selected Lighting Controller can be controlled together. | - |

---

## Basic Programming

Double-clicking on the block opens the following window.

![CentralLight Menu](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/CentralLight_Menu.jpg)

Central commands are not blocked by an active (DisPc) input at the respective function block. If a function block is used in the central block, this is indicated by the central symbol on the respective block.
The functions that can be used on the central block depend on the linked blocks and are set via their parameters. If a function block does not support a function, it cannot be controlled.