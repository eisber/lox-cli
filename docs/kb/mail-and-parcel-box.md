# Mail and Parcel Box

Source: https://www.loxone.com/enen/kb/mail-and-parcel-box/

---

The block can be used to notify the user of the delivery of parcels or mail.
The block can be configured either with the Paketsafe Air or manually using the block inputs.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| DisN | Disable Notifications | 0/1 |
| P | Package received | This input is only visible in certain configurations. | 0/1 |
| M | Mail received | This input is only visible in certain configurations. | 0/1 |
| Cp | Confirm package | This input is only visible in certain configurations. | 0/1 |
| Cm | Confirm mail | This input is only visible in certain configurations. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| P | Package present | ON when a package is in the drop box. | 0/1 |
| M | Mail present | ON when there is mail in the mailbox. | 0/1 |
| Pon | Pulse on package received | 0/1 |
| Poff | Pulse on package confirmed | 0/1 |
| Mon | Pulse on mail received | 0/1 |
| Moff | Pulse on mail confirmed | 0/1 |
| Txle | Text of last event | Provides date, time and description of the last event. | - |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Number of Entries | Maximum number of saved messages. | 2...100 | 20 |
| Assigned Paketsafe Air device | The Paketsafe Air device that is assigned to this function block. | - | - |

---