# GEIGER SOLIDline Air

Source: https://www.loxone.com/enen/kb/tubular-motor-solidline-air/

---

The GEIGER SOLIDline Air is a tubular motor for roller shutters and awnings, controlled via Loxone Air technology.

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Setting the end limits](#setlimits)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Follow the instructions to install the SOLIDLine Air motor:

[Installation and operating instructions with technical data (English)](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/GEIGER_E_BAL_SOLIDline-Flex-AIR_100W1580-000_EN.pdf)

[Installation and operating instructions with technical data (German)](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/GEIGER_E_BAL_SOLIDline-Flex-AIR_100W1580-000_DE.pdf)

> **ℹ️ Note:** Please ensure that the power cable plug is fully inserted into the tubular motor, as shown in the following image:

![solidline insertplugfully](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/solidline-insertplugfully.jpg)

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by an audible "clack-clack" (short up/down movement).

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, switch off the power for 10 seconds, then switch it back on. If no connection to a Miniserver can be established for two minutes, then pairing mode is activated for 30 minutes.

---

## Setting the end limits

After pairing the device, it will be available in the periphery of Loxone Config. Drag the SOLIDLine Air from the periphery tree onto the program page and save the program into the Miniserver, then set the end positions.

The direction of rotation depends on the installation position. The shading should be in a middle position when programming the end limits so that it can move freely in both directions.

To set the end limits, activate the service mode in the App. Then use the buttons in the app to adjust the position of your shading. Once the end position has been reached, save it by clinking "Save end position".

![Solidline SetEndLimits](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/Solidline-SetEndLimits.png)

The end limits are independent of each other and can be set with or without stoppers. The motor confirms the setting of the lower limit with "clack-clack" and the upper limit with "clack-clack-clack", which also indicates the end of the setting procedure. The time between setting the lower and upper limit must not exceed 2 minutes, otherwise the previous or factory settings will be restored, which the motor would indicate with a "clack-clack-clack-clack-clack-clack".

Once the end limits are programmed, you can exit the service mode. If the limits need to be readjusted, briefly disconnect the motor from power. The service mode can then be activated within the first 30 minutes of re-establishing power to the motor and the end limits can be set again. No end positions are set in delivery state.

---

## Actuators

| Summary | Unit |
| --- | --- |
| API Connector | Text |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status GEIGER SOLIDline Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| System temperature | Provides the internal device temperature. This is often the temperature of the CPU or another location in the device. | ° | ∞ |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Disable Repeater functionality | Disable repeater functionality of this Air device. Loxone Air is based on mesh technology. Any air device connected to the power supply can repeat packets from other Air devices, thus extending the range and stability of the overall system. In large systems with a large number of air devices in a confined space, the communication between the air devices can lead to a very high radio channel utilization. A reliable accessibility of the air devices can not be guaranteed. Disabling repeater functionality on individual Air devices can help. Do not disable this function recklessly as this may affect the range and stability of the system. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| Application | Sets what the GEIGER SOLIDline Air is used for. | - |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

Please note the safety instructions in the installation and operating manual.

---

## Documents

[Installation and operating instructions with technical data (English)](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/GEIGER_E_BAL_SOLIDline-Flex-AIR_100W1580-000_EN.pdf)

[Installation and operating instructions with technical data (German)](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/GEIGER_E_BAL_SOLIDline-Flex-AIR_100W1580-000_DE.pdf)

---