# Install Sub 10 Client

Source: https://www.loxone.com/enen/kb/install-sub-10-client/

---

This active loudspeaker is a subwoofer with a dual voice coil, ideal for supporting smaller speakers.

It is powered by 24V and connected through the Master Client Interface (MCI) to a Master or Client Speaker. It always plays the same source as the Master Speaker but can be controlled independently in volume or muted.

For optimal sound, the speaker must be installed in a suitable enclosure or a closed ceiling or wall.

[**Datasheet Install Sub 10 Client**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_InstallSub10Client_610163.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Programming examples](#examples)
- [LED Status](#led_state)
- [Inputs, Outputs, Properties](#Actor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Connect the 24V DC power supply and the MCI data lines to the speaker as follows:

![InstSpkClientPCB](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/InstSpkClientPCB.png)

Via the Master-Client Interface (MCI), the Client Speaker is connected in a line comprising one Master Speaker and up to 20 Client Speakers:

![MCISystemWiring](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/MCISystemWiring.png)

> **ℹ️ Note:** The audio signal is transmitted digitally from speaker to speaker. The system compensates for differences in signal transit time and therefore only works with the wiring shown in a linear configuration. On the MCI data lines, there must be no branching or parallel connection to additional speakers!

> **ℹ️ Note:** When only a few speakers are used, the power lines can also be run in a linear fashion. When using many speakers in a line, voltage drop becomes the limiting factor. This can be addressed by using wires with a larger cross-section, shorter cables, or additional 24V power lines. It is essential to always connect the grounds of different power lines.

The connection cable must be secured to the metal bridge of the speaker basket with the cable tie for strain relief:

![InstSpkStrainRelief](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/InstSpkStrainRelief.png)

**Mounting the Speaker:**
Ensure that there are no loose foreign objects in the mounting opening to prevent any background noise.
Insert the speaker into the opening and screw it in place at the front.
The retaining clips will fold out inside and secure the speaker tightly.
Make sure that the speaker fits snugly all around.
Attach the front grille, which is held magnetically.

For the Install Sub 10, the grille must also be screwed on.

---

## Commissioning

After the power supply is switched on, the Client Speaker is ready for pairing. For pairing, the Master Speaker must also be operational and already paired with the Miniserver Compact or Audioserver.

Click on the Master Speaker in the peripheral tree, and then start the Client Speaker search. The connected Client Speakers will be listed:

![ClientSpkSearch](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ClientSpkSearch.png)

When you click on one of the found speakers, it will identify itself with an acoustic signal. This allows you to assign and name them.
By clicking on the right arrow, the Client Speakers are added to the programming. They are then available in the peripheral tree and ready for use in the programming after being saved to the Miniserver.

> **ℹ️ Note:** The Clients are automatically indexed according to the order of connection before pairing. This only takes a few seconds and is necessary for synchronizing the speakers. The order of connection must not be changed after pairing, as this would require re-pairing.

---

## Programming examples

### Example 1:

**A Master and Client Speaker line on the same Audio Player function block**

**Environment:**
A room with limited space

**Installation:**
3 Client Speakers are connected and paired with one Master Speaker

**Programming:**
1 Audio Player function block:

![ori ex1](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ori_ex1.png)

**Result:**
The speakers always behave identically in terms of selected source and volume.

### Example 2:

**A Master and Client Speaker line on different Audio Player function blocks**

**Environment:**
Small restaurant with a bar, the two areas are acoustically adjacent

**Installation:**
5 Client Speakers are connected and paired with one Master Speaker

**Programming:**
2 Audio Player function blocks, 1x for restaurant, 1x for bar, both combined in a fixed group:

![ori ex2](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ori_ex2.png)

**Result:**
Both areas can sensibly only play the same audio source. However, the volume at the bar can be changed or muted independently of the restaurant.

If different audio sources are to be played simultaneously in two or more rooms, an additional Master Speaker must be used per room.

### Example 3:

**2 lines consisting of 1 Master and 1 Client Speaker each**

**Environment:**
Two separate living spaces

**Installation:**
One Client Speaker is connected to each respective Master Speaker per room and paired

**Programming:**
1 Audio Player function block per room:

![ori ex3](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ori_ex3.png)

**Result:**
Both rooms can be operated completely independently in terms of source and volume.

---

## LED Status

> **ℹ️ Note:** Device is in pairing mode.

---

## Actuators

| Summary | Value Range |
| --- | --- |
| Install Sub 10 Client 1 | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Amplifier limit | If the temperature of the amplifier reaches a critical point, the volume of the zone is reduced. This may be due to overloading or excessively high ambient temperature. | - | 0/1 |
| Online Status Install Sub 10 Client 1 | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
| Temperature Shutdown | Input is active, when the outputs of the device have been switched off due to high device temperature. Possible reasons: Ambient temperature too high, outputs overloaded. | Digital | 0/1 |

---

## Properties

| Summary | Description | Unit | Value Range | Default Value |
| --- | --- | --- | --- | --- |
| Serial Number | Specifies the serial number of the device. | - | - | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - | - |
| Maximum Volume | Determines the maximum (physical) output power of the amplifier in percent, thus limiting the maximum possible volume for this output. The volume values from 0-100% of the Audio Player or App are scaled accordingly. | % | 0...100 | 100 |
| Gain | Adjusts the volume of this output in decibels. This control helps balance the sound levels across different speakers or environments, ensuring consistent audio output. The volume is scaled and limited according to the specified maximum volume. | - | -6...6 | 0 |

---

## Safety Instructions

Installation must be carried out by a qualified electrician in accordance with the applicable regulations.

Install the speaker in a way that the electronics on the back are protected from contact, damage, dirt and moisture.

---

## Documents

[**Datasheet Install Sub 10 Client**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_InstallSub10Client_610163.pdf)

---