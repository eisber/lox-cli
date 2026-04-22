# Smart Battery Charging Limit

Source: https://www.loxone.com/enen/kb/smart-battery-charging-limit/

---

## Brief: I want to set a smart battery charging limit to preserve the battery life.

There is no one-size-fits-all solution to battery management as there are many variables such as the type of battery, material of the components, etc. However there are some general rules that apply to almost all batteries such as batteries should ideally never go completely flat, neither should they be charged permanently. Doing either of these things can result in a reduction in the battery operating life. Therefore the best way to maintain battery life is to implement a smart battery charging limit.

## Solution: Configure a smart battery charging limit with Loxone.

With the Switch Function Block and the [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html) (a switchable socket), the battery charging process of various tools, machines and devices can be stopped automatically and easily. On the one hand, a timer can be used to set a certain period of time during which the socket may supply electricity. On the other hand, the Smart Socket Air recognizes when the battery is full (energy < 100W) and switches off automatically.

At the same time, you can switch the charger on and off manually using a physical button on the socket (= Input 1). When it is on, the LED display flashes green.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html)

### Configuration:

[
![Smart Battery Charging Limit - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/06/Use-Case-11-Automatic-Power-off_2020-06-19.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/06/Use-Case-11-Automatic-Power-off_2020-06-19.png)

### Download the sample file:

### Automatic Power Off

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/06/Use-Case-11-Automatic-Power-off_2020-06-19.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/06/Use-Case-11-Automatic-Power-off_2020-06-19.loxone)

## Why you and your customer should consider having a smart battery charging limit?

With Loxone, a smart battery charging limit can be put into place to avoid permanent charging of the battery and therefore increase the lifespan of the battery. This programming can be used for many different devices: smartphones, e-bikes, cordless screwdrivers, tablets, headphones, etc. This solution is thus applicable in many areas of life from smart homes to workshops, offices, building yards, etc.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)