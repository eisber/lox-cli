# Roof Window Shading

Source: https://www.loxone.com/enen/kb/skylight-blinds/

---

Controls the shading for a roof window.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Direction parameters](#Direction)
- [Start / End offsets](#AutoTime)
- [Enable Sun Position Automatic via User Interface](#SunPosAutoApp)
- [Timing Diagram](#timediag)
- [History](#history)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Tg | Toggle | Toggles between open, stop, close. For single button control. | - | 0/1 |
| Po | Partial open with push & hold | - | 0/1 |
| Pc | Partial close with push & hold | - | 0/1 |
| Co | Complete open | If in motion, stops. | - | 0/1 |
| Cc | Complete close | If in motion, stops. | - | 0/1 |
| Dwc | Door/window contact | Opens shading completely and locks the block when On. Manual operation via user interface is still possible. (0 = closed, 1 = open). | - | 0/1 |
| Wa | Wind alarm | Moves shading to the wind alarm position set in parameter (Wap) and locks the block(can still be operated through the App/Webinterface). Used for storm protection. Active Automatic is only suspended instead of cancelled. After Wa end, automatic is restarted with a positive edge on Sps or of the sunshine-systemvariable. Automatic conditions are not re-evaluated on end on windalarm. If both conditions are permanently active during windalarm, automatic can be restarted via pulse on Spr. The name of the connected sensor is used in the user interface. | - | 0/1 |
| Off | Off | Pulse stops movement. On locks the block. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |
| Pos | Position of shading | Move the shading to the specified position. | % | 0...100 |
| T5 | T5 control | Button 1: Complete open Button 4: Complete close | - | ∞ |
| DisPc | Disable periphery control | Disables inputs Tg, Po, Pc, Co, Cc, So, T5 when On. (e.g Child lock, cleaning) Control via user interface is still possible. | - | 0/1 |
| So | Slightly open | Venetian blinds close completely and move slats in horizontal position according to parameter (Rd). Roller blinds, curtains and awnings move into position according to parameter (Rd). | - | 0/1 |
| Sps | Sun position automatic start | Activates the sun position automatic when On at the beginning of the shading period, or also with a pulse during the shading period. The sun position automatic is deactivated for the rest of the day, if block is operated manually. Pulse on (Spr) followed by a rising edge on (Sps) or pulse on (Spr) while (Sps) is active, restarts the sun position automatic. | - | 0/1 |
| DisSp | Disable sun position automatic | Disables the sun position automatic when On. | - | 0/1 |
| Spr | Sun position automatic restart | Pulse followed by a rising edge at input (Sps) or pulse while input (Sps) is active, restarts the sun position automatic. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Op | Open | This output is only visible in certain configurations. | 0/1 |
| Cl | Close | This output is only visible in certain configurations. | 0/1 |
| Pos | Position of shading | Position of the shading (0.0 = open, 1.0 = closed) | 0...1 |
| Wds | Wind, door/window contact state | On when Input (Wa) of the block or of a linked central block is active or when input (Dwc) is active. | 0/1 |
| Off | Off | Active when input (Off) is 1. | 0/1 |
| Sp | Sun position automatic | On, if input (Sps) = 1 and input (DisSp) = 0 ...and if the setting “Use sunshine variable” is checked and the sun is shining, or if the Automation based on sun position is switched on in the App. | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Wap | Wind alarm position | 0 = Completely open 1 = Completely closed | - | 0/1 | 0 |
| Opd | Opening duration | s | 0...∞ | 75 |
| Cld | Closing duration | s | 0...∞ | 75 |
| Mld | Motor lock duration | Duration of motor lock between direction changes. | s | 0...∞ | 0,5 |
| Tdc | Time double-click | Double click duration on inputs (Po), (Pc) for complete open / close. 0 = Not used | s | 0...∞ | 0,3 |
| Tlc | Time long-click | Long-click duration on inputs (Po), (Pc) for complete open / close. If you prefer a double-click, then set the value > (Opd) or (Cld). 0 = Always start complete travel. | s | 0...∞ | 3 |
| minTd | Minimum travel duration | Minimum travel time at pulse on input (Po) or (Pc). | s | 0...∞ | 0,4 |
| Spm | Sun position automatic mode | 0 = Automatic stays Off if shading is closed. If the input (Sps) is active, a pulse on (Spr) restarts the sun position automatic. 1 = Automatic always allowed. | - | 0...1 | 1 |
| Spe | Sun position automatic end action | 0 = No action 1 = Complete open 2 = Complete close | - | 0...2 | 1 |
| Dir | Compass direction | Compass direction of the window: 0 = north 90 = east 180 = south 270 = west -1 = not configured | ° | -1...359 | -1 |
| Dts | Direction tolerance start | Direction tolerance for the sun position automatic when the sun enters the shading area. | ° | 0...90 | 85 |
| Dte | Direction tolerance end | Direction tolerance for the sun position automatic when the sun exits the shading area. | ° | 0...90 | 85 |
| Pi | Pitch | Roof or window pitch. 0 = horizontal 90 = vertical | ° | 0...90 | 30 |
| Spos | Sun position automatic start offset | Start offset of the sun position automatic relative to Sunrise. | min | -90...90 | 30 |
| Spoe | Sun position automatic end offset | End offset of the sun position automatic relative to Sunset. | min | -90...90 | -30 |
| Sop | Slightly open position | Position used for input (So) and (Sps). | % | 0...100 | 80 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Use sunshine variable | Sun position automatic is only activated if the system variable Sunshine and the input Sps are active. | - | - |
| Activity Log Entries | Number of entries in the activity log. 0: log is disabled The activity log tracks relevant changes since program start. | 0...100 | 20 |

---

## Direction parameters

![Autojalousie Parameter D DT DTe](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Autojalousie_Parameter_D_DT_DTe.jpg)

Dir: Compass direction of the shading device relative to the geographical north. (0 = North, 90 = east, 180 = south, 270 = West)

Dts: Direction tolerance when the sun enters the shading device. The directional tolerance is relative to the perpendicular angle of the shading device. Consider, for example, building protrusions. If your shading device is not affected, you can leave this at its default value of 85º.

Dte: Direction tolerance when the sun exits the shading device. The directional tolerance is relative to the perpendicular angle of the shading device. Consider, for example, building protrusions. If your shading device is not affected, you can leave this at its default value of 85º.

---

## Start / End offsets

With parameter (Spos), the sun position automatic will be delayed by this number of minutes after sunrise.
This is relevant for windows that may not get the sun directly after sunrise.

With parameter (Spoe), the sun position automatic will be brought forward by this number of minutes before sunset.
This is relevant for windows that may be affected by the sun prior to sunset.

There are two possible applications for the two parameters:

Often the sun is obscured by other buildings, structures or trees after sunrise and before sunset. Here, the start or end time of the sun position automatic can be adapted to accommodate for these.

Another possibility is to shift the times so that sunrise or sunset can be seen with the shading open.

---

## Enable Sun Position Automatic via User Interface

The sun position automatic can be started manually via the user interface, thereby blinds are lowered when the [sun is at the window](#Direction). Whether the sun is shining, it is cloudy, or the **Sps** input of the block is active or not is not taken into account.

![AutomaticShading tilted](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AutomaticShading_tilted.png)

---

## Timing Diagram

![AutomaticShading timediag](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/AutomaticShading-timediag.png)

---

## History

In the user interface, the history of the function block can be displayed.
A maximum of 100 entries can be shown.
When you restart or save to the Miniserver, the history is cleared.

![History RoofWindowShading](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/History_RoofWindowShading.png)