# Frost Protection for Crops

Source: https://www.loxone.com/enen/kb/frost-protection-for-crops/

---

## Brief: I want a solution that automates frost protection for crops.

Farmers are dependent on the yield of their crops. The more fruit the plants bear, the more potatoes or apples the harvest brings, etc., the more money made. If the crops are damaged, this results in loss of income. That is why farmers continuously monitor their crops. They have to make sure that the plants are healthy, have enough water and do not suffer damage from frosty nights.

To prevent frost damage, farmers always keep an eye on the weather and temperatures. If the weather forecast predicts a frosty night, farmers are alerted and get up at night to check the temperature and protect the plants if necessary. If the temperature moves into a frosty range, the plants are watered. A protective layer of frozen water forms around the flower. Within this ice layer, the flower is protected, because inside this ice layer it is warmer than outside. The flowers or plants must be watered as long as there are frosty temperatures. When it gets warmer, the watering is stopped and the ice layer around the flower melts. The blossom will survive the frost.

The frosty season is a stressful time for farmers. They have to get up several times during the night to check the temperature and the plants – when frost is forecast. This is necessary to protect the harvest and the business.

Loxone can be used to help farmers out by creating automated frost protection for crops. From monitoring the temperature to watering the plants. So they can get a good nights sleep knowing their crops are safe.

## Solution: Using Loxone to intelligently monitor and automate frost protection for crops.

The following is necessary for automated frost protection sprinkling:
- Temperature monitoring
- An automated irrigation system
- Notifications to the farmer

**Temperature monitoring**

With Loxone the outside temperature can be monitored around the clock. The soil temperature is measured with a [1-Wire Temperature Probe](https://shop.loxone.com/enuk/1-wire-temperature-probe.html). The [Weather Station](https://shop.loxone.com/enuk/weather-station.html) in combination with the [Weather Service](https://shop.loxone.com/enuk/weather-service-1-year.html) provides information about air temperature, wind, rain and a weather forecast. With the help of these three components, the temperature can be perfectly monitored. The irrigation is controlled depending on the outside temperature. If the temperature drops below 1°C, watering is activated and remains active until the temperature is above 10°C.

**Automated irrigation system**

An irrigation system is installed in the fields. Depending on the size of the field, it can be divided into several smaller fields, each of which has its own irrigation system. In our example, we’ve assumed that there are two smaller fields. The water pipe on the field is connected to the water source – a cistern for example. At each water pipe, there is a valve that regulates the water flow. These valves are integrated into the Loxone system through the [Relay Extension](https://shop.loxone.com/enuk/relay-extension.html).

The water in the cistern is pumped into the water pipes. This pump is also integrated into the Loxone system. The pump is activated automatically as soon as the temperature drops below 1°C or it is activated manually. In either case, the pump is only activated if the water level in the cistern is sufficient. Depending on which field needs watering, the respective valve is opened and watering starts.

The level in the cistern is very important and can be monitored with the [ultrasonic sensor](https://shop.loxone.com/enuk/ultrasonic-sensor.html). This monitoring can be used to ensure that the pump is not switched on if there is not enough water, as this could cause damage to the pump.

If the field is divided into several areas that can be irrigated independently, one valve after the other is opened. This ensures that the pressure in the water pipe is sufficient for irrigation. Each area is watered for 20 minutes at a time.

If the farmer wants to enter the field that is being watered, he can stop the watering through the [Loxone App](https://www.loxone.com/enen/products/apps/). The valves remain open for 5 minutes after the manual stop to make sure that all water is pumped out of the pipes. Otherwise, freezing can occur in the pipes. If the water level in the cistern falls below the threshold value, the pump is also switched on for another 5 minutes.

**Notifications to the farmer**

Automated frost protection for crops is great however, it only works if all of the components in the irrigation system are working. Therefore, if the irrigation stops, the farmer will be automatically notified via the Loxone App.

In addition to the weather forecast, it is also important to know if something is wrong with the pump. With the [Nano 2 Relay Tree](https://shop.loxone.com/enuk/nano-2-relay-tree.html), the pump is integrated into the Loxone system which allows its current flow to be monitored. Should the pump break down, the system recognizes through the current flow detection. The farmer is immediately informed by a call from the [Loxone Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html). He can then react as quickly as possible and avoid damage to the pump and plants.

**Configuration in Loxone Config**

Threshold value switches are used to activate the pump depending on the outside temperature. With the “Switch” Function Block the pump can also be switched on or off manually via the Loxone App. The valves on the water pipes are also connected to the “Switch” block.

The operating time counter provides information on how long the irrigation system has been running.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [1-Wire Temperature Probe](https://shop.loxone.com/enuk/1-wire-temperature-probe.html)
- [Weather Station](https://shop.loxone.com/enuk/weather-station.html) with [Weather Service](https://shop.loxone.com/enuk/weather-service-10-year.html)
- [Relay Extension](https://shop.loxone.com/enuk/relay-extension.html)
- [Nano 2 Relay Tree](https://shop.loxone.com/enuk/nano-2-relay-tree.html)
- [Ultrasonic Sensor 0-10V](https://shop.loxone.com/enuk/ultrasonic-sensor.html)
- [Pressure Sensor 0-10V](https://shop.loxone.com/enuk/pressure-sensor-0-10v.html)
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)

### Configuration:

[
![Frost Protection for Crops - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-89_Frost-Protection-Watering.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-89_Frost-Protection-Watering.jpg)

### Download the sample file:

### Frost Protection Watering

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-89_Frost-protection-watering.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-89_Frost-protection-watering.loxone)

## What are the benefits of Loxone automating frost protection for crops?

Farmers are dependent on the yield of their harvests. If the crop is damaged, the farmer loses a lot of money and in the worst-case scenario their business. If the plants die, they have to be replaced – which costs both time and money.

The more information the farmer gets about temperature and weather, the better – automated notifications make this easier. Automatic antifreeze irrigation not only relieves them of physical tasks but also helps them save time and get more sleep – knowing that the plants are safe.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)