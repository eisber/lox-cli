# Using excess solar with the Wallbox and charging EVs at a cheaper rate

Source: https://www.loxone.com/enen/kb/excess-solar-wallbox/

---

## Brief: Using excess solar with the Wallbox and charging EVs at a cheaper rate.

Electric vehicles are swiftly taking the lead of the global shift in transportation. Driven by concerns over climate change, technological advancements, and supportive policies, the adoption of EVs is accelerating at an unprecedented rate. Customers are actively researching and purchasing eco-friendly alternatives to contribute to the green movement for a sustainable future. With fuel prices also on the rise, many have swapped to electric cars in hopes of saving on travelling costs in the long term.

That said, the cost of electricity is still a great concern. Energy costs on the spot market are usually at their highest between 4-7pm. Most people get home from work around this time and start with the evening’s activities, such as plugging their electric vehicle in to charge, which makes this process very expensive. Avoiding this time of day and using power when the wholesale price of the spot market is lower can lead to significant savings that will add up day after day. This low-cost window is normally between the hours of 2-5am – an inconvenient time to actually plug your car in.

Range anxiety is a big factor for many EV drivers. It is advised to keep your car battery charged between 20-80%, however a lot EV drivers may wish to always make sure that their car is topped up every day when they get back from home. Considering the daily commute average in the UK is 18-25 miles, it’s not likely that plugging in every day is actually needed.

In one scenario, drivers that do this might only be charging their car for an hour or two each day which probably starts when they get home at 6pm, meaning they probably pay a higher rate per unit of electricity as this is peak time. In another scenario, those with more range or those less anxious might only top up on a Friday night as they drive more on the weekend than their short commute to work during the week, and so they want to fill the battery. In the second scenario, getting home and plugging in at 6pm still presents the same instance of likely paying the higher rate per unit for 5-8 hours.

With all this in mind, let’s introduce a third hypothetical calculation. The average electric car battery on the market is around 40kWh. Let’s say 3 trips will use up 75% of this capacity, leaving approximately 8kWh, and will require charging. If the goal is to maintain 80% (32kWh) of 40kWh, then the charge will use 24kWh worth of energy to get us back to this goal. This from the grid will cost £2.40 a day if the energy is pulled at a higher rate during the day (£0.30/kWh), such as from 6pm after work. Maintaining this every 3 days will cost £7.20/week. Choosing a different time of day, or in this case, the night time to start the actual charging will reduce this. If your customer has solar panels, or is a commercial premises, we can also look at the benefit of using surplus solar to mitigate the cost of recharging even more.

[
![PH EN UseCase Wallbox App 394x800](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/PH-EN-UseCase-Wallbox-App.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/PH-EN-UseCase-Wallbox-App.png) [
![PH EN UseCase Wallbox App 1 408x800](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/PH-EN-UseCase-Wallbox-App-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/PH-EN-UseCase-Wallbox-App-1.png)[
![PH EN UseCase Wallbox App 2 396x800](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/PH-EN-UseCase-Wallbox-App-2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/PH-EN-UseCase-Wallbox-App-2.png)

*Click to enlarge images*

## Solution: When exporting surplus solar power, divert excess to the Wallbox or use Loxone to automate the charging using the cheapest tariffs at night.

The Loxone Wallbox works in harmony with other features within the Loxone installation such as the solar PV system to maximise reusable energy consumption. The Loxone Wallbox offers variable charging rates (as low as 1.38kW), so a threshold for solar surplus can be set (for example over 1.4kW) that will trigger the system to activate the charging of the Wallbox from the generated solar energy. The Loxone building management system will automatically divert this directly into the car, providing a sustainable and no-cost alternative to exporting electricity back to the grid, for example, where the rate can be as low as £0.05. If the exported power continues to increase every 60 seconds, the Energy Manager will tell the Wallbox to increase the charging rate to maximise self-consumption. This free method minimises costs by maximising self-consumption.

Yes, charging an EV with surplus solar is great, but there are times when your customer might need to charge at maximum power, or there hasn’t been enough solar power (let alone surplus) so with Loxone there is always the option to adjust the charging profile – that flexibility is there for your customers.

If your customer doesn’t have solar, or they only plug in during the evening when they aren’t likely to have a surplus, then the Loxone Wallbox can automatically start charging at the most convenient time based on the energy rates on the spot price market (if they have such setup with their electricity provider).  Alternatively, if they have an Economy 7 tariff, you will be able to take advantage of a lower rate over the 7-hour period at night.

If the selected tariff is Economy 7, a schedule defining the 7-hour period will be used to enable charging during these times. This offers the ultimate convenience meaning that EV owners can plug their car in when they get home at 6pm and not need to remember to open an app to trigger the charging later that evening or as they go to bed in an attempt to only use the cheaper tariffs… Loxone will automatically trigger this at the cheapest time for your customer.

### Hardware:
- Miniserver (can be [Miniserver](https://shop.loxone.com/enuk/miniserver.html), [Miniserver Compact](https://shop.loxone.com/enuk/miniserver-compact.html) or [Miniserver Go](https://shop.loxone.com/enuk/miniserver-go.html))
Wallbox (can be 7.4kW [Tree](https://shop.loxone.com/enuk/wallbox-tree.html)/[Air](https://shop.loxone.com/enuk/wallbox-air.html) or 11kW [Tree](https://shop.loxone.com/enuk/wallbox-tree-16a.html)/[Air](https://shop.loxone.com/enuk/wallbox-air-16a.html))
- Included electricity meter

###

### Configuration:

[
![How to use excess solar with the Wallbox and charge EVs at a cheaper rate 800x289](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/How-to-use-excess-solar-with-the-Wallbox-and-charge-EVs-at-a-cheaper-rate.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/How-to-use-excess-solar-with-the-Wallbox-and-charge-EVs-at-a-cheaper-rate.png)

*Click to enlarge images*

### Download the sample Config file:

### Using excess solar with the Wallbox and charging EVs at a cheaper rate

			[14.4.9.25](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/Using-excess-solar-with-the-Wallbox-and-charging-EVs-at-a-cheaper-rate.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/10/Using-excess-solar-with-the-Wallbox-and-charging-EVs-at-a-cheaper-rate.loxone)

For this specific configuration, go to the ‘Energy’ page within the sample file.

##

## Video:

					Local regulations and standards need to be observed. The contents of this page make certain installation and electricity cost assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.