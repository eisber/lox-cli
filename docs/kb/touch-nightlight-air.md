# Touch Nightlight Air

Source: https://www.loxone.com/enen/kb/touch-nightlight-air/

---

## Contents
- [Technical Data](#TechData)
[Power Supply](#PoweSupp)
[Commissioning Air](#CommAir)
[Programming](#Prog)
[Functions](#Func)
- [Alarm clock](#AlarCloc)
- [Set alarm time](#SetAlarTime)
- [Time format](#TimeForm)
- [Display brightness](#DispBrig)
- [Temperature display](#TempDisp)
- [Downloads](#Down)

## Technical Data

					Detailed technical data can be found on the Product Datasheet [here](https://www.loxone.com/enen/datasheets/#100340).

## Power Supply:

			**Touch Nightlight Air**
![cLoxone Touch Nightlight Air back free 300x242](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/03/cLoxone_Touch-Nightlight-Air-back-free.png)
The Touch Nightlight Air is powered by the supplied USB power supply. Only use the original power supply provided.

## Commissioning Air:

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the light flashing red/green/orange.

[Then follow the pairing procedure](https://www.loxone.com/enen/kb/setting-up-air-devices/)

To activate the pairing mode, hold down the pairing button for at least 5 seconds after establishing power supply. The [pairing button](#Func) is button 7, centered at the bottom.

## Programming

			The Touch Nightlight Air is controlled via the [Alarm Clock block](https://www.loxone.com/enen/kb/alarm-clock/). If the Alarm Clock block is linked to a Touch Nightlight, a static alarm entry is displayed for it. It is best to use [Auto Configuration](https://www.loxone.com/enen/kb/auto-configuration/) for the initial setup. This will automatically configure the Nightlight and Alarm Clock together..

### Inputs

| Name | Description | Type | Values |
| --- | --- | --- | --- |
| T5 | Combined T5 input. | T5 | – |
| Buttons 1-5 | These inputs are hidden by default and can be displayed via the properties of the device. | Digital | 0/1 |
| Button 6 | This input is controlled via the upper middle button on the Touch Nightlight Air. | Digital | 0/1 |

### Outputs

| Name | Description | Type | Values |
| --- | --- | --- | --- |
| Smart Actuator RGBW | RGBW Lighting output for the Backlight on the Touch Nightlight Air. This is a standard Smart Actuator output to be used with a Lighting controller. | Smart Actuator | – |
| Dimmer RGB | Combined RGB output. This only appears if the device is set to “standard” type. | RGB | – |
| Dimmer WW | WW output. This only appears if the device is set to “standard” type. | Analog | 0…100 |
| Temperature | Displays the temperature from different sensors. If used it will swap between it every 5 seconds. | Analogue | ∞ |
| Status LED | LED in the top button (button 6). | Digital | 0/1 |

## Functions

![PH Nightlight Hand.min  3 800x450](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/03/PH_Nightlight-Hand.min_-3.png)

![TouchNightlightAir KeyAssignment](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/03/TouchNightlightAir_KeyAssignment.png)

### Alarm Clock function

The lower button can be used to activate and deactivate the alarm clock. The status LED indicates whether the alarm clock is currently active or inactive. An alarm can be deactivated by pressing the lower button. If another key is pressed during an active alarm, the snooze mode is activated. This is indicated by a pulse of the lower LED.

### Set Alarm time

![PH Nightlight Einstellung Weckzeit.min  3 800x367](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/03/PH_Nightlight-Einstellung-Weckzeit.min_-3.png)

To change the alarm time, hold down the lower button until the display shows the current alarm time. The alarm time can now be set using the left and right buttons. Press the lower button again to confirm the alarm time.

### Time Format

When using the 12-hour time format, the distinction between morning and afternoon is indicated by a dot on the display. The time format can be changed in the project’s properties in Loxone Config.

### Display Brightness

The Touch Nightlight Air automatically switches to “inactive mode” after 20 seconds. A distinction is made here between two display brightness levels (active/inactive). These brightness levels can also be adjusted via the visualization in the app or in Loxone Config on the assigned alarm clock block. If desired, the display can also be deactivated during inactivity.

### Temperature display

If the temperature output is used in Loxone Config, the Touch Nightlight Air changes every 5 seconds between the current time and the current temperature during normal operation. If the output is not used, only the current time is displayed during normal operation.

## Downloads:

If available, the following documents can be downloaded:
- Product Datasheet
- Pairing Instructions
- Quick Start Guide
- Declaration of Conformity

You can find available downloads [here](http://loxone.com/enen/datasheets/#100340).