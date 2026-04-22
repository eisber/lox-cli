# Remote Air

Source: https://www.loxone.com/enen/kb/remote-air/

---

The Remote Air is a wireless remote control with 12 buttons, including 5 buttons that follow the **[Loxone switch standard](https://www.loxone.com/enen/smart-home/standards-and-recommendations/)**.

[**Datasheet Remote Air**](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Documents/Datasheet_RemoteAir_100624,100625.pdf)

## Table of Contents
- [Commissioning](#Commissioning)
- [Programming](#baseconf)
- [T5 buttons as separate inputs](#T5Inputs)
- [Inputs, Outputs, Properties](#Sensor)
- [Safety Instructions](#SafetyInstructions)
- [Documents](#Documents)

---

## Commissioning

In delivery state, pairing mode will be active after inserting the battery. This is indicated by the status LED flashing red/green/orange.

**[Then follow the pairing procedure on the Air Interface.](https://www.loxone.com/help/air-interface#AirPair)**

To activate the pairing mode manually, hold down any button for at least 5 seconds immediately after inserting the batteries.

![RemoteAir comm](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RemoteAir_comm.png)

---

## Programming

Once the Remote Air has been paired, the T5 input can be dragged onto the [Automatic Shading](https://www.loxone.com/help/auto-shading), [Lighting Controller](https://www.loxone.com/help/lighting-controller) and [Audio Player](https://www.loxone.com/help/audioplayer).

![RemoteAir progT5](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RemoteAir_progT5.gif)

Additionally, there are three buttons for audio configuration, one button dedicated to the Leave room function, and three freely configurable buttons.

![RemoteAir buttons](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RemoteAir_buttons.png)

In Simulation/LiveView a representation of the Remote Air can be opened.

![RemoteAir simulation](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RemoteAir_simulation.png)

---

## T5 buttons as separate inputs

Alternatively, the T5 buttons can be activated in the properties and used as digital inputs:

![RemoteAir T5DI](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RemoteAir_T5DI.png)

The assignment of the T5 inputs is as follows:

![RemoteAir T5buttons](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/RemoteAir_T5buttons.png)

---

## Sensors

| Summary | Unit | Value Range |
| --- | --- | --- |
| Previous | Digital | 0/1 |
| Play/Pause | Digital | 0/1 |
| Next | Digital | 0/1 |
| Leave room | Digital | 0/1 |
| I | Digital | 0/1 |
| II | Digital | 0/1 |
| III | Digital | 0/1 |
| T5 | - | ∞ |

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Battery Low | This input activates when the battery level is