# Smart Humidor Monitoring

Source: https://www.loxone.com/enen/kb/smart-humidor-monitoring/

---

## Brief: Set up smart humidor monitoring.

A humidor can offer cigars the ideal climatic conditions to keep them fresh, even after several years. The temperature and the humidity in the humidor play a crucial role. The ideal climate for cigars is at a temperature of 20 to 21°C and relative humidity of 69-72%. It is often forgotten to refill the humidifier in the humidor, which leads to the cigars becoming dry and unenjoyable. This can be very expensive, as there are usually several cigars in a humidor. So, one effective way to keep track of the conditions is with smart humidor monitoring.

In our example, we show you how to set up smart humidor monitoring to ensure your customer can monitor the temperature and humidity in the humidor and have peace of mind that their cigar collection is in the best conditions to encourage a long-lasting lifespan.

## Solution: How to commission smart humidor monitoring.

In this use case, we use the [Room Comfort Sensor Air](https://shop.loxone.com/enuk/room-comfort-sensor-air.html) for smart humidor monitoring of both the temperature and humidity levels. Thanks to [Loxone Air](https://shop.loxone.com/enuk/air-base-extension.html) technology, the sensor can be inserted into the humidor completely wirelessly.

Once the Room Comfort Sensor is connected to the [Miniserver](https://shop.loxone.com/enuk/miniservers.html), you can insert a minimum and the maximum value in [Loxone Config](https://www.loxone.com/enen/products/loxone-config/). To be notified in case of any deviation, add a notification type to the temperature and humidity block – in our case we’ve used the [mailer service](https://www.loxone.com/enen/kb/mailer-service/). As soon as the relative humidity falls below 65% the Miniserver sends a message that the humidifier needs to be refilled. Emails are then sent until the humidity is over 65% again. Thus, the cigars remain enjoyable and fresh even after several years. With the [Loxone App](https://www.loxone.com/enen/products/apps/), your customer is also able to monitor the current temperature and humidity of the humidor at any time.

Hardware:
- [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html)
- [Room Comfort Sensor Air](https://shop.loxone.com/enuk/room-comfort-sensor-air.html)

### Configuration:

[

![Loxone Use Case 53 Humidor Monitoring 1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-53-Humidor-Monitoring-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-53-Humidor-Monitoring-1.png)

### Download the sample file:

### Humidor Monitoring

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-53-Humidor-Monitoring.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-53-Humidor-Monitoring.loxone)

## Why you and your customer should consider passive monitoring for assisted living

If humidity and temperature in the humidor are not consistent, it can cause that the cigars become too dry or too moist. Dry cigars are unstable, break more easily and also burn faster. In addition, the cigar has a bitter aftertaste. Cigars that are too moist, on the other hand, usually have an odd burn-off and taste heavier and sour.

For reliable monitoring of a humidor, we recommend automatic temperature and humidity control with the Room Comfort Sensor Air. Your customers will always be notified in the cause of a humidity deviation and they no longer have to worry about the storage of their cigars.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)