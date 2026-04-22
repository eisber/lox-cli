# Sunshine Input

Source: https://www.loxone.com/enen/kb/sunshine-brightness-thresholds/

---

The **Sunshine** input is used to indicate whether or not the sun is shining.

It can be used in programming, for example in heating, air conditioning or shading applications.
The Sunshine input is available on the [Weather Station Air](https://www.loxone.com/enen/kb/weather-station-air/), the [Weather Station Tree](https://www.loxone.com/enen/kb/weather-station-tree/), and the [Loxone Weather Service](https://www.loxone.com/enen/kb/weather-service/).

## Weather Station

The sunshine detection of the Weather Station is based on the brightness measured by the Weather Station.

The sun elevation is calculated using the geographical coordinates of the installation (project settings) and the time and date.

A threshold value is calculated from both values, since the brightness depends greatly on the angle of the solar radiation.

If the brightness is above the calculated threshold, the Sunshine input is active.

| Examples of calculated thresholds (Setting: Default) |
| --- |
| Sun Elevation | Threshold | Note |
| 90° | 86780 lx | Sun at its highest point (zenith) |
| 65° | 74507 lx | Noon in central Europe at summer solstice |
| 42° | 46555 lx | Noon in Central Europe at equinox |
| 18° | 14057 lx | Noon in central Europe at winter solstice |
| 10° | 5753 lx | Sun is low |

The sensitivity can be adjusted in the properties of the Weather Station under “Sunshine Threshold”.

Default +30% is the lowest sensitivity, the sunshine threshold is increased.

Default -30% is the highest sensitivity, the sunshine threshold is decreased.

[
![sunshine adjustment station](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/09/sunshine_adjustment_station.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/09/sunshine_adjustment_station.png)

**Example:**

If the Weather Station is installed facing a highly reflective surface (e.g. white building), a higher brightness value is measured and the sunshine detection is activated earlier. This can be corrected by setting the threshold a little higher, e.g. +15%.

## Weather Service

The Sunshine detection of the Weather Service depends on the absolute irradiance value provided by the Weather Service.

The sun elevation is calculated using the geographical coordinates of the installation (project settings) and the time and date.

A threshold value is calculated from both values, since the irradiance depends greatly on the angle of the solar radiation.

If the irradiance is above the calculated threshold, the Sunshine input is active.

| Examples of calculated thresholds (Setting: Default) |
| --- |
| Sun Elevation | Threshold | Note |
| 90° | 512 W/m² | Sun at its highest point (zenith) |
| 65° | 464 W/m² | Noon in central Europe at summer solstice |
| 42° | 343 W/m² | Noon in Central Europe at equinox |
| 18° | 158 W/m² | Noon in central Europe at winter solstice |
| 10° | 89 W/m² | Sun is low |

The sensitivity can be adjusted in the properties of the Weather Service Sunshine input under “Sunshine Threshold”.

Default +30% is the lowest sensitivity, the irradiance threshold is increased.

Default -30% is the highest sensitivity, the irradiance threshold is reduced.

[
![sunshine adjustment service](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/09/sunshine_adjustment_service.png)
](https://www.loxone.com/dede/wp-content/uploads/sites/2/2017/09/sunshine_adjustment_service.png)