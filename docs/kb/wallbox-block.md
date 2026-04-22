# Wallbox

Source: https://www.loxone.com/enen/kb/wallbox-block/

---

The Wallbox function block is used to control a charging station for electric vehicles.

Charging a vehicle with the right power at the right time is essential for efficient energy management.

Therefore the function block comes with up to five customizable charging modes with which the charging power can be limited by individual logic.

With a dedicated input a charging session can be paused to avoid grid power peaks or equal.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Charging session](#session)
- [Minimum and maximum Limits](#limits)
- [Calculation of charging costs](#costcalc)
- [Programming example](#baseconf)
- [Authorization Sequence for Charging](#authsequence)
- [Detection of Invalid Meter Readings](#WallboxMeterInvalidReadings)
- [Regional Regulation](#WallboxRegional Limitation)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Ec | Enable charging | A pulse =1s or permanent on, activates charging and switches off again after a falling edge. If this input is inverted and not connected, charging is allowed. Eco charging if managed by Wallbox Manager. | - | 0/1 |
| Ecp | Enable charging priority | A pulse =1s or permanent on, activates Priority and ends the charging process completely after a falling edge. This input is only visible in certain configurations. | - | 0/1 |
| Vc | Vehicle Connected | - | 0/1 |
| Cp | Current charging power | - | ∞ |
| Mr | Meter reading | - | 0...∞ |
| Cac | Charging active | - | 0/1 |
| Sm1-5 | Set charging mode 1-5 | This input is only visible in certain configurations. | - | 0/1 |
| Lm1-5 | Limit charging mode 1-5 | Define up to 5 charging modes with different charging limits. This input is only visible in certain configurations. | kW | ∞ |
| Pm1-5 | Price charging mode 1-5 | Define up to 5 charging modes with different pricing per kWh. This input is only visible in certain configurations. | - | ∞ |
| Pmm | Price charging mode manual | Pmm is Manual charging mode 99 (App only). This input is only visible in certain configurations. | - | ∞ |
| Uid | User ID | - | - |
| Ls | Load shedding | If activated, the device pauses charging to prevent grid power peaks or similar issues, remaining paused as long as load shedding is active. | - | 0/1 |
| Off | Off | Pulse: reset outputs except meter outputs On: Block is locked. Dominating input. Resetting the Wallbox with input Off sets the mode from the (Muv) parameter. | - | 0/1 |
| R | Reset meter reading outputs | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Ca | Charging allowed | Charging is allowed when: -Ec (Enable Charging) is active -Ec is inverted and not connected -Charging is not disabled via Ls or Off -Charging limit is not below minimal charging power. | - | 0/1 |
| Vc | Vehicle connected | - | 0/1 |
| Cp | Current charging power | - | ∞ |
| M | Current charging mode | Charging mode 1-5. 99 = Manual mode (App only) | - | ∞ |
| Managed by Wallbox Manager: Current charging mode | 0 = Charging not allowed, 1 = Eco charging allowed, 2 = Priority charging allowed | - | ∞ |
| Tp | Target charging power | Target charging power according to current mode. Output is 0 if charging Limit is below minimal charging power or output Ca is 0. | kW | ∞ |
| Ls | Load shedding | - | 0/1 |
| Mr | Meter reading | - | ∞ |
| Ccc | Consumption current charge | - | ∞ |
| Clc | Consumption last charge | - | ∞ |
| Cd | Consumption today | - | ∞ |
| Cw | Consumption this week | - | ∞ |
| Cm | Consumption this month | - | ∞ |
| Cy | Consumption this year | - | ∞ |
| Lcl | Last charge log | Text output of - Vc On and Off time - Duration (Vc on/off) - Charged energy (kWh) - User ID. | - | - |
| Cac | Charging active | If the Cac input is unused and no Loxone Wallbox is connected, the output is 1 when the vehicle is connected and the Cp input is greater than the minimum charging power. | - | 0/1 |
| Cld | Consumption yesterday | - | ∞ |
| Clm | Consumption last month | - | ∞ |
| Cly | Consumption last year | - | ∞ |
| Cclc | Charging costs last charge | Provides the calculated costs of the last charging session. If a charging session is active, its running costs are output. | - | ∞ |
| Uid | User Id | - | - |
| Se | Pulse Session ended | - | 0/1 |
| Ss | Pulse Session started | - | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Muv | Mode after unplugging the vehicle | 0 = Keep current mode 1-5 = Switch to mode 1-5 This parameter is only visible in certain configurations. | - | ∞ | 0 |
| Cfp | Connection fee per hour | Price per hour while the vehicle is connected. This parameter is only visible in certain configurations. | Currency | ∞ | 0 |
| Mro | Meter reading offset | Value is added to output (Mr). | - | ∞ | 0 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Max. charging power | Max. charging power [kW] | kW | 0...∞ | 11 |
| Min. charging power | Min. charging power [kW] | kW | 0...∞ | 4.16 |
| Configuration | Configuration of the Inputs and Outputs Used. | - | - | - |
| Number of Entries | Maximum number of saved messages. | - | 1...100 | 100 |
| Relative metering | Active: The physical meter only sends partial quantities at intervals (relative), the block adds them up and uses them to form the meter reading. Not active: The physical meter provides its own total meter reading (absolute), the function block only maps it. | - | - | - |
| Report invalid Meter Reading | If checked, you will be notified if invalid Meter Reading values have been detected. For instance, when a physical counter sends unrealistic values due to transmission errors. | - | - | - |
| Regional regulation | Select the regulation applicable to your region. The Wallbox will adjust its behavior accordingly (e.g., voltage/frequency response). Additional equipment may be required (e.g. Energy Meter 3-Phase Tree). | - | - | - |

---

## Charging session

A charging session is the time between connecting and disconnecting a vehicle. The session can be paused / resumed with inputs Ec, Ls or App.

Charge log on output Lcl is set when starting / ending a session or if the userID changes during an active session.

---

## Minimum and maximum Limits

Minimum and maximum limits define the allowed range of Output Tp (Target charging Limit).

Charging Limit can be configured for each mode via Dialog/App, or via logic in Inputs Lm1-5.

If the charging limit falls below the minimum charging limit, charging is paused and target charging limit is set to 0.

The actual charging power is determined by the vehicle but will not exceed the maximum power settings.

---

## Calculation of charging costs

The calculated costs of a charging session are provided at the Cclc output.
This is done via the energy consumed and it’s price per charging mode, and the connection fee per hour.

During a charging session, the current costs are output and continuously updated.
After the end of the session, the costs remain available at the output until the start of the next session.

Calculation:
kWh mode 1 * price mode 1 + kWh mode 2 * price mode 2 + (other modes) + duration of charging session * Cfp

The calculation is done on a rolling basis, meaning changes in modes and prices are considered from the moment of the change onward, not retroactively.

When managed by the Wallbox Manager, its prices are used instead.

By setting separate prices for energy and connection fees, users can tailor cost calculations to their preferences.
With enough Wallboxes, it makes sense to determine the costs primarily by energy prices.
In cases where the number of Wallboxes is limited, opting for a higher hourly connection fee (Cfp) helps prevent prolonged vehicle occupancy.

---

## Programming example

In the properties window, you can define the number of entries in the history, the minimum and maximum charging power as well as a connection fee per hour:

![wb properties](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wb_properties.png)

Double click on the block to open the configuration window, here up to 5 charging modes can be created:

![wb chargingModes](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wb_chargingModes.png)

With additional logic at the input (Ec), charging is enabled when a user is successfully authenticated via the NFC Code Touch and the vehicle is connected.
An energy meter supplies the current charging power and meter reading to the function block.
The ID of the authenticated user is transferred at the "Uid" input.

![wb example](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/wb_example.png)

---

## Authorization Sequence for Charging

In commercial or communal installations where charging process authorization is necessary, the vehicle must be connected first, and only then can the charging be authorized via a pulse on Ec or Ecp (e.g., through NFC). If not, no charging will occur.
This sequence is critical because otherwise, an unauthorized vehicle could potentially charge at the expense of another.

In the programming example above, programming has been developed that allows for prior authorization. Following this, the charging process can be initiated for a limited time by connecting the vehicle.

In private households, the Ec input can often remain permanently active (negated) when unauthorized charging is not a concern. Under these circumstances, charging is generally permitted.

---

## Detection of Invalid Meter Readings

The Wallbox function block detects and ignores invalid readings at the Mr input(s), which may result from faulty data transmission from a physical absolute value meter.

In general, only increasing or constant meter readings are allowed; decreasing or jumping to 0 readings are discarded.

Unrealistically high increases in meter readings are discarded if:
- The change in value is greater than the total previous consumption and greater than 10000.
- If less than 8 hours have passed since the last value change.

---

## Regional Regulation

Regulations based on energy grid measurements can be configured with the setting 'Regional regulation'

This functionality requires a connected Energy Meter 3 Phase Tree to monitor relevant parameters for the specific regulation.

For Mode 'TOR 1.2 (AT / OVE R37)', charging is stopped when the grid voltage drops below the configurated settings and then resumed when conditions for Section 5.7.4.2 of this regulations are met again.

The target charging power (Tp) is gradually increased once per minute by 10% of the configured maximum charging power, starting with minimum charging power.