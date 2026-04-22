# Mould prevention in a Passivhaus

Source: https://www.loxone.com/enen/kb/mould-prevention-in-a-passivhaus/

---

## Brief: I need a solution to prevent mould in my Passivhaus using ventilation.

The air can only absorb a certain amount of water. This amount depends on the temperature. The higher the temperature, the more water vapour the air can absorb. If the warm air cools down, the excess moisture is released into walls, ceilings, furniture, and so on – which can result in mould. This is a very important issue in private homes but also in commercial and large scale residential buildings.

Condensation occurs mainly in the bathroom after the shower has been used, in the kitchen whilst cooking or in the basement. In winter the **masonry** is cooler due to the outside temperatures. Therefore the house is heated to keep it warm inside. If rooms are not kept warm enough or are not sufficiently ventilated, condensation can occur. This condensation is the perfect environment for fungi, mites and other parasites. The development of mould can lead to health problems.

The ideal humidity in a living room or bedroom is between 40% and 60%. In the kitchen and bathroom, the normal range is slightly higher at 50% to 70%.

In an ordinary house, windows can be opened if the humidity is too high. In energy-saving houses or a Passivhaus this is a controversial issue. Energy-saving houses are built in such a way that the temperature and air circulation is self-regulating. The temperature throughout the house is balanced. Opening a window is therefore not considered energy efficient.

So how can Loxone be used to aid mould prevention in a Passivhaus or energy-saving house? We will show you in this Use Case.

## Solution: Using Loxone for automatic mould prevention in a Passivhaus.

**Humidity monitoring**

In order to prevent mould, it is necessary to monitor the temperature and humidity. The [Loxone Touch](https://shop.loxone.com/enuk/loxone-touch.html) can be used here, as it has an integrated temperature and humidity sensor (in the Air and Tree version). Or you could use the [Room Comfort Sensor](https://shop.loxone.com/enuk/room-comfort-sensor.html).

**The integration of the ventilation system**

The most energy-efficient way to maintain suitable humidity levels in a Passivhaus is to use a ventilation system. If the ventilation system uses a heat exchanger, the ventilation system can regulate the air humidity without affecting the temperature in the house. But the ventilation system must be given the information as to when to start the air exchange.

By using the temperature and humidity sensor in the Touch or Room Comfort Sensor, every room can be monitored. If one of the rooms has excessive humidity, the ventilation system will start automatically. To configure this, the humidity levels collected from the sensors are connected with the “Min Max” Function Block in Loxone Config. Here the minimum and maximum values are defined.

To integrate the ventilation system, the “Room Ventilation Controller” Function Block is used. Different ventilation systems can be integrated into a Loxone System via suitable interfaces. This Function Block is connected to the outputs of the “Min Max” block. This allows the Miniserver to control the ventilation system depending on the measured humidity.

With the activation of the ventilation system, an air exchange takes place and the air humidity decreases. As soon as the humidity level returns to normal, the ventilation stops.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Touch](https://shop.loxone.com/enuk/loxone-touch.html) (with inbuilt humidity sensor) or
- [Room Comfort Sensor](https://shop.loxone.com/enuk/room-comfort-sensor.html)

### Configuration:

[
![Mould prevention in a Passivhaus - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-49-Prevent-mould-in-energy-efficient-homes.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-49-Prevent-mould-in-energy-efficient-homes.png)

### Download the sample file:

### Mould prevention in energy efficient homes

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-49-Prevent-mould-and-condensation-in-energy-efficient-homes.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-49-Prevent-mould-and-condensation-in-energy-efficient-homes.loxone)

## Why you and your customer should consider Loxone for mould prevention in a Passivhaus?

By monitoring temperature and humidity, you can give your customers intelligent mould prevention in their Passivhaus. Too high humidity can cause health problems and also lead to damage to the building. Your customer will incur costs to fix this.

By using the integrated temperature and humidity sensor in the Loxone Touch, your customers do not need any additional hardware (if a Touch is already installed in every room anyway). Only the monitoring and automation needs to be configured.

Especially in a Passivhaus or in energy-saving houses it is often unclear how to ventilate energy-efficiently and at the same time avoid mould. If there is a ventilation system, it can be easily integrated and thus prevented.

The comfort level in the house can be increased even further if a decentralised ventilation system is installed instead of a central one. In our application example, the ventilation system is activated when the humidity in the bathroom is too high. This also affects all the other rooms. However, with a decentralized ventilation system, your customers can control the ventilation room by room, so it only comes on in the rooms it’s needed in.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)