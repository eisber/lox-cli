# Wine Cellar Humidity Control

Source: https://www.loxone.com/enen/kb/wine-cellar-humidity-control/

---

## Brief: I need to monitor and control humidity in a wine cellar.

Wine cellar humidity control is important for maintaining the integrity of both the bottle of wine and the contents itself. In addition to being able to monitor the humidity level, it is useful to be able to control available components in a wine cellar that can either reduce or increase the humidity level.

Relative humidity of between 50% to 70% is generally recognised as suitable levels for wine cellar humidity. 60% would be a good common ground if we work within this range.

So, if you are needing a solution for wine cellar humidity control, with the added benefit of temperature control and automation, then read on to see how we would achieve this using Loxone Hardware and Loxone Config software.

## Solution: Using Loxone for wine cellar humidity control.

With Loxone it is possible to monitor the humidity in a variety of ways using our dedicated Room Comfort Sensor or, if Loxone is used for other parts of control, any of our other products with in-build humidity sensors such as our Touch Switches, or any third-party sensors using standards such as Modbus or 0-10V inputs.

Once a humidity level is known, the Miniserver can take different actions dependant on the type of mechanical installation that is available for climate control in the cellar. The relative humidity level can be effected either through temperature control – which of course needs to be used with caution as wine storage also relies on a specific temperature range or through the use of a ventilation fan if one is in place. This can be done simply through relays or, for via 0-10V analogue outputs for fans. More sophisticated climate controls, such as the one in our configuration example, would typically use Modbus control.

We are using a threshold control, so the Miniserver is able to increase or decrease the forced ventilation of the space to reduce the humidity level. Should the Miniserver, due to environmental circumstances (ie. a very high external temperature, or high humidity levels) not be able to maintain the ideal humidity level for the wine cellar, then the alert/notification services of the Miniserver can be used to alert the proprietor to take manual action as they deem necessary.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Room Comfort Sensor](https://shop.loxone.com/enuk/room-comfort-sensor-air.html)
- [Modbus Extension](https://shop.loxone.com/enuk/modbus-extension.html) (optional)

### Configuration:

[

](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-22-Wine-Cellar-Humidity-Control-2.png)[
![Wine Cellar Humidity Control - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-22-Wine-Cellar-Humidity-Control-2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-22-Wine-Cellar-Humidity-Control-2.png)

### Download the sample file:

### Wine Cellar Humidity Control

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-22-Wine-Cellar-Humidity-Control.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-22-Wine-Cellar-Humidity-Control.loxone)

## Why you and your customer should consider wine cellar humidity control?

Experts advise that humidity below 50% means there is a risk that the environment is too dry which has a direct impact on the longevity of the cork. If the cork becomes too dry, it can start to degrade and ultimately this can let air into the bottle and spoil the wine.

There is also a need to ensure the humidity level for wine storage does not exceed 70%. While a humidity level above 70% won’t affect the wine itself (as long as the bottle is properly sealed) it does increase the risk of mould and degradation of the label on the bottle.

Besides monitoring humidity levels, there are also several ways to increase or decrease humidity levels as well as to control other important aspects of storing wine such as the room temperature (considering its correlation to humidity levels) and light levels.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)