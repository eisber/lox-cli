# Garage Frost Protection

Source: https://www.loxone.com/enen/kb/garage-frost-protection/

---

## Brief: I want a system that automatically offers garage frost protection.

Especially in winter, vehicles such as electric bikes and/or cars are exposed to extreme conditions due to frost in the garage. However, many people use their garage not only to park their vehicles but also as a storage place for food, plants, etc. Often, there is also a sink and/or a water connection in a typical garage.

If the temperature in the garage drops below 5°C, frost damage can occur to the water pipes and the items stored in the garage. However, damage caused by frost is not normally covered by a standard household insurance policy. Therefore implementing a garage frost protection system is very important.

## Solution: How to use Loxone for frost protection in a domestic garage.

With Loxone, the heating of the garage can be easily automated – ensuring garage frost protection.

In this example, we assume that there is an electric heating system in the garage. In order for the heating to be monitored, an intelligent socket – the [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html) – is necessary. The temperature sensor integrated in it reliably transmits the garage’s room temperature to the Miniserver. The Miniserver, in turn, sends the command for “heating on” or “heating off” to the switchable socket. If the temperature in the garage is below 5 °C, the electrical heating switches on, if it is above 8 °C, the heating switches off again. In [Loxone Config](https://www.loxone.com/enen/products/loxone-config/) this is realised with simple logic using the Threshold Switch Function Block.

**Tip:** With the [Room Comfort Sensor Air](https://shop.loxone.com/enuk/room-comfort-sensor-air.html), the humidity in the garage can also be measured and controlled. A must-have for classic car fans!

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html)
- [Room Comfort Sensor Air](https://shop.loxone.com/enuk/room-comfort-sensor-air.html) (Optional)

### Configuration:

[
![Garage Frost Protection - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-18-Frost-Protection-Garage.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-18-Frost-Protection-Garage.png)

### Download the sample file:

### Frost Protection Garage

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-18-Frost-Protection-Garage.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-18-Frost-Protection-Garage.loxone)

## Considering the advantages of a frost-proof garage.

Thanks to this simple configuration, the garage heating can be controlled completely automatically – giving you effective garage frost protection. This ensures that the heating only runs when it is too cold in the garage. This means that energy is only consumed when needed. Making frost damage to electric bikes, water pipes etc. a thing of the past.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)