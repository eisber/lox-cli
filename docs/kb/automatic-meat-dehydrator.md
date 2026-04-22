# Automatic Meat Dehydrator

Source: https://www.loxone.com/enen/kb/automatic-meat-dehydrator/

---

## Brief: I want an automatic meat dehydrator.

Many lovers of dried and cured meat would like to make this speciality themselves. In principle, this is possible with the help of a self-constructed meat dehydrator. However, for successful dehydration, constant temperature and humidity must be maintained. No matter whether it is biltong or beef jerky – for this to succeed, these values must not fluctuate too much. With Loxone, this process can be automated giving you an automatic meat dehydrator. In this Use Case, we’ll show you how to achieve this.

## Solution: Using Loxone to create an automatic meat dehydrator.

You can construct your own DIY meat dehydrator with a 40W bulb for heat generation and a 24V fan for humidity control. Then using Loxone, this dehydrator can be automated quickly and easily.

As Biltong is traditionally produced outdoors in South Africa, it’s important to replicate these conditions.  **

**

The ideal temperature for production is 30 – 35°C. The perfect humidity is 40%. However, it must not fall below 30% and must not exceed 50%. In our example, the [Room Comfort Sensor Air](https://shop.loxone.com/enuk/room-comfort-sensor-air.html) is used to record these values exactly. If the humidity or temperature falls below or exceeds one of these thresholds, the [Miniserver](https://shop.loxone.com/enuk/miniserver.html) sends a push notification.

The 40W bulb (for heat generation) and 24V fan (for humidity regulation) are controlled through the two relays on the [Shading Actuator Air](https://shop.loxone.com/enuk/shading-actuator-air.html). Using the “[2 Position Controller](https://www.loxone.com/enen/kb/2-position-controller/)” Function Block we can ensure that both the temperature and humidy stay within the desired ranges.

If a measured value is outside the desired range, a push notification will be sent with the help of the “Analogue Value Monitor” Function Block.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Room Comfort Sensor](https://shop.loxone.com/enuk/room-comfort-sensor-air.html)
- [Shading Actuator Air](https://shop.loxone.com/enuk/shading-actuator-air.html)

### Configuration:

[
![Automatic Meat Dehydrator - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-94-DIY-Draying-Champer.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-94-DIY-Draying-Champer.png)

### Download the sample file:

### Drying Champer

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Drying-Champer.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Drying-Champer.loxone)

## Why you and your customer should consider commissioning an automatic meat dehydrator?

We have to admit that this Use Case is one for true enthusiasts. However, it still demonstrates how quickly and easily you can Create Automation using the Loxone Miniserver – in almost any scenario.

Good luck with the production of your own dried meat!

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)