# Automated MVHR based on air quality & occupancy

Source: https://www.loxone.com/enen/kb/automated-mvhr-air-quality/

---

## Brief: I want automated MVHR that provides optimum air quality and energy efficiency at the same time.

Due to their air-tight build, controlled ventilation is a central component of almost all new building projects. It ensures adequate ventilation of spaces, preventing the formation of mould and ensuring healthy air quality.

However, in certain circumstances, ventilation systems can be brought on when they shouldn’t be.

This is especially true in office buildings. The intensity of the ventilation in any given room is often based on how it will be used – essentially how many people will be in there. However, this can cause issues. For example, a conference room may be busy Monday to Friday however it will usually be empty on weekends and bank holidays. With many systems, the ventilation will work just as hard regardless of occupancy – this is a waste of both energy and money.

With Loxone the intensity of the ventilation system can be controlled based on the actual use of a room. Giving you Automated MVHR.

## Solution: Using Loxone for an automated MVHR control solution.

Let’s stick to the example of the conference room. With the “Room Ventilation Controller” Function Block and the [Room Comfort Sensor](https://shop.loxone.com/enuk/room-comfort-sensor.html), we can measure the CO2 levels and the humidity of the room. If one of the two values moves out of a defined range, the intensity of the ventilation system is automatically adjusted.

Additionally, we’ll use a [Presence Sensor](https://shop.loxone.com/enuk/motion-sensor.html). This gives us information about the actual presence in the room. If there are people in the room, the ventilation system runs with greater intensity than if there is no one in the room.

If CO2 and humidity values exceed a certain limit, a “Boost Mode” can be activated. This ensures the greatest possible air exchange in a short time. However, to avoid excessive noise, this “Boost Mode” is deactivated when the room is being used.

At the same time, the current outdoor temperature is taken into account and compared with the current indoor temperature. If, for example, it is colder outside than inside and the room is currently warmer than desired, the ventilation system can be used to passively cool (or, of course, to passively heat if the reverse is true).

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Room Comfort Sensor](https://shop.loxone.com/enuk/room-comfort-sensor.html)
- [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html) or [Presence Sensor](https://shop.loxone.com/enuk/presence-sensor-tree.html)
- [Modbus Extension](https://shop.loxone.com/enuk/modbus-extension.html) (optional)

### Configuration:

[
![Automated MVHR - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-4-Needs-Based-Ventilation-Room-Ventilation-Controller.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-4-Needs-Based-Ventilation-Room-Ventilation-Controller.png)

[
![Automated MVHR - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-4-Needs-Based-Ventilation-Leaf.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-4-Needs-Based-Ventilation-Leaf.png)

### Download the sample file:

### Needs Based Ventilation

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-4-Needs-Based-Ventilation.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/04/Use-Case-4-Needs-Based-Ventilation.loxone)

## Why you and your customer should consider automated MVHR?

Sufficient fresh air is essential for well-being and comfort in every building. In old buildings, air exchange takes place through the natural gaps and cracks in the building’s shell. However, in new buildings, new energy standards mean that the buildings are becoming increasingly air-tight and therefore air exchange cannot naturally occur. This results in poor air quality which can severely impact health and well-being. Also, air accumulates moisture and this can result in the formation of mould.

Controlled ventilation ensures that there is sufficient fresh air in new buildings, thus ensuring comfort and well-being. It also helps to prevent the formation of mould. The demand-based activation of ventilation ensures that the system is operated as energy-efficiently as possible.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)