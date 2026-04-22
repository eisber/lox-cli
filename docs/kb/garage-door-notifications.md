# Garage Door Notifications

Source: https://www.loxone.com/enen/kb/garage-door-notifications/

---

## Brief: I want to receive a garage door notification if it is left open.

While windows and doors normally get checked to see if they are closed before going to bed, the garage door is often overlooked as part of this check. This could be because the garage is not in direct sight, it’s not convenient, or it simply gets forgotten. This is, of course, taking it for granted that you are actually at home to do these checks. If are leaving the house, and perhaps you’ve left in a hurry, you may not have waited to check that the garage door closes. Whether you’ve pressed the button for it to close, and it didn’t, or you simply drove off in too much of a hurry. Regardless of the scenario, there’s an overall benefit of having garage door notifications that can let you know if the garage door has been left open.

Now, having the garage door open during the day is not always unintentional. It might be open because the gardener is there, or you’re cleaning the car, or the kids are cycling in the street on a sunny Saturday afternoon. In this case, we would specifically only want to know if the garage door has been left open after dusk – as this is when we’re likely settling in for the evening and security becomes more of a concern. Especially as a garage is not only used to park your prized possession but often it’s also used to store valuable things like tools, sports equipment, etc.

## Solution: How to commission notifications for the garage door.

The solution for this use case is to use Loxone to set up garage door notifications. These will specifically trigger a push notification from the Loxone Miniserver to your customer’s smartphone or tablet – if the garage door is still open after dusk.

If the geo-coordinates of the specific location are saved correctly in Loxone Config, the Loxone Miniserver knows the time for dusk for every day so a garage door notification will be dynamic as the time of dusk changes throughout the year.

With the addition of a Door & Window Contact Air, the Loxone system will know whether the garage door is closed or open when it reaches dusk for that location. If at this time the garage door is still open, the Miniserver will trigger the garage door notification so that someone in the household is made aware and they can manually take the appropriate action.

					Theoretically, it is also possible to close the garage door if it has been left open for a certain about of time or after dusk. However, for safety reasons, this has been omitted from the use case.

Hardware:
- [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html)
- [Door & Window Contact Air](https://shop.loxone.com/enuk/door-window-contact-air.html)

### Configuration:

[
![Garage Door Notifications - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Config-Use-Case-Garage-Door-Notification.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Config-Use-Case-Garage-Door-Notification.png)

### Download the sample file:

### Garage Door Notification

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/use-case-60-garage-door-notification.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/use-case-60-garage-door-notification.loxone)

## Why you and your customer should consider having garage door notifications.

Overall, having garage door notifications if the garage door is left open can offer peace of mind and, albeit simple, is one of the many smart capabilities of Loxone hardware and software that can improve the way your customer lives at home.

The cost of realising this feature, especially if Loxone is already installed in the house, is near negligible when you consider the benefit of convenience and the added protection it offers by ensuring you don’t leave your garage door open at night inviting opportunistic thieves to help themselves.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)