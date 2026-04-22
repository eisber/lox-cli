# Plant Monitoring and Control System

Source: https://www.loxone.com/enen/kb/plant-monitoring-and-control-system/

---

## Brief: I want a plant monitoring and control system.

Fresh herbs, salad and vegetables: indoor gardening can be great! Growing your own food is becoming increasingly popular. However, to ensure a good harvest, the plants must be well cared for, especially indoors.

If the plants are provided with sufficient lighting, the right amount of water and the correct temperature, the risk of failure is reduced. However, it can be quite challenging to consistently meet these demands. Hence a plant monitoring and control system would make life much easier.

## Solution: Using Loxone to create an intelligent plant monitoring and control system.

With the help of the [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html), you can create an intelligent plant monitoring and control system, meaning that many indoor gardening related tasks can be easily monitored and/or automated. Therefore increasing the chances of a successful yield.

Plants can only thrive if they get enough light. To help with this, there are special plant LED lamps. These can be controlled, for example, via an [RGBW 24V Compact Dimmer Air](https://shop.loxone.com/enuk/rgbw-24v-compact-dimmer.html). However, it is not simply a case of providing the plants with light – the amount of time the plant is exposed to light must also be monitored and controlled. This can be individually adjusted to the specific plants’ needs with the help of the [“Schedule” Function Block.](https://www.loxone.com/enen/kb/timerscheduler/)

In addition to sufficient lighting, the plants must be provided with the right temperature and humidity. With the [Room Comfort Sensor Air](https://shop.loxone.com/enuk/room-comfort-sensor-air.html), these values can be recorded precisely. If the temperature is too low, for example, a heating mat can be controlled via a digital output on the Miniserver to provide the necessary heat. Within Loxone Config, this is implemented with the help of the [“Intelligent Room Control” Function Block](https://www.loxone.com/enen/kb/irc-v2/). If the air humidity or the temperature is too high, a fan takes care of regulating this. In our example, we’ve used the [“Room Ventilation Controller” Function Block](https://www.loxone.com/enen/kb/ventilation/) for this.

Certain plants are very sensitive and require a very precise soil pH value. The current pH value is measured with a sensor from a third party supplier. If the pH value deviates from the pre-defined value, your customers will be informed immediately through a push notification.

Last but not least, plants need enough water to grow and thrive. This process can also be easily automated with the Miniserver. If the moisture levels in the soil fall below a certain value (which can be selected by your customer), a solenoid valve is opened via a digital output on the Miniserver and the plants are supplied with water.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Room Comfort Sensor](https://shop.loxone.com/enuk/room-comfort-sensor-air.html)
- [RGBW 24V Compact Dimmer Air](https://shop.loxone.com/enuk/rgbw-24v-compact-dimmer.html)
- Fan (third-party supplier)
- Ph-sensor (third party)
- Soil moisture sensor (Third Party)
- Solenoid valve (third party)
- LED lighting for plants (third-party supplier)

### Configuration:

[
![Plant Monitoring and Control System - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-35-Indoor-Gardening.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-35-Indoor-Gardening.png)

### Download the sample file:

### Indoor Gardening

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/07/Use-Case-35-Indoor-Garden-Monitoring.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/07/Use-Case-35-Indoor-Garden-Monitoring.loxone)

## Why you and your customer should consider having a plant monitoring and control system?

With the help of the Miniserver, you can implement an intelligent plant monitoring and control system. This will save your customers a lot of manual work and also allow for a longer period of time away from the plants.

Through the precise monitoring of temperature, humidity and pH-value, your customers’ plants will enjoy the best of health which will give your customers a high-yielding harvest.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)