# Deep Frying Control

Source: https://www.loxone.com/enen/kb/deep-frying-control/

---

## Brief: I want a greater level of deep frying control.

Although deep frying may not be the healthiest way to cook food – it’s certainly a popular (and tasty) method. To achieve the best result when deep frying food, the temperature must be spot on. To help achieve this it would make sense to implement intelligent deep frying control.

Dedicated deep fryers usually have some form of an integrated thermostat. However, if you want to deep-fry at home (in a saucepan, for example) it’s much more difficult to maintain a constant temperature. This Use Case will show you how to implement intelligent deep frying control.

## Solution: Using Loxone to commission intelligent deep frying control.

The ideal temperature for deep-frying is between 170 and 180° C. A crispy crust is quickly formed around the food, which means less fat or oil can penetrate the middle. This keeps the food nice and juicy inside.

With a hot plate, a [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html), a [Touch & Grill](https://shop.loxone.com/enuk/touch-grill-air.html) and a little configuration work, the perfect temperature can be easily maintained even in your own pot.

The hot plate is connected to the mains via Smart Socket Air (ensuring amps are not exceeded) and switched on to full power. The temperature of the fat/oil is measured with the Touch & Grill. The [“Threshold Switch”](https://www.loxone.com/enen/kb/threshold-switch/) Function Block within [Loxone Config](https://www.loxone.com/enen/products/loxone-config/) ensures that the Smart Socket Air disconnects the hot plate from the power when the fat/oil reaches a temperature of 180° C – then reactivates it when it drops below 175° C. Thus, the temperature of the fat/oil will constantly be kept between 175 and 180° C.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Touch & Grill Air](https://shop.loxone.com/enuk/touch-grill-air.html)
- [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html)
- Suitable Hot Plate

### Configuration:

[
![Deep Frying Control - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-106-Deep-frying-with-Loxone.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-106-Deep-frying-with-Loxone.jpg)

### Download the sample file:

### Deep Frying

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-106-Deep-frying-with-Loxone.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-106-Deep-frying-with-Loxone.loxone)

## Why you and your customer should consider implementing deep frying control?

Intelligent deep frying control will give you perfectly cooked chips, churros or whatever else you want to make. It really is as simple as that.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)