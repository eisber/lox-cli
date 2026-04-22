# Wallbox Manager

Source: https://www.loxone.com/enen/kb/wallbox-manager/

---

The Wallbox Manager block takes over the energy management of multiple Wallboxes.
It fulfills the following functions:

**Load limitation**
To prevent upstream fuses from tripping, all wallboxes and their electrical cabling structure can be replicated in the block. Both the total power and the power per fused strand can be limited.

**Power distribution**
Distributes the available power to all actively charging wallboxes, considering the predefined power limits.
If a wallbox is in Eco or Priority mode, but no charging session is active, the defined minimum charging power is allocated and vehicle charging is allowed. If the minimum charging power is 0, then the available power will be allocated, but with a maximum of 1.3 kW. This does not affect the total assigned power "Ap". Once a charging session becomes active, the available power is redistributed among the wallboxes and added to "Ap".
By default, vehicles charge in Eco mode, sharing only the available surplus power. Only vehicles charging in Priority mode are assigned as much power as possible.

**Presetting costs**
Definition of prices per kWh for charging in Eco and Priority modes, as well as a connection fee per hour. These prices are transferred to the linked Wallbox blocks.

**User Interface**
Shows an overview of the energy distribution with status and power of the linked Wallbox blocks.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Application](#application)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Pmax | Maximum total power | Maximum total power available to the Wallboxes. Used primarily as maximum value for overload protection. Can also be used to restrict charging power during periods of high electricity prices or high consumption. | kW | 0...∞ |
| Peco | Power for Eco charging | Surplus power, which is distributed to the Wallboxes for Eco charging. | kW | 0...∞ |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. The name of the connected sensor is used in the user interface. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Cp | Current power | Current total power consumption of all Wallboxes. For wallboxes whose power consumption is not known (no meter available or Cp input not used), the assigned power is assumed. | kW | 0...∞ |
| Ap | Assigned Power | Total power assigned to the wallboxes. Sum of Tp of all linked wallboxes. | kW | 0...∞ |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| PrEco | Price Eco | Price per kWh for Eco charging. | Currency | ∞ | 0.1 |
| PrPrio | Price Priority | Price per kWh for Priority charging. | Currency | ∞ | 0.2 |
| Cfp | Connection fee per hour | Price per hour while the vehicle is connected. | Currency | ∞ | 0 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Configure... | Configure the Wallbox Manager by adding Wallboxes and grouping them together. | - | - |
| Activity Log Entries | Number of entries in the activity log. 0: log is disabled The activity log tracks relevant changes since program start. | 0...100 | 20 |

---

## Application

First, existing [Wallbox blocks](https://www.loxone.com/help/wallbox-block) are added in a tree structure and grouped into strands. A strand is a group of wallboxes that are protected by a common higher-level fuse. The power can be limited for the individual strings. Select a value that matches the fuse protection or load capacity of the string:

![WBM ConfigStrands](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/WBM-ConfigStrands.png)

As soon as a Wallbox block has been added, only two fixed charging modes (Eco and Priority) are available. The energy distribution is controlled by the Wallbox Manager.

The maximum power and the surplus power available for Eco charging must be specified at the inputs of the Wallbox Manager:

![WBM ConfigInputs](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/WBM-ConfigInputs.png)