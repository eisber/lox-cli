# Watering a sports field

Source: https://www.loxone.com/enen/kb/watering-a-sports-field/

---

## Brief: A smarter way to manage the watering of a sports field.

A football field has its own specific requirements when it comes to irrigation. It differs entirely in the water consumption, water distribution and timing from other sports fields. The suggested parameters for watering a sports field also differs regionally, so the irrigation system also needs to be adjusted based on local conditions.

Sports fields with uneven watering develop uneven consistencies that are not conducive to playing football and may cause injuries. The associated repair if this occurs can be costly and render the pitch unusable for a period.

The surface of football fields is mostly natural or rolled turf. This lawn needs to be well maintained and watered to remain both luscious and hard-wearing. In many cases, a groundskeeper usually has to manually monitor conditions and keep an eye on the weather forecast – so the watering of a sports field is not simply a scheduled task.

To sum up, a watering system for a football field means a lot of resources in terms of time, staff, money and water.

## Solution: How to set up a watering system for a football field.

In this use case for watering a sports field, we’re going to look at how Loxone can reduce the demand on such resources. For this, we’ll take into account these certain characteristics.

**Water Tank Level:**

The cistern has to be filled sufficiently, to avoid a pump running dry. The water tank, or cistern, is monitored by a Loxone Ultrasonic Sensor. This sensor has an integrated 0-10v transducer. Thus, you can define in which range you want to measure the water level (f.e. 0-400 cm).

**2x Water Pumps:**

The water from the cistern is carried via the “Water Pump” into the water pipes of the irrigation system. The pump can be activated manually via the Loxone App, a Loxone Remote or automated at a predefined time.

If there’s not enough water in the cistern, the “Cistern Pump” will then automatically refill the cistern from an available water source or simply deactivate. A “Cistern Pump” is only necessary if the water has to be pumped up from anywhere – so this is not needed if a water tank or water supply is readily available.

**Valve Distribution & Sprinklers:**

We have 9x pop-up sprinklers which are connected to a water pipe. At the water pipe, there’s a valve installed that is connected to the Loxone System and the pop-up sprinklers are activated by water pressure. The sprinklers are installed just beneath the ground level and as soon as the valve on the water pipe is open, the sprinkler pops up into operation. It would not be unusual for water pressure to not be enough to have all valves open concurrently, so you can implement a cascading operation of the valves via the Radio Buttons module.

**We have two modes:**

Mode 1: Irrigation of max. 60 minutes → each pop-up sprinkler is open for 8 minutes (one after another), automatic safety shutdown after 60 minutes.

Mode 2: Irrigation shortly before the match and in the half-time each pop-up sprinkler is open for 1 min (one after another). This avoids injuries of the football players and to allow a slick and fast passing. The mode has to be started manually.

The automation will depend on the amount of recent or forecast rain, and the water tank level:

So in our use case, we’ll want to know how much has it has rained the last 30 hours and how much is it going to rain in the next 35 hours. If the Weather Service tells the Miniserver that it rained less than 2 l/*m2*, the automatic watering takes place twice a week (Tuesday/Wednesday and Friday/Saturday). A virtual input enables setting an interval in the Loxone App, but consideration needs to be made regarding the water pressure. Hence, another option is to send a notification “Please, water the sports field” to allow for watering to only start if this notification is acknowledged manually.

In creating this Config, we have used our existing case study – the football field in Kollerschlag, Upper Austria.

Hardware:
- [Weather Service](https://shop.loxone.com/enuk/weather-service-10-year.html)
- Pop-Up Sprinkler (3rd party)
- Sprinkler Valve (3rd party)
- 24V AC Power Supply

If cistern exists:
- [Ultrasonic Sensor](https://shop.loxone.com/enuk/ultrasonic-sensor.html)
- Suction pump

### Configuration:

[

](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-1.png)[
![Automatic sports field watering - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-1.png)

[

](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-2.png)[
![Rain detection for sports field watering - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-2.png)

[

](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-3.png)[
![Smart irrigation for sports fields - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-3.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-3.png)

[

](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-4.png)[
![Watering a sports field - Loxone Config Screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-4.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-55-Watering-Sports-Field-4.png)

### Download the sample file:

### Watering of Sports Field

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-55-Watering-Sports-Field.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-55-Watering-Sports-Field.loxone)

## Why you and your customer should consider smart automation for watering a sports field.

The amount of water on the surface of a football field can considerably affect conditions of play for football. For this reason; sports clubs, managers and groundskeepers can use this to their advantage. An accurately-watered field will allow for slick and fast passing. The correct amount of water on any given turf will encourage the streamlined movement of the ball.

Hence, watering a sports field with intelligent automation from Loxone is a sensible investment for the sports club in order to optimally irrigate the playing field and to avoid over or under-watering the sports facility. Another advantage of an intelligent system is that it requires very little personnel expenses, which is very important nowadays.

Overall, the costs for the care of the football field can be greatly reduced by automating the irrigation with Loxone – in particular by reducing the water requirement, through the targeted and properly timed use of the water. And while it might not prevent Oscar-worthy performances from some players, precise irrigation improves conditions that encourage a safer playing environment and less injury due to poor conditions of the pitch.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)