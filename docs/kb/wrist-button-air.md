# Wrist Button Air

Source: https://www.loxone.com/enen/kb/wrist-button-air/

---

The Wrist Button Air is a battery-operated wristband with a large red button and an integrated LED.
When the Wrist Button Air is pressed, freely configurable actions can be carried out. The integrated LED is controlled either directly as an output with on/off duration or via the [Emergency Alarm](https://www.loxone.com/help/EmergencyAlarm) Block.

[**Datasheet Wrist Button Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WristButtonAir_100496.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Configuration](#Programmierung)
- [Battery replacement](#battery_change)
- [Inputs, Outputs, Properties](#Sensor)
- [Documents](#Documents)

---

## Mounting

Open the packaging and insert the CR2032 battery into the bottom part with the flat + side down.
Then fit the top part onto the bottom part with the red button.
Make sure that the top and bottom are correctly aligned with each other, as shown in the following diagram:

![100496 insertbatt](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100496 insertbatt.png)

Then fix the lower part to the upper part with the enclosed screws:

![100496 bottomscrews](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100496 bottomscrews.png)

A screwdriver is included in the package for this purpose.

---

## Commissioning

The pairing mode is active in the delivery state after inserting the battery. This is indicated by the status LED under the red button flashing red.
The pairing mode is only active for a limited time. If necessary, activate pairing mode again by pressing and holding the red button.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

The status LED shows through the red button, which also serves as a pair button.

If you want to activate pairing mode manually, press the pair button for at least 5 seconds after inserting the batteries.

---

## Configuration

There are 2 options available for configuration:

If the Wrist Button Air is selected in the [Emergency Alarm Block](https://www.loxone.com/help/EmergencyAlarm) and therefore used for an alarm, the Wrist Button Air lights up when an alarm has been activated by a button press. The alarm can be deactivated again by a long press on the Wrist Button Air. In addition, configuration for any purpose can be created with the digital input of the unit. To do this, drag either the device or the input from the periphery tree to the configuration page. The LED of the unit cannot be controlled manually.

Alternatively, by activating the tick box "Use LED" in the parameters of the Wrist Button Air, the input/output for button and LED become visible in the periphery tree. By configuring the switch-on and switch-off duration in the parameters of the LED output, any flashing behaviour can be defined.
Use in the emergency Alarm Block is no longer possible in this variation.
In order to control the LED without delay, the wake-up function is activated in the Wrist Button Air. This will reduce the battery life.

---

## Battery replacement

To replace the battery, open the unit by removing the four screws (Torx 5) on the bottom.
Remove the top section from the bottom and insert a new CR2032 battery in the bottom.

![100496 insertbatt](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100496 insertbatt.png)

When reassembling, make sure that the upper and lower parts are correctly aligned with each other and then fix the screws back in.

Immediately after inserting the new battery or assembling the device, the status LED flashes red 3 times to confirm that the connection to the Miniserver has been restored.

---

## Sensors

| Summary | Description | Value Range |
| --- | --- | --- |
| I | Input is active when the button is pressed. | 0/1 |

---

## Actuators

| Summary | Description | Value Range |
| --- | --- | --- |
| LED | As long as this output is active, the LED flashes at the set interval. The output is only visible if the option "Use LED as Output" has been activated on the unit. | 0/1 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Wrist Button Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Battery level | Provides the current battery level. | % | 0...100 |
| Battery low | Indicates low battery, battery needs to be replaced. | - | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | You will be notified via System Status or Could Mailer if the device is no longer available or offline. As this device and it's functionality are critical to safety, it is not possible to disable this setting for this device. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Use LED as Output | If ticked, the LED can be used as an Output in the configuration. In this case, use with the Emergency Alarm block is no longer possible. If the device is already used in an Emergency Alarm block, this option is deactivated. | - |
| Display Error Output | If checked, error output will be displayed in 2nd row. | - |

---

## Documents

[**Datasheet Wrist Button Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_WristButtonAir_100496.pdf)

---