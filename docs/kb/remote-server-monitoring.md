# Remote Server Monitoring

Source: https://www.loxone.com/enen/kb/remote-server-monitoring/

---

## Brief: I want to have remote server monitoring and receive an alert if something is wrong.

Whether it is for a small or large company, or a holiday home with a network, remote server monitoring can be a fantastic feature to stay in-the-know. Companies are storing crucial data on servers which they always need to access. Should one of the servers have a problem and it is not noticed immediately, it can not only cause inconvenience but can also result in an inability for staff to work or even affect profits.

A more abstract example is the monitoring of large refrigerator rooms in a hotel or restaurant environment. Should the server which is monitoring the refrigerator become unavailable, then any notification about the refrigerator itself won’t be sent. Therefore remote server monitoring can alert you to a potential network issue so that this can be corrected as soon as possible.

Sites that are not regularly visited, whether these be off-location commercial premises or even a second home, a solution that will monitor a server can be vital in alerting the customer about potential damage rather than someone only finding out when they are next scheduled to be there. If such a second location has potentially been damaged after a storm due to a lighting strict taking out the network, then the remote server monitoring feature can send a notification right away.

That’s why it is important to have a solution to monitor a server and to be informed as soon as there is a potential problem. The responsible person will be notified and can react and deactivate the alarm and notification as soon as everything is ok.

## Solution: How to set up smart remote server monitoring.

This use case is based on four servers which should be monitored. As soon as one of the servers is offline for a predefined time – for example, like 30 seconds – this information has to be written into the logfile serverstatus.log. Additionally, an alert sequence is started and triggers several calls (max. 10 run-throughs) which can phone various contacts, such as the IT technician on duty, the IT manager, or even the CEO.

The notifications via the calls can be deactivated by pressing the number 1. Afterwards, the call will be ended. It’s now up to the technician to check the server and start manual maintenance.

In this example, the Miniserver pings the four servers every 10 seconds to check that they are online. If one of the servers can not be pinged within 10 seconds, the Miniserver will do 5 pings every 30 seconds. If the miniserver still doesn’t get a pingback from the monitored server, the alarm sequence will be activated.

This is realized with the [Ping](https://www.loxone.com/enen/kb/ping-function-block/) Function Block for each monitored server. With each Ping Function Block, you fill in the ping address of the server. Every 10 seconds the Ping block will make sure that the monitored server is pinged. Via a [Status Block](https://www.loxone.com/enen/kb/status/), you can define the status message that should be shown as soon as the monitored server is offline. By using the block “Alarm Sequence” the calls to the specific people can be defined.

Hardware:
- [Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)

### Configuration:

[
![Rremote server ping sequence - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Cases-57-Server-Monitoring-2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Cases-57-Server-Monitoring-2.png)

[

](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Cases-57-Server-Monitoring-3.png)[
![Remote server monitoring - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Cases-57-Server-Monitoring-3.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Cases-57-Server-Monitoring-3.png)

### Download the sample file:

### Server Monitoring

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/Use-Cases-57-Server-Monitoring.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/Use-Cases-57-Server-Monitoring.loxone)

## Why you and your customer should consider this effective solution for remote server monitoring.

The outage of a server can cost your customer a lot of money and trouble. To mitigate the effect of a server outage remote server monitoring should be paired with a series of notifications so that your customer will be notified in an emergency. This way, your customer can make sure that in case of an outage, the responsible employees can react immediately.

If Loxone has already been installed in the building, then you can monitor a server using the existing hardware already in place, reducing the overall cost of setting up remote server monitoring while enhancing functionality.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)