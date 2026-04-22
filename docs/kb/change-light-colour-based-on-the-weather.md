# Change light colour based on the weather

Source: https://www.loxone.com/enen/kb/change-light-colour-based-on-the-weather/

---

## Brief: Change the colour of my lights so I know if it is going to rain today.

It is very beneficial to know about the current weather situation before you leave the house. After waking up, you can see what the forecast for that day is because your home can change the light colour based on the weather. For example blue is rain – orange is sunny – magenta is cloudy. Now, changing all of the lights might not be practical if you’re using that lighting to get ready. So, can we use just one of the accent light sources, such as LED strip to change light colour based on the weather? Yes!

So, you can now dress properly and prepare yourself for what mother nature has in store for the day.

Furthermore, if rain is predicted in the next 12 hours, you can use text-to-speech to have the house share valuable information like: “please do not forget your umbrella!”

So if you are interested in a solution to change light colour based on the weather, then continue reading to see how we can achieve this with Loxone Hardware and Loxone Config Software.

## Solution: How to change light colour based on the weather forecast.

With Loxone it is possible to change light colour based on the weather forecast.

The respective coloured lighting mood is only visualised between 6:30 am and 7:30 am in order not to use unnecessary energy. In this example the LED Strip is activated from the first movement in the room. Then at 7:30 am, the LED Strip is turned off. The customer can adjust this time window individually in the app.

At the first movement, a text-to-speech is sent if rain is expected in the next 12 hours. This text can be changed in the Status block. This is because it is very possible that rain is expected within the next 12 hours however it is currently it is sunny.

For cloudless and cloudy the “weather type” is used, a value which is then compared.

This value can, of course, be adjusted at any time by consulting the documentation:

| 1 | Clear | 16 | Light rain showers |
| --- | --- | --- | --- |
| 2 | Bright | 17 | Heavy rain showers |
| 3 | Cloudy | 18 | Thunderstorm |
| 4 | Very cloudy | 19 | Heavy thunderstorm |
| 5 | Overcast | 20 | Light snow |
| 6 | Fog | 21 | Snow |
| 7 | Low Fog | 22 | Heavy snow |
| 8 | Not used | 23 | Light snow showers |
| 9 | Not used | 24 | Strong snow showers |
| 10 | Light rain | 25 | Light sleet |
| 11 | Rain | 26 | Sleet |
| 12 | Heavy rain | 27 | Heavy sleet |
| 13 | Drizzle | 28 | Light sleet showers |
| 14 | Light freezing rain | 29 | Heavy sleet showers |
| 15 | Heavy freezing rain |  |  |

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Weather Service](https://shop.loxone.com/enuk/weather-service-10-year.html)
- [RGBW Dimmer 24V](https://shop.loxone.com/enuk/rgbw-24v-dimmer.html)
- [LED Strip RGBW](https://shop.loxone.com/enuk/led-strip-rgbw.html)

### Configuration:

[
![Change light colour based on the weather - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-41-Smart-Weather-Forecast-2.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-41-Smart-Weather-Forecast-2.jpg)

### Download the sample file:

### Smart Weather Forecast

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/09/Use-Case-41-smart-weather-forecast.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/09/Use-Case-41-smart-weather-forecast.loxone)

## Why you and your customer should consider coloured lighting that changes based on the weather?

It is a comfort and time-saving function to know about the weather as soon as you get up. If your home can change the light colour based on the weather, then this will be especially useful if you have children. I can dress myself or my children according to the forecast, get the raincoats out and choose the right shoes without having to Google what the weather is like and if it might rain during the day. I am reminded via text-to-speech not to forget the umbrella if rain is predicted in the next 12 hours. It’s one thing being able to look outside and see what the weather is like, but it’s another feature altogether having your home let you know what the weather will be like later in the day, too.

Very practical also on the weekend, when you see that it is raining, you can easily get a few more minutes of sleep.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)