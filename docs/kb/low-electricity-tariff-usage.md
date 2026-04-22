# Low electricity tariff usage

Source: https://www.loxone.com/enen/kb/low-electricity-tariff-usage/

---

## Brief: Reduce energy bills with low electricity tariff usage.

It is not uncommon these days for energy providers to offer low electricity tariff rates if devices or appliances are used at certain times where demand on the grid is low, usually late at night. But most devices can’t automatically benefit from low electricity tariff usage because they simply don’t know when these apply. With a bit of help from Loxone, everybody can use devices like a washing machine at a time when electricity tariffs are low – ultimately reducing grid demand and saving on energy bills.

With the help of a Loxone Miniserver, you can get info and even automatically switch on / off chosen devices. You can even charge your electric car for the best price without thinking about it.

In this Use Case, we’ll look at how you can implement tariff-dependent electricity use for your customers.

## Solution: How to set up appliances to take advantage of low electricity tariffs.

Low tariffs are usually signalized by a digital output (relay) on the energy suppliers energy meter. This information can be easily read by Loxone, using the digital input of a Loxone Miniserver.

This information can be used in a variety of ways, depending on needs inside the household, to then make the most of low electricity tariff usage.

Some energy providers offer so-called nighttime tariffs while some offer time-dependent tariffs. If you are wanting to base the usage on specific time – perhaps a window of a few hours – this can be optimised in Loxone COnfig using the Timer Function Block.

Some energy suppliers can provide electricity meters that provide information when a nighttime tariff is available. This information is signalled by a digital output on the energy meter and processed via the digital input on the Loxone Miniserver. This information is then used to supply various devices with electricity. Here are a few examples of such devices that could take advantage of low electricity tariff usage:

**Electric Underfloor Heating**

If the energy meter detects that electricity is available at a low tariff, the electric UFH is activated. For this purpose, relays of the Miniserver or the Relay Extension are connected to the heating circuit of the underfloor heating (we recommend the use of a coupling relay to limit the load on the Loxone devices). Electric boilers can also be activated with a relay to heat up the collected water.

[
![Low electricity tariff usage - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-33-Low-Electricity-Tariff_2.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-33-Low-Electricity-Tariff_2.png)

**Electric Car Charger**

The electric car is charged when electricity is available at low tariffs. Most charging stations offer an interface for communication with devices from other suppliers. For this, only the Loxone Miniserver is necessary as central control to realise low electricity tariff usage. Should a charging process be necessary when electricity is provided at the high tariff, the charging process can be started manually via the Loxone App.

[
![Low electricity tariff usage car charging - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-33-Electricity-Tariff-Ecar_4.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-33-Electricity-Tariff-Ecar_4.png)

**Switching for appliances**

Every mains-power device can be switched on automatically (where it is safe to do so) as soon as the low electricity tariff is active. To realise low electricity tariff usage for appliances, you can use either a relay output on the Miniserver or Relay Extension, or a Smart Socket Air. An example of such appliances is a washing machine. If the appliance needs to have completed a task before the power supply is disconnected again (still using the washing machine as an example here), it is necessary to use the “[Energy Manager](https://www.loxone.com/enen/kb/energy-manager/)” Function Block for additional configuration. Further optimisation can also be achieved with PV systems.

[
![Low electricity tariff usage for appliances - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-33-Low-Electricity-Tariff_1.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-33-Low-Electricity-Tariff_1.png)

**Heated Pool**

In the UK, if you have a swimming pool it’s not uncommon for it to be an indoor pool which can be heated. Heating a mass of water isn’t cheap, so low electricity tariff usage would obviously be a great help Depending on interoperability with the heating source, heating of the pool can be controlled via the Aquastar Air or a relay on the Miniserver. It is then possible to activate the pool’s heating source when electricity is available at a lower tariff. A great tip to keep the pool warm while saving money.

[
![Heating pool with low electricity tariff - Loxone config screenshot](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-33-Low-Electricity-Tariffs-5.png)
](https://www.loxone.com/enen/wp-content/uploads/sites/3/2020/03/Loxone-Use-Case-33-Low-Electricity-Tariffs-5.png)

### Hardware:
- [Loxone Miniserver](https://shop.loxone.com/enuk/miniserver.html)
- [Relay Extension](https://shop.loxone.com/enuk/relay-extension.html) (optional)
- [Smart Socket Air](https://shop.loxone.com/enuk/smart-socket-air.html) (optional)
- [AquaStar Air](https://shop.loxone.com/enuk/aquastar-air.html) (optional)

### Download the sample file:

### Low Electricity Tariff

			[Config 15.1](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/Use-Case-33-Low-Electricity-Tariffs.loxone)

			[Download Config](https://www.loxone.com/enen/wp-content/uploads/sites/3/2024/08/Use-Case-33-Low-Electricity-Tariffs.loxone)

**Important: The conditions of this Use Case rely on the supply of a so-called nighttime electricity tariff. The configuration will respond as depicted herein only if the energy meter provides the information as to when this tariff is available/active. For the use of time-dependent nighttime electricity tariffs, we recommend using the Timer Function Block.**

## Why you and your customer should consider setting up low electricity tariff usage for some appliances?

This feature can be a big money and worries saver. No one will have to check the time whenever they want to use the dishwasher, washing machine, oven etc.

					Local regulations and standards need to be observed. The contents of this page make certain installation assumptions. The information herein does not replace the contextual know-how of a Loxone Partner. As a professionally-installed solution, please consult with a Loxone Partner to realise any of the functionality listed in this Use Case.

#### More Use Cases in this series
-  [AC Control Air for Panasonic](https://www.loxone.com/enen/kb/ac-control-air-for-panasonic/)
-  [Wireless Speaker](https://www.loxone.com/enen/kb/wireless-speaker/)
-  [Door Lock Air Inductive](https://www.loxone.com/enen/kb/door-lock-air-inductive/)
-  [Mail Generator](https://www.loxone.com/enen/kb/mail-generator/)