# IR Control Air

Source: https://www.loxone.com/enen/kb/ir-control-air/

---

The Loxone IR Control Air can be used to control infrared-capable devices such as TVs, AV receivers, projectors, air conditioners etc. In addition to the internal IR transmitter and receiver, a wired external IR receiver allows the hidden placement of the IR Control Air. Up to four external IR transmitter diodes can be placed directly on your devices.

> **ℹ️ Note:** Bang & Olufsen and Daikin remote controls are not compatible with IR Control Air.

[**Datasheet IR Control Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_IRControlAir_100141.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Programming](#configuration)
- [Assigning IR commands](#ir_commands)
- [Inputs, Outputs, Properties](#Diagnostic)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Use the supplied Velcro tape to install the device near the devices you want to control.

Connect the supplied IR receiver and IR transmitter to the back of the unit if required

Place the IR transmitter diodes close to the devices you want to control

Connect the supplied Micro USB power adapter to the back of the unit.

![100141 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100141 install.png)

---

## Commissioning

In delivery state, pairing mode will be active after the power supply has been established. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down the pairing button for at least 5 seconds after establishing power supply.

---

## Programming

After pairing, the IR Control Air is available in Loxone Config. First, remote controls or commands must be defined. There are two ways to do this:

**Insert template:** In Loxone Config some remote controls are already available as templates. CCF files can also be imported:

![100141 10.3 remote templates](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100141 10.3 remote templates.png)

**Pair remote control:** If there is no template for the remote control, it must be paired manually. Select the IR Control Air in the peripheral tree and then click on "Learn button". Please read and confirm the information displayed in the window.

The IR Control Air indicates that it is ready to learn IR commands by flashing orange/green.

If you now repeatedly press the desired button on the remote control, the signal is displayed in the IR Monitor Learn.

Note the value in the "Quantity" column. For each button press it increases by one.

If several lines appear for the same button, select the one with the highest number.

Highlight the line, enter the name of the remote control and the button, and then click Assign.

The IR command has now been assigned to the button of the respective remote control. Repeat this procedure for all desired buttons.

**Toggle:** Some remote controls send two commands with a single button press. In this case, the first command is taught in with "Assign", the second is assigned to the same button with "Assign Toggle".

> **ℹ️ Note:** Beware of possible interference from monitors, bright lighting, fluorescent lamps or direct sunlight.

---

## Assigning IR commands

The previously defined buttons are now assigned to the IR Control via drag & drop either for transmitting or receiving.

For example, the "Power" button is highlighted and with the left mouse button pressed, dragged to the "IR Transmitter" of the IR Control.

This allows the command of the "Power" button to be sent from the IR Control Air. If you want to receive the command, drag the button to "IR Receiver"

Alternatively, an IR Transmitter or Receiver can be created manually and then the command can be assigned in the properties.

As the IR Control also provides 4 connections for external transmitters in addition to the internal one, the transmitter can be selected separately for each command.

The IR Transmitters and Receivers can then be dragged to the programming page and connected to various function blocks.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status IR Control Air | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |
| Serial Number | Serial number of Air device. Automatic pairing can be enabled on the Air Base. Automatic pairing can be enabled on the Airbase for a set time. | - |
| LED Acknowledge | Acknowledge the reception of IR signals with LED | - |

---

## Safety Instructions

Ensure that the device is protected from water.

---

## Documents

[**Datasheet IR Control Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_IRControlAir_100141.pdf)

---