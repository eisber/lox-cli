# Automatically switch off an iron not in use

Source: https://www.loxone.com/enen/kb/automatically-switch-off-an-iron/

---

## Brief: Reduce the risk of fire by implementing an automatic switch off for an iron

All too often irons are accidentally left on, whether that’s people switching them on to heat up and then getting distracted or forgetting to turn them off after they’re done. An unattended hot iron can lead to a fire. This is a big issue in Ambient Assisted Living for older residents, however, by no means are the younger generation immune to this either.

In this Use Case, we show you how to implement an automatic switch off for an iron to avoid potential hazards such as a fire.

## Solution: Using Loxone to automatically switch off an iron not in use.

**In this example, we assume that your customer wants to avoid that someone forgets to switch off the iron at home.**

To do this, it is necessary to know the energy consumption of the iron and to have the ability to detect movements in the room. The [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html) enables power and energy measurement. Based on this we’ll know when the iron is switched on or off. The [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html), in the room where the iron is located, monitors whether there is someone in the room. If the iron is switched on for more than 15 minutes and no movement is registered, the Miniserver receives this information and switches off the iron via Smart Socket Air. In addition, the resident receives a push notification. If the resident is in an assisted living facility, their carer will also be notified.

In Loxone Config the information recorded by the Smart Socket Air is processed via the “Utility Meter” Function Block. If the predefined value is reached here and the Motion Sensor does not detect presence in the room, the “Switch” Function Block triggers the relay and the iron is automatically turned off.

It is also possible to automatically switch off the iron as soon as the resident leaves the house or when “Night Mode” is activated. Reducing the risk of a fire and also saving energy.

As a further option, it is possible to integrate the [Loxone Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html). So the resident and the carer receive a phone call as well as a push notification. Several telephone numbers can be stored here.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html)
- [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html)
- [Caller Service](https://shop.loxone.com/enuk/caller-service-10-year.html)

### Configuration:

[
![automatically switch off an iron - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-77-Iron-shut-down-for-safety.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-77-Iron-shut-down-for-safety.png)

### Download the sample file:

### Iron shut down for safety

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-77-Identify-Potential-Risks-iron-alarm.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-77-Identify-Potential-Risks-iron-alarm.loxone)

## Why you and your customer should consider an automatic switch off for an iron?

Europe as a whole has an ageing population and Ambient Assisted Living is becoming an increasingly important issue. The aim of AAL is to use technology to improve the quality of life of elderly people or people with disabilities. Older people in particular often forget to switch off devices. In order to avoid fire or other dangers, an automatic switch off for an iron can be used to prevent this. Although this Use Case has focused on an iron, the same logic could be applied to any electrical device.

The automatic switch-off serves to protect both the occupants and the building. Relatives can relax in the knowledge that the iron will be switched off automatically in an emergency.

However, as we mentioned earlier, it’s not only elderly people who leave the iron on. People of all ages find themselves thinking “did I definitely turn the iron off?” – especially when they’ve just left the house. With this automatic switch-off, you can give your customers more security and peace of mind.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)

n