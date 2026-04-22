# Fire Door Monitoring

Source: https://www.loxone.com/enen/kb/fire-door-monitoring/

---

## Brief: I want a fire door monitoring feature that will sound an alert if a fire door is kept open.

A fire door has the task of securing openings in fire-retardant or fire-resistant walls against the passage of fire. Fire protection doors can only fulfil their function if they are kept closed or if they automatically close in the event of a fire. Therefore, they must never be wedged or tied open!

With Loxone the open state of the doors can be monitored – enabling effective fire door monitoring. In the case of an open fire door the Miniserver sends a message to inform the relevant parties.

## Solution: Using TTS (Text-to-Speech) to deliver an alert as part of fire door monitoring.

A [Door & Window Contact Air](https://shop.loxone.com/enuk/door-window-contact-air.html) from Loxone, gives you fire door monitoring as the open status of fire protection doors can be seen easily. In our example, the Miniserver uses TTS to send a message via the [Music Server](https://shop.loxone.com/enuk/loxone-music-server.html) as soon as the fire door is open for more than 3 minutes. You can also choose another type of notification if you want. Either via a push notification in the Loxone App or via Mailer or Caller Service if you don’t have a Music Server integrated into your Loxone system.

### Hardware:

-
[Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html)
- [Door & Window Contact Air](https://shop.loxone.com/enuk/door-window-contact-air.html)
- [Music Server](https://shop.loxone.com/enuk/loxone-music-server.html) + [12-channel Amplifier](https://shop.loxone.com/enuk/loxone-12-channel-amp.html) + [Speaker](https://shop.loxone.com/enuk/speaker.html)

### Configuration:

[
![Fire Door Monitoring - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-51-Fire-Protection-Doors.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-51-Fire-Protection-Doors.png)

### Download the sample file:

### Keep Fire Doors Closed

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-51-Keep-Fire-Doors-Closed.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-51-Keep-Fire-Doors-Closed.loxone)

## The advantages of intelligent emergency exit monitoring of escape routes.

In case of fire, it is essential that all fire doors are closed to prevent the fire from spreading through the building. Loxone offers the possibility to optimally monitor the open status of fire doors and to notify the relevant parties in if one is left open.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)