# Smart Manufacturing Monitoring

Source: https://www.loxone.com/enen/kb/smart-manufacturing-monitoring/

---

## Brief: I want to monitor a production line and receive error notifications.

A production or assembly line is commonplace in factories. It is a process in which parts are added per workstation in order to ultimately arrive at the fully finished product. From cars to microchips. Via a conveyor belt, the product can be finished faster than if the production relied solely on manual labour. Machines aren’t always perfect though… Should the conveyor belt in a production line experience a blockage, it is important to quickly inform a technician so that production can be resumed as fast as possible. So, I want to monitor a production line in my factory and alert the responsible member of staff of a blockage straight away, so that they can act to mitigate the potential impact this would have on the output.

This can be automatically integrated via the Loxone system. Read on to find out how setting up smart manufacturing monitoring can ensure that the notification of a blockage is immediately and automatically reported to the right person, in this case, the technician.

## Solution: How to commission smart manufacturing monitoring.

The Loxone system is an open system. This means that the Loxone Miniserver does not only speak with Loxone components, but also with products of other manufacturers. We could check the functioning of the conveyor belt with our Motion Sensor or with the Pressure Sensor, and it is even possible to integrate a sensor with an optical lens. This type of sensor would closely monitor a production line and trigger an alert in the event of a malfunction.

If the conveyor belt blocks, the optical lens will immediately register this and will send a signal to the Miniserver. With a smart manufacturing monitoring feature in place, the Miniserver will, in turn, send a push notification to the person in charge, in this case, the technician.

In the above situations, one technician is notified each time. However, the parameters in Loxone Config allow for more than one person to be notified. An extra option that can be useful is the integration of the Loxone Caller Service. This means that in addition to a push notification, the on-duty technician will also receive a telephone call. Again, multiple phone numbers can be linked to this.

It would also be good for the general factory staff to be aware of a blockage, so the smart manufacturing monitoring feature can also trigger an alarm siren or light internally in the workplace. The Loxone Alarm Siren would be a suitable product to integrate for this feature. From the moment the optical lens detects a blockage, a signal is sent to the Miniserver which will control the Alarm Siren so that employees in the workshop are made aware as soon as possible to avoid the situation becoming worse or to clear potential blockages that can safely be done so.

To get an overview of the functioning of the conveyor belt, a log file can also be created in which any blockages are stored. In this way, the smart manufacturing monitoring doesn’t only inform you of a potential issue there and then, but it can also keep a record of potential issues that were noted for that month, for example.

Hardware:
- [Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- Optical Lens Sensor (3rd party)
- [Alarm Siren](https://shop.loxone.com/enuk/alarm-siren.html)
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)

### Configuration:

[

](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-65-Smart-Manufacturing-Process-1.png)[
![Assembly line notification - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-65-Smart-Manufacturing-Process-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-65-Smart-Manufacturing-Process-1.png)

[

](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-65-Smart-Manufacturing-Process-2.png)[
![Smart manufacturing monitoring - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-65-Smart-Manufacturing-Process-2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-65-Smart-Manufacturing-Process-2.png)

### Sample file:

### Smart Manufacturing Process

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-65-Smart-Manufacturing-Process.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-65-Smart-Manufacturing-Process.loxone)

## Why you and your customer should consider smart manufacturing monitoring on a production line.

Time is money! Mass production is the production of large quantities of standard products, popularised by Henry Ford’s T-Ford at the beginning of the 20th century. Mass production often uses a conveyor belt to move products to workers who specialise in performing one particular task in the production process.

Mass production is capital intensive because it requires a relatively large number of machines per employee. This also means that there must be a large sales market for a product in order to make mass production worthwhile in the first place and to recoup the large investments in machinery.

So when a conveyor belt blocks and the technician isn’t immediately informed, a manufacturing company can very quickly incur a loss. For this reason, monitoring a production line can be invaluable.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)