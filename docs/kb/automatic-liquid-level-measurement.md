# Automatic Liquid Level Measurement

Source: https://www.loxone.com/enen/kb/automatic-liquid-level-measurement/

---

## Brief: Is there a way to automatically measure the liquid level in a cistern?

Whether for watering a garden, cleaning cars, sprinkling a football pitch, flushing toilets or filling swimming pools, the use of rainwater is a good way of remaining environmentally friendly. After all, drinking water is an expensive commodity. In addition, high energy costs are incurred for extraction and treatment. So it would definitely be a waste of natural and financial resources. A water cistern serves as a catchment and storage tank for rainwater. With a cistern, the irrigation of a garden can be implemented in an economically-friendly manner. This Use Case will show how automatic liquid level measurement can be integrated into a Loxone project.

## Solution: Using Loxone for automatic liquid level measurement of rainwater collection.

With the help of Loxone Config, automatic liquid level measurement of the cistern and a warning function can be implemented in a few minutes.

The cistern is monitored by an [Ultrasonic Sensor](https://shop.loxone.com/enuk/ultrasonic-sensor.html), which is placed on the lid of the water tank. This sensor has an integrated 0-10V converter. This allows you to determine which range the water level should be measured (here: 0-400 cm). Every 60 minutes the Ultrasonic Sensor measures the level in the cistern and passes the information on to the Miniserver.

If a critical water level is reached (below 50%), the owner is informed via push notification through the Loxone App. This prevents the water pump from running dry. For this, different status values for water level measurement were set in advance with the help of the [Status](https://www.loxone.com/enen/kb/status/) Function Block
- Empty (water level = 0)
- Major Error (water level < 100 cm)
- Warning (water level between 100 and 200 cm)
- Done (water level between 200 and 350 cm)
- Full (water level >= 350 cm)

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Ultrasonic Sensor 0-10V](https://shop.loxone.com/enuk/ultrasonic-sensor.html)

### Configuration:

[
![Automatic Liquid Level Measurement - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-64-Liquid-Levels.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-64-Liquid-Levels.png)

### Download the sample file:

### Fill Levels

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-64-Fill-Levels.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-64-Fill-Levels.loxone)

## What are the advantages of using Loxone automatic water level measurement?

Automatic liquid level measurement is a great addition for new or existing Loxone projects. Without much effort, an ultrasonic sensor can be installed in the cistern and integrated into a smart system. The owners are informed about critical water levels by push notifications. The warning function thus prevents the water pump from running dry and saves a lot of trouble and costs. The level can also be viewed remotely.

Automatic liquid level measurement can also be applied to other applications. For example, the automatic level measurement could also be used in rain barrels, wells or tanks*. The example can also be extended further and an automatic switch-off of the water pump could be implemented, a notification via the Caller Service can be played out or the display in the app can be adapted. There are no limits to your creativity!

**Please check the related legal regulations.*

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)