# Husqvarna

Source: https://www.loxone.com/enen/kb/husqvarna/

---

The Husqvarna plugin allows the integration of compatible [Husqvarna Automower® X-Line models](https://www.husqvarna.com/automower), with built-in cellular connection. It can be used to initiate a mowing session, park the robot, or display the charge status and current activity. More information about Husqvarna can be found [here](https://www.husqvarna.com).

> **ℹ️ Note:** A Husqvarna account is required for use! Please note the Miniserver Gen. 1 is not supported.

The status of the lawnmower is transmitted to the Miniserver every 15 minutes, in the Husqvarna App it is displayed live.

## Table of Contents
- [Configuration](#config)
- [Notes for Use](#cut_park)
- [Too many Commands](#too_many_commands)
- [Inputs, Outputs, Properties](#Diagnostic)

---

## Configuration

### Setup

First, Husqvarna must be added. To do so, click on Network Periphery in the periphery tree, then Add Network Device and select Husqvarna.

### Authentication

After initially saving to the Miniserver, you must authenticate with your Husqvarna account. Please wait for the system status message asking you to allow your Miniserver access to your Husqvarna devices.

If the Husqvarna status changes to red after saving to the Miniserver, the connection to the Husqvarna server could not be established.

If communication to the Husqvarna server is interrupted for more than 10 days, the authentication process must be repeated.

### Add Devices

If the authentication was successful, your Husqvarna devices can be added via Device Search in Loxone Config.

---

## Notes for Use

### Mowing and Parking

Each robot lawnmower has an output for starting the mowing process and one for parking the unit. A command is triggered by a rising edge on the corresponding output.

A rising edge on the Park actuator, parks the robot until a new command is sent to the unit.

Both commands will override the actions from the Calendar in the Husqvarna App.

---

## Too many Commands

For commands on the Husqvarna service, the following limits per account apply:
- Max. 1 command per second. Any additional commands will be throttled.
- Max. 10000 commands per month. Any additional commands will be throttled.

> **ℹ️ Note:** Avoid configuration that changes actuator state frequently.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Husqvarna | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor service | If checked, you will be notified by the System Status or Cloud Mailer if this service is no longer available or offline. | - |

---