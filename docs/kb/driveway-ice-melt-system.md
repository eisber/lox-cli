# Driveway Ice Melt System

Source: https://www.loxone.com/enen/kb/driveway-ice-melt-system/

---

## Brief: I want to set up a driveway ice melt system.

As a child, snow is one of the most exciting things you can experience. However, sadly, in adult life, it can be quite an inconvenience and even a danger.

Snow, and the resulting black ice, bring about an exponentially increased risk of falling. Especially on your own driveways, pavement, terraces etc. as these have lower footfall and hence the snow/black ice is not cleared.

There are usually two options here. Option 1 – simply do nothing and hope that no one hurts themselves. Option 2 – reach for your shovel and manually scrape the snow/ice away. However, what if we told you there was an option 3 – one that combines the convenience of option 1 with the safety of option 2? Well, option 3 is an automated driveway ice melt system.

However, it gets even better. With Loxone, not only can your driveway ice melt system be automated based on whether or not there is snow. We can go one step further and activated the system before the first snowflake has even landed – ensuring that none of the snow settles at all. We’ll show you how to set that up in this Use Case.

## Solution: Using Loxone to create a driveway ice melt system.

To create an effective driveway ice melt system you’ll need a heating mat, [Temperature & Humidity Sensor](https://shop.loxone.com/enuk/outside-temperature-humidity-sensor.html) and an accurate weather forecast – from the [Loxone Weather Service](https://shop.loxone.com/enuk/weather-service-10-year.html), for example.

The heating system will be activated under the following conditions:
- If the outdoor temperature is below 3° C and humidity is detected
- If the outside temperature is below 3° C and rain or snowfall is predicted within the next 30 minutesOf course, the system can also be activated manually through the [Loxone App](https://www.loxone.com/enen/products/apps/).

After activation, the temperature and humidity will continue to be monitored. The driveway ice melt system will be switched off when the conditions for its activation are no longer met – i.e. the temperature rises above 3° C.

If the system was activated manually, it will be automatically switched off after 12 hours.

**Note**: To get the best results the [Temperature & Humidity Sensor](https://shop.loxone.com/enuk/outside-temperature-humidity-sensor.html) must be placed as close to the ground as possible.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Weather Service](https://shop.loxone.com/enuk/weather-service-10-year.html) (included for 10 years when using the Loxone Weather Station)
- [Temperature & Humidity Sensor](https://shop.loxone.com/enuk/outside-temperature-humidity-sensor.html)
- [Weather Station Tree](https://shop.loxone.com/enuk/weather-station.html) (alternative)

### Configuration:

[
![Driveway Ice Melt System - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-80-Snow-Melt-System.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-80-Snow-Melt-System.png)

### Download the sample file:

### Snow Melt System

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-80-Snow-Melt-System.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-80-Snow-Melt-System.loxone)

## Why you and your customer should consider implementing a driveway ice melt system?

An automated driveway ice melt system reduces the risk of slipping on snow/ice. It will automatically ensure that the worries that come along with snow are a thing of the past. It will also reduce the time and effort of manually scraping the snow up.

In a commercial environment, where you have a duty-of-care a driveway ice melt system can automatically take away one of your worries. It could also save you the costs of paying someone to removed the snow/ice for you.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)