# Miele@home

Source: https://www.loxone.com/enen/kb/mielehome/

---

Miele@home allows the integration of compatible Miele@home household appliances. Status information can be retrieved, settings can be changed and programs can be started and stopped. More information about Miele@home can be found [here](https://www.miele.com/com/mielehome-5235.html).

[**List of supported functions and devices**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/MieleSupportedFunctionsDevices.pdf)

> **ℹ️ Note:** A Miele@home account is required for use! The Miniserver Gen. 1 is not supported. The Miele@home devices and the Miniserver must have an active internet connection.

## Table of Contents
- [Configuration](#config)
- [Important usage notes](#usage)
- [Command failed](#request_failed)
- [Inputs, Outputs, Properties](#Diagnostic)

---

## Configuration

### Setup

To add Miele@home to the program, click on 'Network Periphery' in the 'Periphery Tree' tab, then 'Add Network Device' and select 'Miele@home'. Or you can quickly add it by clicking F5 and searching for 'Miele'.

### Authentication

After saving the program to the Miniserver, you will need to authenticate with your Miele@home account. A System Status message asking you to allow your Miniserver to access your Miele@home devices will be displayed.

If the status of Miele@home changes to red after saving to the Miniserver, the connection to the Miele@home server could not be established.

### Add devices

If the authentication was successful, your Miele@home devices can be added via Device Search in Loxone Config.

Therefore the devices should be switched on and in an idle and ready state. This can for example be verified in the Miele@home app. If a device is in the wrong state, the Miniserver cannot retrieve all of the required information from it. If this is the case, a corresponding message will be displayed. Please confirm the following points and restart the search:
- Remote controlling the device is allowed.
- The device is switched on and reachable (state 2).
- No program is currently running on the device, and none is in the process of being ended/stopped.
- No manual input or action is required on the device (e.g. confirming a notification).

After successfully adding the devices, the program can be saved to the Miniserver. The online status of each device then changes to green.

---

## Important usage notes

In order to correctly configure the available devices, please familiarize yourself with their functionality and features. Also review them in the Miele@home app. The following information must be observed when creating automations:

### Remote control permission

To be able to control a device remotely, the appropriate permission must be set on the device.

### Device state

Some commands may not be executed, depending on the current state of the device. In such cases, the Miniserver will inform you via the System Status. For more information on failed commands, see [Command failed](#request_failed).

### Remote control allowance

To remote start programs successfully, the following requirements must be met:
- Remote controlling the device is allowed.
- The device is switched on and reachable (state 2).
- The device has to be in state 4 (Waiting for start).

For more information on failed program commands, see [Command failed](#request_failed).

---

## Command failed

If a command to a Miele@home device could not be executed, the Miniserver will notify you via System Status. If the exact reason can be determined, you will be informed of this in the message. In any case, the Miele@home app or the device itself can provide more detailed information.

Listed below are some possible reasons for failed commands:
- Remote controlling the device is not allowed.
- The device is not switched on or not reachable (powered off, offline, in standby mode).
- The device is currently being operated manually.
- A program is already running on the device or one is ending/stopping.
- A manual input or action is required on the device.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Onlinestatus | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor service | If checked, you will be notified by the System Status or Cloud Mailer if this service is no longer available or offline. | - |

---