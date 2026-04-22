# Overcurrent Protection

Source: https://www.loxone.com/enen/kb/overcurrent-protection/

---

## Brief: I want to implement overcurrent protection.

In a typical household having multiple electrical devices on at once is completely fine. However, problems can, occasionally, occur when multiple bigger devices which require more power are on at the same time.

When used together things such as electric car chargers and saunas could cause the fuse board to trip.

In this Use Case, we’ll show you how to implement intelligent overcurrent protection to ensure that this never happens.

## Solution: Using Loxone to set up overcurrent protection.

As electrical devices – such as a car charger and a sauna – are already controlled by the [Miniserver](https://shop.loxone.com/enuk/miniserver.html), the system knows whether a device is currently on or off.

Therefore simple logic was created within [Loxone Config](https://www.loxone.com/enen/products/loxone-config/). This logic ensures that if the sauna is currently active then the car charger cannot be switched on and visa versa. It’s a first come first serve policy.

This simple bit of logic gives you intelligent overcurrent protection.

If someone tries to turn on the sauna while the car charger is on, for example, they’ll be made aware (through the [Loxone App](https://www.loxone.com/enen/products/apps/)) that the sauna is currently disabled. Making it clear for everyone.

**Note:** In our example, there is no automatic prioritization. However, it is perfectly possible to create logic which would give the sauna preferential treatment over the car charger, for instance. Then if the sauna is activated while the car charger is on, the charger would simply be switched off until the sauna is done with.

### Configuration:

[
![Overcurrent Protection - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-105-Overload-Protection.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-105-Overload-Protection.png)

### Download the sample file:

### Overload Protection

			[Config 15.1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/Use-Case-105-Overload-Protection.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/Use-Case-105-Overload-Protection.loxone)

## Why you and your customer should consider using an overcurrent protection system?

We’ve all been there. One moment everything is normal and then the next you’re completely in the dark. You’ve got to search for a torch and venture over to the fusebox to see what’s going on. By implementing automated overcurrent protection you’ll never have to worry about the electricity tripping again.

The best bit about this Use Case is that no additional hardware is required – it’s simply extra logic within Loxone Config.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)