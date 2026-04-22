# Open window alert if it rains

Source: https://www.loxone.com/enen/kb/open-window-alert-if-it-rains/

---

## Brief: Alert me if a window is open and it starts to rain.

Many people have terrible experiences (especially in summer) with open windows (especially roof windows) and rain. It is great to have fresh air inside the building, but you have to be careful with the weather. Rain can easily ruin your furniture, electronic devices, books etc. So getting an early warning if it is about to rain would be ideal. This is why this brief will look at configuring an open window alert if it rains. (In the UK, it is not a case of if it will rain, but when…). Roof windows are being used more and more in self builds, they are a fantastic source of light and can really open up a space that has a sloping ceiling – but if a roof window is open and it starts to rain, you would want to know about it pretty quickly.

The Loxone Miniserver, with the help of a few accessories and actual weather data, can easily provide an open window alert if it rains. Getting a push notification, for example, means you can react quickly and close that one window upstairs that you had forgotten was open.

## Solution: Sending an alert if it rains and a window is open.

First, we need to get information about incoming rain. The Loxone Weather Service provides you with a precise weather forecast and of course the current situation. The other required value is the status of the window – open/closed. For this function, we recommend using the Loxone Door & Window Contact Air. Now we’re almost instantly able to now configure an open window alert if it rains.

In the example, we have used the current precipitation to send a notification (urgent) and forecast from the next two hours to send a warning to email (less urgent). The notifications only happen, if the window contact detects an open window. If the customer has electronically controlled windows, you could simply configure the windows to automatically close. The method of notification is completely up to you.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Air Base Extension](https://shop.loxone.com/enuk/air-base-extension.html) (unless you are using a [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html))
- [Door & Window Contact Air](https://shop.loxone.com/enuk/door-window-contact-air.html)
- [Loxone Weather Service](https://shop.loxone.com/enuk/weather-service-10-year.html) (Or Loxone [Weather Station](https://shop.loxone.com/enuk/weather-station.html) with included weather service subscription)

### Configuration:

[
![Open window alert if it rains - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-34-Open-Window-Alarm.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-34-Open-Window-Alarm.png)

### Download the sample file:

### Open Window Alarm

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-34-Open-Window-Alarm.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-34-Open-Window-Alarm.loxone)

## Why you and your customer should consider an alert for open windows when it starts to rain?

It may sound like just nice-to-have feature until you experience the damage that having an open window during a downpour can do. Especially in summer, where unexpected heavy rains are not uncommon – having an open window alert if it rains can mitigate a lot of trouble and damage. When it starts to rain, instead of thinking “did I leave that window open?” or “I wonder if the kids closed their window?” or physically going to check the windows throughout the house, you can simply rely on an ‘open window’ alert if it rains This would certainly go a long way in not only reducing damage and nuisance but probably saving a fair bit of sanity.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)