# Dewpoint Calculator

Source: https://www.loxone.com/enen/kb/dewpoint-calculator/

---

Calculates the dewpoint based on current temperature and relative humidity.

The Calculation Method uses the Magnus-Formula, so the allowed temperature range is between -65°C/-85°F and 60°C/140°F

Used approximation formula: k3 * ((k2 * ϑ) / (k3 + ϑ) + ln(H/ 100)) / ((k2 * k3) / (k3 + ϑ) - ln(H/ 100))

For ϑ > 0: k2 = 17.62, k3 = 243.12. For ϑ below 0: k2 = 22.46, k3 = 272.62

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| ϑ | Temperature | Current Temperature | ° | -65°C/-85°F...60°C/140°F |
| H | Relative Humidity | Relative Humidity of the Air | % | 1...100 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| ϑd | Dew Point | Calculated Dew Point Temperature | ° | ∞ |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| O | Output Offset | Adds a fixed offset to the calculated dew point | ° | ∞ | 0 |

---