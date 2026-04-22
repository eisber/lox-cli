# Monitor CO levels in a Garage

Source: https://www.loxone.com/enen/kb/monitor-co-levels-in-a-garage/

---

## Brief: I want a garage carbon monoxide detector system

Carbon monoxide (CO) is a highly toxic, odourless and colourless gas. It is produced, among other things, during the combustion process of car engines. It is, therefore, a serious health risk, especially in garages.

It is therefore, very important to monitor CO levels in a garage – whether that’s public or private. If there is no monitoring in place, there can be very serious health effects, which can be fatal in the most extreme cases.

In a Loxone installation, a CO alarm system can be implemented with a few simple steps.

## Solution: Using Loxone to intelligently monitor CO levels in a garage.

Only a 0-10V CO sensor is required to take a reading. If the CO value in the garage rises above a certain threshold value, the garage door is opened automatically (provided it is completely closed) and the owner is informed of this via a push notification.

If sufficient air exchange has taken place with the door open and the CO levels have fallen below a defined threshold value, the garage door will be automatically closed.

**Note:** In this example, we assume that the garage has an electric door. If there is only a manual door, you can still monitor CO levels in a garage and an alarm system can still be implemented simply by sending a push notification when the levels become too high – allowing the door to be manually opened.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- 0-10V CO Sensor

### Configuration:

[
![Monitor CO levels in a Garage - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-81-CO-Level-Garage.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-81-CO-Level-Garage.png)

### Download the sample file:

### Garage CO Monitoring

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-81-CO-Level-Garage.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-81-CO-Level-Garage.loxone)

## Why you and your customer should consider monitoring CO in a garage?

The fact that Carbon Monoxide is both colourless and orderless is what makes this gas so dangerous as it is virtually undetectable to humans. As soon as the gas is inhaled, it gradually displaces the oxygen from the body. It binds to haemoglobin (red blood cells) in the blood at the exact place where oxygen is normally bound and blocks the receptors.

The symptoms of carbon monoxide poisoning depend on the concentration of CO and the length of time one has been exposed to the gas.

A low CO concentration = headache, dizziness, vomiting, drowsiness, etc.

A medium CO concentration = severe headache, severe fatigue, confusion, accelerated pulse, altered breathing, etc.

A high CO concentration = unconsciousness, cardiac arrest, respiratory arrest and, as a consequence, brain damage and even death.

Which means that a garage carbon monoxide detector system can save lives in both private and public garages.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)