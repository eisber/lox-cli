# Pool Controller

Source: https://www.loxone.com/enen/kb/pool-controller/

---

The Pool Controller provides the ability to control the various cycles of Filter, Circulate, Back & Rinse, Drain, and Out of Service.
These cycles can be activated via operating modes, inputs of the block but also in the User Interface.
In addition, the block also allows the ability to give pool temperature control.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Automatic (Changes mode according to schedule)](#OperatingMode)
- [Service mode (for maintenance and verification purposes)](#ServiceMode)
- [Out of service](#NoService)
- [Timer for automatic operation](#Schedule)
- [Temperature Control](#Temperatur)
- [Cycles](#cycles)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Om | Set operating mode | 1 = Automatic 2 = Service | - | 0...2 |
| ϑm | Set temperature mode | 0 = Off 1 = Automatic heating or cooling using the set point. Input (Eco) is taken into account. 2 = Automatic heating using the set point. Input (Eco) is taken into account. 3 = Automatic cooling using the set point. Input (Eco) is taken into account. 4 = Heating permanently on 5 = Cooling permanently on | - | 0...5 |
| Eco | Eco | Output (H) and output (C) are switched off when 1. | - | 0/1 |
| ϑt | Target temperature | ° | ∞ |
| ϑc | Current temperature | ° | ∞ |
| Wlvl | Water level | cm | 0...∞ |
| Cpos | Cover position | Position of the cover (0.0 = open, 1.0 = closed) | - | 0...1 |
| Sm | Swimming machine | Digital or analog input depending on output (Wm). | - | ∞ |
| I1 | Custom input 1 | Input value is displayed in the user interface. | - | ∞ |
| I2 | Custom input 2 | Input value is displayed in the user interface. | - | ∞ |
| Fi | Activate Filter cycle | This input is only visible in certain configurations. | - | 0/1 |
| Bw | Activate Backwash & rinse cycle | This input is only visible in certain configurations. | - | 0/1 |
| Ci | Activate Circulation cycle | This input is only visible in certain configurations. | - | 0/1 |
| Dr | Activate Drain cycle | This input is only visible in certain configurations. | - | 0/1 |
| Vp | Set valve position | Valve postion can only be set when in service mode! (Note: No Cycle is started!) Valve positions: 0 = Filtration 1 = Backwash 2 = Rinse 3 = Circulate 4 = Closed 5 = Draining This input is only visible in certain configurations. | - | ∞ |
| Fp | Filtration pump | For manual control of the filtration pump. Can only be activated in service mode! This input is only visible in certain configurations. | - | 0/1 |
| Dv | Drain valve | For manual control of the drain valve. Can only be activated in service mode! This input is only visible in certain configurations. | - | 0/1 |
| Error | Error Input | Can be used as error input. Pulse disables the valve until a reset via input (Off) or an acknowledgment via the user interface. | - | 0/1 |
| Off | Off | Pulse = Resets the function block to its initial state: - Operating mode: automatic - Temperature control mode: automatic - Jets: off or 0 - Active cycle will be stopped - Valve is set to filtration position - Pump: off - Drain valve: closed 1 = Locks the funcktion block - All Off - Aquastar in winter position The name of the connected sensor is used in the user interface. | - | 0/1 |
| DisPc | Disable periphery control | Disables inputs (Om), (ϑm), (ϑt), (Fi), (Bw), (Ci), (Dr), (Vp), (Fp), (Dv) when On. (e.g Child lock, cleaning) Control via user interface is still possible. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| H | Heating demand | Only activated when the filter pump is active and something is connected to the output. If there is a heating demand while in temperature mode "automatic" and no cycle is active, the circulation cycle is started. | - | 0/1 |
| C | Cooling demand | Only activated when the filter pump is active and something is connected to the output. If there is a cooling demand while in temperature mode "automatic" and no cycle is active, the circulation cycle is started. | - | 0/1 |
| Om | Current operating mode | 0 = Not in Use 1 = Automatic 2 = Service | - | 0...2 |
| ϑm | Current temperature control mode | 0 = Off 1 = Automatic heating or cooling using the set point. Input (Eco) is taken into account. 2 = Automatic heating using the set point. Input (Eco) is taken into account. 3 = Automatic cooling using the set point. Input (Eco) is taken into account. 4 = Heating permanently on 5 = Cooling permanently on | - | 0...5 |
| Cϑm | Cycle started via (ϑm) | On if the current cycle was started by a temperature control mode. | - | 0/1 |
| ϑt | Current target temperature | ° | ∞ |
| Wlvl | Current water level | cm | ∞ |
| Op | Open pool cover | Pulses when the button is pressed in the user interface. | - | 0/1 |
| Cl | Close pool cover | Pulses when the button is pressed in the user interface. | - | 0/1 |
| Wm | Water machine | Output for a water machine or jets. Output is analog or digital depending on the connection. | - | ∞ |
| Fi | Filtration cycle state | This output is only visible in certain configurations. | - | 0/1 |
| Bw | Backwash cycle state | This output is only visible in certain configurations. | - | 0/1 |
| Ci | Circulation cycle state | This output is only visible in certain configurations. | - | 0/1 |
| Dr | Draining cycle state | This output is only visible in certain configurations. | - | 0/1 |
| Rtc | Remaining duration of the active cycle | This output is only visible in certain configurations. | s | ∞ |
| Fpet | Filtration pump extend time | Indicating whether the filtration pump is still running due to the parameter (Fpet). This output is only visible in certain configurations. | - | 0/1 |
| Vpos | Current valve position | -1 = Valve is moving / unknown position 0 = Filtration 1 = Backwash 2 = Rinse 3 = Circulate 4 = Closed 5 = Draining 6 = Release This output is only visible in certain configurations. | - | ∞ |
| Fp | Filtration pump | Output for filtration pump control. This output is only visible in certain configurations. | - | 0/1 |
| Dv | Drain valve | Output for drain valve control. This output is only visible in certain configurations. | - | 0/1 |
| Error | Error code | Is activated by input (Error). Is active until a reset via input (Off) or acknowledgment via the user interface. | - | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| ϑh | Hysteresis temperature | When in temperature mode "automatic", the decision whether to heat or cool is made at midnight: If the water temperature is lower than the target temperature minus the hysteresis, heating will be activated. If the water temperature is lower than the target temperature plus the hysteresis, cooling will be activated. | - | ∞ | 0,5 |
| Fpet | Filtration pump extend time | Filtration pump extend time after a cycle with heating or cooling demand. To prevent heat build-up, the water is kept in motion. This parameter is only visible in certain configurations. | s | 0...1800 | 0 |
| Fid | Filtration cycle duration | This parameter is only visible in certain configurations. | s | 0...∞ | 18000 |
| Bwd | Backwash cycle duration | This parameter is only visible in certain configurations. | s | 20...600 | 120 |
| Rid | Rinsing cycle duration | This parameter is only visible in certain configurations. | s | 20...300 | 30 |
| Cid | Circulation cycle duration | This parameter is only visible in certain configurations. | s | 0...∞ | 43200 |
| Drd | Draining cycle duration | This parameter is only visible in certain configurations. | s | 0...∞ | 3600 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Aquastar Air | Aquastar Air linked to this function block. Devices that can be assigned: Aquastar Air | - |

---

## Automatic (Changes mode according to schedule)
- Uses the cycles set in the schedule

-When a manual cycle is started, it has priority. After completion of the manual cycle, the scheduled sequence continues according to the schedule

-Allows automatic temperature control

-Manual changes of valve position, pump, and drain valve are ignored.

---

## Service mode (for maintenance and verification purposes)

-Allows manual control of the valve via the inputs (Vp), (Fp) and (Dv).

-Start manual cycles

-Manual control of filter pump, drain valve, and heating and cooling release via the app

-Timer is ignored

-Active until the user switches back to automatic

-Temperature control is ignored. In this mode, mutual interlocks and safety features are disabled. Having the pump active while the valve is in motion should be avoided. This mode is used to check the individual functions.

---

## Out of service

-All inputs (even commands from the app), as well as the timer, are ignored. Valve position is set to discharge

-At the end of this mode, a reset is triggered (falling edge at the input (Off)).

---

## Timer for automatic operation

Predictable cycles (always with pump):

-Filter

-Backwashing + Rinsing

-Circulate

---

## Temperature Control

-Make sure that the set temperature for the day is reached.

-Controls the outputs heating or cooling release

-In Eco mode these two outputs are deactivated.

---

## Cycles

![PoolController Clean](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PoolController_Clean.jpg)

Filter:

Filter pump on, drain valve closed, heater release valve (if heating required)

Backwash + Rinse:

-Drain: backwash-> turn on rinse -> rinse -> turn on filters

-Filter pump on (not when changing position), drain valve open

Circulate:

Filter pump on, drain valve closed

Empty:

Filter pump on, drain valve open