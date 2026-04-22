# Automated Intelligent Legionella Prevention

Source: https://www.loxone.com/enen/kb/automated-intelligent-legionella-prevention/

---

## Brief: I need a solution that automates Legionella prevention.

Legionella is a dangerous form of bacteria that occur in water that multiplies at a water temperature of 20°C to 45°C. Coming into close contact with Legionella can lead to Legionnaire’s disease – a deadly form of pneumonia.

Although anyone can be infected with the disease, the most vulnerable groups are those who have a history of smoking, chronic lung disease, poor immune function and the elderly.

Hot and cold water tanks in buildings are potential breeding grounds for the bacteria. The water stagnates and is stored within the temperature range that allows the bacteria to thrive. There is a greater risk of Legionella in buildings with long pipes and rarely used outlets, such as schools and restaurants.

If the hot water storage is kept above 60°C, there is no danger, as the bacteria are killed at this temperature. The flow and return flow temperatures of the hot water storage tanks should be at least 60°C in the flow and 50°C in the return every month. Rarely used outlets should be flushed weekly to clean the pipes.

With Loxone many of these processes can be monitored and automated – giving you automatic Legionella prevention.

## Solution: Using Loxone for Legionella prevention.

To monitor and ensure that the hot water is kept above 60°C, temperature sensors must be installed in the storage tank. If the temperature falls below a defined threshold value, a notification is sent to the relevant person.

The process can also be automated to ensure that the tank reaches the Legionella flushing temperature at least once per defined number of days. A maintenance counter is used to achieve this. As long as the water in the tank remains below the temperature we require, the counter counts backwards. When the counter reaches 0, the tank should be brought back to temperature using a heat source.

To monitor the flow and return temperatures of the hot water tank, we need a temperature probe for each pipe. The measurement is triggered a certain time after the main valve has been opened. If the flow is less than 60°C or the return is less than 50°C, a notification is sent.

Finally, flow meters can be attached to each outlet to check if they have been run weekly. If the weekly flow was insufficient, a notification is sent again to the responsible person.

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Temperature Probe 1-Wire](https://shop.loxone.com/enuk/1-wire-temperature-probe.html)
- Flow Detector

### Configuration:

### Download the sample file:

### Automatic Legionella Prevention

			[Config 14.4.9.25](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-2-Automatic-Legionella-Prevention.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2023/11/Use-Case-2-Automatic-Legionella-Prevention.loxone)

## Why you and your customer should consider Loxone for automated legionella prevention?

Legionella prevention is very important in every building with water storage. In larger buildings, the time required for building management is enormous, as the water outlets have to be maintained manually every week.

With automatic monitoring of the water outlets, the time required can be drastically reduced. At the same time, statistics can be provided to show that these outlets have been used.

By monitoring the hot water temperatures in the storage tanks, it can be ensured that people in the building are protected from the harmful effects of Legionella.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)