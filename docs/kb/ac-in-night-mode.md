# AC in Night Mode

Source: https://www.loxone.com/enen/kb/ac-in-night-mode/

---

Noise reduction and airflow adjustment according to schedule for better sleeping comfort.

The goal is to automatically adjust the air conditioning to minimize noise and avoid direct airflow in the bedroom during the night, based on a predefined schedule.

**How it works**
- **Activation:** Automatically or manually by the user (app, switch, touch)
- **Deactivation:** Automatically at the set time (“Disable night mode”, default 8:00 a.m.)
- **Changes only occur when the air conditioning is in operation.** If there is no cooling needed and the AC is off, it is not affected by the default night mode times

**Time planning – example bedroom:**
- Monday to Friday: 9:00 PM – 6:00 AM
- Saturday and Sunday: 9:00 PM – 10:00 AM

**Behavior when night mode is activated**

The AC Unit Controller adjusts settings to improve sleep comfort:
- Fan speed to Quiet, Very Low or Low (depending on the device)
- Airflow position on position 5 to direct the air towards the ceiling and avoid direct airflow onto the bed
- (Optional) Dry mode in child-relevant rooms for gentle cooling

**Advantages**
- Noise is reduced to a minimum for undisturbed sleep
- Airflow is redirected to prevent direct airflow onto the bed

### Hardware:
- [AC Control Air](https://shop.loxone.com/enen/product/100553-ac-control-air-for-gree)

### Configuration:

[
![UseCase AC NightMode 1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-AC-NightMode-1.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-AC-NightMode-1.jpg)

[
![UseCase AC NightMode 2](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-AC-NightMode-2.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-AC-NightMode-2.jpg)

### Download the sample file

### AC Night Mode

			[Config 15.5.3.4](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-AC-NightMode.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-AC-NightMode.loxone)

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)