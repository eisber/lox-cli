# Touch Tree

Source: https://www.loxone.com/enen/kb/touch-tree/

---

For installation in the UK, please refer to the use of the EU back box for mounting [here](#backbox)

The Loxone Touch Tree features five touch points to control the most important functions of a room. When a button is touched, an audible click confirms the action. An integrated sensor measures temperature and relative humidity.

The large centre touch zone is ideal for controlling the lighting, while the corner zones are suitable for controlling music and shading. It is based on the [Loxone switch standard](https://www.loxone.com/enen/smart-home/switch-standard/). The buttons can also be freely used for other applications.

Please note that there is a certain delay when measuring humidity due to the housing. The Room Comfort Sensor is better suited for a fast detection of changes in humidity.

## Table of Contents
- [Mounting](#Assembly)
- [Commissioning](#Commissioning)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Mounting

Install the device on a suitable installation box.

Connect the power supply (orange/white terminal) and Tree communication wires (green/white terminals).

![100221 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100221 install.png)

**Touch Tree US:**

![100425 install](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/100425 install.png)

Finally, secure the device by snapping it onto the mounting frame or screwing it onto the frame of the installation box, depending on the model.

---

## Commissioning

Shortly after power-up, the status LED on the back of the device will blink orange if the wiring is correct (connection to Tree Extension and Miniserver is established).

**[Then follow the pairing procedure on the Tree Interface.](https://www.loxone.com/help/tree-interface#TreePair)**

---

## Sensors

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| T5 | Combined input for the 5 touch points according to the Loxone Switch Standard. | - | ∞ |
| Temperature | Provides the air temperature. | ° | -40...125 |
| Humidity | Provides the air humidity. | % | 0...100 |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status Touch Tree | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Value Range | Default Value |
| --- | --- | --- | --- |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - | - |
| Serial Number | Specifies the serial number of the device. Enter 'Auto' to automatically pair a Tree device with unknown serial number. This can only be used if there is only one Tree device of the same type on a standalone Miniserver (not in a Client-Gateway setup). Save in the Miniserver to pair the Tree device. Afterwards the program must be loaded from the Miniserver to transfer the actual serial number of the Tree device into the program. | - | - |
| Show Button 1 | Show individual button | - | - |
| Show Button 2 | Show individual button | - | - |
| Show Button 3 | Show individual button | - | - |
| Show Button 4 | Show individual button | - | - |
| Show Button 5 | Show individual button | - | - |
| Audible acknowledgement | Audible acknowledgement on button press | - | - |
| Transmission cycle | Specifies how often analog values from sensors are transmitted [s] 0 ... Values are updated at least every 60 min, or if temperature deviates by 0.2 °C or humidity deviates by 2% to the last transmitted value | 0...3600 | 0 |

---

## Safety Instructions

The installation must be carried out by a qualified technician in accordance with all applicable regulations.

The installation requires a suitable enclosure to ensure protection against contact, water and dust.

---

## Documents

[**Datasheet Touch Tree**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchTree_100221.pdf)

[**Datasheet Touch Tree US**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_TouchTreeUS_100425.pdf)

---

## UK Installation

The Loxone Touch is a slimline product, as such it should be mounted on a European style circular back box for correct fitting. If it is mounted on a standard UK back box/pattress box then it will have an improper fitting and will be likely to fall in at the corners or fall off the back box. [Click on the image to purchase them.](https://shop.loxone.com/enuk/circular-dry-lining-box.html)

[
![European Dry Lining Backbox](https://www.loxone.com/enen/wp-content/uploads/sites/3/2021/04/Orange-circular-back-box-300x300-1.jpg)
](https://shop.loxone.com/enuk/circular-dry-lining-box.html)