# AC with Spotprice

Source: https://www.loxone.com/enen/kb/ac-with-spotprice/

---

### Avoid expensive electricity hours for air conditioning units using the AC-Control Air.

With **variable electricity prices,** the air conditioners are automatically **switched off** **during the most expensive hours** of the day to avoid unnecessary costs.

The **spot price module** determines the **20 cheapest hours** of the following day at midnight every day.

Within this time window, the air conditioning of the rooms can be **enabled** via the AC units, while during the **4 most expensive hours, **the Spot Price Optimizer ensures that the units are not activated

The air conditioning modules have an input for **load shedding** (“LS”), which is activated as soon as the expensive hours of the day apply.

This status change is also clearly displayed **directly in the visualization** of the block.
- If necessary, the time periods for calculating cheap and expensive hours can also be easily adjusted in the app’s **“Spot Price Optimizer”** .
- If individual rooms **have different requirements** for heating and cooling, the **central air conditioning unit **intelligently **decides** which operating mode is appropriate.

### Hardware:
- [AC Control Air](https://shop.loxone.com/enen/ac-control)

### Configuration:

[
![UseCase AC SpotPrice](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-AC-SpotPrice.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/UseCase-AC-SpotPrice.jpg)

### Download the sample file

### AC SpotPrice

			[Config 15.5.3.4](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/Spot-Price-Optimizer-AC-Control.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2025/05/Spot-Price-Optimizer-AC-Control.loxone)

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)