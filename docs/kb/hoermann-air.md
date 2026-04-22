# Hörmann Air

Source: https://www.loxone.com/enen/kb/hoermann-air/

---

Loxone Hörmann Air is a compact interface module for integrating Hörmann garage door operators.

[**Datasheet Hörmann Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_HoermannAir_100552.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Programming](#baseconf)
- [Limitations](#limits)
- [Softwareindex](#softwareindex)
- [Diagnostic](#diagnostic)
- [Inputs, Outputs, Properties](#Actor)
- [Documents](#Documents)

---

## Mounting

Hörmann Air is connected to the BUS socket of a compatible Hörmann door operator using the enclosed adapter cable. It is also supplied with voltage via this socket. Always refer to the manual to ensure you are using the correct BUS connector.

![hoem BusConnect](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/hoem_BusConnect.png)

For HCP1 operators, the device is then ready for pairing.
The door operator is then recognized automatically.

For HCP2 operators, the BUS-Scan and thus the power supply for the device must first be activated at the door operator in order to be able to pair the device afterwards. Activation varies depending on the door operator:

![hoem BusActivate](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/hoem_BusActivate.png)

After activating the BUS-Scan (power supply), the device can be paired.
The door operator is then recognized automatically.

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for two minutes, then pairing mode is activated for 5 minutes.

---

## Programming

Loxone Hörmann Air is programmed and set up using the [Garage / Gate function block](https://www.loxone.com/help/garage-gate) in Loxone Config:

![hoem dragdrop block](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/hoem_dragdrop_block.gif)

The (Po) input (partially open) of the Garage/Gate block is also supported by Hörmann Air.

---

## Limitations

With HCP1 drives, an error may briefly be displayed on the drive during an update. The function is not affected. After sending a command again, the error is no longer displayed.

When using the Hörmann Air, the Hörmann HKSI climate sensor cannot be used in parallel.

---

## Softwareindex

With the softwareindex the compatibility to Loxone Hörmann Air can be checked, the supported operators are listed in the [datasheet](#Documents).

On the label of the Hörmann garage door operator you will find the serial number with the softwareindex, which begins with a capital letter.

![hoem software](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/hoem_software.png)

---

## Diagnostic

If the Hörmann drive has detected a warning or error, a system message should be active. Please check the device directly to view the error code or status displayed. For detailed troubleshooting instructions, refer to the device’s manual based on the indicated status.

---

## Actuators

| Summary | Description | Value Range |
| --- | --- | --- |
| API Connector | Intelligent API based connector. API Commands | - |
| Light | Activates the lighting of the door operator manually. Independently of this, the door operator always activates its lighting itself when the door is opened or closed. After that, the lighting remains active for a time defined in the operator. | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Hörmann Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Actuator information | Digital | 0/1 |
| Communication error | Digital | 0/1 |
| No communication | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Automatically learn travel durations | Automatically learn travel durations | - |

---

## Documents

[**Datasheet Hörmann Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_HoermannAir_100552.pdf)

---