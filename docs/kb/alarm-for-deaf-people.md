# Alarm for Deaf People

Source: https://www.loxone.com/enen/kb/alarm-for-deaf-people/

---

## Brief: I want to set up an alarm for deaf people.

Everyday things that many people take for granted often cause difficulties for people with impaired hearing. This starts with simple things, such as not hearing the doorbell, and goes on to more serious things, such as not hearing a fire alarm. To ensure that deaf people can live an independent life a solution must be found to make sure that they’re still made aware of alarms and doorbells.

So in order to set up an alarm for deaf people, we need to convert audible signals into visual signals. In this Use Case, we use the lighting to visually signify alerts such as doorbells, fire and water alarms.

## Solution: Using Loxone to commission an alarm for deaf people.

In order to be able to use the lighting as an alarm for deaf people, RGBW-capable lighting must be used. It’s important that this is implemented in every room or alerts could be missed. If there isn’t currently RGBW lighting installed, some [LED Strip](https://shop.loxone.com/enuk/led-strip-rgbw.html) and an [RGBW 24V Compact Dimmer Air](https://shop.loxone.com/enuk/rgbw-24v-compact-dimmer.html) is a perfect retrofit option.

Our example has been configured as follows:

The doorbell signal, which can come either from the Intercom, NFC Code Touch or digital input, is connected to the (A) input of the [Lighting Controller](https://www.loxone.com/enen/kb/lighting-controller-v2/) via the “Bell” Function Block.

The “[Retractive Switch](https://www.loxone.com/enen/kb/retractive-switch/)” Function Block limits the pulse on the doorbell to a few seconds.

Input A (alarm) now causes the light to flash with white light. After the doorbell signal, the lighting mood reverts back to whatever was previously set.

If a [Fire or Water Alarm](https://www.loxone.com/enen/kb/fire-water-alarm/) is now triggered by a [Smoke Detector Air](https://shop.loxone.com/enuk/smoke-detector-air.html) or [Water Sensor Air](https://shop.loxone.com/enuk/water-sensor-air.html), a “Pulse Generator” is used to generate a flashing signal. These two flashing signals are connected to the Lighting Controller via a “Memory Flag”.

Two dedicated lighting scenes have been created for this purpose.

Red = Fire Alarm

Blue = Water Alarm

Your customers are thus immediately visually made aware of the current alarm status.

If the alarms are acknowledged, the lighting is centrally set to 100%. This is important in order to determine and locate the potential danger.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
[RGBW lighting controlled by the Miniserver](https://shop.loxone.com/enuk/accessories.html?c=lighting#content)
- [LED Spots RGBW Tree](https://shop.loxone.com/enuk/led-spot-rgbw-tree-white-uk.html)
- [LED Ceiling Light RGBW](https://shop.loxone.com/enuk/led-ceiling-light-rgbw.html)
- [RGBW 24V Compact Dimmer](https://shop.loxone.com/enuk/rgbw-24v-compact-dimmer.html) + [LED Strip](https://shop.loxone.com/enuk/led-strip-rgbw.html)
- Doorbell
- [Intercom](https://shop.loxone.com/enuk/intercom.html)
- [NFC Code Touch](https://shop.loxone.com/enuk/nfc-code-touch.html)
- [Smoke Detector Air](https://shop.loxone.com/enuk/smoke-detector-air.html)
- [Water Sensor Air](https://shop.loxone.com/enuk/water-sensor-air.html)

### Configuration:

[
![Alarm for Deaf People - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-70-lights-to-flash-for-deaf-people_1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-70-lights-to-flash-for-deaf-people_1.png)

[
![Alarm for Deaf People - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-70-lights-to-flash-for-deaf-people.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-70-lights-to-flash-for-deaf-people.png)

### Download the sample file:

### Lights to flash for deaf occupant

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-70-Program-lights-to-flash-for-deaf-occupan.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-70-Program-lights-to-flash-for-deaf-occupan.loxone)

## Why you and your customer should consider implementing an alarm for deaf people?

The aim of this intelligent solution is to give people with a hearing impairment more independence and above all more security in everyday life. Automation technology can make an important contribution to enabling people with hearing impairments to live more independently and safely in their own homes.

Unlike cameras, GPS trackers or similar, Loxone works discreetly in the background. Thus, it not only provides a higher level of security but also creates a good feeling!

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)