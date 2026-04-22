# Wind Gauge

Source: https://www.loxone.com/enen/kb/wind-gauge/

---

With the function block Wind Gauge, the frequency input from a wind speed sensor can be converted to a wind speed value
Please note that only inputs with frequency counter functionality can be used.
If the wind sensor is connected to a Digital Input of the Multi Extension Air, the wind speed is transmitted every minute. As a result, the value for calculation must be divided by 60!

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Frequency Input](#Frequency)

---

## Inputs

| Abbreviation | Summary | Value Range |
| --- | --- | --- |
| F | Frequency | ∞ |

---

## Outputs

| Abbreviation | Summary | Unit | Value Range |
| --- | --- | --- | --- |
| Avg | Average wind speed | km/h | 0...∞ |
| G | 3 second average for gusts | km/h | 0...∞ |
| AvgMax | Maximum wind speed in the averaging period | km/h | 0...∞ |
| Wa | Wind alarm | - | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Avgt | Averaging-time | min | 0...∞ | 10 |
| F | Factor | Conversion coefficient Hz to km/h resp. m/s or other units as stated on datasheet | - | ∞ | 1 |
| W | Wind speed alarm | km/h | 1...∞ | 50 |

---

## Frequency Input

The wind sensor is connected to a digital input. The input must support frequency counter functionality, which needs to activated in the input's settings:

![Wind Frequency](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Wind_Frequency.jpg)