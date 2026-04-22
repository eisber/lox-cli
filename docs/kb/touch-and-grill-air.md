# Touch and Grill Air

Source: https://www.loxone.com/enen/kb/touch-and-grill-air/

---

## Contents
- [Technical Data](#TechData)
[Power Supply](#PoweSupp)
[Commissioning Air](#CommAir)
[Programming](#Prog)
[Functions](#Func)
- [Keys ](#keys)
- [Temperature Sensor](#TempSens)
- [Temperature Monitor](#TempMoni)
- [Timer function](#TimeFunc)
- [Standby mode](#StanMode)
- [Display Brightness](#DispBrig)
- [Downloads](#Down)

## Technical Data

					Detailed technical data for the Touch & Grill Air can be found [here](https://www.loxone.com/enen/datasheets/#100341).

Do not use the temperature sensors on an induction hob. Malfunction in the operation of the device and/or incorrect temperature readings may occur.

## Power Supply

			**Touch & Grill Air****
**
![tgbackconnections 296x300](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/07/tgbackconnections.png)

![cLoxone Touch and Grill back free 1024x936](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/07/cLoxone_Touch-and-Grill-back-free.png)

![cLoxone Touch  Grill Air 5 300x200](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/07/cLoxone_Touch__Grill_Air_5.jpg)
The Touch & Grill Air has an integrated rechargeable battery for flexible use. The battery is charged via the included power adapter. Only use the original mains adapter.

## Commissioning air:

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

[Then follow the pairing procedure](https://www.loxone.com/enen/kb/setting-up-air-devices/)

To activate the pairing mode manually, first switch the device off with [Key 6](#keys) and then on again. Now hold down the [pairing button (Key 7)](#keys) for at least 5 seconds.

## Programming

			The Touch & Grill Air is controlled via the Touch & Grill block. One unit can be connected with up to two blocks. This allows the device to be used in several rooms, e.g. kitchen and barbecue area. In the app, you can easily switch between the two rooms.

### Inputs

| Name | Description | Type | Values |
| --- | --- | --- | --- |
| T5 | Combined T5 input. This can be used multiple times throughout the config project. | T5 | – |
| Buttons 1 – 5 | These inputs are hidden by default and can be displayed via the properties of the device. | Digital | 0/1 |

###

## Function

##
![cLoxone Touch  Grill Air 20 300x200](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/07/cLoxone_Touch__Grill_Air_20.jpg)

### Keys

![TouchGrillAir Keys 1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/03/TouchGrillAir_Keys-1.png)

### Temperature Sensor

The Touch & Grill Air has two temperature sensors. A distinction is made between a green and a yellow sensor. The temperature of the sensors is displayed accordingly on the device’s display. The toggle button can be used to switch between the green and yellow sensors (if connected) or the current time and the timer if it is active.

![TouchGrillAir Sensors](https://www.loxone.com/enen/wp-content/uploads/sites/3/2019/03/TouchGrillAir_Sensors.png)

### Temperature Monitoring

Different temperature thresholds can be defined for both temperature sensors. As soon as a sensor falls below its threshold, the Touch & Grill Air emits an alarm tone, and the user is also notified via the App.

### Timer Function

A timer can be started via the timer button (middle bottom of the device). To change the duration, hold down the toggle button until the display shows the current set duration. The duration can now be set using the left and right buttons. Press the timer button again to finalise and confirm the timer. Press and hold the toggle button again to cancel the active timer.

![IG Touch and Grill Tastenstandard 01 1024x451](https://www.loxone.com/dede/wp-content/uploads/sites/2/2019/03/IG_Touch-and-Grill-Tastenstandard-01.png)

### Standby Mode

The Touch & Grill Air has a built-in rechargeable battery. Therefore it is also necessary to be able to switch off the device if necessary. This can be achieved by keeping the power button (middle top of the device) pressed until the display shows “OFF”. The unit can be switched on in the same way by keeping the toggle button pressed or alternatively by connecting the power supply.

### Display Brightness

The brightness of the Touch & Grill Air display is determined by the block within Config. The app can also be used to define whether the display should switch off after a longer period of inactivity. A brightness distinction is made between battery and permanent power source.

## Downloads:

If available, the following documents can be downloaded:
- Product Datasheet
- Pairing Instructions
- Quick Start Guide
- Declaration of Conformity

You can find available downloads [here](https://www.loxone.com/enen/datasheets/#100341).