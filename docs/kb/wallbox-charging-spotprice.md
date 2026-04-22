# Wallbox Charging Spotprice

Source: https://www.loxone.com/enen/kb/wallbox-charging-spotprice/

---

## Brief: I want to charge my EV at the cheapest possible time

The cheapest way to buy electricity is often via the spot price market. On the spot price market, electricity is traded at prices that are based on supply and demand in real time and are therefore subject to fluctuations. In order to be able to charge electric vehicles as cheaply as possible, electricity must be purchased at precisely the times when it has the lowest price.

## Solution: Comfort charging mode with the Loxone Wallbox and the spot price optimizer

Thanks to the [spot price optimizer](https://www.loxone.com/enen/kb/spot-price-optimizer/), the Loxone Wallbox can charge fully automatically at the cheapest hours according to the spot price market.

The initial configuration in this application example is the same as in the [Wallbox charging modes](https://www.loxone.com/enen/kb/wallbox-charging-modes/) use case. There you will also find information on the “Eco” and “Full Power” charging modes, which are also used in this application example.

In this example configuration, we use the “Comfort” charging mode to be able to charge as cheaply as possible with the Spot Price Optimizer and the Loxone Wallbox. The user defines the number of hours to be charged and the maximum electricity price they are willing to pay.

For example, it can be set to charge for 2 hours within the next 10 hours from activation at a price of up to €0.4 per kWh (40ct/kWh). The spot price optimizer searches for the cheapest price during this period and controls the wallbox. When the cheapest time slot is reached, the wallbox switches to full power (11 kW) to charge the vehicle.

This means that the car can be connected in the evening, the “Comfort” charging mode can be selected via the Touch Pure Flex, and the car is ready for use again the next day, charged as cheaply as possible.

### Requirements/Hardware:
- [Loxone Wallbox 11kW](https://shop.loxone.com/enen/catalogsearch/result/?q=wallbox+11kw)
- [Touch Pure Flex Tree/Air Wallbox](https://shop.loxone.com/enen/catalogsearch/result/?q=touch+pure+flex+wallbox)

### Configuration:

[
![Use Case 110 Wallbox Charging Spotprice](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/12/Use-Case-110-Wallbox-Charging-Spotprice.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/12/Use-Case-110-Wallbox-Charging-Spotprice.jpg)

### Download the sample file:

### Wallbox Charging Spotprice

			[14.4.9.25](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/WallboxFlexSpotprice.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/WallboxFlexSpotprice.loxone)

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)