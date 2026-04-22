# Automatic Safety Shutdown

Source: https://www.loxone.com/enen/kb/automatic-safety-shutdown/

---

## Brief: I want an automatic safety shutdown for my electrical devices.

An automatic safety shutdown of electrical devices has many benefits from energy-saving to reducing fire risks. With Loxone, a safety shutdown can be based on a wide range of factors such as presence, temperature, the triggering of an alarm and much more. Loxone’s versatile nature means that there is plenty of scenarios where an automatic safety shutdown would be used. Here are some examples:
- Electric heaters, irons, straighteners, etc. being switched off when the room temperature exceeds a pre-defined value.
- Depending on regulations – the ventilation being disabled if the fire alarm is triggered. (Important for containing the fire.)
- Switching off any water pumps in the event of a water alarm.

## Solution: Using Loxone to configure an automatic safety shutdown.

With Loxone, an automatic safety shutdown can be configured quickly and easily. In our example, we’ll take a heating device (such as a radiant heater) and switch it off when no presence is detected in the room it’s in for 30 minutes.

To achieve this, the heater must be connected via a socket which can be controlled with Loxone. A [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html) is an easy way to achieve this or relay control can be used – through a [Relay Extension](https://shop.loxone.com/enuk/relay-extension.html), [Nano IO Air](https://shop.loxone.com/enuk/nano-io-air.html), etc. – while observing the amperage.

It’s important to understand that (in our example) the heater itself is not directly switched on or off with Loxone. The heater is manually switched on or off by a person. Loxone merely ensures that an automatic safety shutdown takes place should the heater be accidentally left off.

When the [Presence Sensor](https://shop.loxone.com/enuk/presence-sensor-tree.html) detects presence in the room, the socket is automatically enabled – allowing the person to turn the heater on. At the same time, a switch-off delay of 1,800 seconds (30 minutes) begins. Presence being detected will reset this switch-off delay – only when no presence has been detected for 30 minutes will the heater be turned off.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Air Base Extension](https://shop.loxone.com/enuk/air-base-extension.html)
- [Presence Sensor](https://shop.loxone.com/enuk/presence-sensor-tree.html)
- [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html)

### Configuration:

[
![Automatic Safety Shutdown - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-108-Automated-Safety-Shutdown.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-108-Automated-Safety-Shutdown.png)

### Download the sample file:

### Automated Safety Shutdown

			[Loxone Config 11.0](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-108-Automatic-Safty-Shutdown.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-108-Automatic-Safty-Shutdown.loxone)

## Why you and your customer should consider setting up an automatic safety shutdown?

An automatic safety shutdown not only increases energy efficiency but it can also greatly reduce the risk of danger. With very limited hardware, an intelligent shutdown can be put in place to avoid any accidents.

This Use Case is just one example of an automatic safety shutdown in action. However, this functionality is very versatile and can be applied in a variety of scenarios.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)