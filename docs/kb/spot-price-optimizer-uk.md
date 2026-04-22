# Spot Price Optimizer: 30 minutes (UK)

Source: https://www.loxone.com/enen/kb/spot-price-optimizer-uk/

---

## Table of Contents
- [Intro](#Intro)
- [Parameters](#Parameter)
- [Formula](#Formula)
- [Inputs](#Inputs) (Distribution Cost and Peak Time Premium)
- [Daylight Saving Time Adjustment](#Daylight)

(If you are looking for general information about the Spot Price Optimizer Function Block – including all parameters and properties, visit the dedicated documentation page [here](https://www.loxone.com/enen/kb/spot-price-optimizer/).)

## Intro

Learn how to set up the Spot Price Optimizer Function Block for use within the UK, utilising the GB 30-minute Spot Market data.

Currently, there is only one energy provider in the UK offering an energy tariff which enables you to take advantage of the spot market energy prices: Octopus Energy, with their Agile tariff.

With this in mind, we will be basing the setup of the Spot Price Optimizer using the Octopus Agile tariff.

We’ve prepared this walkthrough video showing you the setup documented below.

**Step 1. To get started, we need to add a Spot Price Optimizer function block to our Loxone Config file.**

[
![Config Spot Price Optimizer 30 minute UK C 800x450](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/Config-Spot-Price-Optimizer-30-minute-UK-C.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/Config-Spot-Price-Optimizer-30-minute-UK-C.png)

##

## Parameters

**Step 2. Adjust the following Parameters**
- Mode → ‘Spot market’
- Market Area → ‘GB 30-minutes (Epex, £/kWh)’

[
![SPO Parameters Spot Price Optimizer 30 minute UK C 1 800x450](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/SPO-Parameters-Spot-Price-Optimizer-30-minute-UK-C-1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/SPO-Parameters-Spot-Price-Optimizer-30-minute-UK-C-1.png)

![SPO Formula Spot Price Optimizer UK EPEX 800x79](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/SPO-Formula-Spot-Price-Optimizer-UK-EPEX.png)

## Formula

**Step 3. Enter the formula for calculating your energy price**

					The formula for the Octopus Agile tariff is **MIN(1;(I1*I2+IF(I4>=960;IF(I4<1140;I3;0);0))*1.05)**

![SPO Formula Spot Price Optimizer 30 minute UK C 1 800x450](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/SPO-Formula-Spot-Price-Optimizer-30-minute-UK-C-1.png)

![Loxone Loxone Spot Price Optimizer Epex Octopus Formula](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/SPO-Formula-Spot-Price-Optimizer-UK-Formula.png)

**What does each part of the formula do?**

**MIN(1;(I1*I2+IF(I4>=960;IF(I4<1140;I3;0);0))*1.05)**

**Price Cap** – This ensures the amount you pay per kWh is capped at £1/kWh (95p/kWh +5% VAT)

**Distribution Costs** – This is a multiplication factor based on your area

**Peak Time Premium **– Between 16:00/4pm (960 minutes past midnight) and 19:00/7pm (1140 minutes past midnight) your peak time premium is added to the unit cost

**VAT** – add 5% VAT to the result

					Note: You do not need to add the VAT. However, this ensures the result displayed in the Spot Price Optimizer is consistent with the Agile Portal dashboard: [https://agile.octopushome.net/dashboard](https://agile.octopushome.net/dashboard)
If you choose not to add the VAT, the formula should be adjusted to **MIN(0.95;(I1*I2+IF(I4>=960;IF(I4<1140;I3;0);0)))**

## General Purpose Inputs

**Step 4. Find your distribution cost and peak time premium for I2 and I3**

For this you will need to know your Distribution Network Operator (DNO) code, otherwise known as region code.

This can be found on the customer’s bill, or via the steps below:

Navigate to **https://api.octopus.energy/v1/industry/grid-supply-points/?postcode=**

After the = sign, you need to add the customers post code, e.g.[https://api.octopus.energy/v1/industry/grid-supply-points/?postcode=RG74RA](https://api.octopus.energy/v1/industry/grid-supply-points/?postcode=RG74RA)

This will return the customer’s region code, as shown in this image:

![Region Code Spot Price Optimizer 30 minute UK 1 800x193](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/Region-Code-Spot-Price-Optimizer-30-minute-UK-1.png)

Based on the result, confirm which region they are in:

A – Eastern England

B – East Midlands

C – London

D – Merseyside and Northern Wales

E – West Midlands

F – North Eastern England

G – North Western England

H – Southern England

J – South Eastern England

K – Southern Wales

L – South Western England

M – Yorkshire

N – Southern Scotland

P – Northern Scotland

Based on the region, refer to this Octopus’ “Agile pricing explained” blog, under the “What is my D and P value?” list to find the correct D and P value:

[https://octopus.energy/blog/agile-pricing-explained/](https://octopus.energy/blog/agile-pricing-explained/)

[
![DandP Value Spot Price Optimizer 30 minute UK 800x608](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/DandP-Value-Spot-Price-Optimizer-30-minute-UK.png)
](https://octopus.energy/blog/agile-pricing-explained/)

**Step 5. Input the D value (distribution cost) into the Input 2 parameter (I2), and the P value (peak time premium) into the Input 3 parameter (I3).**

					Note: It is important that you add a decimal place to the peak time premium, e.g. 12 should be inputted as 0.12 to ensure you are adding 12p/kWh, rather than £12/kWh.

![I2 I3 Spot Price Optimizer 30 minute UK C 1 800x450](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/I2-I3-Spot-Price-Optimizer-30-minute-UK-C-1.png)

![Loxone Loxone Spot Price Optimizer Epex Octopus Inputs](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/SPO-Formula-Spot-Price-Optimizer-UK-I2-I3.png)

**Step 6. Save in to the Miniserver**

View the Spot Price Optimizer within the Loxone App, and enjoy!

|  |  |  |
| --- | --- | --- |

					Note: As the Spot market data is based on CET, pricing data for 23:00 & 23:30 is released with the pricing data for the following day. You can expect to see data for the 23:00 – 23:30 and 23:30 – 00:00 periods between 17:00 and 21:00.
For further information about the Spot Price Optimizer Function Block – including a **programming example** for a heat to only run during the 4 cheapest hours – visit the dedicated documentation page [here](https://www.loxone.com/enen/kb/spot-price-optimizer/).

## Daylight Saving Time Adjustment

When the clocks move forward by one hour at 01:00 on the last Sunday in March (start of Daylight Saving Time), the unit rates for the current 24-hour period will have already been updated earlier that day. For example, in the case of Octopus Agile, these rates are typically updated between 16:00 and 19:00.

As a result, on the day of the time change, there will be a one-hour discrepancy in the Spot Price Optimizer within Loxone. For instance, if the Agile portal displays a price for 16:30, it may appear as 17:30 in the Loxone App. This behavior is expected due to the time shift and only affects this changeover day.

On the following day (Monday), the unit rates will appear to end at 22:30 instead of the usual 23:30. This is because the system is realigning with the adjusted time schedule.

From Monday evening onwards, when the new rates are collected (typically between 16:00 and 19:00), the time display will return to normal. For example, Tuesday’s rates will once again display correctly until 23:30.