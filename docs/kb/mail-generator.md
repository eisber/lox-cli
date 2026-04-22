# Mail Generator

Source: https://www.loxone.com/enen/kb/mail-generator/

---

Sends an email on rising edge. All properties can be customized with placeholders (e.g. <v1>, <user.email>, <sys.rain>, etc). Double-click on the logic block to open the text editor where you can edit the body (message) of the email.

The Cloud Mailer is a free email service from Loxone, available after [registering](https://portal.loxone.com/products) the Miniserver.

The Mailer requires your own email account, the Miniserver does not need to be registered for this.

Up to 200 emails can be sent per day per Miniserver.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tr | Triggers output | Sends the email on rising edge. | 0/1 |
| V1-8 | Value 1-8 | Input values can be used in the email message and properties | - |
| Uid | User-ID | User-ID. If set before being triggered, the user fields of the corresponding user can be used. | - |

---

## Outputs

| Abbreviation | Summary | Description |
| --- | --- | --- |
| API | API Connector | Intelligent API based connector. API Commands |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Td | Trigger Delay | Delays email-generation after trigger to make sure all inputs are set | ms | 0...2147483647 | 0 |
| Tu | Update Interval | Interval how often the email gets sent while the Trigger input is active. Can be used to update the email parameters using the new input values in a regular interval. 0 = Disabled | s | 0...2147483647 | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Mailer | Mailer service to be used for sending the email | - |
| Recipient Email | Email address of the recipient. To enter multiple recipients, use a semi-colon (;) as separator. Placeholders can be used (e.g. , , , etc) | - |
| Subject | Subject of the email. Variables can be used (e.g. , , , etc) | - |
| Body | Message (body) of the email. Placeholders can be used (e.g. , , , etc) | - |

---