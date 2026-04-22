# Automatic Shading

Source: https://www.loxone.com/enen/kb/automatic-blinds/

---

The Automatic Shading function block is used to control shading devices such as blinds, roller shutters, curtains or awnings.
In addition to manual operation, shading can be automated, based on sunlight and room temperature, in combination with Intelligent Room Controller.
The goal is to avoid overheating of a room due to solar radiation.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Type of shading device](#Type)
- [Programming example](#desc6)
- [Functional Description](#functional)
- [Direction parameters](#Direction)
- [Start / End offsets](#AutoTime)
- [Backlash duration](#desc4)
- [Slat adjustment](#AutotrackingSlats)
- [Complete movement, end positions and remanence](#desc8)
- [Function for setting the end limits](#setlimits)
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
| So | Slightly open | Venetian blinds close completely and move slats in horizontal position according to parameter (Rd). Roller blinds, curtains and awnings move into position according to parameter (Rd). | - | 0/1 |
| Sps | Sun position automatic start | Activates the sun position automatic when On at the beginning of the shading period, or also with a pulse during the shading period. The sun position automatic is deactivated for the rest of the day, if block is operated manually. Pulse on (Spr) followed by a rising edge on (Sps) or pulse on (Spr) while (Sps) is active, restarts the sun position automatic. | - | 0/1 |
| DisSp | Disable sun position automatic | Disables the sun position automatic when On. | - | 0/1 |
| Spr | Sun position automatic restart | Pulse followed by a rising edge at input (Sps) or pulse while input (Sps) is active, restarts the sun position automatic. | - | 0/1 |
| Wa | Wind alarm | Moves shading to the wind alarm position set in parameter (Wap) and locks the block(can still be operated through the App/Webinterface). Used for storm protection. Active Automatic is only suspended instead of cancelled. After Wa end, automatic is restarted with a positive edge on Sps or of the sunshine-systemvariable. Automatic conditions are not re-evaluated on end on windalarm. If both conditions are permanently active during windalarm, automatic can be restarted via pulse on Spr. The name of the connected sensor is used in the user interface. | - | 0/1 |
| Dwc | Door/window contact | Opens shading completely and locks the block when On. Manual operation via user interface is still possible. (0 = closed, 1 = open). | - | 0/1 |
| Off | Off | Pulse stops movement. On locks the block. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |
| Pos | Position of shading | Move the shading to the specified position. | % | 0...100 |
| Slat | Position of slats | Move the slats to the specified position. This input is only visible in certain configurations. | % | 0...100 |
| T5 | T5 control | Button 1: Complete open Button 4: Complete close | - | ∞ |
| DisPc | Disable periphery control | Disables inputs Tg, Po, Pc, Co, Cc, So, T5 when On. (e.g Child lock, cleaning) Control via user interface is still possible. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Op | Open | This output is only visible in certain configurations. | - | 0/1 |
| Cl | Close | This output is only visible in certain configurations. | - | 0/1 |
| Pos | Position of shading | Position of the shading (0.0 = open, 1.0 = closed) | - | 0...1 |
| Slat | Position of slats | Position of the slats (0.0 = horizontal, 1.0 = vertical) This output is only visible in certain configurations. | - | 0...1 |
| Sp | Sun position automatic | On, if input (Sps) = 1 and input (DisSp) = 0 ...and if the setting “Use sunshine variable” is checked and the sun is shining, or if the Automation based on sun position is switched on in the App. | - | 0/1 |
| Wds | Wind, door/window contact state | Active when input (Wa) or input (Dwc) is 1. | - | 0/1 |
| Off | Off | Active when input (Off) is 1. | - | 0/1 |
| AQpp | Command output | Used with specific devices. Command * 1000000 + Blind position in % * 1000 + Slat position in °. Command 0 = Stop, 1 = Blind position + Slat position, 2 = Only blind position, 3 = Only slat position. This output is only visible in certain configurations. | - | 0/1 |
| TPos | Target position | Target position of the shading. Can be used for Hunter Douglas shades, for example. | % | 0...100 |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Type | Shading type | Type of shading device 0 = Venetian blinds 1 = Roller blinds 2 = Curtains opening to both sides 3 = Schlotterer Retrolux 4 = Curtain left 5 = Curtain right 6 = Awning | - | 0...6 | 0 |
| Wap | Wind alarm position | 0 = Completely open 1 = Completely closed | - | 0/1 | 0 |
| Spe | Sun position automatic end action | Defines what the system does when the sun position automatic ends. This action is only triggered when calculated “automatic shading until” time is reached. Any manual intervention or other events that cancel the sun position automatic will not trigger this action. 0 = No action 1 = Complete open 2 = Complete close 3 = Set slats horizontal | - | 0...3 | 1 |
| Tlc | Time long-click | Long-click duration on inputs (Po), (Pc) for complete open / close. If you prefer a double-click, then set the value > (Opd) or (Cld). 0 = Always start complete travel. | s | 0...∞ | 3 |
| Opd | Opening duration | s | 0...∞ | 75 |
| Cld | Closing duration | s | 0...∞ | 70 |
| Mld | Motor lock duration | Duration of motor lock between direction changes. | s | 0...∞ | 0,5 |
| Tdc | Time double-click | Double click duration on inputs (Po), (Pc) for complete open / close. 0 = Not used | s | 0...∞ | 0,3 |
| Rd | Return duration | Venetian blinds: Return duration until slats are aligned horizontally. Roller blinds, curtains, awnings: Set Position for input (So) in a range from open [0.1] to closed [1.0]. Value must be > 0. | s | 0.1...∞ | 0,8 |
| Bldo | Backlash duration opposite | Backlash duration when moving in opposite direction. This parameter is only visible in certain configurations. | s | 0...∞ | 0,15 |
| Bld | Backlash duration | Backlash duration when moving in same direction. This parameter is only visible in certain configurations. | s | 0...∞ | 0 |
| minTd | Minimum travel duration | Minimum travel time at pulse on input (Po) or (Pc). | s | 0...∞ | 0,4 |
| Dir | Compass direction | Compass direction of the window: 0 = north 90 = east 180 = south 270 = west -1 = not configured | ° | -1...360 | -1 |
| Dts | Direction tolerance start | Direction tolerance for the sun position automatic when the sun enters the shading area. | ° | 0...90 | 85 |
| Dte | Direction tolerance end | Direction tolerance for the sun position automatic when the sun exits the shading area. | ° | 0...90 | 85 |
| Sw | Slat width | Width of the slats. This parameter is only visible in certain configurations. | mm | 0...∞ | 70 |
| Sd | Slat distance | Distance between two horizontal slats. This parameter is only visible in certain configurations. | mm | 0...∞ | 60 |
| Spm | Sun position automatic mode | 0 = Optimal brightness - no direct sunlight, with as much light as possible. Automatic remains off with closed shading. If the input (Sps) is active, a pulse on (Spr) restarts the sun position automatic. 1 = Optimal cooling - blocks even more solar radiation, but also results in lower brightness. Automatic remains off with closed shading. If the input (Sps) is active, a pulse on (Spr) restarts the sun position automatic. 2 = Optimal brightness - no direct sunlight, with as much light as possible. Automatic is activated even when the shading is closed. 3 = Optimal cooling - blocks even more solar radiation, but also results in lower brightness. Automatic is activated even when the shading is closed. This parameter is only visible in certain configurations. | - | 0...3 | 1 |
| Spi | Sun position automatic interval | Specifies how often the slats may adjust during sun position automatic control. This parameter is only visible in certain configurations. | min | 1...180 | 120 |
| Spos | Sun position automatic start offset | Start offset of the sun position automatic relative to Sunrise. | min | -90...90 | 30 |
| Spoe | Sun position automatic end offset | End offset of the sun position automatic relative to Sunset. | min | -90...90 | -30 |
| Rdd | Reference Drive Down | When ON, every close command triggers a full drive down, ensuring the shades move for the entire configured closing duration. Even if the sahdes are already closed, the block will reactivate the outputs to ensure the shades align with their hardware-defined end position. The Automatic Shading block does not define the lower limit—this is determined by your blinds’ setup. This parameter is only visible in certain configurations. | - | 0/1 | 0 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Use sunshine variable | Sun position automatic is only activated if the system variable Sunshine and the input Sps are active. | - | - |
| Activity Log Entries | Number of entries in the activity log. 0: log is disabled The activity log tracks relevant changes since program start. | 0...100 | 20 |

---

## Type of shading device

The function block and the user interface adapt based on the following types:

0: Venetian blinds and external blinds

1: Roller shutters and roller blinds

2:Curtains on both sides

3: Schlotterer Retrolux (special slats prevent direct sunlight, which makes readjusting the slats unnecessary)

4: Curtain one-sided left

5: Curtain one-sided right

6: Awning

---

## Programming example

A Loxone Touch is connected to the T5 input, to operate the function block according to the Loxone switch standard.
The outputs (Op) and (Cl) are connected to the outputs that control the shading device.
The parameters (Opd) and (Cld) specify the run times.
The parameter (Type) defines the type of shading device.

![10.5 autoblinds basic](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/10.5 autoblinds basic.png)

In addition, the compass direction is specified in parameter (Dir), to ensure that the sun position automatic is only activated when the sun is actually shining on the window. On demand, the Intelligent Room Controller requests shading via the output (Shd) to the input (Sps) of the Automatic Shading block.

![10.5 autoblinds irc](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/10.5 autoblinds irc.png)

When using venetian blinds, the parameters Rd, Bld, Bldo, Sw, Sd must also be set for them to be controlled correctly.

---

## Functional Description

When configured with the recommended settings, the sun position automatic is activated when the room controller enables it via input (Sps). If however, the sun position automatic is activated manually, e.g. via App, the signal from the room controller is not required.

The sun position automatic begins when the sun reaches the shading area and ends when it passes.

During this time, the slats are adjusted according to the angle of the sun. At the end, the blinds move to the position specified with parameter (Spe).

Manual shading, independent of the sun's position, can be triggered via input (So) or in the App with "Slightly open". The slats are then adjusted horizontally.

The sun position automatic can be deactivated via input (DisSp).

If the blinds are operated manually during shading period (e.g. using an app), then the sun position automatic is deactivated for that shading period. To reactivate the automatic control, a pulse on input (Spr) or activation via user interface is necessary.

> **ℹ️ Note:** When the sun position automatic is activated, the blinds may not adjust immediately. Shading only starts based on the position of the sun and the related parameters.

Output (Sp) indicates whether the sun position automatic is active.

To control multiple blinds simultaneously, the corresponding central function block is used.

Additional inputs, outputs and parameters may be required if standard push buttons are used or additional logic is programmed.

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

## Backlash duration

The parameter (Bld) is added to the required percentage of (Rd) when adjusting the slats, if the preceding run was in the same direction. The parameter only affects slat adjustment, as a change of direction is always necessary when the sun position automatic is activated. If the slat position is correct at the start of sun position automatic, but becomes inaccurate over the course of the day, this parameter must be adjusted.

The parameter (Bldo) is added to the required percentage of (Rd) when adjusting the slats, if the preceding run was in the opposite direction. If the slat position is already incorrect at the start of sun position automatic, the sum of (Rd + Bldo) is either too high (sun shines in between the slats) or too low (blind closed too far)

For windows facing east, there is only one change of direction, as the slats are opened more and more during the course of the day; for windows facing west, the blind changes direction when adjusting for the first time.

---

## Slat adjustment

When the sun position automatic is active, the blind slats are adjusted based on the sun's position.
The slats are tilted to an angle that prevents any direct sunlight from entering in between them.

Generally, the adjustment is carried out at the intervals set with parameter (Spi), but only at 15° increments, and only if necessary.
If, for example, the sun's path is at a relatively constant angle, its elevation will hardly change. For this reason, an adjustment is not necessarily carried out at every interval.

When the sun is rising, the value required at each interval is calculated and the slats are adjusted if necessary.
When the sun is setting, the value required for the following interval is calculated at each interval and the slats are adjusted if necessary.

---

## Complete movement, end positions and remanence

When fully opening the blinds (via **Co**, central function block or the App), they are always activated for the entire opening duration (**Opd**), regardless of the current position. This ensures that the blinds move into the defined end position. This is equivalent to a reference run.

However, for safety reasons, when fully closing the blinds, they are only activated for the determined remaining run time.

The actual position of the blinds is stored in the Miniserver using the block's remanence function.

If e.g. the blinds are moving up and a power failure occurs immediately thereafter, the new position can not be registered by the Miniserver. When power returns, the system will remember the previous position of the blinds, although they are actually raised.

If a downwards movement is then triggered, it will not be executed at all or not far enough, because the previous position is still stored. Also the status displayed in the App will not match the actual position.

In such and similar cases, a complete open movement must first be executed, then the shading will again operate as expected.

---

## Function for setting the end limits

Since Loxone Config version 12.1, shading technology specialists have had a function available in the visualisation of the block for setting the end positions of the shading.
This function is comparable to commercially available setting cables, and requires precise knowledge and observance of the operating instructions of the respective motor.

In the open visualisation of the block, the function can be called up in the menu. For users with full access, the function is then accessible after entering the password:

![autoblinds setlimits](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/autoblinds-setlimits.png)

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

![History AutoShade](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/History_AutoShade.png)