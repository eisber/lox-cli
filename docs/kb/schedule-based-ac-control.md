# Schedule-based AC control

Source: https://www.loxone.com/enen/kb/schedule-based-ac-control/

---

### Time control of the air conditioning according to room type including pause function when windows are open.

The goal is to automatically turn the air conditioning on and off based on a weekly schedule defined by room type. This improves energy efficiency and ensures comfort during active hours.

**Furnishings**

Use the “Timing by Room Type” feature in Auto Configuration for scheduling.

**Sample schedule – living room:**
- Sunday to Thursday: 6:00 – 21:00
- Friday and Saturday: 6:00 a.m. – 12:00 a.m.

**Behave**

The air conditioner only turns on when the schedule is active. Outside of these times, it turns off automatically.

**Customization options**
- Schedule and behavior can be customized via the AC Unit Controller settings.
- Default values ​​can be changed in the “Properties” tab in the settings.
- Link to specific operating modes such as “vacation” to temporarily override the default behavior.

**Function Door and Window Contact Air**

When the air conditioner is on, it will pause as soon as a door or window is opened. Operation will resume automatically once all are closed.
- 0 = closed
- 1 = open

This feature helps save energy by avoiding cooling or heating while windows or doors are open.

Tip: Recommend customers to use operating modes to easily change system behavior without having to manually adjust the schedule.

### Hardware:
- [AC Control Air](https://shop.loxone.com/enen/product/100553-ac-control-air-for-gree)
- [Door and Window contact Air](https://shop.loxone.com/enen/product/100210-door-window-contact-air-white)

### [
![UseCase schedule based AC](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-schedule-based-AC.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-schedule-based-AC.jpg)

### Download the sample file

### AC Schedule

			[Config 15.5.3.4](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/Schedule-Based-AC-Control.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/Schedule-Based-AC-Control.loxone)

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)