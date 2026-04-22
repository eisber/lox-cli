# Daylight Responsive Lighting

Source: https://www.loxone.com/enen/kb/brightness-control/

---

Function block for constant brightness control. Requires a well-positioned sensor that quickly detects the brightness in the room.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Basic Programming](#basic)
- [Selecting and positioning the brightness sensor](#SensorPlacement)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Br | Current brightness | ∞ |
| Set | Sets output (Lc) to value of input (Set) | 0...100 |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |
| Act | Activation | Activates constant brightness control when 1. | 0/1 |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Lc | Light circuits | Supported are: Smart Actuator RGBW or WW, Lumitech, RGB, 0-100%, 0-10V, 1-10V. If multiple actuators are connected, they must be of the same type. | ∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Hys | Hysteresis | If the difference between target and actual value is greater than this hysteresis, the controller will adjust. | % | 0...100 | 10 |
| Sts | Step size | The larger the step size, the quicker the target value is reached. However, large steps may cause the light change to be visible. | % | 0...100 | 5 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Target brightness | This brightness is to be maintained when the controller is active. | ∞ | 5000 |
| Set target brightness | Determine the target brightness to be maintained when the controller is active. | - | - |

---

## Basic Programming

The Constant Brightness Controller is typically used in combination with the Lighting Controller. It is activated via the input (Act), the dimmable lighting is connected to the output (Lc):

![10.5 daylight harvesting basic](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/10.5 daylight harvesting basic.png)

In this configuration, the Constant Brightness Controller is only used to switch the lighting on or off, the Constant Brightness Controller adjusts the brightness of the lights.

If the lighting is switched on by the Lighting Controller, but the measured brightness in the room is already sufficient, then the Constant Brightness Controller does not switch on the lighting.

Use the "Set target brightness" function in the block properties to adjust the lighting to the desired brightness. Fine-tuning is recommended after the room has been furnished and set up.

If necessary, use several controllers, sensors and lighting groups for different room areas. For example, lighting is often not necessary for areas near the window, but it is often necessary for areas with less daylight.

---

## Selecting and positioning the brightness sensor

Sensors that quickly detect changes in brightness are recommended, such as the Motion Sensor Tree. When using the Motion Sensor Air, the transmission interval for the light sensor can be reduced to 1 minute. However, the system will still react slower in this case.

The location of the sensor is crucial for satisfactory operation. Select a location where the brightness for the desired area can be measured effectively, e.g. directly above a desk, or even better on the desk surface.

Note that direct or reflected daylight or sunlight may hit the sensor but not the desk surface, or vice versa. This should be considered and avoided by carefully positioning the sensor.