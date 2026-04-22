# Motion Activated Stair Lights

Source: https://www.loxone.com/enen/kb/motion-activated-stair-lights/

---

## Brief: I want motion activated stair lights.

Illuminating a staircase at night not only increases visibility (making using them safer), it quite simply just looks beautiful. In this Use Case, we’ll show you how to implement motion activated stair lights – for each individual step.

Each step will be lit up using warm white LED strip as this reduces the number of dimmers needed. However, should you want to use RGBW strip the process will remain largely the same. We’ll show you how to configure the lighting to ensure that the steps are lit up in relation to whether the person is going up or down them. Then, once the person is done using the steps the lights should turn off in a similar fashion – with the first step’s lights be deactivated first.

## Solution: Using Loxone to commission motion activated stair lights.

Commissioning motion activated stair lights will ensure that the lights will only come on when the stairs are actually being used. If the steps are sufficiently lit, during the day, by natural light, then there is no need for the stair lights – hence they won’t be brought on. In our example, we’ve got 12 steps – all of which will be individually lit with [Warm White LED Strip](https://shop.loxone.com/enuk/led-strip-ww.html). This strip is controlled using [RGBW 24V Dimmers](https://shop.loxone.com/enuk/rgbw-24v-dimmer.html) – which have four 50W outputs. Meaning that they can control a maximum of 5m of Warm White LED Strip per channel.

Two [Presence Sensors](https://shop.loxone.com/enuk/presence-sensor-tree.html) are used to detect whether a person enters the staircase from below or above. If, for example, motion is detected downstairs, the stair lights will gradually be brought on one-by-one – from bottom to top. Then, once the lights have reached the top step, the lights will be turned off, again, from the bottom step. If motion is detected upstairs then the opposite will happen.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [3x RGBW 24V Dimmer Tree](https://shop.loxone.com/enuk/rgbw-24v-dimmer.html)
- [WW LED Strip](https://shop.loxone.com/enuk/led-strip-ww.html)
- [Presence Sensor Tree](https://shop.loxone.com/enuk/presence-sensor-tree.html)
- [24V Power Supply (10A)](https://shop.loxone.com/enuk/psu-24v-10a.html)

### Configuration:

[
![Nr. 101 Running Stairway Light New](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/06/Nr.-101-Running-Stairway-Light-New.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/06/Nr.-101-Running-Stairway-Light-New.jpg)

### Download the sample file:

### Running Stairway Light

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-101-Running-Stairway-Light-New.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-101-Running-Stairway-Light-New.loxone)

## Why you and your customer should consider installing motion activated stair lights?

Motion activated stair lights not only increase visibility in the dark, but they’re also a real eye-catcher. In this Use Case, we’ve demonstrated how motion activated stair lights can be achieved with LED Strip, however, thanks to Loxone’s flexible nature almost any lighting source could be used.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)