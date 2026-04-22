# Refrigerator Temperature Monitoring System

Source: https://www.loxone.com/enen/kb/refrigerator-temperature-monitoring-system/

---

## Brief: I want a refrigerator temperature monitoring system.

In the hospitality sector, it’s crucially important that the temperature of refrigerators, freezers and cold rooms are continuously monitored and recorded. Manually recording this data is extremely painstaking and time-consuming. However, with Loxone you can commission an automated refrigerator temperature monitoring system – allowing exact temperature recording, but also immediate notification should the temperature fall below or exceed a pre-defined range.

## Solution: Using Loxone to create a refrigerator temperature monitoring system.

With Loxone, there are many ways of collecting and storing data. In this Use Case, the temperature of a cold storage room is being measured with the [Room Comfort Sensor Tree](https://shop.loxone.com/enuk/room-comfort-sensor.html). It has a measuring range of -40°C to 120°C, making it the perfect tool for precise temperature recording of a cold rooms, refrigerators or freezers.

The recorded values will then be displayed within the Loxone App – so they can be checked at any time. By using the [Logger function](https://www.loxone.com/enen/kb/logger/) on the [Minisever](https://shop.loxone.com/enuk/miniserver.html), these values can also be stored on external data servers or on FTP (File Transfer Protocol).

Particularly in the commercial sector, it may be necessary to log temperature undershoot or overshoot. In our example, this was solved using the function block “Min Max Since Reset”. The maximum and minimum temperatures witnessed for each day are captured with a daily impulse at 23:59:39. At the same time, these daily minimum and maximum values are displayed in the Loxone App using the “[Tracker](https://www.loxone.com/enen/kb/tracker/)” logging feature.

If the temperature deviates for more than 3 minutes, then a phone call will be made via the [Loxone Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html) to inform the relevant people.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Room Comfort Sensor Tree](https://shop.loxone.com/enuk/room-comfort-sensor.html)
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)

### Configuration:

[
![Refrigerator Temperature Monitoring System - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-103-Monitoring-and-data-logging-of-cooling-room.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-103-Monitoring-and-data-logging-of-cooling-room.png)

### Download the sample file:

### Monitoring and data logging of cooling rooms

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-103-Cooling-Room-Hells-Kitchen.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-103-Cooling-Room-Hells-Kitchen.loxone)

## Why you and your customer should consider installing a refrigerator temperature monitoring system?

In the hospitality sector, cold rooms, refrigerators and freezers often contain significant amounts of food – costing significant amounts of money. Hence, it’s crucially important that the temperature is constantly monitored. Furthermore, an intelligent refrigerator temperature monitoring system will alert your customers should the temperature deviate from a certain range.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)