# System Status

Source: https://www.loxone.com/enen/kb/systemstatus/

---

## Overview

Comparable to the warnings and information lights on the dashboard of a vehicle, your home now also gives you important information via System Status.

System Status is a method by which the Miniserver can communicate messages about the current system. This can be simple and minor information or messages that require immediate action.

The individual messages provide information on the cause and the nature of the incident and will also help to restore the system to its normal state. The residents are informed about individual events in various ways. System Status can be viewed in Loxone Config as well as in the Loxone App.

## Glossary
- [System Status in App](#app)
[System Status in Config](#config)
[Message Types](#messages)
[Potential Messages](#potential-messages)
[Notifications](#notifications)

### System Status in The Loxone App

In the Loxone App, System Status is only actively visible when active message are present. If this is the case, a coloured heart symbol appears in the title bar, alongside a bar of the same colour. Tapping on the bar or the heart symbol takes you to a detailed view of the System Status. If no messages are currently active, System Status can be found in the Miniserver settings on the App.

![App System Status](https://www.loxone.com/enen/wp-content/uploads/sites/3/2018/07/Loxone-App.png)

### System Status in Loxone Config

In Loxone Config, System Status becomes visible as soon as a connection to a Miniserver has been established.
On the right-hand side of the status bar, an area appears, which describes the current state of the System. Clicking on the area takes you to a detailed view.

![Unbenannt](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/07/Unbenannt.jpg)

## Messages

There are 2 important distinguishing features for individual messages. One is the severity and the other is who the message is displayed to. Some messages are only displayed to Administrators, other messages are displayed to all users, or even to a specific user. A distinction is made between the following severities:
- **Information** – E.g. System Reboot following Update
- **Warning** – E.g. High SD Card Write Load
- **Important Message** – E.g. Air Device Offline
- **Critical Message** – E.g. Extension Offline

### Potential Messages
- SD Card Diagnostics

[SD Card Damaged](https://www.loxone.com/enen/kb/sd-card-diagnostics/#worn)
- [SD Card Defective](https://www.loxone.com/enen/kb/sd-card-diagnostics/#broken)
- [SD Card High Write Load](https://www.loxone.com/enen/kb/sd-card-diagnostics/#writeload)
- Shading Diagnostics
- [Obstacle Detected](https://www.loxone.com/enen/kb/shading-diagnostics/#obstacle-detected)
- [Motor Blocked](https://www.loxone.com/enen/kb/shading-diagnostics/#motor-blocked)
- Valve Actuator Diagnostics
- [Valve Stuck](https://www.loxone.com/enen/kb/valve-diagnostics/#valve-stuck)
- [No Valve Detected](https://www.loxone.com/enen/kb/valve-diagnostics/#no-valve)
- [Mechanical Defect](https://www.loxone.com/enen/kb/valve-diagnostics/#valve-hardware)
- Ventilation Diagnostics
- Filter Change Due [[Internorm](https://www.loxone.com/enen/kb/internorm-ventilation/#filter) & [Leaf](https://www.loxone.com/enen/kb/leaf-air-tree/#maintenance)]
- Ventilation Error (Mechanical or Electrical) [[Internorm](https://www.loxone.com/enen/kb/commissioning-internorm-i-tec-devices/#error-codes) & [Leaf](https://www.loxone.com/enen/kb/leaf-air-tree/#diagnostics)]
- Gate Diagnostics
- [Gate Stopped by Safety Mechanism](#gate-safety)
- [Gate Not Closed](#gate-object)
- Sauna Diagnostics
- [Sauna Exceeded Maximum Operating Time](#operating-time)
- [Sauna Exceeded Maximum Operating Temperature](#operating-temperature)
- Pool Diagnostics
- [Pool Control Suspended](#pool-control)
- [Pool Out of Order](#pool-offline)
- Miscellaneous
- [Air Channel Occupied](https://www.loxone.com/enen/kb/loxone-air/#frequency-change) — Advanced Issue – Consult Loxone Partner
- [Device Offline](https://www.loxone.com/enen/kb/device-offline-diagnostics/)
- [Insecure User](https://www.loxone.com/enen/kb/password-security/)
- [Insecure Admin](https://www.loxone.com/enen/kb/password-security/)
- [Client Miniserver(s) Offline](https://www.loxone.com/enen/kb/device-offline-diagnostics/#networkdevice)
- [Miniserver Restart](#miniserver)

### Gate Diagnostics

#### Gate Stopped by Safety Mechanism

When configured, the gate controller can detect obstacles in the way of the gate itself. This is brought up as the safety mechanism stopping the motor.

#### Gate Not Closed

When configured, the gate controller can detect objects crossing the photo cell(s)/light barrier(s). This is brought up as the gate being unable to complete its closing cycle.

### Sauna Diagnostics

#### Exceeded Operating Time

The Sauna Controller in Loxone Config is configured with a defined shutdown timer. After this period has elapsed, the sauna will automatically shutdown for safety. To continue use, simply acknowledge the message and reactive the sauna.

#### Exceeded Operating Temperature

The Sauna Controller in Loxone Config is configured with a defined shutdown temperature. When this temperature is met or exceeded, the sauna will automatically shutdown for safety. To continue use, allow the room to cool before attempting to use it again.

### Pool Diagnostics

#### Pool Control Suspended

The AquaStar Air can detect issues with its normal function. If an issue is detected, it will shutdown its automated control, resulting in your Pool Control being suspended. To solve this issue, check the pool system for any anomalies and acknowledge the message.

#### Pool Out of Order

If the AquaStar Air goes offline, you will receive a message alerting you that the Pool is Out of Order. For further diagnosis of this issue, see [here](https://www.loxone.com/enen/kb/device-offline-diagnostics/#air).

### Miniserver Restart

There are a number of reasons why a Miniserver may restarts. The three key reasons that you will receive as a System Status Message are: Config Change, Firmware Upgrade, and Power Cut

#### Config Change

In the event that a change to the configuration file requires a system restart, you will be notified of this occurrence.

#### Firmware Upgrade

When the Miniserver is successfully upgraded to a new version of firmware, you will be notified of this occurrence.

#### Power Cut

In the event of the Miniserver being restarted as a result of power failure, you will be notified of this occurrence.

---

## Notifications

The residents in the Smart Home are informed of new events from the System Status in the following ways:
- Push Notifications
- Emails
- Daily Report
- Miniserver LED
- Log File

### Push Notifications

To receive push notifications from System Status, active the corresponding setting in your Loxone Smart Home App. In the App settings, navigate to “User Interface & App” and click on “Notifications”. On the follow screen, select “Push Notifications”, activate Push Notifications and select “System Status”.

### Emails

To be notified by Email about System Status messages, the option “Use for System Messages” must be enabled for the Mailer. This can be found in the Mailer options.

### Daily Report

All messages that do not trigger either a direct push notification or a direct email due to their low severity are listed in a daily report. The report only informs the user of a event if has not appeared in an email before.

### Miniserver LED

If a message is, at least, visible to all administrators, the secondary LED of the Miniserver will light up in the message’s corresponding colour, analogous to the coloured displayed in the app.

### Log File

Information about System Status messages are written to the log file “def.log”. This can be accessed via the Web Service “http://miniserver/dev/fsget/log/def.log”.

## Documents

[Overview of all system status messages](https://www.loxone.com/dede/wp-content/uploads/sites/2/2018/10/Systemstatus_Message_Overview.pdf)