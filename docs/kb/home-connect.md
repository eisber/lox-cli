# Home Connect

Source: https://www.loxone.com/enen/kb/home-connect/

---

Home Connect allows integration of compatible household appliances. Statuses can be retrieved, settings changed, and programs started and stopped. More information about Home Connect can be found [here](https://www.home-connect.com).

The available functions depend on the respective Home Connect device:

[**List of supported functions and devices**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/HomeConnectSupportedFunctionsDevices.pdf)

> **ℹ️ Note:** A Home Connect account is required for use! The Miniserver Gen. 1 is not supported. The Home Connect devices and the Miniserver must have an active internet connection.

## Table of Contents
- [Configuration](#config)
- [Important usage notes](#usage)
- [Too many requests](#too_many_requests)
- [Command failed](#request_failed)
- [Inputs, Outputs, Properties](#Diagnostic)

---

## Configuration

### Setup

To add Home Connect to the program, click on 'Network Periphery' in the 'Periphery Tree' tab, then 'Add Network Device' and select 'Home Connect'. Or you can quickly add it by clicking F5 and searching for 'Home Connect'.

### Authentication

After saving the program to the Miniserver, you will need to authenticate with your Home Connect account. A System Status message asking you to allow your Miniserver to access your Home Connect devices will be displayed.

If the Home Connect status changes to red after saving to the Miniserver, the connection to the Home Connect server could not be established.

### Add Devices

If the access permission was successful, your Home Connect devices can be added via Device Search in Loxone Config.

The units must be switched on and in a state in which programs can be started. This can be checked in the Home Connect app. If a device is in the wrong state, the Miniserver cannot retrieve all the necessary information about it. If this is the case, a corresponding message will be displayed. Please confirm the following points and restart the search:
- Remote control of the unit is allowed.
- Remote start (if available) is enabled.
- The device is switched on and reachable (operating state 1).
- The device is not currently being operated manually.
- No program is currently running on the device, and none are in the process of being ended/stopped.
- No manual input or action is required on the device (e.g. confirming a notification).

After successfully adding the devices, the program can be saved to the Miniserver. The online status of each device then changes to green.

---

## Important usage notes

In order to correctly configure the available devices, please familiarise yourself with their functionality and features. Also review them in the Home Connect app. The following must be observed when creating automations:

### Remote control permission

To be able to control a device remotely, the appropriate permissions must be set on the device.

### Remote start permission

To be able to start programs, remote start must be activated (if available). On some appliances the remote start permission is reset each time a manual operation is performed, and must be re-enabled again afterwards.

### Device state

Some commands may not be executed, depending on the operating state of the device. In such cases, the Miniserver will inform you via the System Status. For more information on failed commands, see [Command failed](#request_failed).

It may also be necessary to perform a manual action locally on the device or in the Home Connect app before starting a program (e.g. confirming a notification).

### Starting programs

To remote start programs successfully, the following requirements must be met:
- The remote control permission must be activated on the unit.
- Remote start (if available) is enabled.
- The device is switched on and reachable (operating state 1).
- The device is not currently being operated manually.

For more information on failed program commands, see [Command failed](#request_failed).

### Maximum number of requests

There are various limits for a Home Connect account, e.g. a maximum of 1000 requests are allowed per day. If one of the limits is reached, further requests are blocked for a certain time. For more information, see [Too many requests](#too_many_requests).

> **ℹ️ Note:** Avoid configuration that frequently operates outputs.

---

## Too many requests

The following per-account limits apply to requests to the Home Connect service:
- Max. 1000 requests per 24h. Further requests will be blocked for the next 24h.
- Max. 50 requests per minute. Further requests will be blocked for 1 minute.
- Max. 5 program start requests per minute. Further start requests will be blocked for 1 minute.
- Max. 5 program stop requests per minute. Further stop requests will be blocked for 1 minute.

If the Miniserver reports too many requests to the Home Connect service via the System Status, certain requests are blocked for a certain time, depending on the limit that has been exceeded. This applies either to all requests or only to one specific type of request. The System Status message will inform you how long the requests will be blocked.

> **ℹ️ Note:** Avoid programming that frequently operates outputs.

---

## Command failed

If a command to a Home Connect device could not be executed, the Miniserver will notify you via System Status. If the exact reason can be determined, you will be informed of this in the message. In any case, the Home Connect app or the device itself can provide more detailed information.

Listed below are some possible reasons for failed commands:
- Remote controlling the device is not allowed.
- Remote start (if available) is not enabled.
- The device is not switched on or not reachable (powered off, offline, in standby mode).
- The device is currently being operated manually.
- A program is already running on the device or one is ending/stopping.
- A manual input or action is required on the device.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Home Connect | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor service | If checked, you will be notified by the System Status or Cloud Mailer if this service is no longer available or offline. | - |

---