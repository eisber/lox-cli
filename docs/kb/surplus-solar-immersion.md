# Using an Immersion Heater as a Solar Diverter

Source: https://www.loxone.com/enen/kb/surplus-solar-immersion/

---

## Brief: How to use excess solar and proportionately control your immersion heater with Loxone.

Solar panels, also known as photovoltaic (PV) panels, convert sunlight into electricity, which can be used to power appliances, lighting, zoned heating and more. By harnessing renewable energy from the sun, homeowners can significantly reduce their electricity bills, decrease their carbon footprint, and contribute to a more sustainable future.

In this scenario we’ll use some hypotheticals and example electricity unit prices. Let’s say your customer’s house needs 10kW to run all needed devices as a practical example. If the solar panels installed on the property cannot meet this requirement and only produce 8kW, the rest will be bought from the grid for £0.34/kWh. If the solar cells manage to produce 12kW, one option is to sell the excess 2kW back to the grid (in other words, export it) for, let’s say, £0.05/kWh. The other option is to use this excess energy across the appliances and installations that require electricity to work. This will maximise energy self-consumption and save on costs.

One area to target with this excess power is the hot water system. Normally a boiler heats up the water, and if it fails or a boost is needed, the immersion heater is switched on. In our scenario, a 170L cylinder switching the immersion on pulls 6kW of power immediately, regardless of how much is actually required. In our previous example, even if the solar system has produced 12kW of energy, we will still need to purchase 4kW to provide for this.

Introducing a Solid State Relay (SSR) can help with this, as it gives control by turning the immersion heater on proportionally to use only the amount of power relevant to our surplus and therefore minimise additional costs.

The Solar Immersion Diverters currently available on the market can do this, but they are standalone solutions focusing the flow of surplus energy to the immersion heater only.

## Solution: Using the hot water tank as energy storage with surplus solar.

Loxone brings logic that tells the SSR how much to use from the produced surplus energy. One RGBW Dimmer channel controlled by the Miniserver informs the SSR how much of the surplus energy is available to send to the immersion.

With Loxone, it’s also possible to check the water temperature with a 1-wire probe or 0-10V probe to make sure it reaches a desired temperature as well as working as overheat protection – although the hot water cylinder will likely have its own failsafe for the latter. Where the Loxone automation system excels is redirecting the additional energy elsewhere once this has been done.

The Loxone building automation system provides holistic energy management instead of a single application solution. It monitors the power production and consumption throughout the whole building, and controls this expanded system by making a decision on whether or not to use the surplus energy to heat the immersion heater, redirect it into a car charger or start appliances like dishwashers.

If we use the surplus power instead of selling it, the following costs and returns can be associated as a rough estimate when using our hypothetical scenario and pricing from above:

There is a 25p difference between buying (£0.30/kWh) and selling (£0.05/kWh) energy. If we use 2kWh surplus a day on average as per our example above, we can divert it and save 50p. Multiplying that with 365 days in a year means we save £182.5 by utilising this surplus. Naturally, not every day will be sunny enough for surplus, but on the flipside we aren’t taking into account the saving of not having to use energy from the grid to provide the equivalent heating.

If we assume this integration is for an existing Loxone installation, the cost associated with the equipment needed to integrate the immersion heater (RGBW dimmer + SSR + Temp Probe + Config) adds up to an estimated £175.

This means we can see a return on investment as soon as 11 months.

Due to this all being controlled from the heart of the installation – the Miniserver – all other automated functions in the building such as the zoned heating and the smart blinds work through a single smart home system instead of multiple expensive single source solutions, making the Loxone system even more cost effective in the long term. Features for lighting, heating, multiroom audio and more are intelligently controlled from the Miniserver to improve comfort, security and energy efficiency in any smart home, building or special application.

### Hardware:
- Miniserver (can be [Miniserver](https://shop.loxone.com/enuk/miniserver.html), [Miniserver Compact](https://shop.loxone.com/enuk/miniserver-compact.html) or [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html))
- RGBW Dimmer (can be [RGBW 24V Dimmer Tree](https://shop.loxone.com/enuk/rgbw-24v-dimmer-tree.html), [RGBW 24V Compact Dimmer Air](https://shop.loxone.com/enuk/rgbw-24v-compact-dimmer-air.html), [RGBW 24V Compact Dimmer Tree](https://shop.loxone.com/enuk/rgbw-24v-compact-dimmer-tree.html), [RGBW 24V Dimmer Air](https://shop.loxone.com/enuk/rgbw-24v-dimmer-air.html))
- SSR – recommendation: PWM Controlled – [Finder SSR 24V PWM Controlled](https://uk.rs-online.com/web/p/solid-state-relays/8002767?cm_mmc=UK-PLA-DS3A-_-google-_-CSS_UK_EN_Fallback-_-All+Products-_-8002767&matchtype=&pla-293946777986&cq_src=google_ads&cq_cmp=9815220214&cq_term=&cq_plac=&cq_net=g&cq_plt=gp&gclid=Cj0KCQjwwJuVBhCAARIsAOPwGATHLKH3v6uq1qi1bgfGJg-BHc6--zucgb5pxQ0_5H8_cEjxrTy_NjcaAuuvEALw_wcB&gclsrc=aw.ds)
- Temperature Probe for the Water Tank – recommendation: 0-10V – PT 100 + Converter or [Temperature Probe 1-Wire](https://shop.loxone.com/enuk/1-wire-temperature-probe.html) + [1-Wire Extension](https://shop.loxone.com/enuk/1-wire-extension.html)
- In line or CT clamp Energy Meter – Modbus ([3-phase](https://shop.loxone.com/enuk/modbus-meter-3-phase.html) or [1-phase](https://shop.loxone.com/enuk/modbus-meter-1-phase.html)), 0-10V, 0-5V, M-Bus, Wi-Fi etc. However it is presumed such an energy meter would already be in place for the solar installation.

###

### Configuration:

[
![PH EN UseCase Immersion Config 800x355](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/PH-EN-UseCase-Immersion-Config.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/PH-EN-UseCase-Immersion-Config.jpg)

### Download the sample Config file:

### Using an Immersion Heater as a Solar Diverter

			[14.4.9.25](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/Using-an-Immersion-Heater-as-a-Solar-Diverter.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/Using-an-Immersion-Heater-as-a-Solar-Diverter.loxone)

For this specific configuration, go to the ‘Energy’ page within the sample file.

##

## Video:

					Local regulations and standards need to be observed. The contents of this page make certain installation and electricity cost assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.