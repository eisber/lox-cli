# EEBUS

Source: https://www.loxone.com/enen/kb/eebus/

---

The commissioning problems with EEBus can be solved with the [public beta](https://www.loxone.com/enen/support/downloads/#section-beta) version 15.1 beta 2 version.

EEBUS is an open standard to connect devices across manufacturers and industries. The primary goal is to increase energy efficiency.
For the latest information on applications and capabilities, visit the [website of the EEBus Initiative](https://www.eebus.org).

EEBUS specifies various UseCases.
The Loxone system currently only supports the HVAC UseCase.
The devices are integrated via the network using the SHIP protocol.

> **ℹ️ Note:** The Miniserver Gen. 1 does not support EEBUS!

Vaillant offers [EEBUS enabled heat pumps](https://www.vaillant-group.com/news-centre/web-special-online-press-conference/neue-regelungen-fuer-mehr-komfort-neues-gateway-fuer-ausfuehrliches-monitoring-1-1680893.html), which can be integrated as described in the configuration example.

## Table of Contents
- [Vaillant Example](#config)
- [Inputs, Outputs, Properties](#Diagnostic)

---

## Vaillant Example

### Requirements:

The current Miniserver is used.
EEBUS devices are connected to the network, configured and accessible.
The EEBUS feature or interface on the device is enabled and authorised.

### Example Vaillant Heat Pump

First, EEBUS is enabled in the settings menu of the unit's multiMATIC app:

![vaillantEEbusActivate](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/vaillantEEbusActivate.png)

Then a Network Device Search is started in Loxone Config, and once the device is found it will be listed in the search results:

![networksearch eebus](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/networksearch_eebus.png)

Use the + button to add the device to the configuration and save the program to the Miniserver.

Next, communication with the Miniserver must be approved using the multiMATIC app, where the Miniserver should already be listed among the available devices.
Select the Miniserver, and then add it to the list of trusted devices:

![vaillantEEBusTrustMS](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/vaillantEEBusTrustMS.png)

The device should now be marked green in the Periphery tree of Loxone Config.
You can then search for available inputs and outputs by clicking on the **Find periphery** button:

![vaillantfindperiphery](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/vaillantfindperiphery.png)

Now the available Inputs and Outputs are listed in the Periphery tree and ready for use in configuration:

![vaillantperipherytree](http://updatefiles.loxone.com/KnowledgeBase/Online/Common/Images/vaillantperipherytree.png)

The screenshot above shows the maximum number of functions currently supported. This depends on the EEBUS standard. Please note that not all of these inputs/outputs may be available, depending on the device configuration.

> **ℹ️ Note:** When Vaillant gateways perform an update, their configuration may change. In such cases, the Miniserver will generate a corresponding system message. It is then necessary to delete and re-add the peripheral in Loxone Config. Until this is done, the integration will not function properly.

---

## Diagnostic Inputs

| Summary | Description | Unit | Value Range |
| --- | --- | --- | --- |
| Online Status EEBUS Device | Indicates whether the device can be reached by the Miniserver. Diagnostics for Air devices Diagnostics for Tree devices Diagnostics for Extensions | Digital | 0/1 |

---

## Properties

| Summary | Description | Default Value |
| --- | --- | --- |
| Address | IP Address and Port of the Device | - |
| Monitor Online Status | If checked, you will be notified via System Status or the Mailer if the device is no longer available or goes offline. | - |

---