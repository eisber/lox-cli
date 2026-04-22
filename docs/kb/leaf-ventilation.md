# Leaf Ventilation

Source: https://www.loxone.com/enen/kb/leaf-ventilation/

---

Controls a Leaf home fan based on humidity, temperature, and CO2. For temperature support, the fan must be assigned to a room control as a source. In addition, the system variable "outside temperature" is required.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Device Errors](#device-error)
- [Maintenance Interval / Filter Change](#filter-change)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Hi | Humidity indoor | % | 0...100 |
| CO2 | CO2 indoor | ppm | 0...∞ |
| Sat | Supply air temperature | Indicates the temperature of the incoming air. Used to support temperature control. If this input is not connected the System Variable "Outdoor Temperature" is used. If this value is not available, support for temperature control is not possible. | ° | ∞ |
| Dwc | Door/window contact | ON: window open, OFF: window closed. Ventilation is deactivated when the window is open. | - | 0/1 |
| P | Presence | - | 0/1 |
| Off | Off | Stops the fan and closes the vent as long as ON. | - | 0/1 |
| Sm | Sleep mode | Switches off ventilation for the time set in parameter (Smt). After that time, ventilation is resumed. | - | 0/1 |
| B | Boost | Stops the control and sets the output to 100 percent. | - | 0/1 |
| Ex | Exhaust air | Stops the control and activates exhaust air at 100 percent. ATTENTION: This input can only be used if the block property "Exhaust air/supply air operation allowed" is activated. This input is only visible in certain configurations. This input is only visible in certain configurations. | - | 0/1 |
| Cfc | Confirm Filter Change | - | 0/1 |
| Rtd | Reset to default | Resets parameters and settings of the block to the default values as specified in the block preset. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| S | Status | Indicates why the fan is active. This output is for informational reasons only. 0: Basic ventilation 1: Increased humidity 2: Supporting temperature control 3: Poor air quality (CO2) 4: Manual stop 5: Window open 6: Manual boost 7: Manual App 8: Manual exhaust air 9: Sleep mode. | 0...9 |
| Fc | Filter change | Indicates whether air filter needs to be replaced. | 0/1 |
| Error | Error | 0=No error, 1=Offline, 2=Stuck, 3=Aperture error. | 0...3 |
| API | API Connector | Intelligent API based connector. API Commands | - |

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
| Ivp | Intensive ventilation presence | The value for the fan when presence is On and (Hi) is greater than the parameter (Hmax). Intensive ventilation stops when (Hi) is less than (Hmax - 3%). In the "Supporting temperature control" and "Poor air quality (CO2)" modes, this value is used as the maximum. | % | 0...100 | 30 |
| Bvp | Basic ventilation presence | Value for fan in automatic mode when presence is On. In temperature mode this value is used as minimum. | % | 0...100 | 20 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Energy Costs | Costs for running this source. Sources that have been configured to be 'Expensive' will only be requested by room controllers if no sources with higher priorities are available. | - |
| Extract / Supply permitted | Sets whether a fan is permitted to operate as extract only. When active, an extra input is available on the Leaf function block which can be used to start the extract mode. WARNING: If a room has an open fireplace an extract fan in close proximity may result in smoke being pulled into the room! | - |
| Airflow direction (heat exchanger disabled) | Sets the airflow direction on a pair of Leaf fans when the heat exchangers are disabled. If the heat exchangers are active then there is no set direction since the direction is reversed in regular intervals. | - |
| Assigned Fan A 1 | Leaf fans are synchronized. Whilst fan A provides fresh air, fan B operates to extract stale air and vice versa. Please ensure that your configuration always contains the same number fans set to type A and B. WARNING: An uneven split may lead to either higher / lower air pressure in a room! | - |
| Assigned Fan B 1 | Leaf fans are synchronized. Whilst fan A provides fresh air, fan B operates to extract stale air and vice versa. Please ensure that your configuration always contains the same number fans set to type A and B. WARNING: An uneven split may lead to either higher / lower air pressure in a room! | - |

---

## Device Errors

If you receive a message indicating that your Leaf unit has experienced an Aperture error or a Fan failure, refer to the [Diagnostics Documentation](https://www.loxone.com/dede/kb/leaf-airtree#DiagnosticInputs) for details.

---

## Maintenance Interval / Filter Change

Information on the filter change or maintenance interval of the device can be found on the [device documentation](https://www.loxone.com/dede/kb/leaf-airtree#filterchange) page.