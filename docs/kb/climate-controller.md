# Heating and Cooling Controller

Source: https://www.loxone.com/enen/kb/climate-controller/

---

This block controls a Heating and/or Cooling source.
Depending on the requirements of the assigned Intelligent Room Controllers, it is decided whether the Heating or Cooling mode is active.

A Service Mode is available in the user interface to override the function block.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Costs](#ModeCosts)
- [Decision heating / cooling process](#ModeSelect)
- [Surplus heat or cooling](#ExcessEnergy)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| ϑo | Outdoor Temperature | If this input is not connected, the System Variable "Outdoor Temperature" is used. If this value is not available, the value -1000 is displayed. | ° | ∞ |
| B | Boost | Activates Stage 2 immediately. When in heating mode, output (Ah) is also enabled. The name of the connected sensor is used in the user interface. | - | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |
| Ah | Additional heating | Activates output (Ah) when 1. The name of the connected sensor is used in the user interface. | - | 0/1 |
| F | Fan | Activates output (F) when 1. | - | 0/1 |
| Cfc | Confirm filter change | - | 0/1 |
| Ec | Excess cooling | Surplus or cheap cooling energy available In cooling mode, Intelligent Room Controllers will overcool or allow premature start of cooling. | - | 0/1 |
| Eh | Excess heating | Surplus or cheap heating energy available. In heating mode, Intelligent Room Controllers will overheat or premature start of heating will be permitted. | - | 0/1 |
| Mh | Manual heating | Activates Manual Heating Mode when 1. As long as (Mh) is active, the requirements of the Intelligent Room Controllers are ignored and the Heating operation is activated. e.g. For Hot Water Treatment. Minimum run times are observed. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| H | Heating | 0/1 |
| H2 | Heating stage 2 | Activates Stage 2 of the heating source after time (Tt2s). When Block is set to Type Heat Pump, the output is activated immediately when the outside temperature is below (ϑminS2). Will be deactivated again when heating is deactivated. | 0/1 |
| C | Cooling | 0/1 |
| C2 | Cooling stage 2 | Activates Stage 2 of the cooling source after time (Tt2s). Will be deactivated again, when cooling is deactivated. | 0/1 |
| Ah | Additional heating | 0/1 |
| Sv | Switching valve | Controls the switching valve. 0 = Heating, 1 = Cooling. Only after the switching valve is in position (Vd), the outputs (H) / (H2) or (C) / (C2) are switched on. This output is only visible in certain configurations. | 0/1 |
| F | Fan | Cooling: The fan starts immediately after cooling is activated and the valve movement is complete. Heating: For heat pumps, the fan starts 15 seconds after heating is activated and the valve movement is complete. For oil/gas, the fan does not operate during heating. | 0/1 |
| Fc | Filter change | 1 when (Dfc) has expired. | 0/1 |
| ϑoa | Average outdoor temperature | Shows the average outdoor temperature of the last 48 hours. The calculated value is available as soon as the first 24 hours have passed! Until then, the value -1000 is displayed! | ∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Mode | Mode | -1 = Heating and Cooling Controller is turned off. 0 = Automatic changeover based on the Intelligent Room Controller demands. 1 = heating only if there is sufficient demand. 2 = cooling only if there is sufficient demand. | - | -1...2 | 0 |
| MinHr | Time minimum HVAC Runtime | The minimum runtime must be met before switching to standby (All inputs OFF) or the opposite mode. Minimum openings of the room controllers are ensured. | min | ∞ | 0 |
| Sot | Switch on threshold | Average valve openings of the Intelligent Room Controllers must exceed (Sot) in order to start operation. | % | 0...100 | 30 |
| Vd | Valve delay | Time required by the switching valve (Sv) to move into position. This parameter is only visible in certain configurations. | s | ∞ | 0 |
| Fod | Fan Overrun Duration | Overrun time of the fan after the end of heating/cooling to bring the residual energy out of the system. | s | ∞ | 120 |
| Don | Duration for On | Duration for ON pulse for (MaxTp). | s | ∞ | 750 |
| Doff | Duration for Off | Duration for OFF pulse for (MaxTp). | s | 180...∞ | 300 |
| MaxTp | Maximum threshold for pulsing | Maximum threshold for pulsing based on the valve opening percentage. If the sum of the valve openings is smaller than this value, the Heating/Cooling outputs will pulse at intervals of (Don) and (Doff). 0 = Disables the timer. | % | ∞ | 0 |
| Dfc | Days until Filter Change | Specify the number of days remaining until the filter needs to be replaced. 0 = Deactivated | Days | ∞ | 0 |
| Tt2s | Time to second stage | Delay before Stage 2 is enabled after the start of a heating/cooling cycle. | min | ∞ | 60 |
| ϑminS2 | Minimum Temperature Stage 2 | If the used outdoor temperature (Parameter Otm) is below (ϑminS2), Stage 2 is activated immediately. This parameter is only visible in certain configurations. This parameter is only visible in certain configurations. | ° | ∞ | -6 |
| ϑminHP | Minimum Temperature Heat Pump | Minimum outdoor temperature required to operate the heat pump. If the outdoor temperature (ϑo) falls below this value, only the auxiliary heating (Ah) is activated in heating mode, the outputs (H) / (H2) remain off. This parameter is only visible in certain configurations. | ° | ∞ | -22 |
| Otm | Outdoor Temperature Mode | 0 = Disabled (ϑLimH and ϑLimC not used) 1 = Average Outdoor Temperature of the past 48h 2 = Value of the System Variable 'Expected Average Outdoor Temperature 48h' 3 = Current Outdoor Temperature If enabled, the average outdoor temperature of the last 48h, next 48h or the current temperature is used to select heating/cooling mode according to (ϑLimH) and (ϑLimC). If the value is not available, this parameter has no effect. | - | 0...3 | 2 |
| ϑLimH | Temperature Limit Heating | If the used outdoor temperature (Parameter Otm) is above (ϑLimH), the system will not switch to heating mode, even if there is a demand. | ° | ∞ | 18 |
| ϑLimC | Temperature Limit Cooling | If the used outdoor temperature (Parameter Otm) is below (ϑLimC), the system will not switch to cooling mode, even if there is a demand. | ° | ∞ | 15 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Assign room controllers | Add or remove the Climate Controller as a source for individual Intelligent Room Controllers. Further settings (Priority, PWM, etc.) can be made in the configuration dialog of each Intelligent Room Controller. | - |
| Heating Type | Type of the heat source | - |
| Energy Costs(Heating) | Costs for running this source. Sources that have been configured to be 'Expensive' will only be requested by room controllers if no sources with higher priorities are available. | - |
| Energy Costs(Cooling) | Costs for running this source. Sources that have been configured to be 'Expensive' will only be requested by room controllers if no sources with higher priorities are available. | - |

---

## Costs

There are separate heating / cooling costs configured for each process Depending on these settings, the requests are sent to the block.

---

## Decision heating / cooling process

The controller determines the operating mode based on the total energy demand of all rooms, calculated in degree square meters (°m²). The demand for each room is calculated with following formula:

(Target temperature - Current temperature) * Room Size

The system initially selects the mode with the higher total demand across the building, but strictly prioritizes occupied rooms to ensure comfort. Consequently, the system will override a general cooling decision and switch to heating if any occupied room requires heat. Conversely, it will only switch from heating to cooling if the cooling demand from occupied rooms is greater than their heating demand. This automatic evaluation only occurs when the minimum opening threshold is exceeded and is further restricted by the configured average temperature limits (ϑLimH and ϑLimC).

Whenever outputs (H) or (C) have been switched off, a delay of 3 minutes is active in addition to (Fod) until they can be switched on again.

---

## Surplus heat or cooling

With the inputs (Eh) and (Ec), the room controllers are informed that surplus energy is available or that a specific mode would be favorable. The room controllers are then allowed to go into preparation mode before the scheduled time. If the room controls are already in Comfort mode, they are allowed to overheat or overcool to the average between the two comfort temperatures. This function can be used in conjunction with solar / PV, off-peak hours or [Spot Price Optimizer](https://www.loxone.com/enen/kb/spot-price-optimizer/) to use more favorable times for heating and cooling.