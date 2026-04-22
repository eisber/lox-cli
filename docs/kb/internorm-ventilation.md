# Internorm Fan

Source: https://www.loxone.com/enen/kb/internorm-ventilation/

---

This block is used to control an Internorm I-tec ventilation system. Humidity, CO2 value, as well as indoor and outdoor temperature can be included.

When using a Motion/Presence Sensor, different ventilation speeds can be defined depending on presence.

For temperature support, the fan must be assigned as a source to a room controller. In addition, the system variable outdoor temperature is required.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Device Errors](#device-error)
- [Maintenance Interval / Filter Change](#filterchange)
- [Frost Protection](#frostprotection)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Hi | Humidity indoor | % | 0...100 |
| CO2 | CO2 indoor | ppm | 0...∞ |
| Sat | Supply air temperature | Used to support temperature control in combination with the Intelligent Room Controller and for frost protection. If this input is not connected the system variable "Outdoor Temperature" is used. If this variable is not available, support for temperature control and frost protection is not available. | ° | ∞ |
| Dwc | Door/window contact | 1 = Open, 0 = Closed | - | 0/1 |
| P | Presence | - | 0/1 |
| Off | Off | 1 = control stopped and set to lock | - | 0/1 |
| Sm | Sleep mode | Digital Input - Sleep Mode: Turns off ventilation for the time set in parameter (Smt). Afterwards, the block will start again with ventilation. | - | 0/1 |
| B | Boost | Stops the control and sets the output to 100 percent. | - | 0/1 |
| Ex | Exhaust air | Stops the control and activates exhaust air at 100 percent. ATTENTION: This input can only be used if the block property "Exhaust air/supply air operation allowed" is activated. This input is only visible in certain configurations. | - | 0/1 |
| Rtd | Reset to default | Resets parameters and settings of the block to the default values as specified in the block preset. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| S | Status | Analog Output - Value indicates which parameter is driving the ventilation. This output is for informational reasons only. 0: Basic Ventilation, 1: Increased Humidity, 2: Temperature Control, 3: Poor Air Quality (CO2), 4: Manual Stop, 5: Window Open, 6: Manual Boost, 7: Manual Control User Interface, 8: Manual Exhaust Air, 9: Sleep Mode, 10: Frost Protection. | 0...10 |
| Fc | Filter change | Digital Output - Filter Change: Indicates whether air filters need to be changed. | 0/1 |
| Error | Error | Analogue error output: Indicates if there is an error: 0 = No Error, 1 = Offline, 2 = Flaps closed, 100-115 = Internorm error code. | ∞ |
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
| Bvp | Basic ventilation presence | Value for fan in automatic mode when presence is On. In temperature mode this value is used as minimum. | % | 0...100 | 22 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Energy Costs | Costs for running this source. Sources that have been configured to be 'Expensive' will only be requested by room controllers if no sources with higher priorities are available. | - |
| Extract / Supply permitted | Sets whether a fan is permitted to operate as extract only. When active, an extra input is available on the Leaf function block which can be used to start the extract mode. WARNING: If a room has an open fireplace an extract fan in close proximity may result in smoke being pulled into the room! | - |
| Airflow direction (heat exchanger disabled) | Sets the direction of ventilation when the heat exchanger is switched off for an Internorm fan. If the heat exchanger is active, there is no prevailing ventilation direction, supply air and exhaust air run simultaneously. | - |
| Linked Fan | Fan linked to this function block. Devices that can be assigned: Internorm I-Tec Ventilation | - |

---

## Device Errors

If one or more ventilation apertures have been reported as closed, they must be reopened before operation can continue. If you device is reporting as offline, please check the power connection and wireless signal. Further information can be found [here](https://www.loxone.com/enen/kb/commissioning-internorm-i-tec-devices/).

---

## Maintenance Interval / Filter Change

The fan filters must be replaced or cleaned at regular maintenance intervals. When the window is open, the two filter covers for supply and exhaust air are visible and can be opened with the aid of a flat head screwdriver. Some pressure must be exerted towards the frame.
The procedure is described in detail in the **[Internorm operating and maintenance manual](https://www.internorm.com/fileadmin/internorm/Konzern-Laender/PDF_Kataloge/Internorm-Fibeln/Internorm_Fibel_UK.pdf)**.

After servicing the filters, press the + and - button on the Internorm control panel simultaneously for at least 5 seconds to confirm the maintenance and reset the warning.

---

## Frost Protection

The Internorm itec Ventilation unit has its own frost protection. If the internal temperature of the ventilation unit is too low the device will stop. When the unit is in this state manual control is currently not possible. Below a temperature of -20°C the ventilation unit will be displayed as disabled via the system status. Depending on how it has been installed this frost protection may already kick in a lot sooner. Due to technical limitations it is unfortunately not possible to display this. Further information about the frost protection of the ventilation unit can be found [here.](https://www.internorm.com/fileadmin/internorm/Konzern-de/Download/Gebrauchsanw/Internorm_Bedienung_I-tec_Lueftung_de.pdf)