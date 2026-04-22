# Text Generator

Source: https://www.loxone.com/enen/kb/text-generator/

---

Create customized texts with the 8 inputs which can be text or numbers. Double-click on the logic block to open the text editor where you can define your texts. The text will be generated after the trigger input is activated.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Tr | Triggers output | Creates the text on rising edge. Reset generated text on falling edge. | 0/1 |
| V1-8 | Value 1-8 | Input values can be used in the text editor | - |
| Uid | User-ID | User-ID. If set before being triggered, the user fields of the corresponding user can be used. | - |

---

## Outputs

| Abbreviation | Summary | Description |
| --- | --- | --- |
| Txt | Generated text | Outputs the generated text defined in the corresponding text editor when the input (Tr) is triggered |
| Txt1-4 | Generated text 1-4 | Outputs the generated text defined in the corresponding text editor when the input (Tr) is triggered |
| API | API Connector | Intelligent API based connector. API Commands |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Td | Trigger Delay | Delays text-generation after trigger to make sure all inputs are set | ms | 0...2147483647 | 0 |
| Tu | Update Interval | Interval how often the text gets updated while the Trigger input is active. Can be used to update the text using the new input values in a regular interval. 0 = Disabled | s | 0...2147483647 | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Text Editor - Txt | Specifies the text to set on the corresponding output when there is a rising edge on Tr. | - |
| Text Editor - Txt1 | Specifies the text to set on the corresponding output when there is a rising edge on Tr. | - |
| Text Editor - Txt2 | Specifies the text to set on the corresponding output when there is a rising edge on Tr. | - |
| Text Editor - Txt3 | Specifies the text to set on the corresponding output when there is a rising edge on Tr. | - |
| Text Editor - Txt4 | Specifies the text to set on the corresponding output when there is a rising edge on Tr. | - |

---