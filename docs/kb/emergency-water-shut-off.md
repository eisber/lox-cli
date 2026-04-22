# Emergency Water Shut Off

Source: https://www.loxone.com/enen/kb/emergency-water-shut-off/

---

## Brief: I need to implement emergency water shut off if there is a leak.

The early detection of a water leak is crucial to prevent damage to a building. It is essential that a water leak is detected immediately and that the relevant parties are informed about the events – whether that’s the owner, caretaker, etc. Simply knowing that water is leaking is a massive help however implementing an emergency water shut off through a solenoid valve is even better.

With the help of a water sensor, a leak can be detected directly at its location. We recommend using a water sensor around appliances such as sinks, washing machines, dishwashers etc.

An additional challenge is the monitoring of pipes that cannot be directly monitored by a sensor – such as pipes that run within the fabric of the building. Although water leaks occur less frequently here if they go undetected for any period of time they can cause massive amounts of damage.

## Solution: Use Loxone to close an emergency solenoid valve if there is a water leak.

**Water leak detection for accessible water pipes**

With the help of the [Water Sensor Air](https://shop.loxone.com/enuk/water-sensor-air.html), a water leak can be detected immediately. Thanks to Loxone Air technology, this small wireless sensor can be placed almost anywhere. No cabling is necessary.

Even wired sensors from third-party manufacturers can be integrated into a Loxone system.

Thanks to the “Fire & Water Alarm” Function Block within [Loxone Config](https://www.loxone.com/enen/products/loxone-config/), a multi-stage alarm is triggered when a water leak is detected. In the [Loxone App](https://www.loxone.com/enen/products/apps/), you can see which sensor has triggered the alarm, so the problem can be addressed quickly. To avoid further leakage, an emergency water shut off kicks into action – this will only be disabled once someone has acknowledged the alarm on the Loxone App.**

**

In our example, the[ Loxone Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html) was used to set off the alarm outside the building. The calls are sent one after the other thanks to the “Alarm Sequence” Function Block and thus the responsible parties are made aware of the water leak.

**Water leak detection for inaccessible water pipes**

Sometimes pipes are inaccessible and hence it’s not always possible to install a water sensor, especially in offices and commercial buildings. In this scenario, water leaks can be detected through a water consumption monitoring system. This is rather difficult in private buildings, where water consumption can fluctuate considerably. In offices and commercial buildings, on the other hand, water consumption is greatly reduced on weekends and holidays.

For example, the average water consumption on a working day is 50 litres per person, while on weekends and holidays, when the workforce is away, the total consumption is less than 50 litres. The water consumption is recorded using the “Utility Meter” Function Block and the average water consumption per day is documented in the “Schedule” Function Block. If the water consumption deviates, e.g. on weekends, this is recorded through the “Status” Function Block and an email is sent.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Water Sensor Air](https://shop.loxone.com/enuk/water-sensor-air.html)
- Electronic Solenoid Valve (mainline)
- Water Meter
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)

### Configuration:

[
![Emergency Water Shut Off - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/3-emergency-water-cut-off.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/3-emergency-water-cut-off.jpg)

### Download the sample file:

### Emergency Water Cut Off

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Emergency-Water-Cut-Off.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Emergency-Water-Cut-Off.loxone)

## Why you and your customer should consider emergency water cut off?

Water damage can cause very high costs in both private and commercial properties. Since a water leak often goes unnoticed for several weeks or months, irreparable damage can often form in the background. Even after the water leak has been properly repaired, consequential damage cannot be avoided.

One of the biggest issues here is the mould forming due to the moisture. Mould formation is not immediately perceived as such visually, although it already has a considerable influence on the health of the people in the building.

Significant damage can be prevented by implementing a notification system and emergency water shut off.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)