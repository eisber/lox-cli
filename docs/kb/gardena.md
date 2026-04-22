# Gardena

Source: https://www.loxone.com/enen/kb/gardena/

---

The Gardena plugin allows the integration of compatible devices connected via a Gardena Smart Gateway. More information about Gardena smart system can be found [here](https://www.gardena.com).

> **ℹ️ Note:** A Gardena account is required for use! Please note the Miniserver Gen. 1 is not supported.

## Table of Contents
- [Configuration](#config)
- [Too many Commands](#too_many_commands)
- [Inputs, Outputs, Properties](#Diagnostic)

---

## Configuration

### Setup

First, Gardena must be added. To do so, click on Network Periphery in the periphery tree, then Add Network Device and select Gardena.

### Authentication

After initially saving to the Miniserver, you must authenticate with your Gardena account. Please wait for the system status message asking you to allow your Miniserver access to your Gardena smart devices.

If the Gardena status changes to red after saving to the Miniserver, the connection to the Gardena smart service could not be established.

If communication to the Gardena smart service is interrupted for more than 10 days, the authentication process must be repeated.

### Add devices

If the authentication was successful, your Gardena smart devices can be added via Device Search in Loxone Config.

---

## Too many Commands

For commands on the Gardena smart service, the following limits per account apply:
- Max. 700 requests per week. Any additional commands will be throttled.
- 10 requests per 10-second interval. Any additional commands will be throttled.

> **ℹ️ Note:** Avoid configuration that changes actuator state frequently.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Gardena | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor service | If checked, you will be notified by the System Status or Cloud Mailer if this service is no longer available or offline. | - |

---