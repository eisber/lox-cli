# Automating zoned heating to save energy and improve comfort

Source: https://www.loxone.com/enen/kb/zoned-heating-comfort/

---

## Brief: Optimising a central heating installation to have individual room temperature control.

Conventional installations have one thermostat that operates the entire heating system from a single point of temperature control. This is usually in the hallway next to the front door – the coldest area in the house. Temperature here changes very rapidly too as the door is frequently opened by people coming and going. The conventional thermostat goes back to the boiler directly and switches it on once the room is below comfort temperature. This creates a constant cycle of on and off, and will require the residents to manually adjust the amount of heat in each room by adjusting each and every radiator valve themselves to prevent some rooms getting too hot.

This also means that switching the boiler on or off is only dependent on temperature, and only in one room at that. A conventional thermostat won’t be able to factor in other elements that smart home and building automation systems can see, like the passive heat from the sun or presence. The system will try to reach the same temperature across the home in each room, whether it’s an empty spare room, a bathroom that only needs to be warm first thing in the morning or in the evening, or an at-home office only used during weekdays.

Radiators are manually adjustable to help with individual room control, but most people won’t remember or want to put the effort into turning the valve when the central thermostat is readily available. If they do turn the valve, it will most likely never be changed back for the sake of convenience. For example, turning the bedroom radiator to 5 in December and leaving it there, then turning the heating back on during a cold night in March will overheat the room and cause both a loss of comfort and an unnecessary expense on energy. Or likewise, on days your client might be working from home, turning the valve higher during the day and then the evening when you’re not using the room forgetting to turn it down.

A conventional thermostat or individual radiator valve will not stop or lower if a window is left open in a particular room, which will waste energy and makes it impossible to maintain a comfortable climate in the room.

In addition, conventional thermostats can’t at the least consider the temperature downstairs, upstairs and outside before making a decision. This can create several issues because the heating will remain centralised from a single source and will try to adjust to the same comfort temperature in every area. This can result in, for example, overheating the upstairs rooms as they are furthest away from the hall on a sunny but cold day – especially as heat travels up and adds to the already too high temperature, and the blinds are left open to let the sunshine in.

Lastly, a conventional thermostat with a scheduling feature only gives the user the option of when to start heating. The issue this presents is that – depending on the heat source, the size of the room, the size of the house, etc – it doesn’t give an idea of when the desired temperature will actually be reached. For example, if the household wakes up at 7am, and you have people having breakfast in the kitchen and using the bathrooms, you want those rooms to already be at temperature so that everyone is comfortable at this time. In reality, people schedule it to come on just before they wake up, which means the heating probably only reaches comfortable temperature as everyone’s leaving the house at 8am.

The alternative to the above is to leave your heating system running overnight, with some believing this is more energy efficient, so that in the morning you wake up to the right temperature but you have wasted energy overnight.

## Solution: Loxone smart heating uses individual room temperature measurements and considers room size for zoned control through intelligent automation.

The Loxone system’s goal is to keep as close to comfort temperature as possible through intelligent learning. The Loxone smart heating system optimises indoor climate control by utilising room temperature data and dimensions. The system monitors how long it takes to increase a room’s temperature by 1°C using the Intelligent Room Controller for a full cycle. This is essential information for smart heating control. By learning this, the Loxone system will be able to schedule the heating and allow for proper cooldown or heat up time to achieve the comfort temperature in the most efficient way – by, for example, turning the heating on at 5pm so the house is at the perfect temperature by 6pm when everyone comes home.

Loxone can track each room’s temperature, for example via the in-built sensors in each Touch switch. These provide precise readings throughout the day to enable the efficient control of the climate in every area of the building. The advantage of taking the temperature reading by the Touch switch in each room is to know what the difference is between current and target temperature for each room so that they can be variably controlled to be within as close as possible to the comfort temperature.

The control of indoor temperature is further supported by the Loxone Valve Actuators. Unlike conventional on/off solutions, the Valve Actuators are adjustable between 0-100%. This way, the heating will not consume too much energy to make a small adjustment of 1-2°C or put strain on the source by short cycling. The Loxone system will further ease this process by intelligently learning your customers’ behaviours and preferences. For example, if the comfort temperature is set at 22.5°C, and the current temperature is 21°C, the Loxone system will instruct the Valve Actuator to regulate itself specific to that room.

Smart heating by Loxone also takes room size into account – a factor often overlooked in standard central heating systems. The Switch On Threshold, set at 30% as default, will signal the boiler or air heat pump to switch on when the room is too cold based on its current temperature and size. For example, if 2 large rooms out of 4 have dropped below comfort temperature, the heating demand will be more than 30% based on their collective size. The Loxone system will automatically adjust the Valve Actuators based on these parameters and turn the boiler on. In another example, 3 smaller rooms can also trigger the Switch On Threshold as their demand based on their collective size can add up to more than 30%.

The calculations including all variables can be summarised via the below formulas:

The [Intelligent Room Controller](https://www.loxone.com/enen/kb/intelligent-room-controller/) calculates what proportion the valve needs to open to based on the following: Target θ [target temperature] – Current θ [current temperature] = Delta θ [difference temperature]

The [Climate Controller](https://www.loxone.com/enen/kb/climate-controller/) then takes these values as well as the room size into consideration when determining the overall demand for heat in the building.

[
![PH EN Use Case Heat Pumps App3 644x800](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-Use-Case-Heat-Pumps-App3.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-Use-Case-Heat-Pumps-App3.png)

If your customer has an air source heat pump, we can further optimise that based on the information given from the Loxone system. For example, we can set the target flow temperatures of the hot water and the target temperature of the buffer tank based on the demand of the system and outside weather temperature to make the pump run more efficiently. Essentially, it is being able to read all of these data points (temperature and size of each room, the outside temperature, the flow temperature) that allows a system like Loxone to truly manage a zoned heating system in the most energy-efficient way. It is the power of the software and the calculations that are happening behind the scenes that deliver the most optimal solution for zoned heating control.

[
![PH EN Use Case Heat Pumps App2 683x800](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-Use-Case-Heat-Pumps-App2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-Use-Case-Heat-Pumps-App2.png)

[
![PH EN Use Case Heat Pumps App 1 800x324](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-Use-Case-Heat-Pumps-App-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-Use-Case-Heat-Pumps-App-1.png)

### Hardware:
- Miniserver (can be [Miniserver](https://shop.loxone.com/enuk/miniserver.html), [Miniserver Compact](https://shop.loxone.com/enuk/miniserver-compact.html) or [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html))
- Valve Actuator [Air](https://shop.loxone.com/enuk/valve-actuator-air.html) / [Tree](https://shop.loxone.com/enuk/valve-actuator-tree.html)
- [Nano IO Air](https://shop.loxone.com/enuk/nano-io-air.html) (if not using a relay on the Miniserver for the heat source)
- [Touch switches](https://shop.loxone.com/enuk/catalogsearch/result/?q=Touch+switch) [/ Room Comfort Sensors](https://shop.loxone.com/enuk/catalogsearch/result/?q=Room+Comfort+Sensor)
- [Modbus Extension](https://shop.loxone.com/enuk/modbus-extension.html) (if integrating a compatible heat pump)

### Configuration:

[
![PH EN Use Case Heat Pumps Config 800x614](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-Use-Case-Heat-Pumps-Config.jpg)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/PH-EN-Use-Case-Heat-Pumps-Config.jpg)

### Download the sample Config file:

### Automating zoned heating to save energy and improve comfort

			[14.4.9.25](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Automating-zoned-heating-to-save-energy-and-improve-comfort.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Automating-zoned-heating-to-save-energy-and-improve-comfort.loxone)

## Video:

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.