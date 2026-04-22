# Honeywell Security

Source: https://www.loxone.com/enen/kb/honeywell-security/

---

Requires MB-Secure Honeywell Server, or MB Secure PRO (minimum version 03.06) with IQ Control Panel (minimum version 16.05).

Additionally, a Smart Building License is required, which has to be aquired from Honeywell Security.

Support for arming and disarming, Detector groups, Macros, Doors, ...

Setup a new user with remote client type Loxone.

Save the changes to MB-Secure. Afterwards load the latest configuration from MB-Secure and export the OPIF file.

Import the latest OPIF into Config. The inputs and outputs will be automatically created.

[**Honeywell Security Plugin Setup**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/HoneywellSecurity_Setup.pdf)

## Table of Contents
- [Inputs, Outputs, Properties](#Diagnostic)

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Honeywell Security | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Address | IP address or hostname and port of the MB-Secure system (e.g., 192.168.178.100:443). | - |
| Username | Username of the MB-Secure user. Important: The user must have appropriate permissions for remote client access (e.g., via Client Type Loxone). | - |
| Password | Password for the MB-Secure user | - |
| Timestamp | System Status Messages related to OPIF timestamp mismatches are shown with this severity | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |

---