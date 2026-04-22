# Remote Airbnb Management

Source: https://www.loxone.com/enen/kb/remote-airbnb-management/

---

## Brief: I want a way of managing my Airbnb remotely.

Landlords of short-term-let property such as Airbnbs are often not based in the immediate vicinity of their rental properties. Therefore, it’s not always possible to know if and when a guest has checked out. This can be problematic for cleaning/servicing the property as there’s no certainty that the guest has left. However, more importantly, when a guest leaves the property, there’s no way of knowing they locked up and set the alarm correctly. Essentially, there’s no way of knowing if the property is safe.

A remote Airbnb management system would make this process much easier. A landlord could remotely check that the guests have left. Also, the alarm system could be automated to ensure that as soon as guests leave it is activated, any electrical devices could be automatically turned off and the heating could be lowered.

## Solution: Using Loxone to enable remote Airbnb management.

With Loxone, you can enable remote Airbnb management but also intelligent automation to take care of a range of tasks. This can be achieved by setting the occupancy schedule of the property withing the Loxone App – the property could then be placed into Away Mode when the property is vacant.

To implement this, two [operating modes](https://www.loxone.com/enen/kb/operating-modes/) called “Booked” and “Not Booked” are created – these can be set up to one week in advance. Check-in and out times are then stored in the “[Schedule](https://www.loxone.com/enen/kb/timerscheduler/)” Function Block. This block is visible within the[ Loxone App](https://www.loxone.com/enen/products/apps/) meaning that booking times can quickly and easily be adjusted.

In our example, we assume that the Airbnb is occupied from Friday 11 am to Sunday 11 am. When the check-in time comes around (Friday 11 am) the alarm will automatically be deactivated. When it’s time to check out (Sunday 11 am), the Airbnb will automatically switch into Away Mode – this means the heating will be lowered, the alarm will be activated and the owner will be informed about any open/windows or doors through the Loxone App.

**Tip:** Times and access rights for cleaners can also be stored in the system.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Motion Sensor Air](https://shop.loxone.com/enuk/motion-sensor.html)
- [Door & Window Contact Air](https://shop.loxone.com/enuk/door-window-contact-air.html)
- [Multi Extension Air](https://shop.loxone.com/enuk/multi-extension-air.html)
- [Tree Extension](https://shop.loxone.com/enuk/tree-extension.html)
- [Valve Actuator](https://shop.loxone.com/enuk/valve-actuator.html)

### Configuration:

[
![Remote Airbnb Management - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-97-AirBnB-Sleep-Mode.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-97-AirBnB-Sleep-Mode.jpg)

### Download the sample file:

### Sleep Mode AirBnB

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-97-Turn-on-sleep-mode-Airbnb.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-97-Turn-on-sleep-mode-Airbnb.loxone)

## Why you and your customer should consider setting up remote Airbnb management?

The automatic activation of Away Mode ensures that the empty AirBnB is protected as the alarm is set. The owner does not need to worry about their property from afar. Also, electrical devices are turned off and the heating is lowered – saving money and essentially increasing profit margins.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)