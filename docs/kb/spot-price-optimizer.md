# Spot Price Optimizer

Source: https://www.loxone.com/enen/kb/spot-price-optimizer/

---

> **ℹ️ Note:** Loxone provides direct integration with leading European spot markets, including EPEX, OTE, and ESIOS. This enables a reliable, efficient, and seamless setup without the need for additional configuration.

The Spot Price Optimizer function block is used to activate a load in hours when the energy price is the lowest.

After a trigger the block will pick a number of hours, defined by 'Demand', within the next hours, defined by 'Period' and activate the output 'O'.

Alternatively the Outputs Very High, High, Low and Very Low can be used to activate/deactivate a load. For this usecase you can for example connect the day pulse to the trigger.

The recommended configuration for this block is in Spot Market mode, which provides the most efficient and automated setup.

Alternatively, the block can operate in either Relative mode or Absolute mode, where each hour is represented by a dedicated input (Relative: +0 - +23, Absolute: 00:00 - 23:00)

If the data on the Inputs +0 to +23 is provided by a virtual http input, the price is only used if it was updated within the current hour, therefore preventing the use of outdated prices.

The Spot Price Optimizer can be used as a source for the export and import price in the [Energy Flow Monitor](https://www.loxone.com/help/energy-flow-monitor). Both function blocks must be on the same Miniserver.

## Table of Contents
- [Inputs](#Input)
- [Outputs](#Output)
- [Parameters](#Parameter)
- [Properties](#Property)
- [Spot Market Mode (Recommended Setup)](#spotmarket)
- [Relative/Absolute Mode](#relabsmode)
- [Price Calculation](#pricecalc)
- [Calculation of outputs](#calcoutputs)

---

## Inputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| Off | Off | Pulse: Outputs are reset / switched off. On: Block is locked. Dominating input. | 0/1 |
| Tr | Trigger | Starts the automatic. The output O will be activated as often as defined by 'demand' within 'period'. | 0/1 |
| +0 to +23 | Relative Mode: Price in the hour now +0 to +23 | Price forecast for the hour now + offset. This inputs are not available when using the Spot market mode. | ∞ |
| Absolute Mode: Price in the hour 00:00 to 23:00 | Price forecast for a specific hour of the current day. This inputs are not available when using the Spot market mode. | ∞ |

---

## Outputs

| Abbreviation | Summary | Description | Value Range |
| --- | --- | --- | --- |
| O | Active Output | Output will be activated at the hours with the lowest price after a trigger. | 0/1 |
| Cv | Current Price | ∞ |
| vHigh | Very High | The current price is Very High compared to the other hours or above the Max parameter. Limits will be recalculated after a Trigger based on the forecast values available at that moment. | 0/1 |
| High | High | The current price is High compared to the other hours. Limits will be recalculated after a Trigger based on the forecast values available at that moment. | 0/1 |
| Low | Low | The current price is Low compared to the other hours. Limits will be recalculated after a Trigger based on the forecast values available at that moment. | 0/1 |
| vLow | Very Low | The current price is Very Low compared to the other hours. Limits will be recalculated after a Trigger based on the forecast values available at that moment. | 0/1 |
| Max | Highest Price | The maximum price will be recalculated after a Trigger based on the forecast values available at that moment. | ∞ |
| Min | Lowest Price | The minimum price will be recalculated after a Trigger based on the forecast values available at that moment. | ∞ |
| Avg | Average Price | The average price will be recalculated after a Trigger based on the forecast values available at that moment. | ∞ |
| Nv | Next Price | The price of the next period. | ∞ |
| API | API Connector | Intelligent API based connector. API Commands | - |

---

## Parameters

| Abbreviation | Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- | --- |
| Demand | Specifies the total active duration of output O within the period after a Trigger. | h | 0...∞ | 4 |
| Period | Time period after a trigger from which the block picks the hours with the lowest price where the output O will be activated. | h | 0...∞ | 24 |
| Minimum Runtime | Defines the minimum continuous duration for which the output must remain active once turned on. This duration is achieved by selecting the cheapest consecutive time intervals that meet or exceed the specified runtime. The output may run for a longer period if required or beneficial, but it will never operate for less than the defined minimum. If set to 0 the output will run according to the set market interval duration. | h | 0...12 | 0 |
| Max | Fixed very high price | If the current price is above this value, the very High output will always activate. | Currency | ∞ | 1 |
| I2 | Variable Input 2 | Value which can be used in the formula with I2. | - | ∞ | 0 |
| I3 | Variable Input 3 | Value which can be used in the formula with I3. | - | ∞ | 0 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Price Calculation | Use a formula to calculate the actual price you are paying. Ask your energy supplier for the formula. I1 = price from the inputs or the spot market price (without tax). I2 = I2 parameter. I3 = I3 parameter. I4 = Minutes since midnight for which the price is calculated. | - |
| Mode | Relative: Inputs provide the price relative to the current hour. Absolute: Inputs provide the price for each hour of the day (00:00 to 23:00). Spot market: Data is obtained from european energy spot markets, Inputs for hourly prices are hidden. | - |
| Market Area | Spot market area for which to fetch the prices. | - |
| Market Interval | Interval at which Spot Market prices will be retrieved. | - |

---

## Spot Market Mode (Recommended Setup)

In Spot Market Mode, energy prices for Europe are obtained directly from leading energy spot markets such as EPEX SPOT and others. This provides a fully automated, reliable, and user-friendly solution with no manual inputs or virtual inputs required. We strongly recommend this setup for the easiest and most accurate operation.

To do so:
1. Select the mode Spot Market in the Spot Price Optimizer block.
2. Choose your market area.

The Miniserver must be [registered](https://portal.loxone.com/products) for this.

Once configured, energy price data is automatically retrieved daily between 13:00 and 14:00 UTC, for the following day. The prices are provided excluding taxes, and the hourly price inputs are hidden, as they are handled internally and automatically.

> **ℹ️ Note:** Due to legal restrictions imposed by the spot market data providers, we cannot provide direct access to the raw data. However, all relevant statistics remain available in the user interface.

**Programming Example:**

In this example, a trigger pulse is used to determine the 4 cheapest hours of the next 6 hours and the heat pump is switched on during these hours.
If the energy price is above the calculated or set very high value, the heat pump is locked.

![spo epex](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/spo_epex.png)

More programming examples can be found on our [Use Cases](https://www.loxone.com/enen/use-cases/) page.

---

## Relative/Absolute Mode

If the Spot Market option is unavailable, it is possible to use either Relative mode (inputs define prices relative to the current hour), or Absolute mode (inputs provide prices for each hour of the day).

The hourly energy prices are provided to the function block, e.g. via Virtual HTTP Input Commands.

You can import suitable [template](https://www.loxone.com/help/templates) from the [Loxone Library](https://library.loxone.com/).

**Programming Example:**

In this example, a trigger pulse is used to determine the 4 cheapest hours of the next 12 hours and the heat pump is switched on during these hours.
If the energy price is above the calculated or set very high value, the heat pump is locked.

![spo properties](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/spo_properties.png)

![spo relative](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/spo_relative.png)

---

## Price Calculation

In addition to the spot price, additional costs and taxes are charged by the electricity provider and grid operator.
Therefore, a [formula](https://www.loxone.com/help/Formula/) for calculating the final price can be defined in the properties window.
Check a recent electricity bill to find out the various costs per kWh or ask the energy supplier. Then create a formula based on this data.

In the example, a formula was formed using prices from the provider aWATTar, adding 3% and a fixed price of 1.5ct per hour.
Additionally, grid costs of 6.7ct were calculated, along with 20% VAT.

![spo pricecalc](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/spo_pricecalc.png)

> **ℹ️ Note:** When using formulas with time boundaries (I4), >= should be used insteasd of > for the lower limit of each interval.

---

## Calculation of outputs

The range between the lowest and highest price is divided into 4 equal parts, according to which the outputs are switched.

Diff = (Max - Min) / 4
Limit1 = Min + Diff
Limit2 = Limit1 + Diff
Limit3 = Limit2 + Diff

![spo calc](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/spo_calc.png)

The outputs of the Spot Price Optimizer are recalculated after each trigger. These calculations are based on the available future data at the time of the trigger. The user interface calculates the colors for the current day and categorizes them in low, high, etc. However, **please note that the outputs of the Spot Price Optimizer in Loxone Config may not always match the colors displayed in the user interface**. This difference can occur due to the timing and scope of data used for recalculations.