# Heating Curve

Source: https://www.loxone.com/enen/kb/heating-curve/

---

Calculates the heating flow temperature based on weather compensation. Inputs (Tt) and (Ct) read the setpoint and outdoor temperatures, respectively.
The slope of the heating curve determines how the system responds to changes in outdoor temperature. This is defined by parameter (S), which must be adjusted to match the characteristics of the heating system.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Curve](#curve)

---

## Inputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Tt | Target temperature | ° | ∞ |
| Ct | Current temperature | Current outdoor temperature | ° | ∞ |
| Dis | Disable | Disables input (Tt) when ON. | - | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Unit | Value Range |
| --- | --- | --- | --- | --- |
| Ft | Flow temperature | ° | ∞ |
| Iv | Invalid values | 1 when (Ft) would exceed (minFt) or (maxFt) | - | 0/1 |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| S | Slope | Slope of the heating or cooling curve | - | 0.05...2.5 | 1,3 |
| O | Offset | Parallel shift of the heating curve or cooling curve (when heating, the flow setpoint temperature is increased by this value, when cooling it is decreased) | - | ∞ | 0 |
| minFt | Minimum flow temperature | The outdoor temperature must be provided to calculate the flow temperature, either using input Ct or the system variable. | ° | ∞ | 15 |
| maxFt | Maximum flow temperature | The outdoor temperature must be provided to calculate the flow temperature, either using input Ct or the system variable. | ° | ∞ | 65 |

---

## Curve

The formula for calculating the flow temperature (Ft) is provided below:

![heatcurve formula](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/heatcurve_formula.png)

![heatcurve curve](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/heatcurve_curve.jpg)

Image source: http://de.wikipedia.org/wiki/Heizkurve

This characteristic relates to a setpoint temperature (input Tt) of 20°C.