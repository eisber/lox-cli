# AI Extension

Source: https://www.loxone.com/enen/kb/ai-extension/

---

The **AI Extension** features 4 inputs for analogue voltage signals like 0-10V or digital, as well as 4 analogue inputs for current signals like 0/4-20mA.

The inputs AI1-AI4 for voltage signals operate differentially, each with two terminals for signal and reference potential. The integrated open-wire detection detects interruptions in the measuring circuit.

The inputs AI5-AI8 for current signals work passively, and can be integrated into current loops.

All inputs can also measure negative values with reversed polarity.

All inputs are galvanically isolated from the supply side of the Extension.

[**Datasheet AI Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AIExtension_100471.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Connecting Analog Inputs](#connect-ai)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

The Extension is installed on a DIN rail in a suitable enclosure.

![100471 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100471 install.png)

Connect the power supply and the link data lines to the Miniserver.

The lines for voltage signals are connected to the analog inputs AI1 to AI4. The GND (negative pole) of the associated signal is also required at each input.

The analogue inputs AI5 to AI8 are integrated in current loops.

The Extension starts after switching on the power supply, and the status LED will flash orange after a short time when the connection to the Miniserver is established.

**[Then follow the pairing procedure on the Link Interface.](https://www.loxone.com/help/link-interface#LinkPair)**

---

## Connecting Analog Inputs

### Connection examples 0-10V

**0-10V transmitter with 2 outputs:**

![conex AnalogIn471 2x0 10](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/conex_AnalogIn471_2x0-10.png)

### Connection examples 4-20mA

**Active 4-20mA transmitter:**

![conex AnalogIn471 4 20 active](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/conex_AnalogIn471_4-20-active.png)

**Passive 4-20mA transmitter:**

![conex AnalogIn471 4 20 passive](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/conex_AnalogIn471_4-20-passive.png)

---

## Sensors

| Summary | Unit | Value Range |
| --- | --- | --- |
| Voltage 1 | V | ∞ |
| Voltage 2 | V | ∞ |
| Voltage 3 | V | ∞ |
| Voltage 4 | V | ∞ |
| Current 1 | mA | ∞ |
| Current 2 | mA | ∞ |
| Current 3 | mA | ∞ |
| Current 4 | mA | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status AI Extension | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Open Wire 1 | Digital | 0/1 |
| Open Wire 2 | Digital | 0/1 |
| Open Wire 3 | Digital | 0/1 |
| Open Wire 4 | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair an Extension with unknown serial number. This can only be used if there is only one Extension of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Extension. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Extension into the program. | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

This device must be mounted on a DIN rail in an electrical distribution enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet AI Extension**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_AIExtension_100471.pdf)

---