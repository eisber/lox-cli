# Standby Killer

Source: https://www.loxone.com/enen/kb/standby-killer/

---

## Brief: I want the power to be turned off for any device on standby.

Standby devices, such as a TV, music systems, coffee machines, monitors etc. can become power guzzlers when their usage is added up. Many people still leave them in standby mode and use up electricity unnecessarily. With a so-called standby killer feature, all devices are automatically disconnected from the mains, ultimately saving on energy bills.

This use case will show you how you can configure a standby killer feature with Loxone to reduce energy wastage. This standby killer feature – also known as a standby eliminator – effectively eliminates or ‘kills’ the electricity to the devices that are in standby. While devices in standby might not individually be considered large users of power, collectively it can quickly add up over time.

## Solution: How to commission the ‘standby killer’ feature.

With the Smart Socket Air, you can turn a normal socket into a switchable socket. With just a few clicks, you can configure that connected devices on standby are completely disconnected from power.

In our Config example, we show you how you can use the Smart Socket Air to switch office monitors off. If the power is below 10 watts and the last employee leaves the office, the relay of the Smart Socket Air cuts the power. Turning the power back on for this switch works very easily with motion detection in the room. For example, when the first employee enters the office in the morning.

Please note that you should set a 5-minute delay so that there is enough time to leave the office after the monitor is turned off to avoid power being activated with motion as the last person leaves the office.

Hardware:
- [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html)
- [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html)

### Configuration:

[
![Standby killer for electrical devices - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-59-Standby-Killer-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-59-Standby-Killer-1.png)

### Download the sample file:

### Standby Killer

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-59-Standby-Killer.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-59-Standby-Killer.loxone)

## Why you and your customer should consider using Loxone as a standby eliminator.

Overall, setting up the standby killer feature with Loxone – in a residential or commercial setting – can reduce energy usage, make a building more energy-efficient, and potentially prolong the lifespan of the device. Having this feature in a second home is an excellent way to ensure non-essential devices are not left on when the property is not in use. Likewise, a standby killer for an office environment saves energy in the evening and throughout the weekend when staff are not in the building.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)