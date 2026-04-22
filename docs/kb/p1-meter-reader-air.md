# P1 Meter Reader Air

Source: https://www.loxone.com/enen/kb/p1-meter-reader-air/

---

The P1 Meter Reader Air reads DSMR P1-compatible smart meters via a plug-and-play setup — wirelessly using Loxone Air and powered directly through the P1 interface. It automatically provides bidirectional power and energy values. Additionally, it detects extra sensors and sub-meters connected to the smart meter. This ensures that all relevant energy data is centrally collected and instantly available for efficient energy management.

> **ℹ️ Note:** The use of passive splitters is not recommended.

[**Datasheet P1 Meter Reader Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_P1MeterReaderAir_100663.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Programming](#baseconf)
- [Compatible DSMR P1 versions](#P1Versions)
- [Additional Supported Sensors](#Sensors)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

The P1 Meter Reader Air is connected to the P1 port of a compatible P1 meter using the enclosed adapter cable. It is also supplied with voltage via this socket.

![P1MeterReaderAir connect](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/P1MeterReaderAir_connect.png)

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for two minutes, then pairing mode is activated for 5 minutes.

---

## Programming

Once the P1 Meter Reader Air has been paired, it can be dragged onto the programming page. The function block [Meter Bidirectional](https://www.loxone.com/help/meter-bidirectional) will be created automatically.

![P1MeterReaderAir MeterBidirect](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/P1MeterReaderAir_MeterBidirect.gif)

By default, the API Connector transmits the meter reading and power at the following intervals:
Power: Every 5 seconds, but only on value change.
Meter readings: Every 60 seconds, but only on value change. The transmission cycles can be individually configured in the device settings.

If additional P1 meter values are required, the corresponding sensors can be searched for and added via "Find periphery":

![P1MeterReaderAir FindPeriphery](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/P1MeterReaderAir_FindPeriphery.gif)

---

## Compatible DSMR P1 versions
- 4.2.2 (14-03-2014)
- 5.0 (27-05-2014)
- 5.0.2 (26-02-2016)

---

## Additional Supported Sensors

The following is a list of possible options, but depending on the main or sub-meter setup, some options may not be available (e.g. L3 Power on a single-phase setup).
- L1 Line to Neutral Volts
- L2 Line to Neutral Volts
- L3 Line to Neutral Volts
- L1 Current
- L2 Current
- L3 Current
- L1 Power
- L2 Power
- L3 Power
- Tariff indicator electricity (to switch tariff dependent loads, e.g. boilers)
- Number of power failures in any phase
- Number of long power failures in any phase
- Breaker control state
- Gas Meter reading (temperature converted)
- Heat or Cold Meter reading
- Water Meter reading
- Electric Meter reading (e.g. slave E meter)
- Meter Reading delivered to client (1 = low tariff)
- Meter Reading delivered to client (2 = normal tariff)
- Meter Reading delivered by client (1 = low tariff)
- Meter Reading delivered by client (2 = normal tariff)
- P1 Version Indicator

---

## Actuators

| Summary | Unit |
| --- | --- |
| API Connector | Text |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status P1 Meter Reader Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

---

## Documents

[**Datasheet P1 Meter Reader Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_P1MeterReaderAir_100663.pdf)

---