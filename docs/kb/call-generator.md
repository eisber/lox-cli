# Call Generator

Source: https://www.loxone.com/enen/kb/call-generator/

---

Calls a number on rising edge. Message and phone number can be customized with placeholders (e.g. <v1>, <user.phone>, <sysvar.rain>, etc). Double-click on the logic block to open the text editor where you can edit the message that will be played during the call. The function block also contains optional pulse outputs when key presses 0-9 are detected

For the services (Weather, Caller) the Miniserver must be [registered](https://portal.loxone.com/products) first.
After the services have been purchased in the [shop](https://shop.loxone.com/), they can be activated in the Loxone Partner Portal. The license number can be found on the invoice.
A valid internet connection is required for these services.

Each purchased Caller Service is assigned its own unique telephone number.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Limitations](#limit)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tr | Triggers output | Calls a phone number on rising edge. | 0/1 |
| V1-8 | Value 1-8 | Input values can be used in the message and in the phone number property field | - |
| Uid | User-ID | User-ID. If set before being triggered, the user fields of the corresponding user can be used. | - |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Dk0-9 | Dial key 0-9 | Pulses on key press during the call | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Td | Trigger Delay | Delays the call after trigger to make sure all inputs are set | ms | 0...2147483647 | 0 |
| Tu | Update Interval | Interval how often the call is initiated while the Trigger input is active. Can be used to update the call parameters using the new input values in a regular interval. 0 = Disabled | s | 0...2147483647 | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Language | Language used when playing the message | - |
| Phone number | Phone number to be called. Placeholders can be used (e.g. , , , etc). Note that only use numbers are supported. (e.g. 0043728770700) | - |
| Message | Message to be played during the call. Placeholders are supported (e.g. , , , etc) | - |

---

## Limitations

The Caller Service is limited to 10 calls to the same number per minute.
Calls exceeding this limit are blocked.

Calls can be placed again once the number of calls to the same number in the last minute drops below 10.
Each call counts towards this limit, even if it was blocked.

The maximum duration of the call is approximately 40 seconds.