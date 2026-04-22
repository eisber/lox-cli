# PV Production Forecast

Source: https://www.loxone.com/enen/kb/pv-production-forecast/

---

The PV Production Forecast function block predicts photovoltaic production without requiring physical metering or orientation data. It uses absolute irradiance (GHI) forecast data from the Weather Service. Please note, it does not account for potential inverter limitations.

An active Weather Service is required to use this block.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Calculation](#productionforecastcalc)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Off | Off input | Sets the Ready output to OFF and the prediction outputs to -1 | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Ppwr | Potential Power | Indicates the PV power output that could currently be generated based on solar radiation provided by the weather service. -1 if no prediction can be provided. | kW | ∞ |
| Pp | Predicted period | Outputs the prediction for the given period of time starting from the next full hour. -1 if no prediction can be provided. | kWh | ∞ |
| Ptd | Predicted today | Outputs the prediction for today. -1 if no prediction can be provided. | kWh | ∞ |
| Pnd | Predicted next day | Outputs the prediction for the next day. -1 if no prediction can be provided. | kWh | ∞ |
| Ready | Prediction provided | OFF if control is locked via Off input or if no prediction can be provided (loss of internet connection, subscription expired, etc) | - | 0/1 |
| API | API Connector | Intelligent API based connector. API Commands | - | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Period | Calculated Period | Forecast is calculated over this period of time starting from the next full hour. | h | 0...72 | 24 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Max PV power [kWp] | PV's theoretical maximum power production | 0...∞ | 0 |

---

## Calculation

The PV production forecast is calculated based on the ratio between the current GHI (Absolute Radiations) forecast and the theoretical maximum GHI. This value is then multiplied by the maximum installed PV power (kWp) to determine the estimated PV production.

Formula:
**(GHI from Weather Service / theoretical GHI max) * installed kWp = estimated PV production**

Example:
- GHI (absolute radiations) from Weather Service: 486.94 W/m²
- Theoretical GHI max (standardized value): 1361 W/m²
- Installed PV system size (kWp): 10 kWp
- Calculation: (486.94 / 1361) * 10 = 3.6 kWh