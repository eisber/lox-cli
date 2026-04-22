# AC Central Controller

Source: https://www.loxone.com/enen/kb/ac-central-controller/

---

This Control represents an outdoor unit combining one or more AC Control indoor units.
Depending on the requirements of the assigned [AC Unit Controllers](https://www.loxone.com/help/AC-Control-block), it is decided whether the Heating or Cooling mode is active.
AC Unit Controllers with opposite requirements will be paused to avoid conflicts on the outdoor unit.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Decision heating / cooling process](#ModeSelect)
- [Surplus heating or cooling](#ExcessEnergy)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| ϑo | Outdoor Temperature | If this input is not connected, the System Variable "Outdoor Temperature" is used. If this value is not available, the value -1000 is displayed. | ° | ∞ |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |
| Ec | Excess cooling | Surplus or cheap cooling energy available. In cooling mode, Intelligent Room Controllers will overcool or allow premature start of cooling. | - | 0/1 |
| Eh | Excess heating | Surplus or cheap heating energy available. In heating mode, Intelligent Room Controllers will overheat or allow premature start of heating. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| H | Heating | Device unit is in heating mode | 0/1 |
| C | Cooling | Device unit is in cooling mode | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Mode | Mode | 0 = Device is turned off. 1 = Automatic changeover based on the AC Unit Controller demands. 2 = heating only if there is sufficient demand. 3 = cooling only if there is sufficient demand. | - | 0...3 | -1 |
| ϑLimH | Temperature Limit Heating | If the used outdoor temperature (Parameter Otm) is above (ϑLimH), the system will not switch to heating mode, even if there is a demand. This parameter is only visible in certain configurations. | ° | ∞ | 18 |
| ϑLimC | Temperature Limit Cooling | If the used outdoor temperature (Parameter Otm) is below (ϑLimC), the system will not switch to cooling mode, even if there is a demand. This parameter is only visible in certain configurations. | ° | ∞ | 15 |
| Otm | Outdoor Temperature Mode | 0 = Disabled (ϑLimH and ϑLimC not used) 1 = Average Outdoor Temperature of the past 48h 2 = Value of the System Variable 'Expected Average Outdoor Temperature 48h' 3 = Current Outdoor Temperature If enabled, the average outdoor temperature of the last 48h, next 48h or the current temperature is used to select heating/cooling mode according to (ϑLimH) and (ϑLimC). If the value is not available, this parameter has no effect. | - | 0...3 | 2 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Assignments | Add or remove AC Unit Controller function blocks | - |
| Energy Costs(Heating) | Costs for running this source. Sources that have been configured to be 'Expensive' will only be requested by room controllers if no sources with higher priorities are available. | - |
| Energy Costs(Cooling) | Costs for running this source. Sources that have been configured to be 'Expensive' will only be requested by room controllers if no sources with higher priorities are available. | - |

---

## Decision heating / cooling process

The influencing factor for the decision is the sum of the requirements for all AC Unit Controllers in °m2 (Temperature difference Target / Actual Temperature * Room Size). Requirement types: heating, cooling... In Heating or Cooling only mode, operation is started when there is sufficient demand for the configured mode, otherwise the controller remains OFF.

---

## Surplus heating or cooling

To use inexpensive cooling or heating energy as efficiently as possible, a room can be „overcooled“ or „overheated“. This function is excellent, for example, when combined with the [Energy Manager](https://www.loxone.com/help/energy-manager) for optimizing self-consumption of solar power or with the Spot Price Optimizer.

With the inputs (Eh) and (Ec), the room controllers are informed that there is surplus/inexpensive energy available or that a specific mode would be favorable. The room controllers are then allowed to go into preparation mode before the scheduled time. If the room controls are already in Comfort mode, they are allowed to overheat or overcool to the average between the two comfort temperatures. When using a single target temperature, or a mode which does not allow switching between heating and cooling (1,2,4,5), the offset is taken from parameter ϑExc from the Intelligent Room Controller.