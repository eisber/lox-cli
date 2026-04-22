# Tesla Powerwall

Source: https://www.loxone.com/enen/kb/tesla-powerwall/

---

Allows the integration of a Tesla Powerwall and provides several status inputs.

> **ℹ️ Note:** A Tesla account is required for use! The Miniserver Gen. 1 is not supported.

> **ℹ️ Note:** Currently only Tesla Powerwall 1 and Tesla Powerwall 3 are supported.

## Table of Contents
- [Configuration](#config)
- [Powerwall 3](#powerwall3)
- [Inputs, Outputs, Properties](#Sensor)

---

## Configuration

### Setup

To add your Tesla Powerwall to the program, click on 'Network Periphery' in the 'Periphery Tree' tab, then 'Add Network Device' and select 'Tesla Powerwall'. Or you can quickly add it by clicking F5 and searching for 'Tesla Powerwall'.

Enter the local IP address or hostname of your Powerwall in the settings of the Tesla Powerwall AddOn.

### Authentication

Credential are required to connect to the device locally. Enter the email address registered with your Tesla account and the local password of your Tesla Powerwall in the settings. The default password is the last 5 characters of the Tesla Gateway password.

If the status of Tesla Powerwall changes to green after saving to the Miniserver, the connection is established successfully.

> **ℹ️ Note:** The Tesla Powerwall calculates the energy values imbalanced (phase specific). Connect only the power input to the meter blocks to let the Meter block calculate the energy.

---

## Powerwall 3

To use a Powerwall 3 with our Tesla plugin, you must use the IP address and password of the Powerwall itself.

### Obtaining Password

> **ℹ️ Note:** Obtaining the password requires physical disassembly of the Powerwall unit.

Remove the plexiglass cover by unscrewing the 6 screws.

The full password is found on a sticker located on the memory unit inside the Powerwall.

![Tesla powerwall3](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Tesla_powerwall3.png)

For the plugin configuration, use only the last 5 characters of this full password.

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Battery state of charge | The current level of charge of the battery. | % | 0...100 |
| Grid disconnected | Activates if the grid is disconnected. | Digital | 0/1 |
| Grid power | The current grid power. Positive values indicate power draw from the grid, negative values indicate exporting power to the grid. | kW | -2147483648...2147483647 |
| Battery power | The current battery power. Positive values indicate power draw from the battery, negative values indicate charging the battery. | kW | -2147483648...2147483647 |
| Production | Current solar power production. | kW | -2147483648...2147483647 |
| Load | The current power used by the home. | kW | -2147483648...2147483647 |
| Grid energy imported | Total energy imported from the grid. | kWh | -2147483648...2147483647 |
| Grid energy exported | Total energy exported to the grid. | kWh | -2147483648...2147483647 |
| Solar energy produced | Total solar energy produced. | kWh | -2147483648...2147483647 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Tesla Powerwall | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Address | IP address or hostname of the device in the local network. | - |
| Email | The mail address which is used for the Tesla account. | - |
| Password | Password for local authentication on the device. If not changed manually, the password consists of the last 5 symbols of the gateway password. | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |

---