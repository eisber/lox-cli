# Install Speaker 7 Client

Source: https://www.loxone.com/enen/kb/install-speaker-7-client/

---

This active loudspeaker features a woofer with a coaxially integrated tweeter.

It is powered by 24V and connected through the Master Client Interface (MCI) to a Master or Client Speaker. It always plays the same source as the Master Speaker but can be controlled independently in volume or muted.

For optimal sound, the speaker must be installed in a suitable enclosure or a closed ceiling or wall.

[**Datasheet Install Speaker 7 Client**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_InstallSpeaker7Client_610151.pdf)

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Speaker Installation Planning](#PlanSpkrs)
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

---

## Commissioning

After the power supply is switched on, the Client Speaker is ready for pairing. For pairing, the Master Speaker must also be operational and already paired with the Miniserver Compact or Audioserver.

Click on the Master Speaker in the peripheral tree, and then start the Client Speaker search. The connected Client Speakers will be listed:

![ClientSpkSearch](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/ClientSpkSearch.png)

When you click on one of the found speakers, it will identify itself with an acoustic signal. This allows you to assign and name them.
By clicking on the right arrow, the Client Speakers are added to the programming. They are then available in the peripheral tree and ready for use in the programming after being saved to the Miniserver.

> **ℹ️ Note:** The Clients are automatically indexed according to the order of connection before pairing. This only takes a few seconds and is necessary for synchronizing the speakers. The order of connection must not be changed after pairing, as this would require re-pairing.

---

## Speaker Installation Planning

More information about speaker planning can be found [here](https://www.loxone.com/enen/products/audio).

Loxone Install Speakers require installation in a closed enclosure or a cavity, such as in ceilings or walls, to fully develop their sound volume.

Suitable **[mounting enclosures for drywall or concrete installation](https://shop.loxone.com/enus/audio.html/?c=install-box)** are available in the Loxone Shop.

![PH Loxone Installation recommendations table](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PH-Loxone-Installation-recommendations-table.png)

While an enclosure is not strictly required for fully enclosed surfaces, speakers must be installed in a rear mounting enclosure when used in open surface structures, such as acoustic ceilings.

The required acoustic installation volume varies based on the speaker size and type:

| Speaker type | Minimal volume | Recommended volume |
| --- | --- | --- |
| Install Speaker 7 | 7.2l | 9l or more |
| Install Speaker 10 | 14.5l | 18l or more |
| Install Sub 10 | 18l | 30l |

Larger enclosures or cavities may also be used, provided they are closed.

### Speaker Quantity

In main living areas, at least two speakers should be used to achieve good sound quality. For small rooms or ancillary spaces, a single speaker is usually sufficient.
Depending on the room size, we recommend planning the following number of speakers per room:

![RoomsizeSpkrCountChart](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RoomsizeSpkrCountChart.png)

### Ceiling installation

Plan the speaker installation positions to be evenly distributed throughout the room. A minimum distance of 50cm/20″ from walls should be maintained to avoid sound reflections.

In ceiling installations, the stereo effect is barely noticeable and can often be neglected. Therefore, a full stereo signal is later assigned to each individual speaker via the connection to the Audio Player block (Downmix).

### Wall installation

At the most frequently used listening position, at least two speakers should be arranged to achieve a good stereo effect.
The left-right assignment of the speakers is made later through the connection to the Audio Player block.

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
| Install Speaker 7 Client 1 | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Amplifier limit | If the temperature of the amplifier reaches a critical point, the volume of the zone is reduced. This may be due to overloading or excessively high ambient temperature. | - | 0/1 |
| Online Status Install Speaker 7 Client 1 | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |
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

[**Datasheet Install Speaker 7 Client**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_InstallSpeaker7Client_610151.pdf)

---