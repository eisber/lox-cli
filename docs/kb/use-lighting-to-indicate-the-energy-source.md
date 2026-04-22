# Use lighting to indicate the energy source

Source: https://www.loxone.com/enen/kb/use-lighting-to-indicate-the-energy-source/

---

## Brief: Change the colour of an LED strip to indicate the energy source I’m using.

You want to know if the energy you are using is the energy you produced or the energy you purchased? No problem. With Loxone you can use lighting to indicate the energy source – encourage wise energy use and saving time. Who wouldn’t like to do something good both for the environment and their own wallet?

First, we need a light source that we can change to show if we are using the grid or our own energy source – for this, we’ll use LED strip. Then, if we use lighting to indicate the energy source we need an intuitive traffic-light approach such as green for when the self-produced energy is currently being used. Then change the LED Strip to orange if we’re using energy from the grid. With just a quick glance, you have the knowledge to decide, if you want, for example, to run the washing machine in the afternoon with the self-produced energy instead of in the morning when you have to pay for the energy.

Furthermore, current electricity consumption and electricity production will also be recorded. This data can be viewed in the Loxone App at any time.

So, if you are interested in a solution for intelligent energy use, then continue reading to see how we would achieve this using Loxone Hardware and Loxone Config software.

## Solution: How to use lighting to indicate the energy source.

In this example, the values for the energy purchased from the supplier and self-produced energy are read out by the Modbus energy meter. This is how we’ll know what colour to show when we use lighting to indicate the energy source. To ensure that the light turns green when using ‘free energy” and orange with purchased energy, the respective operating modes are assigned to the light scenes under “Automatic” on the Lighting Controller Function Block. If movement in room we set up this indicator in is detected, the relevant light colour is displayed.

If energy is consumed, the “Energy Consumption” mode switches on and the light colour changes. If energy is produced, the “Energy Production” mode switches on and the light changes to green. Both operating modes are reset when the sum of energy produced and energy consumed is below 0. This is done via the Adder block and the Less or Equal block. We recommend not adding this to the App, otherwise, the customer might change or delete these light scenes.

Hardware:
- [Miniserver](https://shop.loxone.com/enuk/miniserver-gen-1.html)
- [Modbus Energy Meter](https://shop.loxone.com/enuk/catalogsearch/result/?cat=0&q=Modbus+Electricity+Meter+()
- [Modbus Extension ](https://shop.loxone.com/enuk/modbus-extension.html)
- [RGBW Dimmer 24V ](https://shop.loxone.com/enuk/rgbw-24v-dimmer.html)
- [Led Strip RGBW](https://shop.loxone.com/enuk/led-strip-rgbw.html)

### Configuration:

[
![Use lighting to indicate the energy source - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-42-Intelligent-Energy-Use-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-42-Intelligent-Energy-Use-1.png)

### Download the sample file:

### Intelligent Energy Use

			[Config 14.02.06.16](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-42-Intelligent-Energy-Use.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/08/Use-Case-42-Intelligent-Energy-Use.loxone)

## Why you and your customer should consider coloured lighting that changes based on the energy source?

Knowing whether the energy you are currently using was produced by yourself or purchased is a definite advantage because you can save energy. This is very achievable with Loxone.

You can see by the colour of LED Strip if the currently required energy is the self-produced (green light) or the purchased energy (orange light). With this information, you can decide when you want to run devices with high consumption.

Another advantage is the possibility to record statistics of electricity consumption and electricity production. The data that you can gather will help you again to increase your energy-saving potential.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)