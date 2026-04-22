# Monitoring and managing all energy aspects of a building

Source: https://www.loxone.com/enen/kb/monitoring-and-managing-all-energy-aspects-of-a-building/

---

## Brief: Monitoring all energy aspects of a building.

Energy management starts with monitoring. It is a vital step in energy management but this alone doesn’t actually save energy. It’s still down to the user to take that data and use the energy in the most optimal way – with very conscious changes to their behaviour. It is what we do with that data that makes the difference… By accurately recording, analysing and managing energy consumption and production, intelligent buildings can cut costs and proportionately reduce their power usage – putting less onus on the user.

The assumption is that installing ‘green’ devices such as solar panels, battery storage, wallboxes, smart meters, etc will make a building energy efficient. However, as standalone devices, they are not inherently connected so they don’t know what the other ‘green’ devices are doing, or don’t know how much electricity is coming from the grid, or what the solar forecast is for the day ahead, etc. This causes complications with energy management especially when we get to what the granular components are doing.

If these devices could be connected with an intelligent system – effectively a building energy management system – then the information can be shared and decisions can be made to truly optimise the energy production and consumption…and that is where true energy management excels.

For example, installing a wallbox is one thing. Having that wallbox know when there is excess solar is a completely different thing. Now start to factor in that same building having an immersion heater, and battery storage, etc. Wouldn’t it be great if we could decide where the excess solar goes, and which device has priority at different times of the day… In a hypothetical example: if we’re generating 15kW power from the solar panels, but only using 10kW throughout the building, the logical next step would be to drop the excess 5kW into the car charger – except the wallbox is not connected to the energy management system and therefore isn’t visible to it, which means this process won’t be done automatically. This can also be the case for the immersion heater and the battery if these are connected with standalone 3rd party devices.

Sounds great. But what is the purpose of these devices being connected? Aren’t we still putting the onus on the user to change their behaviour to actually save energy? No. The beauty of using the right building energy management system is that all that logic is done for you – you get all the convenience without compromising on how you live in your own home or work in your own office.

## Solution: Using Loxone to monitor energy in a single app and manage it more intelligently.

Energy monitoring and management with Loxone is the solution. It is as overarching or as granular as it needs to be. Our system can monitor every component from the grid and the solar panels down to light fittings and immersion heaters. This takes the responsibility away from your customers and keeps their comfort and convenience at the heart of every installation: they won’t compromise this to become more energy efficient or save on electricity bills.

**Energy Flow Monitor Function Block**

The Energy Flow Monitor is responsible for the visualisation of the flow of electricity in a building. To do this, it depicts consumers (appliances, lighting fixtures, air conditioners, etc), producers (solar panels and wind turbines), storage units (batteries) and the grid on one user-friendly interface. These elements are all connected to different Meter Function Blocks that record and track their power generation and usage. The power consumption can be monitored in different ways in order to be displayed in the Loxone App – this can be through using the Power Supply & Backup, using Modbus energy meters, using CT Clamps, Smart Socket Airs, etc. If the individual power consumption cannot be monitored directly, but it is a fixed load, then there are additional metering Function Blocks that can be used so that you can still identify these in the app. The amazing thing about the Energy Flow Monitor is that once this is configured in Loxone Config, the visualisation is automatically made so your customer has a user-friendly view of their entire setup.

[
![EN How to energy Energy Flow Monitor and Energy Manager config screenshot 3 800x241](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/EN-How-to-energy-Energy-Flow-Monitor-and-Energy-Manager-config-screenshot-3.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/EN-How-to-energy-Energy-Flow-Monitor-and-Energy-Manager-config-screenshot-3.png)

**Energy Manager Function Block

**But…seeing all this information is only the first step. Bring in the Energy Manager for even more magic! The Energy Manager Function Block distributes excess energy in the most efficient way through automation and, consequently, the onus is no longer on the user to consciously change their behaviour. You can use the block to control up to 12 devices with different priorities. The Energy Manager prioritises devices based on their power requirements, starting with the device that has the lowest demand and therefore is the fastest to supply. The goal is to turn on the maximum number of devices when there is excess energy available. If the Grid Power input (Gpwr) in combination with Solar Power (Spwr) indicates available excess energy, the Energy Manager will automatically switch on as many loads as possible, starting with the highest priority. If the surplus is not enough to supply the power needed, the digital load is switched off. For the Energy Manager to operate effectively, real-time data on electricity production and consumption is required at all times.

[
![EN How to energy Energy Flow Monitor and Energy Manager config screenshot 2 800x278](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/EN-How-to-energy-Energy-Flow-Monitor-and-Energy-Manager-config-screenshot-2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/EN-How-to-energy-Energy-Flow-Monitor-and-Energy-Manager-config-screenshot-2.png)

As an example, let’s look at a building that has a wallbox, an immersion heater and an aircon. This is a common scenario for both commercial buildings and smart homes. Your goal will always be to proportionately control these appliances with the excess energy available. The energy manager will see that you are producing excess solar of 2kW and recognise that there is a car plugged in to the wallbox waiting for charge. As the Loxone Wallbox can charge at a rate of just 1.38kW, the Energy Manager will divert this excess to the wallbox. If there was no car plugged in, then it would automatically divert this to the next device on the list, such as an immersion heater, or battery storage, etc. Or, it can direct this surplus energy into pre-heating or pre-cooling a space to reach comfort temperate at the most affordable time. Essentially making sure you use every bit of self-produced energy as possible. Now that is real energy management!

### Hardware:
- Miniserver (can be [Miniserver](https://shop.loxone.com/enuk/miniserver.html), [Miniserver Compact](https://shop.loxone.com/enuk/miniserver-compact.html) or [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html))

### Configuration:

[
![EN How to energy Energy Flow Monitor and Energy Manager config screenshot 800x505](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/EN-How-to-energy-Energy-Flow-Monitor-and-Energy-Manager-config-screenshot.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/EN-How-to-energy-Energy-Flow-Monitor-and-Energy-Manager-config-screenshot.png)

### Download the sample Config file:

### Monitoring and managing all energy aspects of a building

			[14.4.9.25](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Energy-Flow-Monitor-and-Energy-Manager.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Energy-Flow-Monitor-and-Energy-Manager.loxone)

## Video:

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.