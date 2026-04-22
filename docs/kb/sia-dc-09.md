# SIA DC-09

Source: https://www.loxone.com/enen/kb/sia-dc-09/

---

Using the SIA DC-09 protocol, the Miniserver can send a predefined message to a SIA server upon ON/OFF/analogue value change, indicating threats such as fire alarm or burglary to a monitoring station.

An introduction to SIA DC-09 and the configuration in Loxone Config is available in this **[video on YouTube](https://www.youtube.com/watch?v=dO9vBhOVPFE)**.

> **ℹ️ Note:** The Miniserver Gen. 1 does not support the SIA DC-09 protocol!

## Table of Contents
- [Properties](#Property)
- [Programming example](#baseconf)
- [SIA Codes, Contact ID Codes](#codes)
- [Documents](#documents)

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Server address | IP-address:port or sia.example.com:port | - | - |
| Backup server address | IP-address:port or sia.example.com:port If specified, this server is used when the main server is not available. | - | - |
| Polling [s] | Server address polling [s] (0 = no polling, max. 86400 seconds) | 0...86400 | - |
| Backup polling [s] | Backup server address polling [s] (0 = no polling, max. 86400 seconds) | 0...86400 | - |
| Server timeout [s] | Timeout [s] for server response (1 to 10 seconds) | 1...10 | - |
| Protocol | IP protocol currently only TCP is supported | - | - |
| Data format | Protocol (expected by the server) | - | - |
| Encryption Key | 0, 32, 48, or 64 characters (allowed characters: 0-9 A-F) If no key is specified, the transmission is unencrypted | - | - |
| Account Number | Server account (3-16 characters allowed: 0-9 A-F) | - | - |
| Account prefix | Prefix to account (1-6 characters allowed: 0-9 A-F) Use 0 if none is specified. | - | - |
| Receiver number | optional receiver number (0-6 characters allowed: 0-9 A-F) | - | - |

---

## Programming example

First, create a SIA DC-09 output under Messages:

![SIAcreatevo](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/SIAcreatevo.png)

Now the SIA server address must be specified in the settings.
The output can then be used, for example, on the text output of an alarm block:

![SIAconfig](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/SIAconfig.png)

---

## SIA Codes, Contact ID Codes

For further information on the protocol structure and the SIA codes, see the following documents:

[List of SIA Codes](https://www.chipkin.com/files/liz/FS-8705-19%20-%20Security%20Industry%20Association%20-%20SIA%20Codes.pdf)

List of Contact ID Codes: www.nexgenerationcentral.com/Portals/7/AdemcoContactID.pdf

---

## Documents

[**Set up SIA DC-09 in Loxone Config**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/SIA_DC-09_Setup.pdf)