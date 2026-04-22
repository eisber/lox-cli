# Hunter Douglas PowerView Hub

Source: https://www.loxone.com/enen/kb/hunter-douglas-powerview-hub/

---

Integrating a Hunter Douglas PowerView Hub enables the control of Hunter Douglas shading elements.

The individual shades are programmed via the PowerView Hub and the PowerView App, and rooms and scenes are defined.
The shades and scenes can be controlled by the Miniserver.

> **ℹ️ Note:** Controlling shades is only supported by the PowerView Gen.3!

> **ℹ️ Note:** The Miniserver Gen. 1 does not support control of the PowerView Hub!

## Table of Contents
- [Configuration Example](#config)
- [Inputs, Outputs, Properties](#Diagnostic)

---

## Configuration Example

### Requirements:

The current Miniserver is used.
The PowerView Hub has been configured and can be accessed via the network.
The individual blinds are programmed and configured via the PowerView App.

### Creating Scenes

First, create the desired scenes in the Hunter Douglas PowerView app:

![PowerViewAppScenes](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PowerViewAppScenes.png)

Then a network periphery search is started in Loxone Config, and once the PowerView Hub is found it will be listed in the search results:

![PowerViewHubNetworkSearch](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PowerViewHubNetworkSearch.png)

Use the + button to add the device to the configuration and save the program to the Miniserver.

### Add Scenes in Loxone Config

The device should now be marked green in the Periphery tree of Loxone Config.
You can then search for available scenes by clicking on the **Find periphery** button:

![PowerViewHubFindPeriphery](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PowerViewHubFindPeriphery.png)

Now the available scenes are listed in the Periphery tree and ready for use in configuration:

![PowerViewHubScenesConfig](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/PowerViewHubScenesConfig.png)

Up to 32 scenes can be controlled in this manner.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Onlinestatus | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Address | IP Address of the PowerView Hub | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |

---