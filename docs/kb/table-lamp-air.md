# Table Lamp Air

Source: https://www.loxone.com/enen/kb/table-lamp-air/

---

The Table Lamp Air is a lamp with warm white light, RGB signal light, has an integrated Li-Ion battery and touch buttons
It is controlled and programmed wirelessly via a [Miniserver](https://www.loxone.com/help/miniserver) and the [Loxone Air technology](https://www.loxone.com/help/air-interface).

[**Datasheet Table Lamp Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TableLampAir_100550,100551.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Sensor)

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.

Pairing mode ONLY works when the device is plugged in. If the lamp is in pairing mode and then unplugged, it returns to the delivery state (switched off).

![TableLamp PairingButton](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/TableLamp_PairingButton.png)

---

## Sensors

| Summary | Description | Value Range |
| --- | --- | --- |
| Charging status | 0: USB not connected 1: Battery is charging 2: Battery is fully charged 3: Charging error | 0...3 |
| T5 | T5 for Lighting and Audio | ∞ |

---

## Actuators

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Disable T5 | Disables T5 Buttons | - | 0/1 |
| Smart Actuator RGB | - | ∞ |
| Smart Actuator W | Smart Actuator Dimmer | % | 0...100 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Table Lamp Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery Low | This input activates when the battery level is