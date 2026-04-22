# Weather Service

Source: https://www.loxone.com/enen/kb/weather-service/

---

## SETUP

The Weather Service function blocks are already in Loxne Config. All you need to do is simply activate it with a Miniserver. In order to do this, you must go to where you register a product and to the right of the registering field, you will have a licence activation field. Select the serial number of the Miniserver you want to link it to and type in the license code.

![Screenshot 081417 031919 PM 300x162](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/Screenshot_081417_031919_PM.jpg)

In order for the weather data to be accurate to the location of the Miniserver, it is important to set a location within the properties of the project.

Once entered, the Geo-coordinates will then be automatically calculated. It is important that the Miniserver has a working internet connection for this to be calculated correctly.

In the same properties list, you can also specify the preferred unit type for the weather data being used.

![Changing The Location Properties In The Loxone Miniserver](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Miniserver_Properties_Weather.png)

Once you have added in the Weather Service, it is important that you Save in Miniserver to ensure weather data will be received and updated.

#### THE FOLLOWING WEATHER INFORMATION IS AVAILABLE

Temperature

Dewpoint

Rel. Humidity

Wind Speed

Wind Direction

Precipitation

Air Pressure

Perceived Temperature

Weather Type

Sunshine Detection

####
![Icon Exclamation Mark Loxone](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Icon_Exclamation_Mark.png)
For information on how to use the Sunshine Detection, please see [here](https://www.loxone.com/enen/kb/sunshine-brightness-thresholds/).

1 = Clear

2 = Bright

3 = Cloudy

4 = Very cloudy

5 = Overcast

6 = Fog

7 = Low Fog

8 = Not used

9 = Not used

10 = Light rain

11 = Rain

12 = Heavy rain

13 = Drizzle

14 = Light freezing rain

15 = Heavy freezing rain

16 = Light rain showers

17 = Heavy rain showers

18 = Thunderstorm

19 = Heavy thunderstorm

20 = Light snow

21 = Snow

22 = Heavy snow

23 = Light snow showers

24 = Strong snow showers

25 = Light sleet

26 = Sleet

27 = Heavy sleet

28 = Light sleet showers

29 = Heavy sleet showers

Time Weather Data – The time of last received weather data

Last Weather Forecast Update – The time of last received weather forecast

Weather Data Error – ON when there has been an error receiving weather data

Error Weather Forecast Data – ON when there has been an error receiving weather forecast

Radiation

Radiation risk classification 0-3 (solar constant).

Class 0: 0-20%

Class 1: 20-40%

Class 2: 40-60%

Class 3: 60-100%

100% corresponds to 1376 W/m2

Weather data is updated every hour (+5 to 10 minutes).

Every time the Miniserver restarts, it will immediately update its current weather data and weather forecast.

#### WEATHER FORECASTING

It is possible to display weather forecasts with the weather server.

![Screenshot 081417 032316 PM 236x300](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/Screenshot_081417_032316_PM.jpg)

You can add a new entry by clicking the “Weather data button” whilst the weather server is selected.

![Screenshot 081417 033053 PM 300x273](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/Screenshot_081417_033053_PM.jpg)

Select which data type the forecast will display.

![Choosing Which Weather Information To Use In Loxone Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Adding_Weather_Forcasting.png)

It is now possible to select the timescale of the forecast data.

![Adjusting The Offset or Time In Loxone Config For Weather Forcasting](https://www.loxone.com/enen/wp-content/uploads/sites/3/2016/09/EN_KB_Config_Timescale_Weather_Forcasting.png)