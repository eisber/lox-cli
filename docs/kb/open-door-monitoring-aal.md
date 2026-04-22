# Open Door Monitoring (AAL)

Source: https://www.loxone.com/enen/kb/open-door-monitoring-aal/

---

## Brief: I want to set up open door monitoring in an Ambient Assisted Living environment.

Managing and identifying potential risks in Ambient Assisted Living environments provides a greater degree of protection for older residents. Loxone is perfectly suited for use in these kinds of installations.

With Loxone, it’s possible to remotely check whether doors & windows are open, if devices such as irons, hobs etc. have been switched off after use and even trigger alarms if no movement has been detected for a certain period of time.

In this Use Case, we’ll look at how Loxone can be used as an open door monitoring system. To make sure that the front door, for example, is never accidentally left open over night.

## Solution: Using Loxone for open door monitoring.

To create an effective open-door monitoring system we, obviously, need a way of knowing if the door is open or not. To do this we’ll use the [Door and Window Contact Air](https://shop.loxone.com/enuk/door-window-contact-air.html). A door being open in the day is less problematic than at night. Which is why in our example a notification will only be sent if the door is open as it’s about to get dark – this is determined by the geocoordinates of the [Miniserver](https://shop.loxone.com/enuk/miniserver.html).

If it’s about to get dark and the Miniserver detects that a door is open, then a push notification will be sent to the occupant to notify them. The [Loxone Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html) could also be used to notify them through a phone call.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Air Base Extension ](https://shop.loxone.com/enuk/air-base-extension.html)
- [Door & Window Contact](https://shop.loxone.com/enuk/door-window-contact-air.html)
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html) (Optional)

### Configuration:

[
![Open Door Monitoring - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-72-AAL-Identify-Potential-Risks.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-72-AAL-Identify-Potential-Risks.png)

### Download the sample file:

### Identify Potential Risks (AAL)

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-72-AAL-Identify-Potential-Risks.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-72-AAL-Identify-Potential-Risks.loxone)

## Why you and your customer should consider implementing open door monitoring?

Getting old is just a part of life. However, automation allows people to stay in their own home for much longer, thanks to subtle support – such as open door monitoring.

Open door monitoring is a great example of how automation can be used to identify a potential risk before it becomes problematic. An open door through the night is essentially an invitation to a burglar. However, with our gentle reminder to shut the door before bed the danger is avoided.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)