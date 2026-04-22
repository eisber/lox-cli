# Smart lighting as an occupancy indicator

Source: https://www.loxone.com/enen/kb/smart-lighting-as-an-occupancy-indicator/

---

## Brief: Change the colour of lighting as an occupancy indicator.

It can be annoying when a room is occupied for a meeting and someone accidentally opens the door just to see if someone is in there. Similarly, in a large office building where the toilet can be quite a distance away, it is not time-effective to simply walk up to the door to see if it is occupied and return to your seat if it is, only to make the same trip moments later to see if the room is no longer occupied – repeating the back-and-forth until so. Another instance where indicating if someone can enter a room or not is commercial environments where only one person is admissible at a time. So, can you use smart lighting as an occupancy indicator?

We can find several examples where we have to indicate from the outside if a room is vacant or engaged.

An easy way to do this is with the lights. The universal sign for green and red; where green means available, and red means it is currently occupied.

## Solution: Using red or green lighting to show if a room is busy or not.

If the room is empty, the light outdoors will be green and whenever the sensor detects someone, it will turn into red, indicating that the room is already occupied. This solution adopts a simple traffic light idea to show if a room is busy or not, using existing smart lighting as an occupancy indicator. We can adapt the timing from the last presence detection to let the system knows when to switch the colour from red to green again.

Whenever you have an existing Loxone installation, with Motion sensors and lighting control, this can be added with just a little additional configuration. If you do not have a Miniserver, it can be also easily done via our wireless technology. The Miniserver Go has already Air technology in it, and you may need only the Motion Sensors Air and the Nano IO Air to control the lighting. The configuration within Loxone Config is the same whether you’re using Air or Tree technology.

Tip: You have to be careful about where you install the Motion Sensors. This is really important as it should detect a person in any part of the room. According to the size and shape of the room, you may need more than one Motion Sensor.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Motion Sensor](https://shop.loxone.com/enuk/motion-sensor.html)
- [LED Spot RGBW](https://shop.loxone.com/enuk/led-spot-rgbw-tree-white-uk.html) or [LED Ceiling Light](https://shop.loxone.com/enuk/led-ceiling-light-rgbw.html) (or a suitable 3rd party RGBW light)

### Configuration:

[

](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-26-Customise-Lights-Occupancy-01.png)[
![lighting as an occupancy indicator config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-26-Customise-Lights-Occupancy-01.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-26-Customise-Lights-Occupancy-01.png)

### Download the sample file:

### Customise Lights based on Occupancy

			[Config 10.3.11.27](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-26-Customise-Lights-Occupancy.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Use-Case-26-Customise-Lights-Occupancy.loxone)

## Why you and your customer should consider using colour lighting to show occupancy?

That is a very easy and useful feature that you can offer to a customer if you think that the project could benefit from using the existing smart lighting as an occupancy indicator. The programming can be done in a few minutes with the hardware that a basic project already likely has installed. That can be useful especially in offices for meetings rooms or single person work cubicles, for example.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)