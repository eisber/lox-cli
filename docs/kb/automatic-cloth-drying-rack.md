# Automatic Cloth Drying Rack

Source: https://www.loxone.com/enen/kb/automatic-cloth-drying-rack/

---

## Brief: I want an automatic cloth drying rack for my restaurant.

In restaurants, tablecloths, towels, etc, are dried at an accelerated rate with the help of heated drying racks. These racks are turned on and off manually by members of staff. However, this usually results in the drying rack being left on as no one remembers to switch it off. On its own this is a waste of electricity however it can also have the knock-on effect of heating up the room. This could then lead to the ventilation system kicking in – wasting even more energy.

To avoid all of this the cloth drying rack should only be on when the restaurant is in operation. In this Use Case, we’ll show you how to set up an automatic cloth drying rack which will reduce electricity costs.

## Solution: Using Loxone to commission an automatic cloth drying rack.

**Automation of the cloth drying rack:**

The cloth drying rack is automatically turned on during the restaurant’s opening hours and switched off when the restaurant is closed. In our example, the restaurant is open from 13:00 to 17:00 and then from 19:00 to 22:00. This is achieved in Loxone Config by using a “Schedule”** **Function Block – which is where the operating times are stored.

However, there may be times where the restaurant is open outside of these times (a pre-booked party for example). For this there’s a Touch Switch located next to the rack which will activate it. However, crucially this will only switch it on for 30 minutes – so no one needs to remember to switch it back off again. For this the “Stairwell Light Switch” Function Block has been used. The “TH” parameter defines how long the drying rack is switched on for – 1800 seconds (30 minutes) in this case.

To prevent the drying rack from being switched on during public holidays, operating times are used. In the peripheral tree new entries are created for each public holiday, e.g. Christmas Eve – which are then assigned to the “Public Holidays” operating mode. In the “Schedule” there are no times stored for “Public Holidays” and hence the drying rack will not be activating on these days.

To control the cloth drying rack, both Function Blocks – Schedule and Stairwell Light Switch – are connected via a digital output to the drying rack.

**Working out kWh**

The “Utility Meter” Function Block in combination with the “Pulse Generator” block records the energy consumption. The “Pulse Generator” will be triggered by either the “Schedule” or “Stairwell Light Switch” blocks – depending on if the dryer has been turned on manually or according to the schedule. If, for example, the clothes dryer is on for one hours, the Pulse Generator will have pulsed 1800 times (because we have set it to pulse every other second). Next, we know that the dryer has a consumption of 2kW. So we take the total pulses (1800) and divide this by the kW (2) = which gives us 900.

Therefore, we set the Impulse parameter to 900 – which corresponds to 1 kWh.

**Working out cost**

To be able to show the monthly energy cost in the Loxone App, the “Virtual State” Function Block is used. In the background, we use the “Formula” function block to multiply the monthly consumption (AQ6) by the price our energy supplier charges for each kW, and then divide this by 100 to show us a cost in tradition currency format.

Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Coupling Relay](https://shop.loxone.com/enuk/coupling-relay.html) (optional – depending on the load of the device)

### Configuration:

[
![Automatic Cloth Drying Rack - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-91-Drying-Rack.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-91-Drying-Rack.png)

### Download the sample file:

### Automated Drying Rack

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-91-Automated-Drying-Racks.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-91-Automated-Drying-Racks.loxone)

## Why you and your customer should consider setting up an automatic cloth drying rack?

No business wants to rack up unnecessary energy costs. An automatic cloth drying rack helps eliminated any wasted energy. The automation of the rack allows it to be switched on based on a schedule so that it’s only on when it actually needs to be on.

Monitoring its consumption is also extremely useful for business owners, as it’s often very difficult to have an overview of how much electricity devices are using. All of this combined makes installing an automatic cloth drying rack a bit of a no brainer.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)