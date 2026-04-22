# Automatic Humidity Control

Source: https://www.loxone.com/enen/kb/automatic-humidity-control/

---

## Brief: I want to have automatic humidity control, specifically in my garage.

The humidity in garages is a major problem in many buildings. In a garage, there is often an increased humidity because wet cars are parked in it. Since the garage is usually not heated, this promotes the formation of moisture. The result is increased mould growth which can cause damage to cars and other objects stored in the garage. It is therefore important to reduce the humidity.

Ideal humidity levels in a garage are as follows:

| 50% – 55% | optimal |
| --- | --- |
| 35% – 65% | acceptable |
| > 65% | too high |

In many garages, the relative humidity is much higher – it’s not uncommon for humidity levels to be 70% or above! In order to reduce the air humidity and thus protect the garage and the objects in it, it makes sense to automate ventilation – giving you automatic humidity control.

Solution: How to reduce humidity levels in a garage with Loxone.

In order to achieve automatic humidity control we need to automatically activate the ventilation and thus to ensure a reduction in humidity, it’s necessary to take the humidity of the outside air into account.

The decisive factor here is not the relative but the absolute humidity. For this reason, the relative humidity is converted into the absolute humidity with the help of the Formula Function Block.

Only when the absolute humidity of the outside air is lower should ventilation be activated.

In our example, we have also included a threshold value of 60%. Only above this threshold is the ventilation activated.

**Tip:** If your customer’s garage has no ventilation, this logic can be used to send information to the customer via a call, push notification or email. This way the humidity can manually be reduced by opening a window.

I1 = Temperature in °C

I2 = Relative Humidity in %.

I3 = air pressure in hPa (please make sure that the air pressure of the outside air is also used for the garage)

**Formula to calculate the absolute humidity:**

0.622 * I2/100 * (1.01325 * 10^(5.426651-2005.1 / (I1 + 273.15) + 0.00013869 * ((I1 + 273.15) * (I1 + 273.15)-293700) / (I1 + 273.15) * (10^(0.000000000011965 * ((I1 + 273.15) * (I1 + 273.15)-293700) * ((I1 + 273.15) * (I1 + 273.15)-293700))-1)-0.0044 * 10^((-0.0057148 * (374.11-I1)^1.25))) + (((I1 + 273.15) / 647.3)-0.422) * (0.577-((I1 + 273.15) / 647.3)) * EXP(0.000000000011965 * ((I1 + 273.15) * (I1 + 273.15)-293700) * ((I1 + 273.15) * (I1 + 273.15)-293700)) * 0.00980665) / (I3/1000-I2/100 * (1.01325 * 10^(5.426651-2005.1 / (I1 + 273.15) + 0.00013869 * ((I1 + 273.15) * (I1 + 273.15)-293700) / (I1 + 273.15) * (10^(0.000000000011965 * ((I1 + 273.15) * (I1 + 273.15)-293700) * ((I1 + 273.15) * (I1 + 273.15)-293700))-1)-0.0044 * 10^((-0.0057148 * (374.11-I1)^1.25))) + (((I1 + 273.15) / 647.3)-0.422) * (0.577-((I1 + 273.15) / 647.3)) * EXP(0.000000000011965 * ((I1 + 273.15) * (I1 + 273.15)-293700) * ((I1 + 273.15) * (I1 + 273.15)-293700)) * 0.00980665)) * I3/1000 * 100000000 / ((I1 + 273.15) * 287.1)

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Room Comfort Sensor Air](https://shop.loxone.com/enuk/room-comfort-sensor-air.html)
- [Weather Service](https://shop.loxone.com/enuk/weather-service-10-year.html)
- [Multi Extension Air](https://shop.loxone.com/enuk/multi-extension-air.html)

### Download the sample file:

### Automatic humidity management

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-13-Automatic-humidity-management.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-13-Automatic-humidity-management.loxone)

## Why you and your customer should consider an automatic workshop dust collection set up?

The exact calculation of the absolute humidity and the comparison with the outside air always ensures that the control system works effectively. By implementing automatic humidity control, you protect your customers’ property from excessive humidity and therefore agasint potential mould growth.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)