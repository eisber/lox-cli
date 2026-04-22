# Smart Garden Watering System

Source: https://www.loxone.com/enen/kb/smart-garden-watering-system/

---

## Brief: I want to have a smart garden watering system that monitors of the flowerbeds.

If the lawn suffers from a lack of water in the summer, this is reflected in unattractive brown spots, weeds and lack of growth. To save you from this, in this Use Case we’ll show you how you can implement a smart garden watering system with Loxone.

## Solution: How to use Loxone to automate a smart garden watering system.

In our example, the garden is watered using water stored in a rainwater tank. The tank is equipped with an [Ultrasonic Sensor](https://shop.loxone.com/enuk/ultrasonic-sensor.html) that checks the water level. If the water level is too low, the garden is watered with normal water. The water level can also be checked at any time from the Loxone App.

Automatic watering can be activated in several ways. Either by using the weather forecast via the [Weather Station](https://shop.loxone.com/enuk/weather-station.html) or the [Weather Service](https://shop.loxone.com/enuk/weather-service-10-year.html) or by using moisture sensors in the garden. Two hours before sunrise watering will take place under the following conditions:
- If there is enough water in the tank (as measured by the Ultrasonic Sensor)
- If it is not currently raining (as determined by the Weather Station / Weather Service)
- If the soil is not moist enough (as measured by the soil moisture sensor)
- If a pre-defined amount of rain is not expected within the next 4 hours
- If the temperature is not expected to fall below 4°C – to prevent frost

If these conditions are all met then water will take place and automatically stop at sunrise. The smart garden watering system can also be manually activated via the Loxone App.

All four valves are connected to a [Radio Buttons](https://www.loxone.com/enen/kb/radio-buttons/) Function Block. The four valves water the garden in 60-second intervals.

Additionally, if there’s a risk of frost, the water in the pipes is blown out with air for 5 minutes to prevent them from freezing.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Tree Extension](https://shop.loxone.com/enuk/tree-extension.html)
- [Ultrasonic Sensor](https://shop.loxone.com/enuk/ultrasonic-sensor.html)
- [Weather Station](https://shop.loxone.com/enuk/weather-station.html) / [Weather Service](https://shop.loxone.com/enuk/weather-service-10-year.html)
- Soil Moisture Sensor
- Valves (4 water valves, 1 main water valve, 1 air valve)

### Configuration:

[
![Smart Garden Watering System - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/06/Use-Case-15-Garden-Watering-EN-1_V2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/06/Use-Case-15-Garden-Watering-EN-1_V2.png)

### Download the sample file:

### Smart Garden Watering

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-15-Smart-Garden-Watering.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-15-Smart-Garden-Watering.loxone)

## Why you and your customer should consider having a smart garden watering system?

We recommend implementing a smart garden watering system to ensure that your garden or the garden of your customers is always sufficiently and regularly watered. Intelligent watering with Loxone can also be used perfectly in large greenhouses and fields.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)