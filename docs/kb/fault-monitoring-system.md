# Fault Monitoring System

Source: https://www.loxone.com/enen/kb/fault-monitoring-system/

---

## Brief: I want to have a fault monitoring system for technical installations.

In commercial and large-scale residential buildings there are many technical components that need to be monitored, maintained and repaired: elevators, air conditioners, heaters, ventilation systems, etc.

If an error occurs, the people in charge often only hear about them once a member of staff, or worse, a customer stumbles across the issue. This can result in a prolonged period of disruption and higher repair costs.

A fault monitoring system that automatically sends an alert as soon as parts such as filters need to be replaced ensures effective building maintenance.

## Solution: A fault monitoring system using an automated status report.

An effective fault monitoring system should automatically make the relevant parties aware of any upcoming maintenance. If we take a filter for instance (e.g. from a central air conditioning system) which has to be replaced regularly, then automated monitoring can be easily implemented with logic created within Loxone Config.

With the help of a [virtual input](https://www.loxone.com/enen/kb/virtual-inputs-outputs/), it is possible to define after which time period a notification should be sent. In the example, a filter has to be changed every six months. Therefore an interval of 180 days is created. Initial notification is sent via email 45 days before the filter needs to be changed. Seven days before, a push notification will be sent through the Loxone App to prompt the relevant parties that a filter change must be arranged. On the 180th day, a call will be made (via the [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)) informing the relevant people that a filter change is now overdue and must be done immediately.

After a successful filter change, the interval can start again from the beginning. To restart the interval, someone simply needs to press the “Replace Filter” button within the [Loxone App](https://www.loxone.com/enen/kb/types-of-visualisation/). This button resets the countdown back to 180 days and starts counting down the time again.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)

### Configuration:

[
![Fault Monitoring System - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-47-Fault-Monitoring.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Loxone-Use-Case-47-Fault-Monitoring.jpg)

### Download the sample file:

### Fault Monitoring

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-47-Fault-Monitoring.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-47-Fault-Monitoring.loxone)

## The advantages of an automatic status message from a fault monitoring system.

By implementing automatic maintenance reminders as part of a wider fault monitoring system, you heavily reduce the risk of components malfunctioning. In commercial buildings, it is especially helpful if all messages converge in one overall fault monitoring system. This way, problems can be solved before they even become noticeable. It also saves the caretakers a lot of organisational work. This, in turn, reduces repair costs and creates a structured overview of all areas of the building. The investment costs for the implementation of a Loxone-based fault monitoring system are kept within limits, as no additional hardware is required.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)