# Event Database Connector

Source: https://www.loxone.com/enen/kb/event-database-connector/

---

This block can be used to transfer and display events, such as billing, in an Exosphere database.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Programming example](#baseconf)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| ETr | Trigger | Pulse on input triggers a write to the database. | 0/1 |
| Uid | User-ID | User-ID. If set before being triggered, the user fields of the corresponding user can be used. | - |
| CI1 | Custom input 1 | Additional custom input 1 | - |
| CI2 | Custom input 2 | Additional custom input 2 | - |
| CI3 | Custom input 3 | Additional custom input 3 | - |
| CI4 | Custom input 4 | Additional custom input 4 | - |
| CI5 | Custom input 5 | Additional custom input 5 | - |
| CI6 | Custom input 6 | Additional custom input 6 | - |
| CI7 | Custom input 7 | Additional custom input 7 | - |
| CI8 | Custom input 8 | Additional custom input 8 | - |
| CI9 | Custom input 9 | Additional custom input 9 | - |
| CI10 | Custom input 10 | Additional custom input 10 | - |
| CI11 | Custom input 11 | Additional custom input 11 | - |
| CI12 | Custom input 12 | Additional custom input 12 | - |
| CI13 | Custom input 13 | Additional custom input 13 | - |
| CI14 | Custom input 14 | Additional custom input 14 | - |
| CI15 | Custom input 15 | Additional custom input 15 | - |
| CI16 | Custom input 16 | Additional custom input 16 | - |

---

## Outputs

| Abbreviation | Summary | Description |
| --- | --- | --- |
| Log | Log output | Log output on every write operation to the database |
| API | API Connector | Intelligent API based connector. API Commands |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Td | Trigger Delay | Delays writing to database after trigger to make sure all inputs are set | ms | ∞ | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Column content | Edit column content. Specify the data that should be written to the database columns. | - |

---

## Programming example

In this example, a food and beverage billing for employees is recorded in a database.

A database must be created in [Exosphere](https://www.loxone.com/help/exosphere/) and integrated into the configuration using the "Database".

With the "Event Database Connector" block, the database can now be populated with data.

![Exo DbConEAPI](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Exo_DbConEAPI.png)

Double-clicking on the block opens the "Database Connector Configuration". Here, the contents of the columns can be defined.
If an option is grayed out, the datatype is not compatible with it.

![Exo DbConEColumns](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Exo_DbConEColumns.png)

Additionally, the block inputs "CI1-CI16" can be named. These will be displayed in the configuration but not in the Exosphere database.

![Exo DbConEUI](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Exo_DbConEUI.png)

The input "ETr" is triggered when a user is successfully authenticated via the NFC Code Touch. On every rising edge of this input, data is written to the database.

The ID of the authenticated user is transferred to the "Uid" input, and with additional logic on "CI1" and "CI2", the food and beverage type as well as the respective price are transferred.

![Exo DbConEExample](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Exo_DbConEExample.png)