# LED strip lighting for stairs

Source: https://www.loxone.com/enen/kb/led-strip-lighting-for-stairs/

---

## Brief: I want to have indirect lighting installed in my stairs.

Stairs always pose a certain risk of accidents. Good staircase lighting is extremely important because injuries can occur as a direct result of poor or no staircase lighting at all. LED strip lighting for stairs is an easy and modern way of achieving this.

Indirect staircase lighting offers a high degree of safety and discreetly embellishes the staircase and corridor. To achieve this the light must be sufficiently bright but must not dazzle.

## Solution: Using Loxone LED strip lighting for stairs.

LED strip lighting for stairs can be easily implemented or retrofitted with Loxone. A [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html) is installed above and below the stairs. Then, [LED strip](https://shop.loxone.com/enuk/catalogsearch/result/?cat=0&q=led+strip) should be installed as desired, for example under each step, to the side of the stairs or under the handrail. An [RGBW dimmer](https://shop.loxone.com/enuk/catalogsearch/result/?cat=0&q=rgbw+dimmer) allows easy control of the RGBW LED lighting.

If the Motion Sensor detects movement, the Miniserver knows that it has to send a signal to the LED strip. In the given example we have created different lighting moods: because in the morning and evening we would like to have softer light than during the day. If you approach the stairs at night, the night lighting mood is selected.

Since Motion Sensors have been configured in a thoroughfare, the staircase lighting will be activated for two minutes once motion has been detected.

The example can, of course, be refined further. For example, the lighting times can be adapted or you configure certain moods which can be activated by a switch (e.g. cleaning mode = bright light, anti-mosquito mode = red lighting on summer evenings, etc.).

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html)
- [LED Strip](https://shop.loxone.com/enuk/catalogsearch/result/?cat=0&q=led+strip)
- [RGBW Dimmer](https://shop.loxone.com/enuk/catalogsearch/result/?cat=0&q=rgbw+dimmer)

### Configuration:

[
![LED strip lighting for stairs - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-87-Lighting-Stairway-01.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-87-Lighting-Stairway-01.png)

[
![LED strip lighting for stairs - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-87-Lighting-Stairway-02.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-87-Lighting-Stairway-02.png)

[
![LED strip lighting for stairs - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-87-Lighting-Stairway-03.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-87-Lighting-Stairway-03.png)

### Download the sample file:

### Lighting Stairway

			[Config 10.3.11.27](https://www.loxone.com/wp-content/uploads/sites/3/2020/04/Use-Case-87-Config-Lighting-Stairway.Loxone)

			[Download Config](https://www.loxone.com/wp-content/uploads/sites/3/2020/04/Use-Case-87-Config-Lighting-Stairway.Loxone)

## Why you and your customer should consider LED strip lighting for stairs?

LED strip lighting for stairs is not only visually attractive it’s also extremely practicality. Good staircase lighting minimises the risk of accidents in private homes, commercial or catering premises. Fear for family members, employees or guests can be greatly reduced.

The LED strip from Loxone is particularly suitable for indirect staircase lighting. It offers warm but still bright light, however, does not dazzle. In addition, it is dimmable (useful for basic lighting), has a long life span and low power consumption.

This Use Case offers different possibilities depending on the shape of the staircase: handrail, wall lighting, etc. – let your creativity run free!

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)

n