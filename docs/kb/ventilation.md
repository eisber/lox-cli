# Room Ventilation Controller

Source: https://www.loxone.com/enen/kb/ventilation/

---

Controls a single room living room fan based on humidity, temperature and CO2. For temperature support, the fan must be assigned to a room control as a source. In addition, the system variable "outside temperature" is required.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Hi | Humidity indoor. | When an Intelligent Room Controller is linked to the Room Ventilation Controller and humidity is connected to input H, the system will use the humidity value from the Intelligent Room Controller and ignore the value on input Hi of the Room Ventilation Controller. | % | 0...100 |
| CO2 | CO2 indoor. | When an Intelligent Room Controller is linked to the Room Ventilation Controller and CO2 is connected to the input, the system will use the CO2 value from the Intelligent Room Controller and ignore the value on input CO2 of the Room Ventilation Controller. | ppm | 0...∞ |
| Sat | Supply air temperature | Used to support temperature control in combination with the Intelligent Room Controller and for frost protection. If this input is not connected the system variable "Outdoor Temperature" is used. If this variable is not available, support for temperature control and frost protection is not available. | ° | ∞ |
| Dwc | Door/window contact | 1 = Open, 0 = Closed If a window is open, the ventilation will be deactivated. Once the window is closed, the block will resume operation based on the previously active ventilation. | - | 0/1 |
| P | Presence | - | 0/1 |
| Off | Off | 1 = control stopped and set to lock | - | 0/1 |
| Sm | Sleep mode | Switches off ventilation for the time set in parameter (Smt). After that time, ventilation is resumed. | - | 0/1 |
| B | Boost | Stops the control and sets outputs (F), (Fea) and (Fsa) to 100% when 1. The heat exchanger is still controlled automatically. | - | 0/1 |
| Ex | Exhaust air | Stops the control and sets outputs (F) and (Fea) to 100% when 1. The heat exchanger is still controlled automatically. This input is only visible in certain configurations. | - | 0/1 |
| Rtd | Reset to default | Resets parameters and settings of the block to the default values as specified in the block preset. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| F | Fan | Combined supply and exhaust air fan. | % | 0...100 |
| Fea | Fan exhaust air | Dedicated exhaust air fan. | % | 0...100 |
| Fsa | Fan supply air | Dedicated supply air fan. | % | 0...100 |
| He | Heat exchanger | Output (He) is always on by default, can be switched off manually. When heating/cooling is required by the Intelligent Room Controller, (He) is switched off or on again, depending on the system variable (Outdoor Temperature) or input (Sat). | - | 0/1 |
| S | Status | 0: Basic ventilation 1: Increased humidity 2: Supporting temperature control 3: Poor air quality (CO2) 4: Manual stop 5: Window/door open 6: Manual boost 7: Manual App 8: Manual exhaust air 9: Sleep mode 10: Frost protection | - | 0...10 |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Hmax | Maximum humidity | The block will attempt to keep (Hi) below set value. | % | 0...100 | 60 |
| CO2max | Maximum CO2 (air pollution) | The block will attempt to keep (CO2) below set value. | ppm | 0...∞ | 1000 |
| Pet | Presence extend time | Starts with the falling edge of input (P). Extends presence by the specified time. | s | 0...∞ | 1800 |
| Smt | Sleep mode timeout | Starts with the falling edge of input (Sm). Keeps the device off for the specified time. | s | 0...∞ | 7200 |
| Iva | Intensive ventilation absence | The value for the fan when presence is Off and (Hi) is greater than the parameter (Hmax). Intensive ventilation stops when (Hi) is less than (Hmax - 3%). In the "Supporting temperature control" and "Poor air quality (CO2)" modes, this value is used as the maximum. | % | 0...100 | 100 |
| Bva | Basic ventilation absence | Value for fan in automatic mode when presence is Off. In temperature mode this value is used as minimum. | % | 0...100 | 10 |
| Ivp | Intensive ventilation presence | The value for the fan when presence is On and (Hi) is greater than the parameter (Hmax). Intensive ventilation stops when (Hi) is less than (Hmax - 3%). In the "Supporting temperature control" and "Poor air quality (CO2)" modes, this value is used as the maximum. | % | 0...100 | 0 |
| Bvp | Basic ventilation presence | Value for fan in automatic mode when presence is On. In temperature mode this value is used as minimum. | % | 0...100 | 20 |
| Fpt | Frost protection temperature | If the outside temperature is below this value, ventilation will be stopped to prevent any damage. WARNING: If no external temperature reading is provided then this safety feature is not operational! | - | ∞ | -1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Energy Costs | Costs for running this source. Sources that have been configured to be 'Expensive' will only be requested by room controllers if no sources with higher priorities are available. | - | - | - |
| Extract / Supply permitted | Sets whether a fan is permitted to operate as extract only. When active, an extra input is available on the Leaf function block which can be used to start the extract mode. WARNING: If a room has an open fireplace an extract fan in close proximity may result in smoke being pulled into the room! | - | - | - |
| Maximum Air Exchange | Specify the maximum air exchange that the ventilation unit can provide for the current room. You will find information on this in the datasheet of the device manufacturer. This information is used for optimum adjustment of the controller. | m³/h | ∞ | 40 |

---