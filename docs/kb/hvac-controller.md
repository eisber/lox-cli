# HVAC Controller

Source: https://www.loxone.com/enen/kb/hvac-controller/

---

Controller for various types of HVAC systems. This block is optimized to work with at least one Intelligent Room Controller and its intelligent mode switching to control the heating/cooling system.

Rooms with presence are prioritized for the heating/cooling decision. If presence is detected in rooms that call for heat and in rooms that call for cooling, the decision is based on the higher demand.

Intelligent Room Controllers provide dedicated outputs for up to three assigned controllers.

This function block belongs to the family of thermal energy sources. It is also suitable for use with hydronic heating systems.

This block is incredibly sophisticated and far superior to energy source controllers available on the market, e.g. luxurious thermostats.

> **ℹ️ Note:** The HVAC Controller is primarily designed for North American HVAC systems. However, it is not limited to this application and can also be used with a wide range of other system types, depending on the project requirements.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Mode | Mode | 0 = off 1 = automatic mode, switches automatically between heating and cooling, based on demand 2 = heating only 3 = cooling only | - | 0...3 |
| ϑo | Outdoor temperature | If this input is not connected, the value of the system variable "Outdoor temperature" is used. | ° | ∞ |
| B | Boost | Activate second stage heating/cooling if the first stage is already active. | - | 0/1 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |
| Emh | Emergency Heat | Activates output (E) when heating demand = 1. Used for emergency heat, e.g. if heat pump defective. | - | 0/1 |
| Fan | Fan | Activates output (G) and opens all Intelligent Room Controllers to 100%. If this input is used, the fan can no longer be activated in the App. | - | 0/1 |
| H | Humidity | Required when output (Hmd) is connected in order to optimize indoor humidity. Note: Use the humidity reading of the most relevant room or an average value. | - | ∞ |
| Ec | Excess cooling | Surplus or cheap cooling energy available In cooling mode, Intelligent Room Controllers will overcool or allow premature start of cooling. | - | 0/1 |
| Eh | Excess heating | Surplus or cheap heating energy available. In heating mode, Intelligent Room Controllers will overheat or premature start of heating will be permitted. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| W/W1 | 1st stage heating | 1st stage heating. Varies depending on selected heating type. For "Oil/gas/electric": active for first stage heat, active for second stage heat. For "Heat pump with fossil fuel backup": active for second stage heat. For "Heat pump with electric backup": active for second stage heat. | 0/1 |
| W2 | 2nd stage heating | 2nd stage heating. Varies depending on selected heating type. For "Oil/gas/electric": active for second stage heat. For "Heat pump with fossil fuel backup": active for second stage heat. For "Heat pump with electric backup": active for second stage heat. | 0/1 |
| Y | Compressor | Compressor. Varies depending on selected heating type. For "Oil/gas/electric": active for first stage cool, active for second stage cool. For "Heat pump with fossil fuel backup": active for first stage heat/cool, active for second stage cool. For "Heat pump with electric backup": active for first stage heat/cool, active for second stage heat/cool. | 0/1 |
| Y2 | 2nd stage cooling | 2nd stage cooling. Varies depending on selected heating type. For "Oil/gas/electric": active for second stage cool. For "Heat pump with fossil fuel backup": active for second stage cool. For "Heat pump with electric backup": active for second stage heat/cool. | 0/1 |
| E | Emergency Heat | Activated depending on input (Eh) or via app activation. | 0/1 |
| O/B | Reversing valve | Changes the operating direction of the heat pump. If (DirV) is deactivated: OFF for heating, ON for cooling. If (DirV) is activated: ON for heating, OFF for cooling. | 0/1 |
| G | Fan | Fan that moves heated or cooled air into the rooms | 0/1 |
| Hmd | Humidifier | Activated in heating mode if (H)