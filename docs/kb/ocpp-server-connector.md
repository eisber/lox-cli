# OCPP Server Connector

Source: https://www.loxone.com/enen/kb/ocpp-server-connector/

---

OCPP (Open Charge Point Protocol) enables integration with public charging station networks via OCPP servers (cloud or self-hosted). Authenticate with a valid contract RFID Card, monitor the Charging Points availability and sessions, as well as providing information for billing purposes.

Supports OCPP versions 1.6 and 2.0.1 via secure and unsecure web socket connections (ws:// & wss://).

Miniserver users can be configured with an OCPP Tag ID, which is used for all transactions initiated by them via App or NFC Code Touch.

By default, Miniserver Authorization based on user permissions is used as fallback only if authorization through the OCPP server via OCPP Tag ID or NFC Tag ID fails. The 'Miniserver Authorization preferred' setting changes priority.

For the best experience we recommend using the Loxone Wallbox Tree with Energy Meter Tree and NFC Code Touch Tree.

More Information about OCPP can be found [here](https://openchargealliance.org/protocols/open-charge-point-protocol/).

Current Miniserver generation required, not supported by Miniserver Gen.1 variants.

Wallbox Tree/Air are 'Chargepoint Compatible' certified with the be.Energised charging network.

## Table of Contents
- [Initial Setup](#ocpp_setpup)
- [Programming Examples](#ocpp_baseconf)
- [Functionality](#ocpp_function)
- [OCPP Cards](#ocpp_card)
- [Supported OCPP Features](#ocpp_features)
- [NFC History](#ocpp_nfc)
- [Wallbox](#ocpp_wallbox)
- [Inputs, Outputs, Properties](#Actor)

---

## Initial Setup

Add OCPP Server Connector Plugin as Network Device:

![ocpp add](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ocpp_add.png)

Specify settings for OCPP Server Connector plugin and Charging Point. Add a separate Charging Point for each Wallbox:

![ocpp settings](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ocpp_settings.png)

---

## Programming Examples

### Wallbox standalone

Connect the Charging Points API Connector to the NFC Code Touch and Wallbox function blocks. When connecting the NFC Code Touch with the Charging Point API Connector, the authentication method will be changed to "OCPP" and Don to 0,7s automatically.

![ocpp example wallbox](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ocpp_example_wallbox.png)

### Wallbox with Wallbox Manager

Connect the NFC Code Touch Blocks outputs to either "Ec" or "Ecp" of the Wallbox function block, depending on if Eco or Prio Mode – or both should be usable.

![ocpp example wallboxmanager](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ocpp_example_wallboxmanager.png)

> **ℹ️ Note:** Connecting a charging station (NFC/wallbox) to multiple backends simultaneously is not supported.

---

## Functionality

To start an OCPP charging session, first connect your vehicle, then authenticate. OCPP charging sessions are stopped when the car is disconnected.

**Authentication and Charging Processes with OCPP and Local Charging:**

**Roaming Users (Charging Card)** initiate charging sessions using the NFC Code Touch, it reads their charging cards OCPP-Tag-ID and passes it to the OCPP server, which will either accept or reject it. The NFC Code Touch will reflect the decision using its LEDs (red or green).

**Miniserver Users** can initiate a charging session either via Loxone App or using NFC Code Touch. If an OCPP Tag ID has been configured on the user object, it will be passed to the OCPP server. If no OCPP Tag ID is configured, or it is rejected by the OCPP server, the Miniserver will decide based on the users permissions and may still allow charging. In this case, the Charging Point will be reported as unavailable and no billing information is passed to the OCPP server for that charging session. The 'Miniserver Authorization preferred' setting can be used to change priority.

**Difference Between OCPP and Local Charging:**
In both situations the wallbox blocks history will track the charging session. If the OCPP server accepted the charging session, billing information will not be part of the history and the last charging cost will be 0.

**Energy Management & OCPP**
The charging power set by the energy management on the Miniserver always has priority. If the charging process is deactivated due to local decisions such as "Load Management", "Energy Management", or "Spot Price Management", the charging process will not start or will be stopped. Charging profiles provided by the OCPP server are not used.

**Outages (Network or Power)**
During network outages, transaction messages are queued and sent once the connection is reestablished. After a reboot (e.g. due to power outages), active transactions are restarted, thus connected cars keep charging. On the OCPP server this will be reflected in two charging sessions, one before and one after the outage.

**Remote Start/Stop**
Transactions can be remotely started by the OCPP server. This can be used, for example, to start a session via QR code or a charging network app. When remotely stopping a transaction, the car must be disconnected and reconnected to start a new transaction. Otherwise authorizing a new transaction won't be possible or billing might get lost.

**Selfcheck**
The charging station checks for device errors on startup and reports a 'faulted' state if any are found. The faulted state is cleared once the errors are resolved. Starting a new transaction is not possible while in the faulted state.

---

## OCPP Cards

The Miniserver supports 4 Byte Card, 7 Byte Card and 8 Byte Card.

**Note:** The Miniserver removes the trailing zeros and the EC specifier from the NFC ID before sending it to the OCPP Server. So make sure the NFC ID is correctly set on the OCPP Server.

Examples:

AA:AA:AA:AA:[00:00:00:00]:[EC] -> 4 Byte Card

AA:AA.AA:AA:AA:AA:AA:[00]:[EC] -> 7 Byte Card

AA:AA:AA:AA:AA:AA:AA:AA:[EC] -> 8 Byte Card

---

## Supported OCPP Features

| Feature | Description | Supported? |
| --- | --- | --- |
| Reset | OCPP 2.0.1 by default hardware reset OCPP 1.6 hard/soft reset | No |
| Charge point certificate | - | No |
| Offline Behaviour | Messages are queued | Yes |
| Set/Get Variables | OCPP Server can make changes to Charging Point | Yes |
| Remote Start/Stop transaction | - | Not yet supported |
| Authorization | - | Yes |
| Local Authorization | Authorized tokens are cached locally | No |
| Online Transactions | Charging Point reports to OCPP server | Yes |

---

## NFC History

The [Authentication NFC Code Touch](https://www.loxone.com/help/AuthenticationNFC) shows who got access by which authority (Miniserver or OCPP). The OCPP plugin name is used as the authority's name.

![ocpp nfchistory](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ocpp_nfchistory.png)

---

## Wallbox

Prices are hidden by default in the [Wallbox](https://www.loxone.com/help/wallbox-block) because billing is handled by the OCPP server.

When authorized locally without OCPP servers' approval, the prices configured on the block or provided from a [Wallbox Manager](https://www.loxone.com/help/wallbox-manager) are shown in the app.

Standalone (no Wallbox Manager):

![ocpp wallbox](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ocpp_wallbox.png)

---

## Actuators

| Summary | Unit |
| --- | --- |
| API Connector | Text |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Onlinestatus | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Server Address | Websocket Address of the OCPP-Server (unsecure or secure).\n e.g.\nws://[address]:[port]/steve/websocket/CentralSystemService\nwss://[address]/ocpp/cp/socket/16\nThis address can be found in your external service portal | - |
| Protocol | Protocol used to communicate with OCPP Server. Applies for all chargepoints. | - |
| Miniserver Authorization preferred | OCPP server authorization used as fallback if Miniserver does not grant permission based on user rights in the first place. | - |
| Monitor service | If checked, you will be notified by the System Status or Cloud Mailer if this service is no longer available or offline. | - |

---